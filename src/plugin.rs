//! Stateful popup editor. Drafts survive shell reload; writes use the shared
//! Kanata validation, atomic save and rollback implementation.
use crate::{dispatch, editor::Editor, protocol, runtime};
use serde_json::{Value, json};
use std::{fs, path::PathBuf, time::Duration};

#[derive(Default)]
pub struct PluginEditor {
    editor: Editor,
    path: Option<PathBuf>,
    source: String,
    original: String,
    layout: Value,
    draft_path: Option<PathBuf>,
}

impl PluginEditor {
    fn persist(&self) -> Result<(), String> {
        let Some(path) = &self.draft_path else {
            return Ok(());
        };
        let parent = path.parent().unwrap();
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        runtime::atomic_write(
            path,
            json!({"path":self.path,"source":self.source,"original":self.original})
                .to_string()
                .as_bytes(),
            0o600,
        )
    }

    fn response(&self) -> Result<Value, String> {
        let path = self.path.as_ref().ok_or("Keine Konfiguration geöffnet")?;
        let selected = self.editor.session.view()?;
        let key = selected["selectedKey"].as_i64().unwrap_or(-1);
        let expression = if key >= 0 {
            selected["actions"][key as usize].clone()
        } else {
            json!("_")
        };
        Ok(json!({"ok":true,"document":{
            "overview":protocol::snapshot(&self.source, &self.original, path, &self.layout)?,
            "dirty":self.source != self.original,"canUndo":self.editor.can_undo(),"canRedo":self.editor.can_redo(),
            "selectedLayer":selected["selectedLayer"],"selectedPhysicalKey":selected["selectedPhysicalKey"],
            "details":selected["details"],"expression":expression
        }}))
    }

    fn load(&mut self, input: &Value, discard: bool) -> Result<(), String> {
        let path = protocol::config_path(input)?;
        if !discard && self.path.as_ref() == Some(&path) && self.source != self.original {
            return Ok(());
        }
        let original = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let layout = input
            .get("layoutInfo")
            .cloned()
            .unwrap_or_else(|| crate::environment::discover()["layout"].clone());
        let directory = input["stateDirectory"]
            .as_str()
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
                    .join(".local/state/kanata-studio-plugin/drafts")
            });
        let hash = path
            .to_string_lossy()
            .bytes()
            .fold(0xcbf29ce484222325u64, |hash, byte| {
                (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
            });
        let draft_path = directory.join(format!("{hash:016x}.json"));
        let mut source = original.clone();
        if !discard && draft_path.exists() {
            let saved: Value =
                serde_json::from_slice(&fs::read(&draft_path).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            let draft = saved["source"]
                .as_str()
                .ok_or("Ungültiger gesicherter Entwurf")?;
            if draft != original {
                if saved["original"].as_str() != Some(&original) {
                    return Err("Datei wurde seit dem gesicherten Entwurf extern geändert. Mit „Verwerfen“ die aktuelle Datei laden.".into());
                }
                source = draft.into();
            }
        }
        let mut editor = Editor::default();
        editor.session.layout = layout["id"].as_str().unwrap_or("de").into();
        editor.session.sync(&source, true)?;
        editor.session.set_baseline(&original)?;
        self.editor = editor;
        self.path = Some(path);
        self.source = source;
        self.original = original;
        self.layout = layout;
        self.draft_path = Some(draft_path);
        if discard {
            self.persist()?;
        }
        Ok(())
    }

    pub fn handle(&mut self, input: &Value) -> Result<Value, String> {
        match input["op"].as_str().unwrap_or("") {
            "pluginOpen" => self.load(input, false)?,
            "pluginDiscard" => self.load(input, true)?,
            "pluginSave" => {
                let path = self.path.as_ref().ok_or("Keine Konfiguration geöffnet")?;
                if self.source == self.original {
                    return self.response();
                }
                let home = PathBuf::from(std::env::var_os("HOME").unwrap_or_default());
                for app in ["kanata-studio-rust", "kanata-studio-qt", "kanata-studio"] {
                    let state = home.join(format!(".local/state/{app}/preview/state.json"));
                    if fs::read(state)
                        .ok()
                        .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
                        .is_some_and(|v| matches!(v["phase"].as_str(), Some("pending" | "testing")))
                    {
                        return Err("Bitte zuerst den laufenden Tastaturtest beenden".into());
                    }
                }
                let service = runtime::command(
                    "systemctl",
                    &[
                        "--user",
                        "show",
                        "kanata.service",
                        "--property=ExecStart",
                        "--value",
                    ],
                    Duration::from_secs(5),
                )?;
                let service_path = service_config(&service)
                    .ok_or("Konfigurationspfad von kanata.service ist unbekannt")?;
                if fs::canonicalize(path).map_err(|e| e.to_string())?
                    != fs::canonicalize(service_path).map_err(|e| e.to_string())?
                {
                    return Err("Diese Datei wird von kanata.service nicht verwendet".into());
                }
                let local = home.join(".local/bin/kanata");
                let binary = if local.is_file() {
                    local.to_str().ok_or("Ungültiger Kanata-Pfad")?
                } else {
                    "kanata"
                };
                runtime::check(binary, path, &self.source)?;
                runtime::save(path, self.original.as_bytes(), self.source.as_bytes())?;
                self.original = self.source.clone();
                self.editor = Editor::default();
                self.editor.session.layout = self.layout["id"].as_str().unwrap_or("de").into();
                self.editor.session.sync(&self.source, true)?;
                // A leftover draft with source == disk is recognized as already saved.
                self.persist()?;
            }
            op => {
                let core_op = match op {
                    "pluginSelect" => "selectPhysicalKey",
                    "pluginLayer" => "select",
                    "pluginSetAction" => "setAction",
                    "pluginSetTapHold" => "setTapHold",
                    "pluginUndo" => "undo",
                    "pluginRedo" => "redo",
                    _ => return Err("Unbekannte Plugin-Operation".into()),
                };
                if self.path.is_none() {
                    return Err("Keine Konfiguration geöffnet".into());
                }
                let mut request = input.clone();
                request["op"] = json!(core_op);
                request["source"] = json!(self.source);
                let response = dispatch(&mut self.editor, &request.to_string())?;
                if let Some(source) = response["source"].as_str() {
                    self.source = source.into();
                    self.persist()?;
                }
            }
        }
        self.response()
    }
}

// Parse systemd's argv display conservatively; never evaluate it as shell code.
fn service_config(command: &str) -> Option<String> {
    let mut tokens = Vec::new();
    let mut token = String::new();
    let mut quoted = false;
    for ch in command.chars() {
        if ch == '"' {
            quoted = !quoted;
        } else if ch.is_whitespace() && !quoted {
            if !token.is_empty() {
                tokens.push(std::mem::take(&mut token));
            }
        } else {
            token.push(ch);
        }
    }
    if !token.is_empty() {
        tokens.push(token);
    }
    for pair in tokens.windows(2) {
        if matches!(pair[0].as_str(), "-c" | "--config") {
            return Some(pair[1].clone());
        }
    }
    tokens
        .iter()
        .find_map(|token| token.strip_prefix("--config=").map(str::to_owned))
}
