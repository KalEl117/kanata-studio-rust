use kanata_studio_core::{
    editor::{Editor, Request},
    parser::{self, Config},
};
use serde_json::{Value, json};

const WRAPPED: &str = include_str!("fixtures/wrapped-layout.kbd");

fn request(source: &str, mut value: Value) -> Request {
    value["source"] = json!(source);
    serde_json::from_value(value).unwrap()
}

#[test]
fn parser_preserves_comments_strings_escapes_and_semicolon_key() {
    let source = ";; Anfang 🦀\n#| Kommentar |#\n(defsrc ; a)\n(deflayer größe \"a\\\"b\" \\()";
    let config = Config::parse(source).unwrap();
    assert_eq!(config.forms[config.src].children[1].value, ";");
    assert_eq!(config.action(0, 0).unwrap().raw(source), "\"a\\\"b\"");
    assert_eq!(config.action(0, 1).unwrap().raw(source), "\\(");
    let encoded = serde_json::to_value(parser::qt_nodes(source, &config.forms)).unwrap();
    assert_eq!(
        encoded[0]["start"],
        source[..config.forms[0].start].encode_utf16().count()
    );
}

#[test]
fn parser_rejects_incomplete_and_excessively_nested_input() {
    for input in ["(", ")", "#|", "\"abc", "\"", "\"abc\\\""] {
        assert!(parser::parse(input).is_err(), "{input}");
    }
    assert!(parser::parse(&format!("{}a{}", "(".repeat(129), ")".repeat(129))).is_err());
    assert!(parser::parse(&format!("{}a{}", "(".repeat(128), ")".repeat(128))).is_ok());
}

#[test]
fn config_rejects_duplicate_source_missing_layers_and_misaligned_keys() {
    for input in [
        "(defsrc a)",
        "(defsrc a)(defsrc b)(deflayer base a)",
        "(defsrc (a))(deflayer base a)",
        "(defsrc a b)(deflayer base a)",
    ] {
        assert!(Config::parse(input).is_err());
    }
}

#[test]
fn tap_hold_edit_preserves_wrapper_comments_and_unrelated_keys() {
    let mut editor = Editor::default();
    let changed = editor
        .execute(&request(
            WRAPPED,
            json!({"op":"setTapHold", "head":"tap-hold",
        "repeatMs":180, "holdMs":220, "tap":"esc", "hold":"lctl"}),
        ))
        .unwrap();
    assert_eq!(
        changed,
        WRAPPED.replacen(
            "(tap-hold 180 200 esc (multi rctrl rshift ralt rmeta))",
            "(tap-hold 180 220 esc lctl)",
            1
        )
    );
    assert_eq!(
        editor
            .execute(&request(&changed, json!({"op":"undo"})))
            .unwrap(),
        WRAPPED
    );
    assert_eq!(
        editor
            .execute(&request(WRAPPED, json!({"op":"redo"})))
            .unwrap(),
        changed
    );
}

#[test]
fn rejected_edit_keeps_undo_and_redo_intact() {
    let mut editor = Editor::default();
    let changed = editor
        .execute(&request(
            WRAPPED,
            json!({"op":"setAction", "expression":"_"}),
        ))
        .unwrap();
    editor
        .execute(&request(&changed, json!({"op":"undo"})))
        .unwrap();
    assert!(
        editor
            .execute(&request(
                WRAPPED,
                json!({"op":"setAction", "expression":"("})
            ))
            .is_err()
    );
    assert!(editor.can_redo());
    assert!(!editor.can_undo());
    assert_eq!(
        editor
            .execute(&request(WRAPPED, json!({"op":"redo"})))
            .unwrap(),
        changed
    );
}

#[test]
fn new_edit_after_undo_clears_redo_but_identical_edit_does_not() {
    let source = "(defsrc a)(deflayer base a)";
    let mut editor = Editor::default();
    let changed = editor
        .execute(&request(
            source,
            json!({"op":"setAction", "expression":"b"}),
        ))
        .unwrap();
    editor
        .execute(&request(&changed, json!({"op":"undo"})))
        .unwrap();
    editor
        .execute(&request(
            source,
            json!({"op":"setAction", "expression":"a"}),
        ))
        .unwrap();
    assert!(editor.can_redo());
    editor
        .execute(&request(
            source,
            json!({"op":"setAction", "expression":"c"}),
        ))
        .unwrap();
    assert!(!editor.can_redo());
    assert!(editor.can_undo());
    editor
        .execute(&request(source, json!({"op":"clear"})))
        .unwrap();
    assert!(!editor.can_undo());
}

