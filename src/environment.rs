//! Desktop discovery and display names. No desktop configuration is modified.
use crate::runtime;
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

pub fn preset(value: &str) -> Option<&'static str> {
    let lower = value.trim().to_lowercase();
    let name = lower.split(',').next().unwrap_or("");
    if name == "de" || name.starts_with("german") {
        Some("de")
    } else if matches!(name, "us" | "en_us") || name.starts_with("english (us)") {
        Some("us")
    } else if matches!(name, "gb" | "uk")
        || name.starts_with("english (uk)")
        || name.starts_with("english (british)")
    {
        Some("gb")
    } else {
        None
    }
}

pub fn display_layout(info: &Value) -> &str {
    preset(info["choice"].as_str().unwrap_or(""))
        .unwrap_or_else(|| info["id"].as_str().unwrap_or("de"))
}

fn excluded(name: &str) -> bool {
    let name = normalized_device_name(name);
    [
        "kanata",
        "virtual",
        "fcitx",
        "power button",
        "sleep button",
        "video bus",
        "extra buttons",
        "consumer control",
        "hotkey",
        "pc speaker",
    ]
    .iter()
    .any(|part| name.contains(&normalized_device_name(part)))
}

fn normalized_device_name(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

/// HID capabilities describe possible outputs, not the physical key positions.
/// Only recognizable firmware/model names are classified; a USB vendor alone
/// (in particular ZMK's shared 1d50:615e) does not identify a keyboard model.
pub fn firmware_keyboard(name: &str) -> bool {
    let name = normalized_device_name(name);
    [
        "adv360",
        "advantage360",
        "advantage2",
        "ergodox",
        "moonlander",
        "zsa voyager",
        "glove80",
        "dygma",
        "qmk",
        "zmk",
    ]
    .iter()
    .any(|part| name.contains(&normalized_device_name(part)))
}

pub fn has_bit(bitmap: &str, code: usize) -> bool {
    bitmap
        .split_whitespace()
        .rev()
        .nth(code / 64)
        .and_then(|word| u64::from_str_radix(word, 16).ok())
        .is_some_and(|word| word & (1u64 << (code % 64)) != 0)
}

pub fn devices(contents: &str) -> Vec<Value> {
    let mut result = Vec::new();
    // Linux separates device records with an empty line (occasionally whitespace-only).
    let mut records = Vec::new();
    let mut record = Vec::new();
    for line in contents.lines().chain(std::iter::once("")) {
        if line.trim().is_empty() {
            if !record.is_empty() {
                records.push(std::mem::take(&mut record));
            }
        } else {
            record.push(line);
        }
    }
    for record in records {
        let field = |prefix: &str| {
            record
                .iter()
                .find_map(|line| line.strip_prefix(prefix))
                .unwrap_or("")
        };
        let name = field("N: Name=").trim_matches('"');
        let handlers = field("H: Handlers=");
        let bitmap = field("B: KEY=");
        if name.is_empty()
            || excluded(name)
            || !handlers.split_whitespace().any(|h| h == "kbd")
            || !has_bit(bitmap, 30)
            || !has_bit(bitmap, 44)
        {
            continue;
        }
        let location = format!("{} {}", field("P: Phys="), field("S: Sysfs=")).to_lowercase();
        let usb = field("I: Bus=").starts_with("0003");
        let bluetooth = field("I: Bus=").starts_with("0005");
        let internal = !usb
            && !bluetooth
            && (["i8042", "serio", "platform"]
                .iter()
                .any(|part| location.contains(part))
                || ["internal keyboard", "built-in keyboard"]
                    .iter()
                    .any(|part| name.to_lowercase().contains(part)));
        let firmware = firmware_keyboard(name);
        let mouse = !field("B: REL=").is_empty() && field("B: REL=") != "0";
        let event = handlers
            .split_whitespace()
            .find(|h| h.starts_with("event"))
            .unwrap_or("");
        result.push(json!({"name":name,"type":if internal {"built-in"} else {"external"},
            "usb": usb, "bluetooth":bluetooth, "firmware":firmware, "mouse":mouse,
            "path":format!("/dev/input/{event}"),
            "geometry":if has_bit(bitmap,86) {"iso"} else {"ansi"}, "fnAvailable":has_bit(bitmap,464),
            "fullSize": !internal && !firmware && !mouse
                && [71,79,82,83,96,102,111].iter().all(|code| has_bit(bitmap,*code))}));
    }
    result
}

pub fn layout(hyprland: &str, console: &str, input: &str, machine: &str, choice: &str) -> Value {
    layout_for_config(hyprland, console, input, machine, choice, None)
}

pub fn layout_for_config(
    hyprland: &str,
    console: &str,
    input: &str,
    machine: &str,
    choice: &str,
    config: Option<&str>,
) -> Value {
    let root: Value = serde_json::from_str(hyprland).unwrap_or_default();
    let keyboards = root["keyboards"].as_array().cloned().unwrap_or_default();
    let mut devices = devices(input);
    let forms = config
        .and_then(|text| crate::parser::parse(text).ok())
        .unwrap_or_default();
    let cfg = forms.iter().find(|form| form.head() == "defcfg");
    let option = |key: &str| {
        cfg.and_then(|cfg| {
            cfg.children[1..]
                .as_chunks::<2>()
                .0
                .iter()
                .find(|pair| pair[0].value == key)
                .map(|pair| &pair[1])
        })
    };
    let names = |key: &str| {
        option(key).map(|node| {
            node.children
                .iter()
                .map(|n| n.value.trim_matches('"').to_owned())
                .collect::<Vec<_>>()
        })
    };
    let include = names("linux-dev-names-include");
    let exclude = names("linux-dev-names-exclude").unwrap_or_default();
    let explicit_paths = option("linux-dev").is_some();
    let keyboard_only =
        option("linux-device-detect-mode").is_some_and(|n| n.value == "keyboard-only");
    for device in &mut devices {
        let name = device["name"].as_str().unwrap_or("").to_string();
        // With explicit paths we cannot infer stable symlink targets from /proc.
        let captured = if explicit_paths {
            None
        } else if config.is_some() {
            Some(
                include.as_ref().is_none_or(|names| names.contains(&name))
                    && !exclude.contains(&name)
                    && !(keyboard_only && device["mouse"] == true),
            )
        } else {
            Some(device["firmware"] != true && device["mouse"] != true)
        };
        device["captured"] = json!(captured);
    }
    let candidates: Vec<&Value> = devices
        .iter()
        .filter(|d| d["captured"] != false && d["mouse"] != true)
        .collect();
    let physical: Vec<&Value> = keyboards
        .iter()
        .filter(|k| {
            !excluded(k["name"].as_str().unwrap_or(""))
                && candidates.iter().any(|d| {
                    normalized_device_name(d["name"].as_str().unwrap_or(""))
                        == normalized_device_name(k["name"].as_str().unwrap_or(""))
                })
        })
        .collect();
    let main = physical
        .iter()
        .find(|k| k["main"].as_bool() == Some(true))
        .copied()
        .or_else(|| physical.first().copied());
    let builtin = candidates.iter().copied().find(|d| d["type"] == "built-in");
    let full_size = candidates
        .iter()
        .copied()
        .find(|d| d["fullSize"] == true && d["usb"] == true)
        .or_else(|| candidates.iter().copied().find(|d| d["fullSize"] == true));
    let selected = (if choice == "thinkpad" { builtin } else { None })
        .or(if choice == "fullsize" {
            full_size
        } else {
            None
        })
        .or_else(|| {
            candidates
                .iter()
                .copied()
                .find(|d| d["type"] == "external" && d["usb"] == true)
        })
        .or_else(|| candidates.iter().copied().find(|d| d["type"] == "external"))
        .or(builtin)
        .or_else(|| candidates.first().copied());
    let mut keyboard = selected.cloned().unwrap_or(json!({}));
    if keyboard["type"] == "built-in" {
        keyboard["thinkpad"] = json!(machine.to_lowercase().contains("thinkpad"));
    }
    let selected_hypr = selected.and_then(|d| {
        physical.iter().copied().find(|k| {
            normalized_device_name(k["name"].as_str().unwrap_or(""))
                == normalized_device_name(d["name"].as_str().unwrap_or(""))
        })
    });
    let mut detected = None;
    let mut source = "Standard";
    if let Some(main) = selected_hypr.or(main) {
        let layouts: Vec<&str> = main["layout"].as_str().unwrap_or("").split(',').collect();
        detected = main["active_layout_index"]
            .as_u64()
            .and_then(|index| layouts.get(index as usize))
            .and_then(|name| preset(name))
            .or_else(|| preset(main["active_keymap"].as_str().unwrap_or("")))
            .or_else(|| preset(layouts[0]));
        if detected.is_some() {
            source = "Hyprland";
        }
    }
    if detected.is_none() {
        for line in console.lines() {
            if let Some((name, value)) = line.split_once('=')
                && matches!(name.trim(), "KEYMAP" | "XKBLAYOUT")
            {
                detected = preset(
                    value
                        .trim()
                        .trim_start_matches(['\'', '"'])
                        .split(['\'', '"', '#', ' ', '\t'])
                        .next()
                        .unwrap_or(""),
                );
                if detected.is_some() {
                    source = "System";
                    break;
                }
            }
        }
    }
    json!({"id":detected.unwrap_or("de"),"source":source,"choice":choice,
        "geometry":keyboard["geometry"].as_str().unwrap_or("iso"),"keyboard":keyboard,
        "devices":devices, "available":selected.is_some(), "explicitPaths":explicit_paths})
}

/// Kanata's Windows hook captures every keyboard and Windows names none, so one generic
/// keyboard stands for all of them; its geometry follows the active layout.
pub fn with_hooked_keyboard(layout: &mut Value) {
    if layout["available"] == true {
        return;
    }
    let geometry = if layout["id"] == "us" { "ansi" } else { "iso" };
    let keyboard = json!({"name":"Tastatur","type":"external","usb":false,"bluetooth":false,
        "firmware":false,"mouse":false,"path":"","geometry":geometry,"fnAvailable":false,
        "fullSize":false,"captured":true});
    layout["keyboard"] = keyboard.clone();
    layout["devices"] = json!([keyboard]);
    layout["geometry"] = json!(geometry);
    layout["available"] = json!(true);
}

pub fn theme(contents: &str) -> Value {
    let mut result = json!({"background":"#0b0c16","surface":"#151828","chrome":"#080910",
        "foreground":"#ddf7ff","muted":"#6a6e95","accent":"#82fb9c","green":"#82fb9c","red":"#f68b91"});
    for line in contents.lines() {
        let Some((name, value)) = line.split_once('=') else {
            continue;
        };
        let name = match name.trim() {
            "lighter_background" => "surface",
            "dark_background" => "chrome",
            "dark_foreground" => "muted",
            name => name,
        };
        if !result.as_object().unwrap().contains_key(name) {
            continue;
        }
        let value = value.trim().split('"').nth(1).unwrap_or("");
        if value.len() == 7
            && value.starts_with('#')
            && value[1..].bytes().all(|c| c.is_ascii_hexdigit())
        {
            result[name] = json!(value);
        }
    }
    result
}

fn home() -> PathBuf {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}
/// Per-user settings directory: XDG_CONFIG_HOME, on Windows %APPDATA%, else ~/.config.
pub fn config_home() -> PathBuf {
    let platform = if cfg!(windows) {
        std::env::var_os("APPDATA")
    } else {
        None
    };
    std::env::var_os("XDG_CONFIG_HOME")
        .or(platform)
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join(".config"))
}
/// Where Kanata itself looks for kanata.kbd: ~/.config/kanata, on Windows %APPDATA%\kanata.
pub fn kanata_config_dir() -> PathBuf {
    if cfg!(windows) {
        std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| home().join("AppData/Roaming"))
            .join("kanata")
    } else {
        home().join(".config/kanata")
    }
}
fn settings_path() -> PathBuf {
    config_home().join("kanata-studio-rust/settings.json")
}
pub fn choice_valid(choice: &str) -> bool {
    matches!(
        choice,
        "auto" | "thinkpad" | "fullsize" | "tkl" | "de" | "us" | "gb"
    )
}
pub fn read_choice() -> String {
    let path = settings_path();
    if !path.exists() {
        // Retain the layout preference from the first Qt/Rust prototype.
        let legacy = path
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("kanata-studio/rust.conf");
        if let Ok(contents) = fs::read_to_string(legacy) {
            let mut section = "";
            for line in contents.lines() {
                if line.starts_with('[') {
                    section = line.trim();
                }
                if section == "[keyboard]"
                    && let Some(choice) = line.strip_prefix("layout=")
                    && choice_valid(choice.trim())
                {
                    return choice.trim().into();
                }
            }
        }
    }
    let value: Value = fs::read(path)
        .ok()
        .and_then(|v| serde_json::from_slice(&v).ok())
        .unwrap_or_default();
    value["layout"]
        .as_str()
        .filter(|v| choice_valid(v))
        .unwrap_or("auto")
        .into()
}
pub fn save_choice(choice: &str) -> Result<(), String> {
    if !choice_valid(choice) {
        return Err("Ungültige Layout-Auswahl".into());
    }
    let path = settings_path();
    fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
    let mut settings: Value = fs::read(&path)
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .filter(Value::is_object)
        .unwrap_or(json!({}));
    settings["layout"] = json!(choice);
    runtime::atomic_write(&path, settings.to_string().as_bytes(), 0o600)
}
pub fn discover() -> Value {
    discover_for_config(None)
}
pub fn discover_for_config(config: Option<&str>) -> Value {
    let hyprland = if std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some() {
        runtime::command("hyprctl", &["devices", "-j"], Duration::from_millis(1200))
            .unwrap_or_default()
    } else {
        String::new()
    };
    let read = |path: &Path| fs::read_to_string(path).unwrap_or_default();
    let default_config = crate::protocol::config_path(&json!({}))
        .ok()
        .and_then(|p| fs::read_to_string(p).ok());
    #[cfg(windows)]
    let console = format!(
        "KEYMAP={}",
        crate::windows::keyboard_layout().unwrap_or_default()
    );
    #[cfg(not(windows))]
    let console = read(Path::new("/etc/vconsole.conf"));
    #[allow(unused_mut)]
    let mut layout = layout_for_config(
        &hyprland,
        &console,
        &read(Path::new("/proc/bus/input/devices")),
        &read(Path::new("/sys/class/dmi/id/product_version")),
        &read_choice(),
        config.or(default_config.as_deref()),
    );
    #[cfg(windows)]
    with_hooked_keyboard(&mut layout);
    let palette = read(&home().join(".local/state/omarchy/current/theme/colors.toml"));
    json!({"ok":true,"layout":layout,"theme":theme(&palette)})
}

