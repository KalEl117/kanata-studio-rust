// These tests drive Linux-only behaviour (systemd and /dev/uinput) through shell scripts.
#![cfg(unix)]

use kanata_studio_core::setup::{Permissions, group_entry, unit_text};
use serde_json::{Value, json};
use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[test]
fn group_file_lines_are_parsed() {
    let groups = "root:x:0:\ninput:x:994:alice,bob\nuinput:x:961:\n";
    assert_eq!(
        group_entry(groups, "input"),
        Some((994, vec!["alice".into(), "bob".into()]))
    );
    assert_eq!(group_entry(groups, "uinput"), Some((961, vec![])));
    assert_eq!(group_entry(groups, "plugdev"), None);
}

#[test]
fn fresh_system_gets_every_command_in_setup_guide_order() {
    let fresh = Permissions::default();
    assert_eq!(
        fresh.commands(),
        [
            "sudo groupadd --system uinput",
            "sudo usermod -aG input,uinput \"$USER\"",
            "sudo modprobe uinput",
            "echo uinput | sudo tee /etc/modules-load.d/uinput.conf",
            "echo 'KERNEL==\"uinput\", MODE=\"0660\", GROUP=\"uinput\", OPTIONS+=\"static_node=uinput\"' | sudo tee /etc/udev/rules.d/99-input.rules",
            "sudo udevadm control --reload-rules && sudo udevadm trigger",
        ]
    );
    assert!(fresh.relogin());
}

#[test]
fn only_missing_steps_are_listed() {
    let configured = Permissions {
        input_readable: true,
        uinput_exists: true,
        uinput_rule: true,
        uinput_group_exists: true,
        in_input: true,
        session_input: true,
        ..Permissions::default()
    };
    assert_eq!(configured.commands(), ["sudo usermod -aG uinput \"$USER\""]);
    // Configured but not yet active in this session: nothing to run, only log in again.
    let pending = Permissions {
        in_uinput: true,
        ..configured
    };
    assert!(pending.commands().is_empty());
    assert!(pending.relogin());
    let ready = Permissions {
        uinput_writable: true,
        session_uinput: true,
        ..pending
    };
    assert!(ready.ready() && ready.commands().is_empty() && !ready.relogin());
}

#[test]
fn unit_quotes_paths_and_disables_specifiers() {
    let unit = unit_text("/opt/kanata bin/kanata", "/home/a/100% \"x\"/$cfg.kbd");
    assert!(unit.contains(
        "ExecStart=\"/opt/kanata bin/kanata\" -c \"/home/a/100%% \\\"x\\\"/$$cfg.kbd\"\n"
    ));
    assert!(unit.contains("WantedBy=default.target"));
    assert!(!unit.contains("ExecReload"));
}

struct Fixture(PathBuf);

impl Fixture {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("kanata-setup-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("bin")).unwrap();
        let systemctl = dir.join("bin/systemctl");
        fs::write(
            &systemctl,
            "#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"$TEST_ROOT/calls\"\n\
             case \"$*\" in *LoadState*) cat \"$TEST_ROOT/load-state\" 2>/dev/null || echo not-found ;; esac\n",
        )
        .unwrap();
        fs::set_permissions(&systemctl, fs::Permissions::from_mode(0o755)).unwrap();
        fs::write(dir.join("kanata.kbd"), "(defsrc a)\n(deflayer base _)\n").unwrap();
        Fixture(dir)
    }

    fn call(&self, request: Value) -> Value {
        let mut child = Command::new(env!("CARGO_BIN_EXE_kanata-studio-helper"))
            .env(
                "PATH",
                format!("{}:/usr/bin:/bin", self.0.join("bin").display()),
            )
            .env("TEST_ROOT", &self.0)
            .env("KANATA_STUDIO_UNIT_DIR", self.0.join("units"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(request.to_string().as_bytes())
            .unwrap();
        serde_json::from_slice(&child.wait_with_output().unwrap().stdout).unwrap()
    }

    fn calls(&self) -> String {
        fs::read_to_string(self.0.join("calls")).unwrap_or_default()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn service_unit_is_written_enabled_and_never_replaced() {
    let fixture = Fixture::new("create");
    let config = fixture.0.join("kanata.kbd");
    let request = json!({"op":"createService","binary":"/usr/bin/kanata","config":config});
    let created = fixture.call(request.clone());
    assert_eq!(created["ok"], true, "{created}");
    let unit = fs::read_to_string(fixture.0.join("units/kanata.service")).unwrap();
    assert!(unit.contains(&format!(
        "-c \"{}\"",
        config.canonicalize().unwrap().display()
    )));
    let calls = fixture.calls();
    assert!(
        calls.contains("--user daemon-reload\n--user enable kanata.service\n"),
        "{calls}"
    );
    // Only started when this machine already has the device permissions.
    assert_eq!(
        calls.contains("--user start kanata.service"),
        created["started"] == true
    );

    fs::write(fixture.0.join("load-state"), "loaded\n").unwrap();
    let again = fixture.call(request);
    assert_eq!(again["error"], "kanata.service ist bereits eingerichtet");
    assert_eq!(
        fs::read_to_string(fixture.0.join("units/kanata.service")).unwrap(),
        unit
    );
}

#[test]
fn service_needs_kanata_and_a_config() {
    let fixture = Fixture::new("missing");
    let missing = fixture
        .call(json!({"op":"createService","binary":"","config":fixture.0.join("kanata.kbd")}));
    assert_eq!(missing["error"], "Kanata wurde nicht gefunden");
    let no_config = fixture.call(json!({"op":"createService","binary":"/usr/bin/kanata","config":fixture.0.join("none.kbd")}));
    assert_eq!(
        no_config["error"],
        "Bitte zuerst eine Konfiguration anlegen oder öffnen"
    );
    assert!(!fixture.0.join("units").exists() && fixture.calls().is_empty());
}
