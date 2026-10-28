pub mod actions;
pub mod editor;
pub mod environment;
pub mod parser;
pub mod plugin;
pub mod protocol;
pub mod runtime;
pub mod session;

use base64::{Engine, engine::general_purpose::STANDARD};
use editor::{Edit, Editor, Request};
use serde::Deserialize;
use serde_json::json;
use std::ffi::{CStr, CString, c_char};
use std::path::PathBuf;

/// Optional document context sent with view and selection requests.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ViewContext {
    source: Option<String>,
    comparison_source: Option<String>,
    layout: Option<String>,
}

/// Every request the core understands. On the wire `op` names the variant; fields are camelCase.
#[derive(Deserialize)]
#[serde(tag = "op", rename_all = "camelCase", rename_all_fields = "camelCase")]
enum Command {
    Environment {
        config: Option<String>,
    },
    LayoutChoice {
        choice: String,
    },
    KeyLabel {
        #[serde(default)]
        key: String,
        layout: Option<String>,
    },
    View(ViewContext),
    LoadView(ViewContext),
    ViewLayout(ViewContext),
    Select {
        #[serde(flatten)]
        context: ViewContext,
        layer: Option<usize>,
        key: Option<usize>,
    },
    SelectPhysicalKey {
        #[serde(flatten)]
        context: ViewContext,
        name: String,
    },
    Check {
        binary: String,
        path: PathBuf,
        source: String,
    },
    Save {
        path: PathBuf,
        original: String,
        source: String,
    },
    Worker {
        directory: PathBuf,
        #[serde(default)]
        recover: bool,
    },
    ActionDetails {
        #[serde(default)]
        source: String,
        layer: Option<usize>,
        key: Option<usize>,
    },
    Parse {
        source: String,
    },
    /// Any other `op` is an edit and is parsed as [`Request`].
    #[serde(other)]
    Edit,
}

fn invalid(input: &str, error: serde_json::Error) -> String {
    match serde_json::from_str::<serde_json::Value>(input)
        .ok()
        .and_then(|value| value["op"].as_str().map(str::to_owned))
    {
        Some(op) => format!("Ungültige Anfrage „{op}“: {error}"),
        None => format!("Ungültige Anfrage: {error}"),
    }
}

