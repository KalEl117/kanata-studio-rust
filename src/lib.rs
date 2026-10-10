pub mod actions;
pub mod editor;
pub mod environment;
pub mod parser;
pub mod plugin;
pub mod protocol;
pub mod runtime;
pub mod session;

use editor::{Editor, Request};
use serde_json::json;
use std::ffi::{CStr, CString, c_char};

pub fn dispatch(editor: &mut Editor, input: &str) -> Result<serde_json::Value, String> {
    let value: serde_json::Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
    match value["op"].as_str() {
        Some("environment") => return Ok(environment::discover()),
        Some("layoutChoice") => {
            environment::save_choice(value["choice"].as_str().ok_or("Layout-Auswahl fehlt")?)?;
            return Ok(json!({"ok":true}));
        }
        Some("keyLabel") => {
            return Ok(
                json!({"ok":true,"label":environment::key_label(value["key"].as_str().unwrap_or(""),value["layout"].as_str().unwrap_or("de"))}),
            );
        }
        Some("view" | "loadView" | "select" | "selectPhysicalKey" | "viewLayout") => {
            if let Some(source) = value["source"].as_str() {
                editor.session.sync(source, value["op"] == "loadView")?;
            }
            if let Some(original) = value["comparisonSource"].as_str() {
                editor.session.set_baseline(original)?;
            }
            if let Some(layout) = value["layout"].as_str() {
                editor.session.layout = layout.into();
            }
            if value["op"] == "select" {
                editor.session.select(
                    value["layer"].as_u64().map(|n| n as usize),
                    value["key"].as_u64().map(|n| n as usize),
                )?;
            }
            if value["op"] == "selectPhysicalKey" {
                editor
                    .session
                    .select_physical(value["name"].as_str().ok_or("Tastenname fehlt")?)?;
            }
            return Ok(json!({"ok":true,"view":editor.session.view()?}));
        }
        _ => {}
    }
    if matches!(value["op"].as_str(), Some("check" | "save" | "worker")) {
        return runtime::dispatch(&value);
    }
    let mut req: Request = serde_json::from_str(input).map_err(|e| e.to_string())?;
    if value["layer"].is_null() {
        req.layer = editor.session.layer;
    }
    if value["key"].is_null() {
        req.key = editor.session.key.unwrap_or(0);
    }
    if matches!(req.op.as_str(), "setAction" | "setTapHold" | "setMacro")
        && editor.session.key.is_none()
        && value["key"].is_null()
        && editor.session.physical_key.is_some()
    {
        req.name = editor
            .session
            .physical_key
            .clone()
            .ok_or("Bitte eine Taste auswählen")?;
    }
    if req.op == "actionDetails" {
        let config = parser::Config::parse(&req.source)?;
        let node = config.action(req.layer, req.key)?;
        return Ok(json!({"ok":true, "details":actions::details(&req.source, node)}));
    }
    if req.op == "parse" {
        let forms = parser::parse(&req.source)?;
        return Ok(json!({"ok": true, "forms": parser::qt_nodes(&req.source, &forms)}));
    }
    let source = editor.execute(&req)?;
    let view = if req.op != "clear" && req.op != "record" {
        editor.session.after_edit(&req.op, &source, req.layer)?;
        Some(editor.session.view()?)
    } else {
        None
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
