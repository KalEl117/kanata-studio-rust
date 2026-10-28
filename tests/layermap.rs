use kanata_studio_core::{dispatch, editor::Editor, parser::Config};
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
fn view_shows_map_layers_with_inherited_keys() {
    let mut editor = Editor::default();
    call(&mut editor, json!({"op":"loadView","source":SOURCE}));
    let view = &call(&mut editor, json!({"op":"select","layer":2}))["view"];
    assert_eq!(view["layers"], json!(["base", "nav", "sym"]));
    assert_eq!(view["actions"], json!(["_", "1", "_", "_"]));
    assert_eq!(view["keyStates"][0]["kind"], "inherited");
}

fn edit(source: &str, layer: usize, key: usize, request: Value) -> String {
    let mut editor = Editor::default();
    call(&mut editor, json!({"op":"loadView","source":source}));
    call(&mut editor, json!({"op":"select","layer":layer,"key":key}));
    let mut request = request;
    request["source"] = json!(source);
    let result = call(&mut editor, request)["source"]
        .as_str()
        .unwrap()
        .to_owned();
    Config::parse(&result).unwrap();
    result
}

fn set(source: &str, layer: usize, key: usize, expression: &str) -> String {
    edit(
        source,
        layer,
        key,
        json!({"op":"setAction","expression":expression}),
    )
}

#[test]
fn map_edits_patch_explicit_mappings_in_place() {
    let changed = set(SOURCE, 1, 1, "b");
    assert_eq!(changed, SOURCE.replace("  a left\n", "  a b\n"));
    // `min` in defsrc is mapped as `Minus`: no second entry.
    let changed = set(SOURCE, 1, 3, "b");
    assert_eq!(changed, SOURCE.replace("Minus grv", "Minus b"));
    let changed = edit(
        SOURCE,
        1,
        2,
        json!({"op":"setTapHold","head":"tap-hold","repeatMs":150,"holdMs":180,"tap":"s","hold":"lalt"}),
    );
    assert_eq!(
        changed,
        SOURCE.replace("(tap-hold 200 200 s lctl)", "(tap-hold 150 180 s lalt)")
    );
}

#[test]
fn map_edits_add_a_mapping_and_never_touch_the_fallback() {
    // caps falls back to `_ XX` in nav; changing it must not change the other fallback keys.
    let changed = set(SOURCE, 1, 0, "esc");
    assert_eq!(changed, SOURCE.replace("  _ XX)", "  _ XX\n  caps esc\n)"));
    let config = Config::parse(&changed).unwrap();
    assert_eq!(config.action(1, 0).unwrap().value, "esc");
    // Transparent on purpose: needs an explicit `caps _` to beat the fallback.
    let changed = set(SOURCE, 1, 0, "_");
    assert!(changed.contains("  caps _\n)"));
    assert_eq!(
        Config::parse(&changed).unwrap().action(1, 0).unwrap().value,
        "_"
    );
    // The fallback already yields the requested action: nothing to write.
    assert_eq!(set(SOURCE, 1, 0, "XX"), SOURCE);

    // sym has no fallback: unmapped keys are transparent.
    assert_eq!(set(SOURCE, 2, 0, "_"), SOURCE);
    let changed = edit(
        SOURCE,
        2,
        2,
        json!({"op":"setTapHold","head":"tap-hold","repeatMs":150,"holdMs":200,"tap":"s","hold":"lctl"}),
    );
    assert!(changed.ends_with("(deflayermap (sym) a 1 s (tap-hold 150 200 s lctl))\n"));
    let changed = edit(
        SOURCE,
        2,
        3,
        json!({"op":"setMacro","head":"macro","steps":["a","10","b"]}),
    );
    assert!(changed.ends_with("(deflayermap (sym) a 1 min (macro a 10 b))\n"));
}

#[test]
fn map_edits_are_one_undo_step() {
    let mut editor = Editor::default();
    call(&mut editor, json!({"op":"loadView","source":SOURCE}));
    call(&mut editor, json!({"op":"select","layer":1,"key":0}));
    let changed = call(
        &mut editor,
        json!({"op":"setAction","source":SOURCE,"expression":"esc"}),
    );
    let changed = changed["source"].as_str().unwrap();
    assert!(editor.can_undo());
    let undone = call(&mut editor, json!({"op":"undo","source":changed}));
    assert_eq!(undone["source"], SOURCE);
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
    let added = call(
        &mut editor,
        json!({"op":"addPhysicalKey","source":SOURCE,"name":"F1","expression":"b","layer":2}),
    );
    let added = added["source"].as_str().unwrap();
    assert!(added.contains("(deflayer base (layer-while-held nav) a s min _)"));
    assert!(added.ends_with("(deflayermap (sym) a 1 F1 b)\n"));
    assert!(added.contains("  _ XX)"));
    assert_eq!(
        Config::parse(added).unwrap().action(2, 4).unwrap().value,
        "b"
    );

    let copied = call(
        &mut editor,
        json!({"op":"addLayer","source":SOURCE,"name":"copy","copyFrom":1}),
    );
    let copied = copied["source"].as_str().unwrap();
    let config = Config::parse(copied).unwrap();
    assert!(config.is_map(3));
    assert_eq!(config.action(3, 1).unwrap().value, "left");
    assert_eq!(config.action(3, 0).unwrap().value, "XX");

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
