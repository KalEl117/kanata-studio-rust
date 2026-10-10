use base64::{Engine, engine::general_purpose::STANDARD};
use kanata_studio_core::runtime;
use serde_json::{Value, json};
use std::{
    fs,
    io::Write,
    os::unix::fs::{PermissionsExt, symlink},
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

static SEQUENCE: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    dir: PathBuf,
    config: PathBuf,
    preview: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let dir = std::env::temp_dir().join(format!(
            "kanata-rust-test-{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&dir).unwrap();
        let config = dir.join("test.kbd");
        fs::write(&config, b"original").unwrap();
        fs::set_permissions(&config, fs::Permissions::from_mode(0o640)).unwrap();
        let preview = dir.join("preview");
        fs::create_dir(&preview).unwrap();
        fs::create_dir(dir.join("bin")).unwrap();
        let script = dir.join("bin/systemctl");
        fs::write(
            &script,
            r#"#!/bin/sh
printf 'reload\n' >> "$KANATA_TEST_DIR/calls"
printf '%s' "${LD_LIBRARY_PATH-unset}" > "$KANATA_TEST_DIR/library-path"
if [ "$KANATA_TEST_FAIL" = once ] && [ ! -e "$KANATA_TEST_DIR/failed" ]; then
  touch "$KANATA_TEST_DIR/failed"
  exit 1
fi
exit 0
"#,
        )
        .unwrap();
        fs::set_permissions(&script, fs::Permissions::from_mode(0o700)).unwrap();
        Self {
            dir,
            config,
            preview,
        }
    }
    fn spawn(&self, value: Value, fail: bool) -> Child {
        let mut child = Command::new(env!("CARGO_BIN_EXE_kanata-studio-helper"))
            .env(
                "PATH",
                format!("{}:/usr/bin:/bin", self.dir.join("bin").display()),
            )
            .env("KANATA_TEST_DIR", &self.dir)
            .env("KANATA_TEST_FAIL", if fail { "once" } else { "" })
            .env("LD_LIBRARY_PATH", "/tmp/kanata-packaged-libraries")
            .env("KANATA_STUDIO_SYSTEM_LD_LIBRARY_PATH", "")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(value.to_string().as_bytes())
            .unwrap();
        child
    }
    fn call(&self, value: Value, fail: bool) -> (bool, Value) {
        let output = self.spawn(value, fail).wait_with_output().unwrap();
        (
            output.status.success(),
            serde_json::from_slice(&output.stdout).unwrap(),
        )
    }
    fn preview_request(&self, deadline: u64) {
        fs::write(self.preview.join("request.json"), json!({"configPath":self.config,
            "original":STANDARD.encode(b"original"), "candidate":STANDARD.encode(b"candidate"), "deadline":deadline}).to_string()).unwrap();
    }
    fn worker_request(&self, recover: bool) -> Value {
        json!({"op":"worker", "directory":self.preview, "recover":recover})
    }
    fn save_request(&self) -> Value {
        json!({"op":"save", "path":self.config, "original":STANDARD.encode(b"original"), "source":"candidate"})
    }
    fn state(&self) -> Value {
        serde_json::from_slice(&fs::read(self.preview.join("state.json")).unwrap()).unwrap()
    }
    fn calls(&self) -> usize {
        fs::read_to_string(self.dir.join("calls"))
            .unwrap_or_default()
            .lines()
            .count()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[test]
fn atomic_write_keeps_permissions_and_leaves_no_temporary_file() {
    let fixture = Fixture::new();
    runtime::atomic_write(&fixture.config, b"candidate", 0o640).unwrap();
    assert_eq!(fs::read(&fixture.config).unwrap(), b"candidate");
    assert_eq!(
        fs::metadata(&fixture.config).unwrap().permissions().mode() & 0o777,
        0o640
    );
    assert!(!fs::read_dir(&fixture.dir).unwrap().any(|entry| {
        entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".kanata")
    }));
}

#[test]
fn save_reloads_service_and_rolls_back_on_reload_failure() {
    let fixture = Fixture::new();
    let (ok, error) = fixture.call(fixture.save_request(), true);
    assert!(!ok, "{error}");
    assert_eq!(fs::read(&fixture.config).unwrap(), b"original");
    assert_eq!(fixture.calls(), 2);
    assert_eq!(
        fs::read_to_string(fixture.dir.join("library-path")).unwrap(),
        "unset"
    );
    assert!(fixture.call(fixture.save_request(), false).0);
    assert_eq!(fs::read(&fixture.config).unwrap(), b"candidate");
}

#[test]
fn save_rejects_external_changes_and_symlinks_before_service_reload() {
    let fixture = Fixture::new();
    fs::write(&fixture.config, b"external").unwrap();
    assert!(!fixture.call(fixture.save_request(), false).0);
    assert_eq!(fs::read(&fixture.config).unwrap(), b"external");
    let link = fixture.dir.join("link.kbd");
    symlink(&fixture.config, &link).unwrap();
    let mut request = fixture.save_request();
    request["path"] = json!(link);
    assert!(!fixture.call(request, false).0);
    assert_eq!(fixture.calls(), 0);
}

#[test]
fn expired_preview_restores_configuration_and_protects_state_file() {
    let fixture = Fixture::new();
    fixture.preview_request(now_ms());
    assert!(fixture.call(fixture.worker_request(false), false).0);
    assert_eq!(fs::read(&fixture.config).unwrap(), b"original");
    assert_eq!(fixture.state()["phase"], "reverted");
    assert!(!fixture.preview.join("request.json").exists());
    assert_eq!(fixture.calls(), 2);
    assert_eq!(
        fs::metadata(fixture.preview.join("state.json"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
}

#[test]
fn cancellation_restores_configuration_before_deadline() {
    let fixture = Fixture::new();
    fixture.preview_request(now_ms() + 20_000);
    let mut child = fixture.spawn(fixture.worker_request(false), false);
    let start = Instant::now();
    while fs::read(&fixture.config).unwrap() != b"candidate"
        && start.elapsed() < Duration::from_secs(3)
    {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(fs::read(&fixture.config).unwrap(), b"candidate");
    fs::write(fixture.preview.join("cancel"), b"").unwrap();
    while child.try_wait().unwrap().is_none() && start.elapsed() < Duration::from_secs(5) {
        std::thread::sleep(Duration::from_millis(10));
    }
    if child.try_wait().unwrap().is_none() {
        child.kill().unwrap();
        panic!("worker did not cancel");
    }
    assert!(child.wait().unwrap().success());
    assert_eq!(fs::read(&fixture.config).unwrap(), b"original");
    assert_eq!(fixture.state()["phase"], "reverted");
}

#[test]
fn recovery_after_interruption_restores_and_retries_reload_of_restored_file() {
    let fixture = Fixture::new();
    fixture.preview_request(now_ms());
    fs::write(&fixture.config, b"candidate").unwrap();
    assert!(!fixture.call(fixture.worker_request(true), true).0);
    assert_eq!(fs::read(&fixture.config).unwrap(), b"original");
    assert!(fixture.preview.join("request.json").exists());
    assert!(fixture.call(fixture.worker_request(true), false).0);
    assert_eq!(fixture.state()["phase"], "reverted");
    assert_eq!(fixture.calls(), 2);
}

#[test]
fn recovery_keeps_external_edits_and_request_for_manual_resolution() {
    let fixture = Fixture::new();
    fixture.preview_request(now_ms());
    fs::write(&fixture.config, b"external").unwrap();
    assert!(!fixture.call(fixture.worker_request(true), false).0);
    assert_eq!(fs::read(&fixture.config).unwrap(), b"external");
    assert_eq!(fixture.state()["phase"], "error");
    assert!(fixture.preview.join("request.json").exists());
    assert_eq!(fixture.calls(), 0);
}

#[test]
fn failed_preview_reload_restores_original_and_reports_error() {
    let fixture = Fixture::new();
    fixture.preview_request(now_ms());
    assert!(!fixture.call(fixture.worker_request(false), true).0);
    assert_eq!(fs::read(&fixture.config).unwrap(), b"original");
    assert_eq!(fixture.state()["phase"], "error");
    assert_eq!(fixture.calls(), 2);
}

#[test]
fn check_uses_config_directory_and_cleans_up_temporary_file() {
    let fixture = Fixture::new();
    let script = fixture.dir.join("bin/kanata");
    fs::write(
        &script,
        r#"#!/bin/sh
[ "$1" = --check ] && [ "$2" = -c ] || exit 1
[ "$(dirname "$3")" = "$KANATA_TEST_DIR" ] || exit 1
[ "$(cat "$3")" = candidate ] || exit 1
"#,
    )
    .unwrap();
    fs::set_permissions(&script, fs::Permissions::from_mode(0o700)).unwrap();
    assert!(
        fixture
            .call(
                json!({"op":"check", "binary":script, "path":fixture.config, "source":"candidate"}),
                false
            )
            .0
    );
    assert_eq!(fs::read(&fixture.config).unwrap(), b"original");
    assert!(!fs::read_dir(&fixture.dir).unwrap().any(|entry| {
        entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".kanata")
    }));
}

#[test]
fn child_process_timeout_is_bounded() {
    let start = Instant::now();
    assert!(
        runtime::command(
            "/bin/sh",
            &["-c", "exec sleep 2"],
            Duration::from_millis(50)
        )
        .is_err()
    );
    assert!(start.elapsed() < Duration::from_secs(1));
}
