use crate::{
    actions, environment,
    parser::{Config, Node},
};
use serde_json::{Value, json};

#[derive(Default)]
pub struct Session {
    source: String,
    config: Option<Config>,
    baseline: Option<Config>,
    baseline_source: String,
    pub layer: usize,
    pub key: Option<usize>,
    pub physical_key: Option<String>,
    pub layout: String,
    parses: u64,
}
impl Session {
    pub fn sync(&mut self, source: &str, reset: bool) -> Result<(), String> {
        if self.config.is_none() || source != self.source {
            let config = Config::parse(source)?;
            self.source = source.into();
            self.config = Some(config);
            self.parses += 1;
        }
        let config = self.config.as_ref().unwrap();
        let keys = config.forms[config.src].children.len() - 1;
        if reset {
            self.baseline = self.config.clone();
            self.baseline_source = source.into();
            self.layer = 0;
            self.key = if keys > 0 { Some(0) } else { None };
        }
        if self.layer >= config.layers.len() {
            self.layer = 0;
        }
        if reset {
            self.physical_key = self
                .key
                .map(|key| config.forms[config.src].children[key + 1].value.clone());
        } else if let Some(name) = &self.physical_key {
            self.key = config.forms[config.src].children[1..]
                .iter()
                .position(|node| {
                    crate::editor::normalized(&node.value) == crate::editor::normalized(name)
                });
        } else if self.key.is_some_and(|key| key >= keys) {
            self.key = None;
        }
        Ok(())
    }
    pub fn set_baseline(&mut self, source: &str) -> Result<(), String> {
        if self.baseline.is_none() || self.baseline_source != source {
            let config = if source == self.source {
                self.config.clone().ok_or("Keine Konfiguration geöffnet")?
            } else {
                Config::parse(source)?
            };
            self.baseline = Some(config);
            self.baseline_source = source.into();
        }
        Ok(())
    }
    pub fn select(&mut self, layer: Option<usize>, key: Option<usize>) -> Result<(), String> {
        let config = self.config.as_ref().ok_or("Keine Konfiguration geöffnet")?;
        let next_layer = layer.unwrap_or(self.layer);
        config.layer(next_layer)?;
        if let Some(key) = key {
            config.action(next_layer, key)?;
        }
        self.layer = next_layer;
        if key.is_some() {
            self.key = key;
            self.physical_key =
                key.map(|key| config.forms[config.src].children[key + 1].value.clone());
        }
        Ok(())
    }
    pub fn select_physical(&mut self, name: &str) -> Result<(), String> {
        let config = self.config.as_ref().ok_or("Keine Konfiguration geöffnet")?;
        if name.is_empty()
            || !name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || "_-".contains(c))
        {
            return Err("Ungültiger Tastenname".into());
        }
        self.key = config.forms[config.src].children[1..]
            .iter()
            .position(|node| {
                crate::editor::normalized(&node.value) == crate::editor::normalized(name)
            });
        self.physical_key = Some(name.into());
        Ok(())
    }
    pub fn after_edit(&mut self, op: &str, source: &str, layer: usize) -> Result<(), String> {
        self.sync(source, false)?;
        let config = self.config.as_ref().unwrap();
        match op {
            "addLayer" => self.layer = config.layers.len() - 1,
            "addPhysicalKey" => self.key = Some(config.forms[config.src].children.len() - 2),
            "deleteLayer" => self.layer = layer.min(config.layers.len() - 1),
            _ => {}
        }
        if let Some(key) = self.key {
            self.physical_key = Some(config.forms[config.src].children[key + 1].value.clone());
        }
        Ok(())
    }
    pub fn view(&self) -> Result<Value, String> {
        let config = self.config.as_ref().ok_or("Keine Konfiguration geöffnet")?;
        let layout = if self.layout.is_empty() {
            "de"
        } else {
            &self.layout
        };
        let layer = config.layer(self.layer)?;
        let nodes = &layer.children[2..];
        let config_forms: Vec<Value> = config
            .forms
            .iter()
            .enumerate()
            .filter(|(_, n)| {
                n.list && !n.head().is_empty() && !matches!(n.head(), "defsrc" | "deflayer")
            })
            .map(|(i, n)| json!({"index":i,"name":n.head(),"expression":n.raw(&self.source)}))
            .collect();
        let details = self
            .key
            .map(|key| actions::details(&self.source, &nodes[key]))
            .unwrap_or(if self.physical_key.is_some() {
                json!({"kind":"simple", "value":"_"})
            } else {
                json!({"kind":"none"})
            });
        let states: Vec<Value> = (0..nodes.len())
            .map(|key| {
                let mut state = key_state(config, self.layer, key, layout);
                if let Some(baseline) = &self.baseline {
                    compare_key(
                        &mut state,
                        config,
                        &self.source,
                        baseline,
                        &self.baseline_source,
                        self.layer,
                        key,
                        layout,
                    );
                }
                state
            })
            .collect();
        let selected_state = self.key.map(|key| states[key].clone()).unwrap_or(json!({"kind":"normal", "label":"Normale Funktion", "output": self.physical_key.as_ref().map(|name| environment::key_label(name, layout)).unwrap_or_default(), "detail":"Diese Taste verwendet ihre normale Funktion. Erst eine geänderte Belegung wird in den Entwurf aufgenommen."}));
        Ok(
            json!({"selectedPhysicalKey":self.physical_key.as_deref().unwrap_or(""), "keyStates":states, "selectedKeyState":selected_state, "selectedLayer":self.layer,"selectedKey":self.key.map(|k|k as i64).unwrap_or(-1),
            "keys":config.forms[config.src].children[1..].iter().map(|n|n.value.as_str()).collect::<Vec<_>>(),
            "keyNames":config.forms[config.src].children[1..].iter().map(|n|(n.value.as_str(),environment::key_label(&n.value,layout))).collect::<std::collections::BTreeMap<_,_>>(),
            "layers":config.layers.iter().map(|&i|config.forms[i].children[1].value.as_str()).collect::<Vec<_>>(),
            "actions":nodes.iter().map(|n|n.raw(&self.source)).collect::<Vec<_>>(),
            "labels":nodes.iter().map(|n|summary(n,layout)).collect::<Vec<_>>(),
            "details":details,"configForms":config_forms,"parses":self.parses}),
        )
    }
}
fn summary(node: &Node, layout: &str) -> String {
    if !node.list {
        return if node.value == "_" {
            String::new()
        } else {
            environment::key_label(&node.value, layout)
        };
    }
    let head = node.head();
    if head.starts_with("tap-hold") {
        return "Tippen / Halten".into();
    }
    if head.starts_with("macro") {
        return "Makro".into();
    }
    match head {
        "" => "Aktion".into(),
        "layer-while-held" => "Ebene halten".into(),
        "layer-switch" => "Ebene wechseln".into(),
        "caps-word" | "caps-word-toggle" => "Wort großschreiben".into(),
        "one-shot" => "Einmalige Zusatztaste".into(),
        "release-key" => "Taste lösen".into(),
        "multi" => {
            if let Some(child) = node.children[1..].iter().find(|n| {
                n.list && (n.head().starts_with("tap-hold") || n.head().starts_with("macro"))
            }) {
                return summary(child, layout);
            }
            node.children[1..]
                .iter()
                .map(|n| summary(n, layout))
                .collect::<Vec<_>>()
                .join(" + ")
        }
        _ => "Spezialaktion".into(),
    }
}

