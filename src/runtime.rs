//! Linux file/service operations. The preview worker runs independently of the GUI.
use base64::{Engine, engine::general_purpose::STANDARD};
use serde::Deserialize;
use serde_json::json;
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

struct Temporary {
    path: PathBuf,
    file: File,
}

impl Temporary {
    fn create(directory: &Path) -> Result<Self, String> {
        for _ in 0..100 {
            let sequence = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
            let path = directory.join(format!(
                ".kanata-studio-rust-{}-{sequence}.kbd",
                std::process::id()
            ));
            match OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&path)
            {
                Ok(file) => return Ok(Self { path, file }),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error.to_string()),
            }
        }
        Err("Temporäre Datei konnte nicht angelegt werden".into())
    }
}

impl Drop for Temporary {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn plain_file(path: &Path) -> Result<(), String> {
    let meta = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if !meta.is_file() || meta.file_type().is_symlink() {
        return Err("Verknüpfte oder nicht reguläre Konfigurationsdatei".into());
    }
    Ok(())
}

pub fn atomic_write(path: &Path, bytes: &[u8], mode: u32) -> Result<(), String> {
    let parent = path.parent().ok_or("Datei hat kein Elternverzeichnis")?;
    let mut temp = Temporary::create(parent)?;
    temp.file
        .set_permissions(fs::Permissions::from_mode(mode))
        .map_err(|e| e.to_string())?;
    temp.file.write_all(bytes).map_err(|e| e.to_string())?;
    temp.file.sync_all().map_err(|e| e.to_string())?;
    fs::rename(&temp.path, path).map_err(|e| e.to_string())?;
    File::open(parent)
        .and_then(|dir| dir.sync_all())
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Drain pipes concurrently so verbose command output cannot block the child.
pub fn command(program: &str, args: &[&str], timeout: Duration) -> Result<String, String> {
    let mut process = Command::new(program);
    if let Some(native_path) = std::env::var_os("KANATA_STUDIO_SYSTEM_LD_LIBRARY_PATH") {
        if native_path.is_empty() {
            process.env_remove("LD_LIBRARY_PATH");
        } else {
            process.env("LD_LIBRARY_PATH", native_path);
        }
    }
    let mut child = process
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("{program}: {e}"))?;
    let mut stdout = child.stdout.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();
    let out = thread::spawn(move || {
        let mut bytes = Vec::new();
        stdout.read_to_end(&mut bytes).map(|_| bytes)
    });
    let err = thread::spawn(move || {
        let mut bytes = Vec::new();
        stderr.read_to_end(&mut bytes).map(|_| bytes)
    });
    let started = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Ok(status),
            Ok(None) if started.elapsed() < timeout => thread::sleep(Duration::from_millis(20)),
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                break Err(format!("{program}: Zeitlimit überschritten"));
            }
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                break Err(error.to_string());
            }
        }
    };
    // On timeout, pipe readers may still wait on a grandchild. Do not block the GUI.
    let status = status?;
    let output = out
        .join()
        .map_err(|_| "Ausgabeleser fehlgeschlagen")?
        .map_err(|e| e.to_string())?;
    let errors = err
        .join()
        .map_err(|_| "Fehlerleser fehlgeschlagen")?
        .map_err(|e| e.to_string())?;
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output),
        String::from_utf8_lossy(&errors)
    );
    if status.success() {
        Ok(text.trim().into())
    } else {
        Err(format!("{program} fehlgeschlagen: {}", text.trim()))
    }
}

fn reload() -> Result<(), String> {
    command(
        "systemctl",
        &["--user", "reload", "kanata.service"],
        Duration::from_secs(10),
    )
    .map(|_| ())
}

pub fn check(binary: &str, path: &Path, source: &str) -> Result<(), String> {
    let directory = path.parent().ok_or("Ungültiger Konfigurationspfad")?;
    let mut temp = Temporary::create(directory)?;
    temp.file
        .write_all(source.as_bytes())
        .map_err(|e| e.to_string())?;
    temp.file.flush().map_err(|e| e.to_string())?;
    let file = temp.path.to_str().ok_or("Ungültiger Dateiname")?;
    command(binary, &["--check", "-c", file], Duration::from_secs(12)).map(|_| ())
}

