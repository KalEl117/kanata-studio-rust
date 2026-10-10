use crate::parser::{self, Config, Node};
use serde::Deserialize;

#[derive(Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Request {
    pub op: String,
    pub source: String,
    pub previous: String,
    pub layer: usize,
    pub key: usize,
    pub name: String,
    pub expression: String,
    pub head: String,
    pub repeat_ms: u32,
    pub hold_ms: u32,
    pub tap: String,
    pub hold: String,
    pub steps: Vec<String>,
    pub copy_from: Option<usize>,
    pub form_index: usize,
}

#[derive(Default)]
pub struct Editor {
    undo: Vec<String>,
    redo: Vec<String>,
    pub session: crate::session::Session,
}

impl Editor {
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub fn execute(&mut self, req: &Request) -> Result<String, String> {
        match req.op.as_str() {
            "clear" => {
                self.undo.clear();
                self.redo.clear();
                return Ok(req.source.clone());
            }
            "record" => {
                if req.previous != req.source {
                    self.undo.push(req.previous.clone());
                    self.redo.clear();
                }
                return Ok(req.source.clone());
            }
            "undo" | "redo" => {
                let stack = if req.op == "undo" {
                    &mut self.undo
                } else {
                    &mut self.redo
                };
                let source = stack.last().ok_or("Keine Änderung im Verlauf")?;
                Config::parse(source)?;
                let source = stack.pop().unwrap();
                if req.op == "undo" {
                    self.redo.push(req.source.clone());
                } else {
                    self.undo.push(req.source.clone());
                }
                return Ok(source);
            }
            _ => {}
        }
        let config = Config::parse(&req.source)?;
        let source = if !req.name.is_empty()
            && matches!(req.op.as_str(), "setAction" | "setTapHold" | "setMacro")
        {
            if req.op == "setAction"
                && (req.expression.trim() == "_"
                    || normalized(req.expression.trim()) == normalized(&req.name))
            {
                return Ok(req.source.clone());
            }
            let staged = edit(
                &Request {
                    op: "addPhysicalKey".into(),
                    expression: "_".into(),
                    ..req.clone()
                },
                &config,
            )?;
            let staged_config = Config::parse(&staged)?;
            edit(
                &Request {
                    source: staged.clone(),
                    key: staged_config.forms[staged_config.src].children.len() - 2,
                    ..req.clone()
                },
                &staged_config,
            )?
        } else {
            edit(req, &config)?
        };
        Config::parse(&source)?;
        if source != req.source {
            self.undo.push(req.source.clone());
            self.redo.clear();
        }
        Ok(source)
    }
}

fn patch(source: &str, node: &Node, value: &str) -> String {
    let mut result = source.to_owned();
    result.replace_range(node.start..node.end, value);
    result
}

fn editable(node: &Node) -> bool {
    node.list && !node.head().is_empty() && !matches!(node.head(), "defsrc" | "deflayer")
}

fn valid_name(config: &Config, name: &str, except: Option<usize>) -> Result<(), String> {
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return Err("Ungültiger Ebenenname".into());
    }
    for (index, form) in config.forms.iter().enumerate() {
        let existing = match form.head() {
            "deflayer" => form.children.get(1).map(|n| n.value.as_str()),
            "deflayermap" => form
                .children
                .get(1)
                .and_then(|n| n.children.first())
                .map(|n| n.value.as_str()),
            _ => None,
        };
        if Some(index) != except && existing == Some(name) {
            return Err("Ebene existiert bereits".into());
        }
    }
    Ok(())
}

