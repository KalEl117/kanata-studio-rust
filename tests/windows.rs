use kanata_studio_core::windows::{
    autostart_command, kanata_command, layout_preset, split_command,
};

#[test]
fn autostart_command_round_trips_paths_with_spaces() {
    let line = autostart_command(
        r"C:\Program Files\Kanata Studio\kanata-studio-rust.exe",
        r"C:\Users\Jörg Müller\kanata\kanata.exe",
        r"C:\Users\Jörg Müller\AppData\Roaming\kanata\kanata.kbd",
    );
    assert_eq!(
        split_command(&line),
        [
            r"C:\Program Files\Kanata Studio\kanata-studio-rust.exe",
            "--start-kanata",
            r"C:\Users\Jörg Müller\kanata\kanata.exe",
            "-c",
            r"C:\Users\Jörg Müller\AppData\Roaming\kanata\kanata.kbd",
        ]
    );
    assert_eq!(
        kanata_command(&line),
        Some((
            r"C:\Users\Jörg Müller\kanata\kanata.exe".into(),
            Some(r"C:\Users\Jörg Müller\AppData\Roaming\kanata\kanata.kbd".into())
        ))
    );
}

#[test]
fn kanata_command_reads_hand_written_entries() {
    assert_eq!(
        kanata_command(r"C:\kanata\kanata.exe --cfg C:\kanata\a.kbd"),
        Some((
            r"C:\kanata\kanata.exe".into(),
            Some(r"C:\kanata\a.kbd".into())
        ))
    );
    assert_eq!(
        kanata_command(r#""C:\k\kanata.exe" --cfg="C:\my files\b.kbd" -d"#),
        Some((r"C:\k\kanata.exe".into(), Some(r"C:\my files\b.kbd".into())))
    );
    assert_eq!(
        kanata_command(r"C:\k\kanata.exe"),
        Some((r"C:\k\kanata.exe".into(), None))
    );
    assert_eq!(kanata_command("   "), None);
}

#[test]
fn layout_presets_follow_language_ids() {
    assert_eq!(layout_preset(0x0407), Some("de"));
    assert_eq!(layout_preset(0x0807), Some("de"));
    assert_eq!(layout_preset(0x0c07), Some("de"));
    assert_eq!(layout_preset(0x0409), Some("us"));
    assert_eq!(layout_preset(0x0809), Some("gb"));
    assert_eq!(layout_preset(0x040c), None);
}