fn compact_action(node: &Node, layout: &str) -> String {
    if !node.list {
        let key = crate::editor::normalized(&node.value);
        return match key.as_str() {
            "esc" => "Esc".into(),
            "ret" => "Enter".into(),
            "spc" => "Leertaste".into(),
            "lctl" | "lctrl" => "Strg".into(),
            "rctl" | "rctrl" => "Strg rechts".into(),
            "lshft" | "lsft" | "lshift" => "Shift".into(),
            "rshift" | "rsft" => "Shift rechts".into(),
            "lalt" => "Alt".into(),
            "ralt" => "AltGr".into(),
            "lmeta" => "Super".into(),
            "rmeta" => "Super rechts".into(),
            "_" | "use-defsrc" => "Normal".into(),
            _ => environment::key_label(&node.value, layout),
        };
    }
    if let Some(name) = node.children.get(1) {
        let suffix = match node.head() {
            "layer-while-held" | "layer-toggle" => Some("halten"),
            "layer-switch" => Some("wechseln"),
            _ => None,
        };
        if let Some(suffix) = suffix {
            return format!("{} {suffix}", name.value);
        }
    }
    if node.head() == "multi" {
        return node.children[1..]
            .iter()
            .map(|n| compact_action(n, layout))
            .collect::<Vec<_>>()
            .join(" + ");
    }
    summary(node, layout)
}
fn action_lines(node: &Node, layout: &str) -> Vec<Value> {
    if let Some(tap_hold) = node.tap_hold() {
        return vec![
            json!({"label":"Tippen", "text":compact_action(&tap_hold.children[3], layout)}),
            json!({"label":"Halten", "text":compact_action(&tap_hold.children[4], layout)}),
        ];
    }
    Vec::new()
}

