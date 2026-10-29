//! Windows: Kanata runs as a hidden process of the user instead of a systemd unit. Autostart
//! is a value under the user's Run key (no admin rights); it starts Kanata through Kanata
//! Studio so no console window appears. Saving and testing restart that process.
//! The command-line helpers are platform independent so they are tested everywhere.

pub const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
/// Run value that starts Kanata at login; its presence corresponds to `kanata.service`.
pub const KANATA_VALUE: &str = "Kanata";
/// Run value that restores an interrupted 30-second test at login.
pub const RECOVER_VALUE: &str = "KanataStudioRecover";

/// Windows file names cannot contain `"`, so plain quoting is unambiguous.
pub fn quote(argument: &str) -> String {
    format!("\"{argument}\"")
}

pub fn autostart_command(launcher: &str, binary: &str, config: &str) -> String {
    format!(
        "{} --start-kanata {} -c {}",
        quote(launcher),
        quote(binary),
        quote(config)
    )
}

/// Splits a Run value into arguments; quotes group, backslashes are literal path separators.
pub fn split_command(line: &str) -> Vec<String> {
    let mut arguments = Vec::new();
    let mut current = String::new();
    let mut quoted = false;
    let mut started = false;
    for ch in line.chars() {
        match ch {
            '"' => {
                quoted = !quoted;
                started = true;
            }
            c if c.is_whitespace() && !quoted => {
                if started {
                    arguments.push(std::mem::take(&mut current));
                    started = false;
                }
            }
            c => {
                current.push(c);
                started = true;
            }
        }
    }
    if started {
        arguments.push(current);
    }
    arguments
}

/// Kanata binary and configuration of a Run value, with or without the Studio launcher.
pub fn kanata_command(line: &str) -> Option<(String, Option<String>)> {
    let arguments = split_command(line);
    let start = arguments
        .iter()
        .position(|a| a == "--start-kanata")
        .map_or(0, |i| i + 1);
    let binary = arguments.get(start)?.clone();
    let mut config = None;
    let mut rest = arguments[start + 1..].iter();
    while let Some(argument) = rest.next() {
        if matches!(argument.as_str(), "-c" | "--cfg" | "--config") {
            config = rest.next().cloned();
            break;
        }
        if let Some(value) = ["-c=", "--cfg=", "--config="]
            .iter()
            .find_map(|prefix| argument.strip_prefix(prefix))
        {
            config = Some(value.into());
            break;
        }
    }
    Some((binary, config))
}

/// Keyboard layout preset of a Windows language identifier (low word of the HKL).
pub fn layout_preset(language: u16) -> Option<&'static str> {
    match (language & 0x3ff, language) {
        (0x07, _) => Some("de"),
        (_, 0x0809) => Some("gb"),
        (0x09, _) => Some("us"),
        _ => None,
    }
}

#[cfg(windows)]
mod sys {
    use std::ffi::c_void;
    pub type Hkey = isize;
    // HKEY_CURRENT_USER is ((HKEY)(ULONG_PTR)((LONG)0x80000001)), i.e. sign-extended.
    pub const HKEY_CURRENT_USER: Hkey = 0x8000_0001_u32 as i32 as isize;
    pub const RRF_RT_REG_SZ: u32 = 0x2;
    pub const REG_SZ: u32 = 1;
    pub const ERROR_FILE_NOT_FOUND: i32 = 2;
    #[link(name = "advapi32")]
    unsafe extern "system" {
        pub fn RegGetValueW(
            key: Hkey,
            subkey: *const u16,
            value: *const u16,
            flags: u32,
            kind: *mut u32,
            data: *mut c_void,
            size: *mut u32,
        ) -> i32;
        pub fn RegSetKeyValueW(
            key: Hkey,
            subkey: *const u16,
            value: *const u16,
            kind: u32,
            data: *const c_void,
            size: u32,
        ) -> i32;
    }
    #[link(name = "user32")]
    unsafe extern "system" {
        pub fn GetForegroundWindow() -> isize;
        pub fn GetWindowThreadProcessId(window: isize, process: *mut u32) -> u32;
        pub fn GetKeyboardLayout(thread: u32) -> isize;
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        pub fn GetTickCount64() -> u64;
    }
}

#[cfg(windows)]
fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

