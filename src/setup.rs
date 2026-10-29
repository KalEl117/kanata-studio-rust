//! Kanata service setup. Kanata Studio writes and enables the user unit (no root needed); the
//! device permissions need root, so it only reports them and lists the commands for the user.
use crate::runtime::command;
use serde_json::{Value, json};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{FileTypeExt, MetadataExt};
use std::path::{Path, PathBuf};
use std::time::Duration;

const UDEV_RULE: &str =
    r#"KERNEL=="uinput", MODE="0660", GROUP="uinput", OPTIONS+="static_node=uinput""#;

/// What Kanata needs on Linux (kanata docs/setup-linux.md): read /dev/input, write /dev/uinput.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Permissions {
    pub input_readable: bool,
    pub uinput_exists: bool,
    pub uinput_writable: bool,
    /// /dev/uinput belongs to group `uinput` with group read/write, i.e. the udev rule applies.
    pub uinput_rule: bool,
    pub uinput_group_exists: bool,
    /// Membership configured in /etc/group, and active in this login session.
    pub in_input: bool,
    pub in_uinput: bool,
    pub session_input: bool,
    pub session_uinput: bool,
}

impl Permissions {
    pub fn ready(&self) -> bool {
        self.input_readable && self.uinput_writable
    }

    /// Root commands for whatever is missing, in the order of the Kanata setup guide.
    pub fn commands(&self) -> Vec<String> {
        let mut commands = Vec::new();
        if self.ready() {
            return commands;
        }
        if !self.uinput_group_exists {
            commands.push("sudo groupadd --system uinput".into());
        }
        let missing: Vec<&str> = [("input", self.in_input), ("uinput", self.in_uinput)]
            .iter()
            .filter(|(_, member)| !member)
            .map(|(group, _)| *group)
            .collect();
        if !missing.is_empty() {
            commands.push(format!("sudo usermod -aG {} \"$USER\"", missing.join(",")));
        }
        if !self.uinput_exists {
            commands.push("sudo modprobe uinput".into());
            commands.push("echo uinput | sudo tee /etc/modules-load.d/uinput.conf".into());
        }
        if !self.uinput_rule {
            commands.push(format!(
                "echo '{UDEV_RULE}' | sudo tee /etc/udev/rules.d/99-input.rules"
            ));
            commands.push("sudo udevadm control --reload-rules && sudo udevadm trigger".into());
        }
        commands
    }

    /// Group changes only take effect in a new login session.
    pub fn relogin(&self) -> bool {
        !self.ready()
            && (!self.in_input || !self.in_uinput || !self.session_input || !self.session_uinput)
    }
}

/// `name:x:gid:member,member` lines of /etc/group: (gid, members) of `group`.
pub fn group_entry(groups: &str, group: &str) -> Option<(u32, Vec<String>)> {
    groups.lines().find_map(|line| {
        let mut fields = line.split(':');
        if fields.next()? != group {
            return None;
        }
        let gid = fields.nth(1)?.parse().ok()?;
        let members = fields
            .next()
            .unwrap_or("")
            .split(',')
            .filter(|m| !m.is_empty())
            .map(str::to_owned)
            .collect();
        Some((gid, members))
    })
}

fn user() -> String {
    std::env::var("USER")
        .ok()
        .filter(|u| !u.is_empty())
        .or_else(|| command("id", &["-un"], Duration::from_secs(2)).ok())
        .unwrap_or_default()
}

