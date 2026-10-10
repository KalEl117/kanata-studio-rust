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

fn excluded(name: &str) -> bool {
    let name = name.to_lowercase();
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
    .any(|part| name.contains(part))
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
        let internal = ["i8042", "serio", "platform"]
            .iter()
            .any(|part| location.contains(part))
            || ["internal keyboard", "built-in keyboard"]
                .iter()
                .any(|part| name.to_lowercase().contains(part));
        result.push(json!({"name":name,"type":if internal {"built-in"} else {"external"},
            "geometry":if has_bit(bitmap,86) {"iso"} else {"ansi"}, "fnAvailable":has_bit(bitmap,464)}));
    }
    result
}

pub fn layout(hyprland: &str, console: &str, input: &str, machine: &str, choice: &str) -> Value {
    let root: Value = serde_json::from_str(hyprland).unwrap_or_default();
    let keyboards = root["keyboards"].as_array().cloned().unwrap_or_default();
    let physical: Vec<&Value> = keyboards
        .iter()
        .filter(|k| !excluded(k["name"].as_str().unwrap_or("")))
        .collect();
    let main = physical
        .iter()
        .find(|k| k["main"].as_bool() == Some(true))
        .copied()
        .or_else(|| physical.first().copied());
    let mut detected = None;
    let mut source = "Standard";
    if let Some(main) = main {
        let layouts: Vec<&str> = main["layout"].as_str().unwrap_or("").split(',').collect();
        let index = main["active_layout_index"].as_u64().unwrap_or(0) as usize;
        detected = preset(layouts.get(index).copied().unwrap_or(layouts[0]))
            .or_else(|| preset(main["active_keymap"].as_str().unwrap_or("")));
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
    let devices = devices(input);
    let main_name = main
        .and_then(|k| k["name"].as_str())
        .unwrap_or("")
        .to_lowercase();
    let matched = devices.iter().find(|device| {
        let name = device["name"].as_str().unwrap_or("").to_lowercase();
        !main_name.is_empty() && (name.contains(&main_name) || main_name.contains(&name))
    });
    let builtin = devices.iter().find(|d| d["type"] == "built-in");
    let external = devices.iter().find(|d| d["type"] == "external");
    let selected = matched
        .or_else(|| {
            if machine.to_lowercase().contains("thinkpad") {
                builtin
            } else {
                None
            }
        })
        .or(external)
        .or_else(|| devices.first());
    let keyboard = selected.cloned().unwrap_or(json!({}));
    json!({"id":detected.unwrap_or("de"),"source":source,"choice":choice,
        "geometry":keyboard["geometry"].as_str().unwrap_or("iso"),"keyboard":keyboard})
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
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}
fn settings_path() -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join(".config"))
        .join("kanata-studio-rust/settings.json")
}
pub fn choice_valid(choice: &str) -> bool {
    matches!(choice, "auto" | "thinkpad" | "de" | "us" | "gb")
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
    runtime::atomic_write(
        &path,
        json!({"layout":choice}).to_string().as_bytes(),
        0o600,
    )
}
pub fn discover() -> Value {
    let hyprland = if std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some() {
        runtime::command("hyprctl", &["devices", "-j"], Duration::from_millis(1200))
            .unwrap_or_default()
    } else {
        String::new()
    };
    let read = |path: &Path| fs::read_to_string(path).unwrap_or_default();
    let layout = layout(
        &hyprland,
        &read(Path::new("/etc/vconsole.conf")),
        &read(Path::new("/proc/bus/input/devices")),
        &read(Path::new("/sys/class/dmi/id/product_version")),
        &read_choice(),
    );
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
