use kanata_studio_core::environment;
use serde_json::json;

fn keyboard(name: &str, location: &str, iso: bool) -> String {
    let mut words = [0u64; 8];
    for code in [30, 44, 464]
        .into_iter()
        .chain(if iso { Some(86) } else { None })
    {
        words[code / 64] |= 1u64 << (code % 64);
    }
    format!(
        "N: Name=\"{name}\"\nP: Phys={location}\nH: Handlers=kbd event0\nB: KEY={}\n\n",
        words
            .iter()
            .rev()
            .map(|w| format!("{w:x}"))
            .collect::<Vec<_>>()
            .join(" ")
    )
}
#[test]
fn layout_uses_active_physical_hyprland_keyboard_and_ignores_virtual_devices() {
    let hypr=json!({"keyboards":[{"name":"kanata","main":true,"layout":"us"},
        {"name":"AT Translated Set 2 keyboard","main":true,"layout":"us,de","active_layout_index":1}]}).to_string();
    let input = keyboard("AT Translated Set 2 keyboard", "i8042/serio0", true);
    let result = environment::layout(&hypr, "KEYMAP=uk", &input, "ThinkPad T14", "auto");
    assert_eq!(result["id"], "de");
    assert_eq!(result["source"], "Hyprland");
    assert_eq!(result["keyboard"]["type"], "built-in");
    assert_eq!(result["geometry"], "iso");
    assert_eq!(result["keyboard"]["fnAvailable"], true);
}
#[test]
fn fallback_prefers_thinkpad_internal_keyboard_and_handles_invalid_hyprland() {
    let input = format!(
        "{}{}{}",
        keyboard("USB keyboard", "usb/1", false),
        keyboard("Internal Keyboard", "platform/1", true),
        keyboard("Kanata virtual", "usb/2", false)
    );
    let result = environment::layout("invalid", "KEYMAP=\"gb\"", &input, "ThinkPad", "thinkpad");
    assert_eq!(result["id"], "gb");
    assert_eq!(result["source"], "System");
    assert_eq!(result["keyboard"]["name"], "Internal Keyboard");
    assert_eq!(environment::devices(&input).len(), 2);
    assert_eq!(
        environment::layout("", "", &input, "Desktop", "auto")["geometry"],
        "ansi"
    );
}
#[test]
fn incomplete_device_records_and_invalid_palettes_are_safe() {
    assert!(environment::devices("N: Name=\"Mouse\"\nH: Handlers=kbd\nB: KEY=bad").is_empty());
    let value = environment::theme(
        "background = \"#f0f0f0\"\nforeground=\"#202020\"\nlighter_background=\"#fafafa\"\naccent=\"javascript:red\"\nunknown=\"#123456\"",
    );
    assert_eq!(value["background"], "#f0f0f0");
    assert_eq!(value["surface"], "#fafafa");
    assert_eq!(value["accent"], "#82fb9c");
    assert!(value["unknown"].is_null());
    assert!(!environment::has_bit("xyz", 999));
}
#[test]
fn labels_follow_layout_and_handle_unicode_and_modifiers() {
    assert_eq!(environment::key_label("Minus", "de"), "ß");
    assert_eq!(environment::key_label("Slash", "de"), "Minus");
    assert_eq!(environment::key_label("Minus", "us"), "Minus");
    assert_eq!(environment::key_label("BracketLeft", "de"), "Ü");
    assert_eq!(environment::key_label("BracketRight", "de"), "+");
    assert_eq!(environment::key_label("semicolon", "de"), "Ö");
    assert_eq!(environment::key_label("semicolon", "us"), "Semikolon");
    assert_eq!(environment::key_label("ArrowUp", "de"), "Pfeil hoch");
    assert_eq!(environment::key_label("Digit1", "de"), "Ziffer 1");
    assert_eq!(environment::key_label("lshft", "de"), "Umschalt links");
    assert_eq!(environment::key_label("größer_Wert", "de"), "Größer Wert");
    assert!(!environment::choice_valid("invalid"));
}

#[test]
fn preferences_migrate_and_persist_without_touching_other_user_settings() {
    use std::{
        fs,
        io::Write,
        os::unix::fs::PermissionsExt,
        process::{Command, Stdio},
    };
    let root = std::env::temp_dir().join(format!("kanata-choice-{}", std::process::id()));
    fs::create_dir_all(root.join("kanata-studio")).unwrap();
    fs::write(
        root.join("kanata-studio/rust.conf"),
        "[keyboard]\nlayout=gb\n",
    )
    .unwrap();
    let call = |input: serde_json::Value| {
        let mut child = Command::new(env!("CARGO_BIN_EXE_kanata-studio-helper"))
            .env("XDG_CONFIG_HOME", &root)
            .env_remove("HYPRLAND_INSTANCE_SIGNATURE")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.to_string().as_bytes())
            .unwrap();
        let output = child.wait_with_output().unwrap();
        (
            output.status.success(),
            serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap(),
        )
    };
    assert_eq!(
        call(json!({"op":"environment"})).1["layout"]["choice"],
        "gb"
    );
    assert!(call(json!({"op":"layoutChoice","choice":"us"})).0);
    assert_eq!(
        call(json!({"op":"environment"})).1["layout"]["choice"],
        "us"
    );
    assert!(!call(json!({"op":"layoutChoice","choice":"invalid"})).0);
    assert_eq!(
        call(json!({"op":"environment"})).1["layout"]["choice"],
        "us"
    );
    assert_eq!(
        fs::metadata(root.join("kanata-studio-rust/settings.json"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    assert_eq!(
        fs::read_to_string(root.join("kanata-studio/rust.conf")).unwrap(),
        "[keyboard]\nlayout=gb\n"
    );
    fs::remove_dir_all(root).unwrap();
}
