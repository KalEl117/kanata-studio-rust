use crate::parser::{self, Config, Node};
use serde::Deserialize;

/// One editing operation. On the wire `op` names the variant and fields are camelCase.
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "op", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum Edit {
    Clear,
    Record {
        previous: String,
    },
    Undo,
    Redo,
    SetAction {
        expression: String,
    },
    SetTapHold {
        head: String,
        repeat_ms: u32,
        hold_ms: u32,
        tap: String,
        hold: String,
    },
    SetMacro {
        head: String,
        steps: Vec<String>,
    },
    AddPhysicalKey {
        name: String,
        #[serde(default = "transparent")]
        expression: String,
    },
    AddLayer {
        name: String,
        copy_from: Option<usize>,
    },
    RenameLayer {
        name: String,
    },
    DeleteLayer,
    ReplaceConfigForm {
        form_index: usize,
        expression: String,
    },
    AddConfigForm {
        expression: String,
    },
}

fn transparent() -> String {
    "_".into()
}

#[derive(Clone, Debug, Deserialize)]
pub struct Request {
    #[serde(flatten)]
    pub edit: Edit,
    #[serde(default)]
    pub source: String,
    pub layer: Option<usize>,
    pub key: Option<usize>,
    /// Selected key that is not in defsrc yet; set by the dispatcher, never read from the wire.
    #[serde(skip)]
    pub physical: Option<String>,
}