/// Inspects the real system. Opens devices only to test access and closes them at once.
pub fn permissions() -> Permissions {
    let groups = fs::read_to_string("/etc/group").unwrap_or_default();
    let user = user();
    let member = |group: &str| {
        group_entry(&groups, group).is_some_and(|(_, members)| members.contains(&user))
    };
    let session = command("id", &["-Gn"], Duration::from_secs(2)).unwrap_or_default();
    let in_session = |group: &str| session.split_whitespace().any(|g| g == group);
    let uinput = Path::new("/dev/uinput");
    let uinput_meta = fs::metadata(uinput)
        .ok()
        .filter(|m| m.file_type().is_char_device());
    let uinput_gid = group_entry(&groups, "uinput").map(|(gid, _)| gid);
    let input_readable = fs::read_dir("/dev/input")
        .map(|entries| {
            entries.flatten().any(|entry| {
                entry.file_name().to_string_lossy().starts_with("event")
                    && File::open(entry.path()).is_ok()
            })
        })
        .unwrap_or(false);
    Permissions {
        input_readable,
        uinput_exists: uinput_meta.is_some(),
        uinput_writable: OpenOptions::new().write(true).open(uinput).is_ok(),
        uinput_rule: uinput_meta
            .as_ref()
            .is_some_and(|m| Some(m.gid()) == uinput_gid && m.mode() & 0o060 == 0o060),
        uinput_group_exists: uinput_gid.is_some(),
        in_input: member("input"),
        in_uinput: member("uinput"),
        session_input: in_session("input"),
        session_uinput: in_session("uinput"),
    }
}

pub fn unit_path() -> PathBuf {
    if let Some(directory) = std::env::var_os("KANATA_STUDIO_UNIT_DIR") {
        return PathBuf::from(directory).join("kanata.service");
    }
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".config")
        })
        .join("systemd/user/kanata.service")
}

/// One ExecStart argument: quoted, with systemd's `%` specifiers and `$` expansion disabled.
fn argument(value: &str) -> String {
    let escaped = value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('%', "%%")
        .replace('$', "$$");
    format!("\"{escaped}\"")
}

pub fn unit_text(binary: &str, config: &str) -> String {
    format!(
        "# Angelegt von Kanata Studio.\n\
         [Unit]\n\
         Description=Kanata keyboard remapper\n\
         Documentation=https://github.com/jtroo/kanata\n\n\
         [Service]\n\
         Type=simple\n\
         ExecStart={} -c {}\n\
         Restart=on-failure\n\
         RestartSec=3\n\n\
         [Install]\n\
         WantedBy=default.target\n",
        argument(binary),
        argument(config)
    )
}

fn systemctl(args: &[&str]) -> Result<String, String> {
    let mut all = vec!["--user"];
    all.extend_from_slice(args);
    command("systemctl", &all, Duration::from_secs(15))
}

pub fn service_loaded() -> bool {
    systemctl(&["show", "kanata.service", "--property=LoadState", "--value"])
        .is_ok_and(|state| state == "loaded")
}

pub fn status() -> Value {
    let permissions = permissions();
    json!({
        "ok": true,
        "service": service_loaded(),
        "unitPath": unit_path(),
        "ready": permissions.ready(),
        "inputReadable": permissions.input_readable,
        "uinputWritable": permissions.uinput_writable,
        "relogin": permissions.relogin(),
        "commands": permissions.commands(),
    })
}

/// Writes and enables the user unit; never replaces an existing unit. Starts it only when
/// Kanata already has the device permissions, otherwise it would fail at once.
pub fn create_service(binary: &str, config: &Path) -> Result<Value, String> {
    if binary.is_empty() {
        return Err("Kanata wurde nicht gefunden".into());
    }
    if !config.is_file() {
        return Err("Bitte zuerst eine Konfiguration anlegen oder öffnen".into());
    }
    if service_loaded() {
        return Err("kanata.service ist bereits eingerichtet".into());
    }
    let path = unit_path();
    let config = config.canonicalize().map_err(|e| e.to_string())?;
    let text = unit_text(binary, &config.to_string_lossy());
    fs::create_dir_all(path.parent().ok_or("Ungültiger Dienstpfad")?)
        .map_err(|e| format!("Ordner kann nicht angelegt werden: {e}"))?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|e| format!("kanata.service kann nicht angelegt werden: {e}"))?;
    file.write_all(text.as_bytes())
        .and_then(|_| file.sync_all())
        .map_err(|e| e.to_string())?;
    systemctl(&["daemon-reload"])?;
    systemctl(&["enable", "kanata.service"])?;
    let ready = permissions().ready();
    if ready {
        systemctl(&["start", "kanata.service"])?;
    }
    Ok(json!({"ok": true, "unitPath": path, "started": ready}))
}