/// A string value under the user's Run key; `None` if it does not exist.
#[cfg(windows)]
pub fn run_value(name: &str) -> Result<Option<String>, String> {
    let (key, value) = (wide(RUN_KEY), wide(name));
    let mut size = 0u32;
    let mut status = unsafe {
        sys::RegGetValueW(
            sys::HKEY_CURRENT_USER,
            key.as_ptr(),
            value.as_ptr(),
            sys::RRF_RT_REG_SZ,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut size,
        )
    };
    if status == sys::ERROR_FILE_NOT_FOUND {
        return Ok(None);
    }
    let mut buffer = vec![0u16; size as usize / 2 + 1];
    if status == 0 {
        size = (buffer.len() * 2) as u32;
        status = unsafe {
            sys::RegGetValueW(
                sys::HKEY_CURRENT_USER,
                key.as_ptr(),
                value.as_ptr(),
                sys::RRF_RT_REG_SZ,
                std::ptr::null_mut(),
                buffer.as_mut_ptr().cast(),
                &mut size,
            )
        };
    }
    if status != 0 {
        return Err(format!(
            "Autostart-Eintrag „{name}“ kann nicht gelesen werden ({status})"
        ));
    }
    let length = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
    Ok(Some(String::from_utf16_lossy(&buffer[..length])))
}

#[cfg(windows)]
pub fn set_run_value(name: &str, command: &str) -> Result<(), String> {
    let (key, value, data) = (wide(RUN_KEY), wide(name), wide(command));
    let status = unsafe {
        sys::RegSetKeyValueW(
            sys::HKEY_CURRENT_USER,
            key.as_ptr(),
            value.as_ptr(),
            sys::REG_SZ,
            data.as_ptr().cast(),
            (data.len() * 2) as u32,
        )
    };
    if status != 0 {
        return Err(format!(
            "Autostart-Eintrag „{name}“ kann nicht geschrieben werden ({status})"
        ));
    }
    Ok(())
}

/// Layout of the foreground window's thread, i.e. what the user is typing with.
#[cfg(windows)]
pub fn keyboard_layout() -> Option<&'static str> {
    let layout = unsafe {
        let thread =
            sys::GetWindowThreadProcessId(sys::GetForegroundWindow(), std::ptr::null_mut());
        sys::GetKeyboardLayout(thread)
    };
    layout_preset((layout & 0xffff) as u16)
}

/// Minutes since the epoch at boot; equal within a minute for the same boot.
#[cfg(windows)]
pub fn boot_id() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    let uptime = unsafe { sys::GetTickCount64() };
    (now.saturating_sub(uptime) / 60_000).to_string()
}

#[cfg(windows)]
fn running(image: &str) -> bool {
    let filter = format!("IMAGENAME eq {image}");
    crate::runtime::command(
        "tasklist",
        &["/FI", &filter, "/NH", "/FO", "CSV"],
        std::time::Duration::from_secs(5),
    )
    .is_ok_and(|output| {
        output
            .to_lowercase()
            .contains(&quote(&image.to_lowercase()))
    })
}

/// Stops running instances of `binary` and starts it hidden with `config`. Kanata exits at
/// once on an invalid configuration, so a process that survives the first second counts.
#[cfg(windows)]
pub fn start_kanata(binary: &str, config: &str) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};
    const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let path = std::path::Path::new(binary);
    let image = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("Ungültiger Kanata-Pfad")?;
    if running(image) {
        let _ = crate::runtime::command("taskkill", &["/F", "/IM", image], Duration::from_secs(10));
        let started = Instant::now();
        while running(image) {
            if started.elapsed() > Duration::from_secs(3) {
                return Err(
                    "Laufendes Kanata konnte nicht beendet werden (läuft es als Administrator?)"
                        .into(),
                );
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    }
    let mut process = Command::new(path);
    process
        .args(["-c", config])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .creation_flags(CREATE_NEW_PROCESS_GROUP | CREATE_NO_WINDOW);
    if let Some(directory) = std::path::Path::new(config).parent() {
        process.current_dir(directory);
    }
    let mut child = process.spawn().map_err(|e| format!("{binary}: {e}"))?;
    let started = Instant::now();
    while started.elapsed() < Duration::from_millis(1200) {
        if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
            return Err(format!("Kanata wurde sofort beendet ({status})"));
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    Ok(())
}

/// Restarts Kanata as the autostart entry describes it.
#[cfg(windows)]
pub fn restart_kanata() -> Result<(), String> {
    let line = run_value(KANATA_VALUE)?.ok_or("Kanata-Autostart ist nicht eingerichtet")?;
    let (binary, config) =
        kanata_command(&line).ok_or("Kanata-Autostart enthält keinen Kanata-Befehl")?;
    start_kanata(
        &binary,
        &config.ok_or("Kanata-Autostart nennt keine Konfiguration")?,
    )
}