pub fn save(path: &Path, original: &[u8], candidate: &[u8]) -> Result<(), String> {
    plain_file(path)?;
    let current = fs::read(path).map_err(|e| e.to_string())?;
    if current != original {
        return Err("Datei wurde extern geändert · bitte neu öffnen".into());
    }
    let mode = fs::metadata(path)
        .map_err(|e| e.to_string())?
        .permissions()
        .mode();
    // Keep the rollback path active even if syncing the parent directory fails after rename.
    let result = atomic_write(path, candidate, mode).and_then(|_| reload());
    if let Err(error) = result {
        if fs::read(path).is_ok_and(|bytes| bytes == candidate)
            && let Err(rollback) = atomic_write(path, original, mode).and_then(|_| reload())
        {
            return Err(format!(
                "{error}; Wiederherstellung fehlgeschlagen: {rollback}"
            ));
        }
        return Err(error);
    }
    if fs::read(path).map_err(|e| e.to_string())? != candidate {
        return Err("Datei wurde während des Speicherns extern geändert".into());
    }
    Ok(())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PreviewRequest {
    config_path: PathBuf,
    original: String,
    candidate: String,
    #[serde(default)]
    deadline: u64,
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn state(directory: &Path, phase: &str, message: Option<&str>) -> Result<(), String> {
    let mut value = json!({"phase": phase});
    if let Some(message) = message {
        value["message"] = json!(message);
    }
    atomic_write(
        &directory.join("state.json"),
        value.to_string().as_bytes(),
        0o600,
    )?;
    if phase == "reverted" {
        match fs::remove_file(directory.join("request.json")) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.to_string()),
        }
    }
    let _ = fs::remove_file(directory.join("cancel"));
    Ok(())
}

fn restore(
    directory: &Path,
    request: &PreviewRequest,
    original: &[u8],
    candidate: &[u8],
) -> Result<(), String> {
    plain_file(&request.config_path)?;
    let current = fs::read(&request.config_path).map_err(|e| e.to_string())?;
    if current != original && current != candidate {
        return Err("Datei wurde extern geändert; Wiederherstellung gestoppt".into());
    }
    let mode = fs::metadata(&request.config_path)
        .map_err(|e| e.to_string())?
        .permissions()
        .mode();
    if current != original {
        atomic_write(&request.config_path, original, mode)?;
    }
    // Retry the service reload even when a previous attempt already restored the bytes.
    reload()?;
    state(directory, "reverted", None)
}

pub fn worker(directory: &Path, recover: bool) -> Result<(), String> {
    let run = || -> Result<(), String> {
        let bytes = match fs::read(directory.join("request.json")) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(error.to_string()),
        };
        let request: PreviewRequest = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        let original = STANDARD
            .decode(&request.original)
            .map_err(|e| e.to_string())?;
        let candidate = STANDARD
            .decode(&request.candidate)
            .map_err(|e| e.to_string())?;
        if recover {
            return restore(directory, &request, &original, &candidate);
        }
        if request.deadline == 0 {
            return Err("Testfrist fehlt".into());
        }
        plain_file(&request.config_path)?;
        if fs::read(&request.config_path).map_err(|e| e.to_string())? != original {
            return Err("Konfiguration wurde vor dem Test geändert".into());
        }
        if directory.join("cancel").exists() {
            return state(directory, "reverted", None);
        }
        let status = json!({"phase":"testing", "pid":std::process::id(), "deadline":request.deadline,
            "bootId":fs::read_to_string("/proc/sys/kernel/random/boot_id").unwrap_or_default().trim()});
        atomic_write(
            &directory.join("state.json"),
            status.to_string().as_bytes(),
            0o600,
        )?;
        let mode = fs::metadata(&request.config_path)
            .map_err(|e| e.to_string())?
            .permissions()
            .mode();
        if let Err(error) =
            atomic_write(&request.config_path, &candidate, mode).and_then(|_| reload())
        {
            return match restore(directory, &request, &original, &candidate) {
                Ok(()) => Err(error),
                Err(rollback) => Err(format!(
                    "{error}; Wiederherstellung fehlgeschlagen: {rollback}"
                )),
            };
        }
        while now_ms() < request.deadline && !directory.join("cancel").exists() {
            thread::sleep(Duration::from_millis(100));
        }
        restore(directory, &request, &original, &candidate)
    };
    match run() {
        Ok(()) => Ok(()),
        Err(error) => {
            let _ = state(directory, "error", Some(&error));
            Err(error)
        }
    }
}
