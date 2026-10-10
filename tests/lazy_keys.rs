use kanata_studio_core::{dispatch, editor::Editor};
use serde_json::{Value, json};

const SOURCE: &str =
    "(defcfg)\n(defsrc a b)\n(deflayer base a (tap-hold 150 200 b lctl))\n(deflayer nav _ _)";
fn call(editor: &mut Editor, value: Value) -> Value {
    dispatch(editor, &value.to_string()).unwrap()
}

#[test]
fn selecting_missing_keys_and_layers_does_not_edit_the_document_or_history() {
    let mut editor = Editor::default();
    call(&mut editor, json!({"op":"loadView","source":SOURCE}));
    let selected = call(
        &mut editor,
        json!({"op":"selectPhysicalKey","name":"Slash"}),
    );
    assert_eq!(selected["view"]["selectedKey"], -1);
    assert_eq!(selected["view"]["selectedPhysicalKey"], "Slash");
    assert_eq!(selected["view"]["details"]["kind"], "simple");
    assert_eq!(selected["view"]["selectedKeyState"]["kind"], "normal");
    call(&mut editor, json!({"op":"select","layer":1}));
    for action in ["_", "Slash"] {
        let unchanged = call(
            &mut editor,
            json!({"op":"setAction","source":SOURCE,"expression":action}),
        );
        assert_eq!(unchanged["source"], SOURCE);
        assert!(!editor.can_undo());
    }
    assert_eq!(editor.session.view().unwrap()["selectedLayer"], 1);
}

#[test]
fn first_assignment_is_one_undo_step_and_preserves_selection_through_redo() {
    let mut editor = Editor::default();
    call(&mut editor, json!({"op":"loadView","source":SOURCE}));
    call(
        &mut editor,
        json!({"op":"selectPhysicalKey","name":"Slash"}),
    );
    let changed = call(
        &mut editor,
        json!({"op":"setAction","source":SOURCE,"expression":"Minus"}),
    );
    let next = changed["source"].as_str().unwrap();
    assert_eq!(changed["view"]["keys"][2], "Slash");
    assert_eq!(changed["view"]["selectedKeyState"]["kind"], "own");
    assert_eq!(changed["view"]["actions"][2], "Minus");
    let nav = call(&mut editor, json!({"op":"select","layer":1}));
    assert_eq!(nav["view"]["actions"][2], "_");
    assert_eq!(nav["view"]["selectedKeyState"]["kind"], "inherited");
    let undone = call(&mut editor, json!({"op":"undo","source":next}));
    assert_eq!(undone["source"], SOURCE);
    assert_eq!(undone["view"]["selectedPhysicalKey"], "Slash");
    assert_eq!(undone["view"]["selectedKey"], -1);
    assert!(!editor.can_undo());
    let redone = call(&mut editor, json!({"op":"redo","source":SOURCE}));
    assert_eq!(redone["source"], next);
    assert_eq!(redone["view"]["selectedKey"], 2);
}

#[test]
fn rejected_first_edit_does_not_add_keys_and_tap_hold_and_macro_are_atomic() {
    for edit in [
        json!({"op":"setTapHold","head":"tap-hold","repeatMs":150,"holdMs":200,"tap":"Slash","hold":"lctl"}),
        json!({"op":"setMacro","head":"macro","steps":["Minus","120","Digit1"]}),
    ] {
        let mut editor = Editor::default();
        call(&mut editor, json!({"op":"loadView","source":SOURCE}));
        call(
            &mut editor,
            json!({"op":"selectPhysicalKey","name":"Slash"}),
        );
        assert!(
            dispatch(
                &mut editor,
                &json!({"op":"setAction","source":SOURCE,"expression":"("}).to_string()
            )
            .is_err()
        );
        assert!(
            dispatch(
                &mut editor,
                &json!({"op":"setTapHold","source":SOURCE,"head":"bad","tap":"a","hold":"lctl"})
                    .to_string()
            )
            .is_err()
        );
        assert!(!editor.can_undo());
        assert_eq!(
            editor.session.view().unwrap()["keys"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        let mut request = edit;
        request["source"] = json!(SOURCE);
        let changed = call(&mut editor, request);
        let undone = call(&mut editor, json!({"op":"undo","source":changed["source"]}));
        assert_eq!(undone["source"], SOURCE);
        assert!(!editor.can_undo());
    }
}

#[test]
fn key_states_distinguish_normal_custom_and_transparent_base_actions() {
    let mut editor = Editor::default();
    let base = call(&mut editor, json!({"op":"loadView","source":SOURCE}));
    assert_eq!(base["view"]["keyStates"][0]["kind"], "normal");
    assert_eq!(base["view"]["keyStates"][1]["kind"], "own");
    let nav = call(&mut editor, json!({"op":"select","layer":1,"key":1}));
    assert_eq!(nav["view"]["keyStates"][0]["kind"], "normal");
    assert_eq!(nav["view"]["keyStates"][1]["kind"], "inherited");
    assert_eq!(nav["view"]["selectedKeyState"]["output"], "Tippen / Halten");
    assert_eq!(nav["view"]["selectedKeyState"]["baseLayer"], "base");
}

#[test]
fn physical_aliases_select_existing_keys_and_do_not_create_duplicate_entries() {
    let source = "(defsrc min / lbrc)(deflayer base Minus Slash BracketLeft)";
    let mut editor = Editor::default();
    call(&mut editor, json!({"op":"loadView","source":source}));
    for (index, name) in ["Minus", "Slash", "BracketLeft"].iter().enumerate() {
        let selected = call(&mut editor, json!({"op":"selectPhysicalKey","name":name}));
        assert_eq!(selected["view"]["selectedKey"], index);
        assert_eq!(selected["view"]["selectedKeyState"]["kind"], "normal");
    }
    assert!(!editor.can_undo());
}
