use kanata_studio_core::{
    dispatch,
    editor::Editor,
    parser::{Config, MAP_READ_ONLY},
};
use serde_json::{Value, json};

const SOURCE: &str = include_str!("fixtures/layermap.kbd");

fn call(editor: &mut Editor, value: Value) -> Value {
    dispatch(editor, &value.to_string()).unwrap()
}

#[test]
fn map_layers_count_in_file_order_and_resolve_like_kanata() {
    let config = Config::parse(SOURCE).unwrap();
    let names: Vec<_> = (0..config.layers.len())
        .map(|i| config.layer_name(i).unwrap().value.as_str())
        .collect();
    assert_eq!(names, ["base", "nav", "sym"]);
    assert!(!config.is_map(0) && config.is_map(1) && config.is_map(2));
    // defsrc: caps a s min
    let action = |layer, key| config.action(layer, key).unwrap().raw(SOURCE).to_owned();
    assert_eq!(action(1, 1), "left");
    assert_eq!(action(1, 2), "(tap-hold 200 200 s lctl)");
    // `Minus` in the map matches `min` in defsrc.
    assert_eq!(action(1, 3), "grv");
    // Unmapped keys fall back to `_` when present, otherwise they are transparent.
    assert_eq!(action(1, 0), "XX");
    assert_eq!(action(2, 0), "_");
    assert_eq!(action(2, 1), "1");
}

#[test]
fn view_marks_map_layers_read_only_and_shows_inherited_keys() {
    let mut editor = Editor::default();
    call(&mut editor, json!({"op":"loadView","source":SOURCE}));
    let view = &call(&mut editor, json!({"op":"select","layer":2}))["view"];
    assert_eq!(view["layers"], json!(["base", "nav", "sym"]));
    assert_eq!(view["layerEditable"], false);
    assert_eq!(view["actions"], json!(["_", "1", "_", "_"]));
    assert_eq!(view["keyStates"][0]["kind"], "inherited");
    let base = &call(&mut editor, json!({"op":"select","layer":0}))["view"];
    assert_eq!(base["layerEditable"], true);
}

#[test]
fn edits_on_map_layers_are_refused_without_touching_history() {
    let mut editor = Editor::default();
    call(&mut editor, json!({"op":"loadView","source":SOURCE}));
    call(&mut editor, json!({"op":"select","layer":1,"key":1}));
    for request in [
        json!({"op":"setAction","source":SOURCE,"expression":"b"}),
        json!({"op":"setTapHold","source":SOURCE,"head":"tap-hold","repeatMs":150,"holdMs":200,"tap":"a","hold":"lctl"}),
        json!({"op":"setMacro","source":SOURCE,"head":"macro","steps":["a"]}),
        json!({"op":"addLayer","source":SOURCE,"name":"copy","copyFrom":1}),
    ] {
        assert!(dispatch(&mut editor, &request.to_string()).is_err());
    }
    let error = dispatch(
        &mut editor,
        &json!({"op":"setAction","source":SOURCE,"expression":"b"}).to_string(),
    )
    .unwrap_err();
    assert_eq!(error, MAP_READ_ONLY);
    assert!(!editor.can_undo());
}

#[test]
fn new_physical_keys_rename_and_delete_keep_map_layers_valid() {
    let mut editor = Editor::default();
    call(&mut editor, json!({"op":"loadView","source":SOURCE}));
    let added = call(
        &mut editor,
        json!({"op":"addPhysicalKey","source":SOURCE,"name":"F1","expression":"b","layer":0}),
    );
    let added = added["source"].as_str().unwrap();
    assert!(added.contains("(defsrc caps a s min F1)"));
    assert!(added.contains("(deflayer base (layer-while-held nav) a s min b)"));
    assert_eq!(
        Config::parse(added).unwrap().forms[3].raw(added),
        Config::parse(SOURCE).unwrap().forms[3].raw(SOURCE)
    );

    let renamed = call(
        &mut editor,
        json!({"op":"renameLayer","source":SOURCE,"layer":1,"name":"arrows"}),
    );
    let renamed = renamed["source"].as_str().unwrap();
    assert!(renamed.contains("(deflayermap (arrows)"));
    assert!(renamed.contains("(layer-while-held arrows)"));

    assert_eq!(
        dispatch(
            &mut editor,
            &json!({"op":"deleteLayer","source":SOURCE,"layer":1}).to_string()
        )
        .unwrap_err(),
        "Ebene wird noch von einer Aktion verwendet"
    );
    let deleted = call(
        &mut editor,
        json!({"op":"deleteLayer","source":SOURCE,"layer":2}),
    );
    assert!(!deleted["source"].as_str().unwrap().contains("sym"));
}

#[test]
fn configs_with_only_map_layers_open_and_broken_maps_are_reported() {
    let only_maps = "(defsrc a b)\n(deflayermap (base) a b)\n";
    let config = Config::parse(only_maps).unwrap();
    assert_eq!(config.action(0, 0).unwrap().value, "b");
    assert_eq!(config.action(0, 1).unwrap().value, "_");
    assert!(Config::parse("(defsrc a)\n(deflayermap (base) a)\n").is_err());
    assert!(Config::parse("(defsrc a)\n(deflayermap base a b)\n").is_err());
}
