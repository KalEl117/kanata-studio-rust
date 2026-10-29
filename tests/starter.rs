use kanata_studio_core::{dispatch, editor::Editor, parser::Config, starter};
use serde_json::{Value, json};
use std::path::PathBuf;

/// Rows of ui/KeyboardModel.js for several keyboards (generated with node).
fn rows(kind: &str) -> Vec<Vec<String>> {
    let all: Value = serde_json::from_str(include_str!("fixtures/keyboard-rows.json")).unwrap();
    serde_json::from_value(all[kind].clone()).unwrap()
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("kanata-starter-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

#[test]
fn every_drawn_key_is_in_defsrc_and_transparent() {
    for kind in ["laptop_de", "fullsize_de", "tkl_us", "fullsize_gb"] {
        let rows = rows(kind);
        let source = starter::source(&rows).unwrap();
        let config = Config::parse(&source).unwrap();
        let keys = config.forms[config.src].children.len() - 1;
        let drawn = rows.iter().flatten().filter(|k| k.as_str() != "fn").count();
        assert_eq!(keys, drawn, "{kind}");
        assert_eq!(config.layers.len(), 1);
        assert!((0..keys).all(|key| config.action(0, key).unwrap().value == "_"));
    }
}

#[test]
fn names_are_short_kanata_names() {
    let source = starter::source(&rows("fullsize_de")).unwrap();
    for name in [
        "esc", "caps", "lsft", "102d", "ssrq", "slck", "menu", "kp0", "kprt", "rght",
    ] {
        assert!(source.split_whitespace().any(|word| word == name), "{name}");
    }
    assert!(!source.contains("CapsLock") && !source.contains("fn "));
}

#[test]
fn creates_directory_and_never_replaces_a_file() {
    let dir = scratch("create");
    let path = dir.join("kanata/kanata.kbd");
    let request = json!({"op":"createConfig","path":path,"rows":rows("laptop_de")});
    let created = dispatch(&mut Editor::default(), &request.to_string()).unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), created["source"]);

    std::fs::write(&path, "(defsrc a)\n(deflayer mine b)\n").unwrap();
    let error = dispatch(&mut Editor::default(), &request.to_string()).unwrap_err();
    assert_eq!(error, "Die Konfigurationsdatei existiert bereits");
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        "(defsrc a)\n(deflayer mine b)\n"
    );
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn kanata_rejection_writes_nothing() {
    let dir = scratch("reject");
    let path = dir.join("kanata.kbd");
    let error = starter::create(&path, &rows("tkl_us"), Some("/bin/false")).unwrap_err();
    assert!(
        error.starts_with("Kanata lehnt die neue Konfiguration ab"),
        "{error}"
    );
    assert!(!path.exists());
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn no_keyboard_is_an_error() {
    assert!(starter::source(&[vec!["fn".into(), String::new()]]).is_err());
}
