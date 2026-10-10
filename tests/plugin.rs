use serde_json::{Value, json};
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

struct Client {
    child: Child,
    input: Option<ChildStdin>,
    output: BufReader<ChildStdout>,
}
impl Client {
    fn start(root: &Path, config: &Path) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_kanata-studio-helper"))
            .arg("--serve")
            .env("HOME", root)
            .env("PATH", root.join("bin"))
            .env("TEST_CONFIG", config)
            .env("TEST_ROOT", root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let input = child.stdin.take();
        let output = BufReader::new(child.stdout.take().unwrap());
        Self {
            child,
            input,
            output,
        }
    }
    fn request(&mut self, value: Value) -> Value {
        writeln!(self.input.as_mut().unwrap(), "{value}").unwrap();
        self.input.as_mut().unwrap().flush().unwrap();
        let mut line = String::new();
        self.output.read_line(&mut line).unwrap();
        serde_json::from_str(&line).unwrap()
    }
}
impl Drop for Client {
    fn drop(&mut self) {
        self.input.take();
        assert!(self.child.wait().unwrap().success());
    }
}
struct Fixture(PathBuf);
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn script(path: &Path, source: &str) {
    fs::write(path, source).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
}

#[test]
fn popup_drafts_survive_restart_and_save_validates_conflicts_and_reload_failure() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let name = format!("kanata-plugin-edit-{}-{unique}", std::process::id());
    let root = Fixture(std::env::temp_dir().join(name));
    fs::create_dir_all(root.0.join("bin")).unwrap();
    let config = root.0.join("keyboard with spaces.kbd");
    let original = "(defsrc caps a)\n(deflayer base (multi (tap-hold 150 200 esc lctl) a) a)\n(deflayer nav _ left)\n";
    fs::write(&config, original).unwrap();
    script(
        &root.0.join("bin/kanata"),
        "#!/bin/sh\nif [ -f \"$TEST_ROOT/check-fail\" ]; then exit 1; fi\nprintf check >> \"$TEST_ROOT/checks\"\n",
    );
    script(
        &root.0.join("bin/systemctl"),
        "#!/bin/sh\ncase \"$*\" in\n*ExecStart*) printf '{ argv[]=/bin/kanata -c \"%s\" ; }\\n' \"$TEST_CONFIG\" ;;\n*reload*) if [ -f \"$TEST_ROOT/reload-fail\" ]; then exit 1; fi; printf reload >> \"$TEST_ROOT/reloads\" ;;\nesac\n",
    );
    let open = json!({"op":"pluginOpen","path":config,"layoutInfo":{"id":"de"}});
    let mut client = Client::start(&root.0, &config);
    assert_eq!(client.request(open.clone())["ok"], true);
    assert_eq!(
        client.request(json!({"op":"pluginSelect","name":"CapsLock"}))["ok"],
        true
    );
    let changed=client.request(json!({"op":"pluginSetTapHold","head":"tap-hold","repeatMs":150,"holdMs":200,"tap":"tab","hold":"lctl"}));
    assert_eq!(changed["document"]["dirty"], true);
    assert_eq!(changed["document"]["details"]["wrapped"], true);
    assert_eq!(fs::read_to_string(&config).unwrap(), original);
    assert_eq!(
        client.request(json!({"op":"pluginUndo"}))["document"]["dirty"],
        false
    );
    assert_eq!(
        client.request(json!({"op":"pluginRedo"}))["document"]["dirty"],
        true
    );
    drop(client);
    let mut client = Client::start(&root.0, &config);
    let restored = client.request(open.clone());
    assert_eq!(restored["document"]["dirty"], true);
    assert_eq!(restored["document"]["details"]["tap"], "tab");
    // External editor changes are never overwritten.
    fs::write(&config, format!("{original}; external\n")).unwrap();
    assert_eq!(client.request(json!({"op":"pluginSave"}))["ok"], false);
    assert!(
        fs::read_to_string(&config)
            .unwrap()
            .ends_with("; external\n")
    );
    fs::write(&config, original).unwrap();
    fs::write(root.0.join("check-fail"), "").unwrap();
    assert_eq!(client.request(json!({"op":"pluginSave"}))["ok"], false);
    assert_eq!(fs::read_to_string(&config).unwrap(), original);
    fs::remove_file(root.0.join("check-fail")).unwrap();
    fs::write(root.0.join("reload-fail"), "").unwrap();
    assert_eq!(client.request(json!({"op":"pluginSave"}))["ok"], false);
    assert_eq!(fs::read_to_string(&config).unwrap(), original);
    fs::remove_file(root.0.join("reload-fail")).unwrap();
    assert_eq!(
        client.request(json!({"op":"pluginSave"}))["document"]["dirty"],
        false
    );
    let saved = fs::read_to_string(&config).unwrap();
    assert!(saved.contains("(multi (tap-hold 150 200 tab lctl) a)"));
    assert!(root.0.join("checks").exists() && root.0.join("reloads").exists());
    // A newly selected, previously unconfigured key is added atomically.
    assert_eq!(
        client.request(json!({"op":"pluginSelect","name":"F1"}))["ok"],
        true
    );
    assert_eq!(
        client.request(json!({"op":"pluginSetAction","expression":"esc"}))["document"]["dirty"],
        true
    );
    assert_eq!(fs::read_to_string(&config).unwrap(), saved);
    let discarded =
        client.request(json!({"op":"pluginDiscard","path":config,"layoutInfo":{"id":"de"}}));
    assert_eq!(discarded["document"]["dirty"], false);
}
