//! File and Kanata service operations (systemd on Linux, a managed process on Windows). The preview worker runs independently of the GUI.
use base64::{Engine, engine::general_purpose::STANDARD};
use serde::Deserialize;
use serde_json::json;
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
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
            let mut options = OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
            match options.open(&path) {
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
    #[cfg(unix)]
    temp.file
        .set_permissions(std::os::unix::fs::PermissionsExt::from_mode(mode))
        .map_err(|e| e.to_string())?;
    #[cfg(not(unix))]
    let _ = mode;
    temp.file.write_all(bytes).map_err(|e| e.to_string())?;
    temp.file.sync_all().map_err(|e| e.to_string())?;
    fs::rename(&temp.path, path).map_err(|e| e.to_string())?;
    // Windows cannot open directories as files; NTFS journals the rename itself.
    #[cfg(unix)]
    File::open(parent)
        .and_then(|dir| dir.sync_all())
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Unix permission bits to keep on rewrite; Windows keeps the directory's inherited ACL.
fn file_mode(path: &Path) -> Result<u32, String> {
    let meta = fs::metadata(path).map_err(|e| e.to_string())?;
    #[cfg(unix)]
    return Ok(std::os::unix::fs::PermissionsExt::mode(&meta.permissions()));
    #[cfg(not(unix))]
    {
        let _ = meta;
        Ok(0o600)
    }
}

/// Identifies the current boot so a stale test state is not mistaken for a live one.
pub fn boot_id() -> String {
    #[cfg(windows)]
    return crate::windows::boot_id();
    #[cfg(not(windows))]
    fs::read_to_string("/proc/sys/kernel/random/boot_id")
        .unwrap_or_default()
        .trim()
        .into()
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
    #[cfg(windows)]
    {
        // A GUI app must not flash a console window for every helper it runs.
        use std::os::windows::process::CommandExt;
        process.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
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

/// Kanata has no signal-based reload (SIGUSR1 terminates it, and a unit with
/// `ExecReload=kill -USR1` only recovers through `Restart=` after `RestartSec`), so the
/// service is restarted. This works for any unit, with or without `ExecReload`.
#[cfg(not(windows))]
fn reload() -> Result<(), String> {
    command(
        "systemctl",
        &["--user", "restart", "kanata.service"],
        Duration::from_secs(15),
    )
    .map(|_| ())
}

#[cfg(windows)]
fn reload() -> Result<(), String> {
    crate::windows::restart_kanata()
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
    let mode = file_mode(path)?;
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
    let mode = file_mode(&request.config_path)?;
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
            "bootId":boot_id()});
        atomic_write(
            &directory.join("state.json"),
            status.to_string().as_bytes(),
            0o600,
        )?;
        let mode = file_mode(&request.config_path)?;
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
