//! Versioned JSON-lines transport for shell clients. One editor per connection.
use crate::{dispatch, editor::Editor};
use serde_json::{Value, json};
use std::io::{self, BufRead, Write};
use std::{fs, path::PathBuf};

pub const VERSION: u32 = 1;

/// Read-only snapshot, independent of the editor connection and its history.
pub(crate) fn config_path(value: &Value) -> Result<PathBuf, String> {
    let path = if let Some(path) = value["path"].as_str().filter(|p| !p.is_empty()) {
        PathBuf::from(path)
    } else {
        let directory =
            PathBuf::from(std::env::var_os("HOME").ok_or("HOME fehlt")?).join(".config/kanata");
        let primary = directory.join("kanata.kbd");
        if primary.exists() {
            primary
        } else {
            directory.join("config.kbd")
        }
    };
    Ok(path)
}

fn overview(value: &Value) -> Result<Value, String> {
    let path = config_path(value)?;
    let source =
        fs::read_to_string(&path).map_err(|error| format!("{}: {error}", path.display()))?;
    let layout_info = value.get("layoutInfo").cloned().unwrap_or_else(|| {
        crate::environment::discover_for_config(Some(&source))["layout"].clone()
    });
    Ok(json!({"ok":true,"overview":snapshot(&source, &source, &path, &layout_info)?}))
}

pub(crate) fn snapshot(
    source: &str,
    baseline: &str,
    path: &std::path::Path,
    layout_info: &Value,
) -> Result<Value, String> {
    let mut session = crate::session::Session::default();
    session.layout = crate::environment::display_layout(layout_info).into();
    session.sync(source, true)?;
    session.set_baseline(baseline)?;
    let initial = session.view()?;
    let count = initial["layers"].as_array().ok_or("Ebenen fehlen")?.len();
    let mut views = Vec::with_capacity(count);
    for layer in 0..count {
        session.select(Some(layer), None)?;
        let mut view = session.view()?;
        view["canonicalKeys"] = json!(
            view["keys"]
                .as_array()
                .unwrap()
                .iter()
                .map(|key| crate::editor::normalized(key.as_str().unwrap()))
                .collect::<Vec<_>>()
        );
        views.push(view);
    }
    Ok(json!({"path":path,"layoutInfo":layout_info,"views":views}))
}

pub fn reply(editor: &mut Editor, input: &str) -> Value {
    let parsed = serde_json::from_str::<Value>(input);
    let id = parsed
        .as_ref()
        .ok()
        .and_then(|value| value.get("id"))
        .cloned();
    let result = match parsed {
        Ok(value) if !value.is_object() => Err("Anfrage muss ein JSON-Objekt sein".into()),
        Ok(value) if value["op"] == "ping" => Ok(json!({
            "ok": true, "protocolVersion": VERSION, "backend": "kanata-studio-rust"
        })),
        Ok(value) if value["op"] == "overview" => overview(&value),
        Ok(_) => dispatch(editor, input),
        Err(error) => Err(error.to_string()),
    };
    let mut response = result.unwrap_or_else(|error| json!({"ok": false, "error": error}));
    if let Some(id) = id {
        response["id"] = id;
    }
    response
}

/// Flush each reply before reading again: callers need not close stdin.
/// Bad requests return errors without ending the session or losing history.
pub fn serve(reader: impl BufRead, mut writer: impl Write) -> io::Result<()> {
    let mut editor = Editor::default();
    let mut plugin = crate::plugin::PluginEditor::default();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let value = serde_json::from_str::<Value>(&line).ok();
        let response = if value
            .as_ref()
            .and_then(|v| v["op"].as_str())
            .is_some_and(|op| op.starts_with("plugin"))
        {
            let value = value.unwrap();
            let mut response = plugin
                .handle(&value)
                .unwrap_or_else(|error| json!({"ok":false,"error":error}));
            if let Some(id) = value.get("id") {
                response["id"] = id.clone();
            }
            response
        } else {
            reply(&mut editor, &line)
        };
        serde_json::to_writer(&mut writer, &response)?;
        writer.write_all(b"\n")?;
        writer.flush()?;
    }
    Ok(())
}