pub(crate) fn normalized(key: &str) -> String {
    let lower = key.to_lowercase();
    match lower.as_str() {
        "min" | "-" => "minus",
        "eql" | "=" => "equal",
        "lbrc" | "[" => "leftbracket",
        "rbrc" | "]" => "rightbracket",
        "scln" | ";" => "semicolon",
        "apo" | "apos" | "'" => "apostrophe",
        "comm" | "," => "comma",
        "." => "dot",
        "/" => "slash",
        "bksl" | "\\" => "backslash",
        "102d" | "lsgt" | "nubs" | "<" => "intlbackslash",
        "escape" => "esc",
        "backquote" => "grv",
        "backspace" => "bspc",
        "enter" => "ret",
        "space" => "spc",
        "capslock" => "caps",
        "shiftleft" => "lsft",
        "shiftright" => "rsft",
        "controlleft" => "lctl",
        "controlright" => "rctl",
        "metaleft" => "lmeta",
        "metaright" => "rmeta",
        "altleft" => "lalt",
        "altright" => "ralt",
        "period" => "dot",
        "quote" => "apostrophe",
        "bracketleft" => "leftbracket",
        "bracketright" => "rightbracket",
        "delete" => "del",
        "insert" => "ins",
        "pageup" => "pgup",
        "pagedown" => "pgdn",
        "arrowleft" => "lft",
        "arrowright" => "rght",
        "arrowup" => "up",
        "arrowdown" => "down",
        _ => return lower,
    }
    .into()
}

fn layer_reference(head: &str) -> bool {
    matches!(
        head,
        "layer-switch"
            | "layer-toggle"
            | "layer-while-held"
            | "release-layer"
            | "layer↑"
            | "layer"
            | "base-layer"
    )
}

fn references<'a>(node: &'a Node, name: &str, found: &mut Vec<&'a Node>) {
    if layer_reference(node.head())
        && let Some(target) = node.children.get(1).filter(|n| !n.list && n.value == name)
    {
        found.push(target);
    }
    for child in &node.children {
        references(child, name, found);
    }
}

