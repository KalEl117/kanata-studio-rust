use kanata_studio_core::{dispatch, editor::Editor};
use serde_json::{Value, json};

const SOURCE: &str = "(defsrc caps a)(deflayer base (tap-hold 150 200 esc (layer-while-held special)) a)(deflayer special _ _)";
fn call(editor: &mut Editor, input: Value) -> Value {
    dispatch(editor, &input.to_string()).unwrap()
}

#[test]
fn comparison_tracks_tap_hold_changes_undo_redo_and_the_saved_baseline() {
    let mut editor = Editor::default();
    let loaded = call(&mut editor, json!({"op":"loadView", "source":SOURCE}));
    assert_eq!(loaded["view"]["keyStates"][0]["changed"], false);
    let edited = call(
        &mut editor,
        json!({"op":"setTapHold", "source":SOURCE, "head":"tap-hold", "repeatMs":150, "holdMs":200, "tap":"esc", "hold":"(layer-switch special)"}),
    );
    let state = &edited["view"]["keyStates"][0];
    assert_eq!(state["changed"], true);
    assert_eq!(state["before"]["lines"][1]["text"], "special halten");
    assert_eq!(state["after"]["lines"][1]["text"], "special wechseln");
    assert_eq!(edited["view"]["keyStates"][1]["changed"], false);
    let next = edited["source"].as_str().unwrap();
    let undone = call(&mut editor, json!({"op":"undo", "source":next}));
    assert_eq!(undone["view"]["keyStates"][0]["changed"], false);
    let redone = call(&mut editor, json!({"op":"redo", "source":SOURCE}));
    assert_eq!(redone["view"]["keyStates"][0]["changed"], true);
    let saved = call(
        &mut editor,
        json!({"op":"view", "source":next, "comparisonSource":next}),
    );
    assert_eq!(saved["view"]["keyStates"][0]["changed"], false);
}

#[test]
fn inherited_changes_show_the_old_and_new_base_output() {
    let mut editor = Editor::default();
    call(&mut editor, json!({"op":"loadView", "source":SOURCE}));
    call(
        &mut editor,
        json!({"op":"setAction", "source":SOURCE, "expression":"Minus"}),
    );
    let selected = call(&mut editor, json!({"op":"select", "layer":1}));
    let state = &selected["view"]["keyStates"][0];
    assert_eq!(state["changed"], true);
    assert_eq!(state["changeKind"], "inherited");
    assert_eq!(state["before"]["lines"][0]["text"], "Esc");
    assert_eq!(state["after"]["output"], "ß");
    assert_eq!(state["after"]["expression"], "Minus");
}

#[test]
fn added_keys_start_with_the_physical_output_in_the_before_comparison() {
    let mut editor = Editor::default();
    call(&mut editor, json!({"op":"loadView", "source":SOURCE}));
    call(
        &mut editor,
        json!({"op":"selectPhysicalKey", "name":"Slash"}),
    );
    let result = call(
        &mut editor,
        json!({"op":"setAction", "source":SOURCE, "expression":"Minus"}),
    );
    let state = &result["view"]["keyStates"][2];
    assert_eq!(state["changed"], true);
    assert_eq!(state["changeKind"], "added");
    assert_eq!(state["before"]["output"], "Minus");
    assert_eq!(state["after"]["output"], "ß");
}

#[test]
fn comments_whitespace_and_layer_order_do_not_mark_unchanged_actions() {
    let mut editor = Editor::default();
    call(&mut editor, json!({"op":"loadView", "source":SOURCE}));
    let reformatted = ";; comment\n(defsrc caps a)\n(deflayer special _ _)\n(deflayer base\n (tap-hold 150 200 esc (layer-while-held special))\n a)";
    let result = call(&mut editor, json!({"op":"view", "source":reformatted}));
    assert!(
        result["view"]["keyStates"]
            .as_array()
            .unwrap()
            .iter()
            .all(|state| state["changed"] == false)
    );
    call(&mut editor, json!({"op":"select", "layer":1}));
    let base = call(&mut editor, json!({"op":"view"}));
    assert!(
        base["view"]["keyStates"]
            .as_array()
            .unwrap()
            .iter()
            .all(|state| state["changed"] == false)
    );
}

#[test]
fn new_layers_and_timing_only_edits_have_an_exact_comparison() {
    let mut editor = Editor::default();
    call(&mut editor, json!({"op":"loadView", "source":SOURCE}));
    let timed = SOURCE.replace("150 200", "150 300");
    let result = call(&mut editor, json!({"op":"view", "source":timed}));
    let state = &result["view"]["keyStates"][0];
    assert_eq!(state["changed"], true);
    assert_ne!(state["before"]["expression"], state["after"]["expression"]);
    let result = call(
        &mut editor,
        json!({"op":"addLayer", "source":timed, "name":"new"}),
    );
    assert_eq!(result["view"]["keyStates"][0]["changeKind"], "newLayer");
}