pub fn dispatch(editor: &mut Editor, input: &str) -> Result<serde_json::Value, String> {
    let session = &mut editor.session;
    let view =
        |session: &mut session::Session, context: ViewContext, reset: bool| -> Result<(), String> {
            if let Some(source) = &context.source {
                session.sync(source, reset)?;
            }
            if let Some(original) = &context.comparison_source {
                session.set_baseline(original)?;
            }
            if let Some(layout) = context.layout {
                session.layout = layout;
            }
            Ok(())
        };
    let command = serde_json::from_str(input).map_err(|e| invalid(input, e))?;
    let mut req = match command {
        Command::Environment { config } => {
            return Ok(environment::discover_for_config(config.as_deref()));
        }
        Command::LayoutChoice { choice } => {
            environment::save_choice(&choice)?;
            return Ok(json!({"ok":true}));
        }
        Command::KeyLabel { key, layout } => {
            let label = environment::key_label(&key, layout.as_deref().unwrap_or("de"));
            return Ok(json!({"ok":true,"label":label}));
        }
        Command::View(context) | Command::ViewLayout(context) => {
            view(session, context, false)?;
            return Ok(json!({"ok":true,"view":session.view()?}));
        }
        Command::LoadView(context) => {
            view(session, context, true)?;
            return Ok(json!({"ok":true,"view":session.view()?}));
        }
        Command::Select {
            context,
            layer,
            key,
        } => {
            view(session, context, false)?;
            session.select(layer, key)?;
            return Ok(json!({"ok":true,"view":session.view()?}));
        }
        Command::SelectPhysicalKey { context, name } => {
            view(session, context, false)?;
            session.select_physical(&name)?;
            return Ok(json!({"ok":true,"view":session.view()?}));
        }
        Command::Check {
            binary,
            path,
            source,
        } => {
            runtime::check(&binary, &path, &source)?;
            return Ok(json!({"ok":true}));
        }
        Command::Save {
            path,
            original,
            source,
        } => {
            let original = STANDARD.decode(original).map_err(|e| e.to_string())?;
            runtime::save(&path, &original, source.as_bytes())?;
            return Ok(json!({"ok":true}));
        }
        Command::Worker { directory, recover } => {
            runtime::worker(&directory, recover)?;
            return Ok(json!({"ok":true}));
        }
        Command::ActionDetails { source, layer, key } => {
            let config = parser::Config::parse(&source)?;
            let node = config.action(
                layer.unwrap_or(session.layer),
                key.unwrap_or(session.key.unwrap_or(0)),
            )?;
            return Ok(json!({"ok":true, "details":actions::details(&source, node)}));
        }
        Command::Parse { source } => {
            let forms = parser::parse(&source)?;
            return Ok(json!({"ok": true, "forms": parser::qt_nodes(&source, &forms)}));
        }
        Command::Edit => serde_json::from_str::<Request>(input).map_err(|e| invalid(input, e))?,
    };
    let explicit_key = req.key.is_some();
    req.layer = req.layer.or(Some(session.layer));
    req.key = req.key.or(Some(session.key.unwrap_or(0)));
    if matches!(
        req.edit,
        Edit::SetAction { .. } | Edit::SetTapHold { .. } | Edit::SetMacro { .. }
    ) && session.key.is_none()
        && !explicit_key
    {
        req.physical = session.physical_key.clone();
    }
    let source = editor.execute(&req)?;
    let view = if matches!(req.edit, Edit::Clear | Edit::Record { .. }) {
        None
    } else {
        let layer = req.layer.unwrap_or(0);
        editor.session.after_edit(&req.edit, &source, layer)?;
        Some(editor.session.view()?)
    };
    Ok(
        json!({"ok": true, "source": source, "view":view, "canUndo": editor.can_undo(), "canRedo": editor.can_redo()}),
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn ks_editor_new() -> *mut Editor {
    Box::into_raw(Box::new(Editor::default()))
}

/// # Safety
/// `editor` must be a live handle returned by ks_editor_new, released exactly once.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ks_editor_free(editor: *mut Editor) {
    if !editor.is_null() {
        unsafe {
            drop(Box::from_raw(editor));
        }
    }
}

/// # Safety
/// Handle must be exclusively borrowed; input must be a valid NUL-terminated C string.
/// Returned string must be released with ks_string_free, never with a C allocator.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ks_editor_request(
    editor: *mut Editor,
    input: *const c_char,
) -> *mut c_char {
    let response = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if editor.is_null() || input.is_null() {
            return Err("Ungültiger Rust-Aufruf".into());
        }
        let input = unsafe { CStr::from_ptr(input) }
            .to_str()
            .map_err(|e| e.to_string())?;
        dispatch(unsafe { &mut *editor }, input)
    }));
    let value = match response {
        Ok(Ok(value)) => value,
        Ok(Err(error)) => json!({"ok": false, "error": error}),
        Err(_) => json!({"ok": false, "error": "Rust-Kern: interner Fehler"}),
    };
    CString::new(value.to_string())
        .expect("JSON escapes NUL")
        .into_raw()
}

/// # Safety
/// String must have been returned by ks_editor_request and not previously freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ks_string_free(value: *mut c_char) {
    if !value.is_null() {
        unsafe {
            drop(CString::from_raw(value));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ffi_reports_errors_and_converts_utf16_offsets() {
        let mut editor = Editor::default();
        let result = dispatch(
            &mut editor,
            &json!({"op":"parse", "source":";; 🦀\n(deflayer größe _)"}).to_string(),
        )
        .unwrap();
        assert_eq!(result["forms"][0]["start"], 6);
        assert_eq!(result["forms"][0]["children"][1]["value"], "größe");
        assert!(dispatch(&mut editor, "invalid JSON").is_err());
        unsafe {
            let handle = ks_editor_new();
            let input = CString::new("{}").unwrap();
            let result = ks_editor_request(handle, input.as_ptr());
            let value: serde_json::Value =
                serde_json::from_str(CStr::from_ptr(result).to_str().unwrap()).unwrap();
            assert_eq!(value["ok"], false);
            ks_string_free(result);
            ks_editor_free(handle);
        }
    }
}
