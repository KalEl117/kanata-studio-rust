use kanata_studio_core::protocol;
use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

const SOURCE: &str = "(defsrc a b)\n(deflayer base a b)\n";

#[test]
fn overview_reads_layers_and_aliases_without_changing_file_or_editor_history() {
    use kanata_studio_core::editor::Editor;
    let path = std::env::temp_dir().join(format!("kanata-overview-{}.kbd", std::process::id()));
    let source = "(defsrc caps Semicolon)\n(deflayer base (tap-hold 150 200 esc lctl) a)\n(deflayer nav _ left)\n";
    std::fs::write(&path, source).unwrap();
    let mut editor = Editor::default();
    let changed = protocol::reply(
        &mut editor,
        &json!({"op":"setAction", "source":SOURCE,"expression":"esc"}).to_string(),
    );
    let snapshot = protocol::reply(
        &mut editor,
        &json!({"id":7,"op":"overview","path":path,
        "layoutInfo":{"id":"de","geometry":"iso"}})
        .to_string(),
    );
    assert_eq!(snapshot["ok"], true);
    assert_eq!(snapshot["id"], 7);
    let views = &snapshot["overview"]["views"];
    assert_eq!(views.as_array().unwrap().len(), 2);
    assert_eq!(views[0]["canonicalKeys"][1], "semicolon");
    assert_eq!(views[0]["keyStates"][0]["lines"][0]["text"], "Esc");
    assert_eq!(views[1]["keyStates"][0]["kind"], "inherited");
    assert_eq!(std::fs::read_to_string(&path).unwrap(), source);
    let undo = protocol::reply(
        &mut editor,
        &json!({"op":"undo","source":changed["source"]}).to_string(),
    );
    assert_eq!(undo["source"], SOURCE);
    std::fs::write(&path, "(defsrc").unwrap();
    assert_eq!(
        protocol::reply(
            &mut editor,
            &json!({"op":"overview","path":path,"layoutInfo":{"id":"de"}}).to_string()
        )["ok"],
        false
    );
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "(defsrc");
    std::fs::remove_file(path).unwrap();
}

#[test]
fn session_retains_history_and_survives_invalid_requests() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_kanata-studio-helper"))
        .arg("--serve")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    let mut output = BufReader::new(child.stdout.take().unwrap());
    let mut request = |value: Value| -> Value {
        writeln!(input, "{value}").unwrap();
        input.flush().unwrap();
        let mut line = String::new();
        output.read_line(&mut line).unwrap();
        serde_json::from_str(&line).unwrap()
    };
    let ping = request(json!({"op":"ping", "id":1}));
    assert_eq!(ping["protocolVersion"], protocol::VERSION);
    assert_eq!(ping["id"], 1);
    assert_eq!(
        request(json!({"op":"loadView", "source":SOURCE}))["ok"],
        true
    );
    let changed = request(json!({"op":"setAction", "source":SOURCE, "expression":"esc", "id":2}));
    assert_eq!(changed["ok"], true);
    assert_eq!(changed["canUndo"], true);
    let source = changed["source"].as_str().unwrap();
    assert!(source.contains("base esc b"));
    assert_eq!(
        request(json!({"op":"does-not-exist", "source":source}))["ok"],
        false
    );
    let undo = request(json!({"op":"undo", "source":source}));
    assert_eq!(undo["source"], SOURCE);
    assert_eq!(undo["canRedo"], true);
    let redo = request(json!({"op":"redo", "source":SOURCE}));
    assert_eq!(redo["source"], source);
    drop(request);
    drop(input);
    assert!(child.wait().unwrap().success());
}

#[test]
fn malformed_json_does_not_end_stream() {
    let mut output = Vec::new();
    protocol::serve(
        &b"broken\n[]\n\n{\"op\":\"ping\",\"id\":\"alive\"}\n"[..],
        &mut output,
    )
    .unwrap();
    let replies: Vec<Value> = String::from_utf8(output)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(replies.len(), 3);
    assert_eq!(replies[0]["ok"], false);
    assert_eq!(replies[1]["ok"], false);
    assert_eq!(replies[2]["id"], "alive");
    assert_eq!(replies[2]["ok"], true);
}

#[test]
fn legacy_one_shot_transport_remains_available() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_kanata-studio-helper"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    write!(
        child.stdin.take().unwrap(),
        "{}",
        json!({"op":"parse", "source":SOURCE})
    )
    .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let reply: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(reply["ok"], true);
    assert!(reply["forms"].is_array());
}