pub fn key_label(key: &str, layout: &str) -> String {
    let token = key.trim().to_lowercase();
    let label = match token.as_str() {
        "esc" | "escape" => "Escape",
        "caps" | "capslock" => "Feststelltaste",
        "spc" | "space" => "Leertaste",
        "ret" | "enter" => "Eingabe",
        "bspc" | "backspace" => "Rücktaste",
        "tab" => "Tab",
        "lctl" | "lctrl" | "controlleft" => "Strg links",
        "rctl" | "rctrl" | "controlright" => "Strg rechts",
        "lsft" | "lshft" | "lshift" | "shiftleft" => "Umschalt links",
        "rsft" | "rshift" | "shiftright" => "Umschalt rechts",
        "lmeta" | "metaleft" => "Super links",
        "rmeta" | "metaright" => "Super rechts",
        "lalt" | "altleft" => "Alt links",
        "ralt" | "altright" => {
            if layout == "us" {
                "Alt rechts"
            } else {
                "AltGr"
            }
        }
        "semicolon" => {
            if layout == "de" {
                "Ö"
            } else {
                "Semikolon"
            }
        }
        "quote" | "apostrophe" => {
            if layout == "de" {
                "Ä"
            } else {
                "Apostroph"
            }
        }
        "comma" => "Komma",
        "period" | "dot" => "Punkt",
        "backquote" | "grv" => {
            if layout == "de" {
                "Zirkumflex"
            } else {
                "Akzenttaste"
            }
        }
        "minus" => {
            if layout == "de" {
                "ß"
            } else {
                "Minus"
            }
        }
        "slash" => {
            if layout == "de" {
                "Minus"
            } else {
                "Schrägstrich"
            }
        }
        "equal" => "Gleich",
        "leftbracket" | "bracketleft" => {
            if layout == "de" {
                "Ü"
            } else {
                "Linke Klammer"
            }
        }
        "rightbracket" | "bracketright" => {
            if layout == "de" {
                "+"
            } else {
                "Rechte Klammer"
            }
        }
        "backslash" => "Rückstrich",
        "intlbackslash" => {
            if layout == "de" {
                "Kleiner als"
            } else {
                "Zusatz-Taste"
            }
        }
        "lft" | "arrowleft" => "Pfeil links",
        "rght" | "arrowright" => "Pfeil rechts",
        "up" | "arrowup" => "Pfeil hoch",
        "down" | "arrowdown" => "Pfeil runter",
        "pgup" | "pageup" => "Bild hoch",
        "pgdn" | "pagedown" => "Bild runter",
        "del" | "delete" => "Entf",
        "ins" | "insert" => "Einfg",
        "home" => "Pos1",
        "end" => "Ende",
        "ssrq" | "sysrq" | "printscreen" => "Druck",
        "nlck" | "numlock" => "Num Lock",
        "kp/" => "Numblock /",
        "kp*" => "Numblock *",
        "kp-" => "Numblock −",
        "kp+" => "Numblock +",
        "kp." => "Numblock ,",
        "kprt" => "Numblock Eingabe",
        "fn" => "Fn",
        "kana" => "Kana",
        "base" => "Basis",
        "nav-num" => "Navigation / Zahlen",
        "special" => "Sonderfunktionen",
        "mute" => "Lautlos",
        "volu" => "Lauter",
        "vold" => "Leiser",
        "pp" => "Wiedergabe",
        "next" => "Nächster Titel",
        "prev" => "Vorheriger Titel",
        "rpt" => "Letzte Taste",
        "rpt-any" => "Letzte Aktion",
        _ => "",
    };
    if !label.is_empty() {
        return label.into();
    }
    if let Some(number) = token
        .strip_prefix("kp")
        .filter(|n| n.len() == 1 && n.bytes().all(|c| c.is_ascii_digit()))
    {
        return format!("Numblock {number}");
    }
    if let Some(number) = token
        .strip_prefix("digit")
        .filter(|n| n.len() == 1 && n.bytes().all(|c| c.is_ascii_digit()))
    {
        return format!("Ziffer {number}");
    }
    if token
        .strip_prefix('f')
        .and_then(|n| n.parse::<u8>().ok())
        .is_some_and(|n| (1..=24).contains(&n))
        || (token.chars().count() == 1 && token.chars().all(char::is_alphabetic))
    {
        return token.to_uppercase();
    }
    let mut readable = String::new();
    let mut previous = ' ';
    for ch in key.trim().chars() {
        if ch.is_uppercase() && previous.is_lowercase() {
            readable.push(' ');
        }
        readable.push(if ch == '_' || ch == '-' { ' ' } else { ch });
        previous = ch;
    }
    let readable = readable.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut chars = readable.chars();
    chars
        .next()
        .map(|first| format!("{}{}", first.to_uppercase(), chars.as_str()))
        .unwrap_or_default()
}