fn key_state(config: &Config, layer: usize, key: usize, layout: &str) -> Value {
    let input = &config.forms[config.src].children[key + 1].value;
    let action = config.action(layer, key).unwrap();
    let normal = |node: &Node| {
        !node.list
            && (node.value == "_"
                || node.value == "use-defsrc"
                || crate::editor::normalized(&node.value) == crate::editor::normalized(input))
    };
    if action.value == "_" && layer > 0 {
        let base = config.action(0, key).unwrap();
        if !normal(base) {
            let base_name = &config.layer(0).unwrap().children[1].value;
            return json!({"kind":"inherited", "label":"Von Basis übernommen", "output":summary(base, layout), "lines":action_lines(base, layout), "detail":format!("Durchlassen: übernimmt bei aktiver Basis „{base_name}“ deren Belegung. Bei einer anderen aktiven Basis gilt deren Belegung."), "baseLayer":base_name});
        }
    }
    if normal(action) {
        json!({"kind":"normal", "label":"Normale Funktion", "output":environment::key_label(input, layout), "detail":"Diese Taste verwendet ihre normale Funktion."})
    } else {
        json!({"kind":"own", "label":"Eigene Belegung", "output":summary(action, layout), "lines":action_lines(action, layout), "detail":"Diese Belegung ist direkt in der ausgewählten Ebene festgelegt."})
    }
}

fn same_action(a: &Node, b: &Node) -> bool {
    a.list == b.list
        && a.value == b.value
        && a.children.len() == b.children.len()
        && a.children
            .iter()
            .zip(&b.children)
            .all(|(a, b)| same_action(a, b))
}
#[allow(clippy::too_many_arguments)]
fn compare_key(
    state: &mut Value,
    config: &Config,
    source: &str,
    baseline: &Config,
    original: &str,
    layer: usize,
    key: usize,
    layout: &str,
) {
    let input = &config.forms[config.src].children[key + 1].value;
    let name = &config.layer(layer).unwrap().children[1].value;
    let old_key = baseline.forms[baseline.src].children[1..]
        .iter()
        .position(|n| crate::editor::normalized(&n.value) == crate::editor::normalized(input));
    let old_layer = baseline
        .layers
        .iter()
        .position(|&i| baseline.forms[i].children[1].value == *name);
    let action = config.action(layer, key).unwrap();
    let mut before = json!({"kind":"normal", "label":"Normale Funktion", "output":environment::key_label(input, layout), "expression":"_"});
    let mut inherited_change = false;
    let changed = match (old_layer, old_key) {
        (Some(old_layer), Some(old_key)) => {
            let old_action = baseline.action(old_layer, old_key).unwrap();
            before = key_state(baseline, old_layer, old_key, layout);
            let old_effective = if before["kind"] == "inherited" {
                baseline.action(0, old_key).unwrap()
            } else {
                old_action
            };
            before["expression"] = json!(old_effective.raw(original));
            if action.value == "_" && old_action.value == "_" && layer > 0 && old_layer > 0 {
                inherited_change = !same_action(
                    config.action(0, key).unwrap(),
                    baseline.action(0, old_key).unwrap(),
                );
            }
            !same_action(action, old_action) || inherited_change
        }
        (None, _) => {
            before["output"] = json!("Ebene noch nicht gespeichert");
            true
        }
        (_, None) => true,
    };
    state["changed"] = json!(changed);
    if changed {
        let kind = if old_layer.is_none() {
            "newLayer"
        } else if old_key.is_none() {
            "added"
        } else if inherited_change {
            "inherited"
        } else {
            "edited"
        };
        let mut after = state.clone();
        let effective = if state["kind"] == "inherited" {
            config.action(0, key).unwrap()
        } else {
            action
        };
        after["expression"] = json!(effective.raw(source));
        state["changeKind"] = json!(kind);
        state["before"] = before;
        state["after"] = after;
    }
}
