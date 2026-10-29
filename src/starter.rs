//! First-run configuration: every key of the detected keyboard in `defsrc`, all transparent.
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

/// Kanata name for a key of the drawn keyboard (ui/KeyboardModel.js). Every name passes
/// `kanata --check` (v1.12.0) and maps back to the same drawn key in the app.
pub fn kanata_name(physical: &str) -> Option<String> {
    let name = match physical {
        "" | "fn" => return None,
        "Escape" => "esc",
        "Backquote" => "grv",
        "Minus" => "min",
        "Equal" => "eql",
        "Backspace" => "bspc",
        "Tab" => "tab",
        "BracketLeft" => "lbrc",
        "BracketRight" => "rbrc",
        "Enter" => "ret",
        "CapsLock" => "caps",
        "Semicolon" => "scln",
        "Quote" => "apo",
        "Backslash" => "bksl",
        "ShiftLeft" => "lsft",
        "IntlBackslash" => "102d",
        "Comma" => "comm",
        "Period" => ".",
        "Slash" => "/",
        "ShiftRight" => "rsft",
        "ControlLeft" => "lctl",
        "MetaLeft" => "lmeta",
        "AltLeft" => "lalt",
        "Space" => "spc",
        "AltRight" => "ralt",
        "MetaRight" => "rmeta",
        "ControlRight" => "rctl",
        "ContextMenu" => "menu",
        "PrintScreen" => "ssrq",
        "ScrollLock" => "slck",
        "Pause" => "pause",
        "Insert" => "ins",
        "Home" => "home",
        "PageUp" => "pgup",
        "Delete" => "del",
        "End" => "end",
        "PageDown" => "pgdn",
        "ArrowUp" => "up",
        "ArrowLeft" => "lft",
        "ArrowDown" => "down",
        "ArrowRight" => "rght",
        other => return Some(other.to_lowercase()),
    };
    Some(name.into())
}

/// Config text for the drawn keyboard rows; columns of `defsrc` and the base layer line up.
pub fn source(rows: &[Vec<String>]) -> Result<String, String> {
    let mut seen = std::collections::HashSet::new();
    let rows: Vec<Vec<String>> = rows
        .iter()
        .map(|row| {
            row.iter()
                .filter_map(|key| kanata_name(key))
                .filter(|name| seen.insert(crate::editor::normalized(name)))
                .collect::<Vec<_>>()
        })
        .filter(|row| !row.is_empty())
        .collect();
    if rows.is_empty() {
        return Err("Keine Tastatur erkannt".into());
    }
    let line = |row: &[String], text: &dyn Fn(&str) -> String| {
        let cells: Vec<String> = row
            .iter()
            .map(|name| format!("{:<width$}", text(name), width = name.len()))
            .collect();
        format!("  {}", cells.join(" ").trim_end())
    };
    let keys: Vec<String> = rows
        .iter()
        .map(|row| line(row, &|name| name.into()))
        .collect();
    let base: Vec<String> = rows.iter().map(|row| line(row, &|_| "_".into())).collect();
    Ok(format!(
        ";; Angelegt von Kanata Studio. Alle Tasten behalten ihre normale Funktion,\n\
         ;; bis sie in der Basisebene belegt werden (`_` = durchlassen).\n\
         (defcfg)\n\n(defsrc\n{}\n)\n\n(deflayer base\n{}\n)\n",
        keys.join("\n"),
        base.join("\n")
    ))
}

/// Writes a new config; never replaces an existing file. With `binary`, Kanata checks it first.
pub fn create(path: &Path, rows: &[Vec<String>], binary: Option<&str>) -> Result<String, String> {
    if path.exists() {
        return Err("Die Konfigurationsdatei existiert bereits".into());
    }
    let source = source(rows)?;
    crate::parser::Config::parse(&source)?;
    let directory = path.parent().ok_or("Ungültiger Konfigurationspfad")?;
    fs::create_dir_all(directory).map_err(|e| format!("Ordner kann nicht angelegt werden: {e}"))?;
    if let Some(binary) = binary.filter(|b| !b.is_empty()) {
        crate::runtime::check(binary, path, &source)
            .map_err(|e| format!("Kanata lehnt die neue Konfiguration ab: {e}"))?;
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| format!("Konfiguration kann nicht angelegt werden: {e}"))?;
    file.write_all(source.as_bytes())
        .and_then(|_| file.sync_all())
        .map_err(|e| e.to_string())?;
    Ok(source)
}
