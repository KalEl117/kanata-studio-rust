use kanata_studio_core::session::Session;

const SOURCE: &str = ";; 🦀\n(defsrc a b)\n(deflayer base a (tap-hold 150 200 b lctl))\n(deflayer nav _ (macro C-a 120 Digit1))";
#[test]
fn selection_reuses_parser_and_returns_matching_actions_and_details() {
    let mut session = Session::default();
    session.sync(SOURCE, true).unwrap();
    assert_eq!(session.view().unwrap()["selectedKey"], 0);
    for i in 0..100 {
        session.select(Some(i % 2), Some(i % 2)).unwrap();
        session.view().unwrap();
    }
    let result = session.view().unwrap();
    assert_eq!(result["parses"], 1);
    assert_eq!(result["selectedLayer"], 1);
    assert_eq!(result["details"]["kind"], "macro");
    assert_eq!(result["labels"][1], "Makro");
    session.layout = "us".into();
    assert_eq!(session.view().unwrap()["keyNames"]["a"], "A");
    session.select(Some(0), Some(1)).unwrap();
    assert_eq!(session.view().unwrap()["details"]["tap"], "b");
}
#[test]
fn invalid_source_and_selection_leave_the_previous_view_usable() {
    let mut session = Session::default();
    session.sync(SOURCE, true).unwrap();
    assert!(session.sync("(", false).is_err());
    assert!(session.select(Some(99), None).is_err());
    assert!(session.select(Some(1), Some(99)).is_err());
    assert_eq!(session.view().unwrap()["selectedLayer"], 0);
    assert_eq!(session.view().unwrap()["parses"], 1);
}
#[test]
fn layer_removal_and_file_replacement_reconcile_selection_in_rust() {
    let mut session = Session::default();
    session.sync(SOURCE, true).unwrap();
    session.select(Some(1), Some(1)).unwrap();
    session
        .after_edit("deleteLayer", "(defsrc a)(deflayer base a)", 1)
        .unwrap();
    let view = session.view().unwrap();
    assert_eq!(view["selectedLayer"], 0);
    assert_eq!(view["selectedKey"], -1);
    session.sync(SOURCE, true).unwrap();
    assert_eq!(session.view().unwrap()["selectedKey"], 0);
}

#[test]
fn keyboard_captions_show_wrapped_tap_hold_outputs_and_inherited_actions() {
    let source = "(defsrc caps a)(deflayer base (multi kana (tap-hold 150 200 esc (layer-switch special)) (macro 5)) (tap-hold 150 200 a (multi lctl lalt lshft)))(deflayer special _ _)";
    let mut session = Session::default();
    session.sync(source, true).unwrap();
    let base = session.view().unwrap();
    assert_eq!(base["keyStates"][0]["lines"][0]["label"], "Tippen");
    assert_eq!(base["keyStates"][0]["lines"][0]["text"], "Esc");
    assert_eq!(base["keyStates"][0]["lines"][1]["text"], "special wechseln");
    assert_eq!(
        base["keyStates"][1]["lines"][1]["text"],
        "Strg + Alt + Shift"
    );
    session.select(Some(1), Some(0)).unwrap();
    let inherited = session.view().unwrap();
    assert_eq!(inherited["keyStates"][0]["kind"], "inherited");
    assert_eq!(
        inherited["keyStates"][0]["lines"],
        base["keyStates"][0]["lines"]
    );
}

#[test]
fn held_layer_captions_do_not_look_like_permanent_layer_switches() {
    let mut session = Session::default();
    session.sync("(defsrc caps)(deflayer base (tap-hold 150 200 esc (layer-while-held special)))(deflayer special _)", true).unwrap();
    assert_eq!(
        session.view().unwrap()["keyStates"][0]["lines"][1]["text"],
        "special halten"
    );
}