#[test]
fn physical_key_is_added_to_all_layers_and_alias_duplicates_are_rejected() {
    let source = "(defsrc esc a)\n(deflayer base esc a)\n(deflayer nav _ _)";
    let mut editor = Editor::default();
    assert!(
        editor
            .execute(&request(
                source,
                json!({"op":"addPhysicalKey", "name":"Escape", "expression":"_"})
            ))
            .is_err()
    );
    let result = editor
        .execute(&request(
            source,
            json!({"op":"addPhysicalKey", "name":"F1", "expression":"b", "layer":1}),
        ))
        .unwrap();
    let config = Config::parse(&result).unwrap();
    assert_eq!(config.action(0, 2).unwrap().value, "_");
    assert_eq!(config.action(1, 2).unwrap().value, "b");
    assert_eq!(config.forms[config.src].children[3].value, "F1");
}

#[test]
fn copied_layer_keeps_comments_and_can_be_removed_with_history() {
    let source = "(defsrc a)\n(deflayer base\n ;; behalten\n a\n)";
    let mut editor = Editor::default();
    let copy = editor
        .execute(&request(
            source,
            json!({"op":"addLayer", "name":"Navigation", "copyFrom":0}),
        ))
        .unwrap();
    assert_eq!(copy.matches(";; behalten").count(), 2);
    let deleted = editor
        .execute(&request(&copy, json!({"op":"deleteLayer", "layer":1})))
        .unwrap();
    assert_eq!(Config::parse(&deleted).unwrap().layers.len(), 1);
    assert_eq!(
        editor
            .execute(&request(&deleted, json!({"op":"undo"})))
            .unwrap(),
        copy
    );
    assert!(
        editor
            .execute(&request(source, json!({"op":"deleteLayer"})))
            .is_err()
    );
}

#[test]
fn rename_updates_nested_layer_references_but_not_comments_or_alias_names() {
    let source = ";; nav bleibt\n(defalias nav a)\n(defsrc a)\n(deflayer base (multi (layer-while-held nav) (macro nav)))\n(deflayer nav _)";
    let mut editor = Editor::default();
    let renamed = editor
        .execute(&request(
            source,
            json!({"op":"renameLayer", "layer":1, "name":"größer"}),
        ))
        .unwrap();
    assert!(renamed.contains("(layer-while-held größer)"));
    assert!(renamed.contains("(deflayer größer _)"));
    assert!(renamed.contains(";; nav bleibt"));
    assert!(renamed.contains("(defalias nav a)"));
    assert!(renamed.contains("(macro nav)"));
    assert!(
        editor
            .execute(&request(&renamed, json!({"op":"deleteLayer", "layer":1})))
            .is_err()
    );
    assert!(
        editor
            .execute(&request(
                source,
                json!({"op":"renameLayer", "layer":1, "name":"base"})
            ))
            .is_err()
    );
}

#[test]
fn unicode_offsets_edit_exact_key_without_corrupting_surrounding_text() {
    let source = ";; 🦀 Größe\n(defsrc a b)\n(deflayer größe a b)";
    let mut editor = Editor::default();
    let result = editor
        .execute(&request(
            source,
            json!({"op":"setAction", "key":1, "expression":"(macro C-a 120 Digit1)"}),
        ))
        .unwrap();
    assert_eq!(
        result,
        ";; 🦀 Größe\n(defsrc a b)\n(deflayer größe a (macro C-a 120 Digit1))"
    );
}

#[test]
fn macro_validates_pauses_and_retains_digit_keys_and_numpad_combinations() {
    let mut editor = Editor::default();
    let result = editor
        .execute(&request(
            WRAPPED,
            json!({"op":"setMacro", "head":"macro-repeat",
        "steps":["C-a", "120", "Digit1", "C-kp/", "C-kp*"]}),
        ))
        .unwrap();
    assert!(result.contains("(macro-repeat C-a 120 Digit1 C-kp/ C-kp*)"));
    for step in [
        "0",
        "5001",
        "9999999999999999999999",
        "(multi a b)",
        "a b",
        ";;abc",
    ] {
        assert!(
            editor
                .execute(&request(
                    WRAPPED,
                    json!({"op":"setMacro", "head":"macro", "steps":[step]})
                ))
                .is_err()
        );
    }
}

#[test]
fn config_edits_preserve_unrelated_forms_and_reject_managed_roots() {
    let mut editor = Editor::default();
    let result = editor
        .execute(&request(
            WRAPPED,
            json!({"op":"replaceConfigForm", "formIndex":0,
        "expression":"(defcfg process-unmapped-keys no)"}),
        ))
        .unwrap();
    assert_eq!(
        result,
        WRAPPED.replacen("process-unmapped-keys yes", "process-unmapped-keys no", 1)
    );
    for expression in [
        "(defsrc a)",
        "(deflayer base a)",
        "a",
        "(defalias x a)(defalias y b)",
    ] {
        assert!(
            editor
                .execute(&request(
                    WRAPPED,
                    json!({"op":"addConfigForm", "expression":expression})
                ))
                .is_err()
        );
    }
}