fn edit(req: &Request, config: &Config) -> Result<String, String> {
    let source = &req.source;
    match req.op.as_str() {
        "setAction" => {
            let value = req.expression.trim();
            parser::expression(value)?;
            Ok(patch(source, config.action(req.layer, req.key)?, value))
        }
        "setTapHold" => {
            if !matches!(
                req.head.as_str(),
                "tap-hold" | "tap-hold-press" | "tap-hold-release"
            ) || !(1..=5000).contains(&req.repeat_ms)
                || !(1..=5000).contains(&req.hold_ms)
            {
                return Err("Ungültige Tap Hold Zeiten oder Variante".into());
            }
            parser::expression(req.tap.trim())?;
            parser::expression(req.hold.trim())?;
            let action = config.action(req.layer, req.key)?;
            let part = action.tap_hold().unwrap_or(action);
            let value = format!(
                "({} {} {} {} {})",
                req.head,
                req.repeat_ms,
                req.hold_ms,
                req.tap.trim(),
                req.hold.trim()
            );
            Ok(patch(source, part, &value))
        }
        "setMacro" => {
            if !crate::actions::MACRO_HEADS.contains(&req.head.as_str()) || req.steps.is_empty() {
                return Err("Makro benötigt eine Variante und mindestens einen Schritt".into());
            }
            let mut steps = Vec::new();
            for step in &req.steps {
                let value = step.trim();
                let first = value.chars().next().ok_or("Leerer Makroschritt")?;
                if !(first.is_ascii_alphanumeric() || "_+-".contains(first))
                    || !value
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || "_+*/.-".contains(c))
                {
                    return Err(format!("Ungültiger Makroschritt: {value}"));
                }
                if value.chars().all(|c| c.is_ascii_digit())
                    && !value
                        .parse::<u32>()
                        .is_ok_and(|ms| (1..=5000).contains(&ms))
                {
                    return Err("Makropause muss zwischen 1 und 5000 ms liegen".into());
                }
                steps.push(value);
            }
            Ok(patch(
                source,
                config.action(req.layer, req.key)?,
                &format!("({} {})", req.head, steps.join(" ")),
            ))
        }
        "addPhysicalKey" => {
            let key = req.name.trim();
            if key.is_empty()
                || !key
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || "_-".contains(c))
            {
                return Err("Ungültiger Tastenname".into());
            }
            parser::expression(req.expression.trim())?;
            config.layer(req.layer)?;
            if config.forms[config.src].children[1..]
                .iter()
                .any(|n| normalized(&n.value) == normalized(key))
            {
                return Err("Diese Taste ist bereits in defsrc enthalten".into());
            }
            let append = |form: &Node, value: &str| {
                let at = form.end - 1;
                let prefix = if source[form.start..at].contains('\n') {
                    if source[..at].ends_with('\n') {
                        "  "
                    } else {
                        "\n  "
                    }
                } else {
                    " "
                };
                let suffix = if source[form.start..at].contains('\n') {
                    "\n"
                } else {
                    ""
                };
                (at, format!("{prefix}{value}{suffix}"))
            };
            let mut patches = vec![append(&config.forms[config.src], key)];
            for (layer, &index) in config.layers.iter().enumerate() {
                patches.push(append(
                    &config.forms[index],
                    if layer == req.layer {
                        req.expression.trim()
                    } else {
                        "_"
                    },
                ));
            }
            patches.sort_by_key(|p| std::cmp::Reverse(p.0));
            let mut result = source.clone();
            for (at, value) in patches {
                result.insert_str(at, &value);
            }
            Ok(result)
        }
        "addLayer" => {
            let name = req.name.trim();
            valid_name(config, name, None)?;
            let body = if let Some(index) = req.copy_from {
                let layer = config.layer(index)?;
                source[layer.children[1].end..layer.end - 1].to_owned()
            } else {
                format!(
                    "\n  {}\n",
                    vec!["_"; config.forms[config.src].children.len() - 1].join(" ")
                )
            };
            Ok(format!("{source}\n(deflayer {name}{body})\n"))
        }
        "renameLayer" => {
            let layer = config.layer(req.layer)?;
            let name = req.name.trim();
            valid_name(config, name, Some(config.layers[req.layer]))?;
            let old = &layer.children[1];
            let mut patches = vec![old];
            for form in &config.forms {
                references(form, &old.value, &mut patches);
            }
            patches.sort_by_key(|n| std::cmp::Reverse(n.start));
            patches.dedup_by_key(|n| n.start);
            let mut result = source.clone();
            for node in patches {
                result.replace_range(node.start..node.end, name);
            }
            Ok(result)
        }
        "deleteLayer" => {
            if config.layers.len() <= 1 {
                return Err("Die letzte Ebene kann nicht gelöscht werden".into());
            }
            let layer = config.layer(req.layer)?;
            let name = &layer.children[1].value;
            let mut found = Vec::new();
            for (index, node) in config.forms.iter().enumerate() {
                if index != config.layers[req.layer] {
                    references(node, name, &mut found);
                }
            }
            if !found.is_empty() {
                return Err("Ebene wird noch von einer Aktion verwendet".into());
            }
            let mut end = layer.end;
            while matches!(source.as_bytes().get(end), Some(b' ' | b'\t')) {
                end += 1;
            }
            if source.as_bytes().get(end) == Some(&b'\n') {
                end += 1;
            }
            let mut result = source.clone();
            result.replace_range(layer.start..end, "");
            Ok(result)
        }
        "replaceConfigForm" | "addConfigForm" => {
            let value = req.expression.trim();
            let node = parser::expression(value)?;
            if !editable(&node) {
                return Err("defsrc und deflayer werden separat verwaltet".into());
            }
            if req.op == "replaceConfigForm" {
                let old = config
                    .forms
                    .get(req.form_index)
                    .filter(|n| editable(n))
                    .ok_or("Ungültiger Konfigurationsblock")?;
                Ok(patch(source, old, value))
            } else {
                Ok(format!("{source}\n\n{value}\n"))
            }
        }
        _ => Err(format!("Unbekannte Operation: {}", req.op)),
    }
}