/// Each history entry is a full copy of the config, so the oldest entries are dropped.
pub const HISTORY_LIMIT: usize = 200;

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

    fn remember(&mut self, source: String) {
        if self.undo.len() >= HISTORY_LIMIT {
            self.undo.remove(0);
        }
        self.undo.push(source);
    }

    pub fn execute(&mut self, req: &Request) -> Result<String, String> {
        match &req.edit {
            Edit::Clear => {
                self.undo.clear();
                self.redo.clear();
                return Ok(req.source.clone());
            }
            Edit::Record { previous } => {
                if *previous != req.source {
                    self.remember(previous.clone());
                    self.redo.clear();
                }
                return Ok(req.source.clone());
            }
            Edit::Undo | Edit::Redo => {
                let undo = matches!(req.edit, Edit::Undo);
                let stack = if undo { &mut self.undo } else { &mut self.redo };
                let source = stack.last().ok_or("Keine Änderung im Verlauf")?;
                Config::parse(source)?;
                let source = stack.pop().unwrap();
                if undo {
                    self.redo.push(req.source.clone());
                } else {
                    self.remember(req.source.clone());
                }
                return Ok(source);
            }
            _ => {}
        }
        let config = Config::parse(&req.source)?;
        let layer = req.layer.unwrap_or(0);
        let key = req.key.unwrap_or(0);
        let source = match (&req.physical, &req.edit) {
            (
                Some(name),
                Edit::SetAction { .. } | Edit::SetTapHold { .. } | Edit::SetMacro { .. },
            ) => {
                if let Edit::SetAction { expression } = &req.edit
                    && (expression.trim() == "_"
                        || normalized(expression.trim()) == normalized(name))
                {
                    return Ok(req.source.clone());
                }
                let add = Edit::AddPhysicalKey {
                    name: name.clone(),
                    expression: transparent(),
                };
                let staged = edit(&req.source, &config, layer, key, &add)?;
                let staged_config = Config::parse(&staged)?;
                let added = staged_config.forms[staged_config.src].children.len() - 2;
                edit(&staged, &staged_config, layer, added, &req.edit)?
            }
            _ => edit(&req.source, &config, layer, key, &req.edit)?,
        };
        Config::parse(&source)?;
        if source != req.source {
            self.remember(req.source.clone());
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
    // Tuple forms whose n-th element is a list of layer names (kanata parser/src/cfg/chord.rs, override.rs).
    let layer_list = match node.head() {
        "defchordsv2" | "defchordsv2-experimental" => Some((5, 4)),
        "defoverridesv2" => Some((4, 3)),
        _ => None,
    };
    if let Some((size, position)) = layer_list {
        for tuple in node.children[1..].chunks(size) {
            if let Some(list) = tuple.get(position).filter(|n| n.list) {
                found.extend(list.children.iter().filter(|n| !n.list && n.value == name));
            }
        }
    }
    for child in &node.children {
        references(child, name, found);
    }
}

fn edit(
    source: &str,
    config: &Config,
    layer: usize,
    key: usize,
    op: &Edit,
) -> Result<String, String> {
    if config.is_map(layer)
        && matches!(
            op,
            Edit::SetAction { .. } | Edit::SetTapHold { .. } | Edit::SetMacro { .. }
        )
    {
        return Err(parser::MAP_READ_ONLY.into());
    }
    match op {
        Edit::SetAction { expression } => {
            let value = expression.trim();
            parser::expression(value)?;
            Ok(patch(source, config.action(layer, key)?, value))
        }
        Edit::SetTapHold {
            head,
            repeat_ms,
            hold_ms,
            tap,
            hold,
        } => {
            if !matches!(
                head.as_str(),
                "tap-hold" | "tap-hold-press" | "tap-hold-release"
            ) || !(1..=5000).contains(repeat_ms)
                || !(1..=5000).contains(hold_ms)
            {
                return Err("Ungültige Tap Hold Zeiten oder Variante".into());
            }
            parser::expression(tap.trim())?;
            parser::expression(hold.trim())?;
            let action = config.action(layer, key)?;
            let part = action.tap_hold().unwrap_or(action);
            let value = format!(
                "({head} {repeat_ms} {hold_ms} {} {})",
                tap.trim(),
                hold.trim()
            );
            Ok(patch(source, part, &value))
        }
        Edit::SetMacro { head, steps: input } => {
            if !crate::actions::MACRO_HEADS.contains(&head.as_str()) || input.is_empty() {
                return Err("Makro benötigt eine Variante und mindestens einen Schritt".into());
            }
            let mut steps = Vec::new();
            for step in input {
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
                config.action(layer, key)?,
                &format!("({head} {})", steps.join(" ")),
            ))
        }
        Edit::AddPhysicalKey { name, expression } => {
            let target = layer;
            let key = name.trim();
            if key.is_empty()
                || !key
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || "_-".contains(c))
            {
                return Err("Ungültiger Tastenname".into());
            }
            parser::expression(expression.trim())?;
            config.layer(target)?;
            if config.is_map(target) && expression.trim() != "_" {
                return Err(parser::MAP_READ_ONLY.into());
            }
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
            // A deflayermap needs no entry: unmapped keys are transparent there.
            for (layer, &index) in config.layers.iter().enumerate() {
                if config.is_map(layer) {
                    continue;
                }
                patches.push(append(
                    &config.forms[index],
                    if layer == target {
                        expression.trim()
                    } else {
                        "_"
                    },
                ));
            }
            patches.sort_by_key(|p| std::cmp::Reverse(p.0));
            let mut result = source.to_owned();
            for (at, value) in patches {
                result.insert_str(at, &value);
            }
            Ok(result)
        }
        Edit::AddLayer { name, copy_from } => {
            let name = name.trim();
            valid_name(config, name, None)?;
            let body = if let Some(index) = *copy_from {
                if config.is_map(index) {
                    return Err(
                        "Eine deflayermap-Ebene kann nicht als Vorlage kopiert werden".into(),
                    );
                }
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
        Edit::RenameLayer { name } => {
            let name = name.trim();
            valid_name(config, name, Some(config.layers[layer]))?;
            let old = config.layer_name(layer)?;
            let mut patches = vec![old];
            for form in &config.forms {
                references(form, &old.value, &mut patches);
            }
            patches.sort_by_key(|n| std::cmp::Reverse(n.start));
            patches.dedup_by_key(|n| n.start);
            let mut result = source.to_owned();
            for node in patches {
                result.replace_range(node.start..node.end, name);
            }
            Ok(result)
        }
        Edit::DeleteLayer => {
            if config.layers.len() <= 1 {
                return Err("Die letzte Ebene kann nicht gelöscht werden".into());
            }
            let form = config.layer(layer)?;
            let name = &config.layer_name(layer)?.value;
            let mut found = Vec::new();
            for (index, node) in config.forms.iter().enumerate() {
                if index != config.layers[layer] {
                    references(node, name, &mut found);
                }
            }
            if !found.is_empty() {
                return Err("Ebene wird noch von einer Aktion verwendet".into());
            }
            let mut end = form.end;
            while matches!(source.as_bytes().get(end), Some(b' ' | b'\t')) {
                end += 1;
            }
            if source.as_bytes().get(end) == Some(&b'\n') {
                end += 1;
            }
            let mut result = source.to_owned();
            result.replace_range(form.start..end, "");
            Ok(result)
        }
        Edit::ReplaceConfigForm {
            expression,
            form_index: _,
        }
        | Edit::AddConfigForm { expression } => {
            let value = expression.trim();
            let node = parser::expression(value)?;
            if !editable(&node) {
                return Err("defsrc und deflayer werden separat verwaltet".into());
            }
            if let Edit::ReplaceConfigForm { form_index, .. } = op {
                let old = config
                    .forms
                    .get(*form_index)
                    .filter(|n| editable(n))
                    .ok_or("Ungültiger Konfigurationsblock")?;
                Ok(patch(source, old, value))
            } else {
                Ok(format!("{source}\n\n{value}\n"))
            }
        }
        Edit::Clear | Edit::Record { .. } | Edit::Undo | Edit::Redo => {
            Err("Verlaufsoperationen ändern keinen Text".into())
        }
    }
}
