use crate::parser::Node;
use serde_json::{Value, json};

pub const MACRO_HEADS: &[&str] = &[
    "macro",
    "macro-release-cancel",
    "macro-cancel-on-press",
    "macro-release-cancel-and-cancel-on-press",
    "macro-repeat",
    "macro-repeat-release-cancel",
    "macro-repeat-cancel-on-press",
    "macro-repeat-release-cancel-and-cancel-on-press",
];

fn modifier(value: &str) -> Option<&str> {
    match value {
        "lmeta" | "rmeta" | "lctl" | "rctl" | "lalt" | "ralt" | "lshft" | "rshift" => Some(value),
        "lctrl" => Some("lctl"),
        "rctrl" => Some("rctl"),
        "lshift" | "lsft" => Some("lshft"),
        "rsft" => Some("rshift"),
        _ => None,
    }
}

pub fn details(source: &str, node: &Node) -> Value {
    if let Some(tap_hold) = node.tap_hold() {
        let hold = &tap_hold.children[4];
        let mut kind = "action";
        let mut value = hold.raw(source);
        let mut modifiers = Vec::new();
        if !hold.list {
            if let Some(normalized) = modifier(&hold.value) {
                kind = "modifier";
                value = normalized;
            }
        } else if hold.head() == "layer-while-held" && hold.children.len() == 2 {
            kind = "layer";
            value = &hold.children[1].value;
        } else if hold.head() == "multi" && hold.children.len() >= 2 {
            let normalized: Option<Vec<&str>> = hold.children[1..]
                .iter()
                .map(|n| if n.list { None } else { modifier(&n.value) })
                .collect();
            if let Some(normalized) = normalized {
                kind = "modifiers";
                modifiers = normalized;
            }
        }
        return json!({"kind":"tap-hold", "head":tap_hold.head(),
            "repeat":tap_hold.children[1].value.parse::<u32>().unwrap_or(0),
            "holdMs":tap_hold.children[2].value.parse::<u32>().unwrap_or(0),
            "tap":tap_hold.children[3].raw(source), "hold":hold.raw(source),
            "holdKind":kind, "holdValue":value, "holdModifiers":modifiers,
            "wrapped":tap_hold.start != node.start || tap_hold.end != node.end});
    }
    if node.list && MACRO_HEADS.contains(&node.head()) {
        return json!({"kind":"macro", "head":node.head(),
            "steps":node.children[1..].iter().map(|n| n.raw(source)).collect::<Vec<_>>()});
    }
    json!({"kind":"simple", "value":node.raw(source)})
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::expression;
    #[test]
    fn recognizes_modifier_aliases_inside_wrapped_tap_hold() {
        let source = "(multi kana (tap-hold 180 200 esc (multi rctrl rsft ralt rmeta)) (macro 5))";
        let value = details(source, &expression(source).unwrap());
        assert_eq!(value["holdKind"], "modifiers");
        assert_eq!(
            value["holdModifiers"],
            json!(["rctl", "rshift", "ralt", "rmeta"])
        );
        assert_eq!(value["hold"], "(multi rctrl rsft ralt rmeta)");
        assert_eq!(value["wrapped"], true);
    }
    #[test]
    fn identifies_layer_holds_and_preserves_unusual_macro_steps() {
        let source = "(tap-hold 150 200 a (layer-while-held nav))";
        assert_eq!(
            details(source, &expression(source).unwrap())["holdValue"],
            "nav"
        );
        let source = "(macro 5 (on-press tap-vkey relkana))";
        assert_eq!(
            details(source, &expression(source).unwrap())["steps"],
            json!(["5", "(on-press tap-vkey relkana)"])
        );
    }
}
