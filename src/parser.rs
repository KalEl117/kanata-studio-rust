use serde::Serialize;

/// Byte offsets refer to the original UTF-8 source; edits preserve all other bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
    pub start: usize,
    pub end: usize,
    pub value: String,
    pub children: Vec<Node>,
    pub list: bool,
}

impl Node {
    pub fn head(&self) -> &str {
        self.children
            .first()
            .filter(|n| !n.list)
            .map_or("", |n| &n.value)
    }

    pub fn raw<'a>(&self, source: &'a str) -> &'a str {
        &source[self.start..self.end]
    }

    pub fn tap_hold(&self) -> Option<&Node> {
        if self.list
            && self.children.len() >= 5
            && matches!(
                self.head(),
                "tap-hold" | "tap-hold-press" | "tap-hold-release"
            )
        {
            return Some(self);
        }
        self.children.iter().find_map(Node::tap_hold)
    }
}

/// Qt uses UTF-16 indices. Convert only at the UI boundary, never inside the editor.
#[derive(Serialize)]
pub struct QtNode {
    start: usize,
    end: usize,
    value: String,
    children: Vec<QtNode>,
    list: bool,
}

pub fn qt_nodes(source: &str, nodes: &[Node]) -> Vec<QtNode> {
    let mut offsets = vec![0; source.len() + 1];
    let mut units = 0;
    for (byte, ch) in source.char_indices() {
        offsets[byte] = units;
        units += ch.len_utf16();
    }
    offsets[source.len()] = units;
    fn convert(node: &Node, offsets: &[usize]) -> QtNode {
        QtNode {
            start: offsets[node.start],
            end: offsets[node.end],
            value: node.value.clone(),
            children: node.children.iter().map(|n| convert(n, offsets)).collect(),
            list: node.list,
        }
    }
    nodes.iter().map(|n| convert(n, &offsets)).collect()
}

pub fn parse(source: &str) -> Result<Vec<Node>, String> {
    let mut forms = Vec::new();
    let mut stack: Vec<Node> = Vec::new();
    let mut at = 0;
    while at < source.len() {
        let rest = &source[at..];
        let ch = rest.chars().next().unwrap();
        // Kanata accepts a leading UTF-8 BOM; treat it like whitespace.
        if ch.is_whitespace() || ch == '\u{feff}' {
            at += ch.len_utf8();
            continue;
        }
        if rest.starts_with(";;") {
            at += rest.find('\n').unwrap_or(rest.len());
            continue;
        }
        if let Some(comment) = rest.strip_prefix("#|") {
            let end = comment
                .find("|#")
                .ok_or("Blockkommentar nicht geschlossen")?;
            at += end + 4;
            continue;
        }
        if ch == '(' {
            if stack.len() >= 128 {
                return Err("Maximale Verschachtelung (128) überschritten".into());
            }
            stack.push(Node {
                start: at,
                end: 0,
                value: String::new(),
                children: Vec::new(),
                list: true,
            });
            at += 1;
            continue;
        }
        let node = if ch == ')' {
            let mut node = stack.pop().ok_or("Unerwartete schließende Klammer")?;
            at += 1;
            node.end = at;
            node
        } else {
            let start = at;
            if ch == '"' {
                at += 1;
                let mut closed = false;
                while at < source.len() {
                    let next = source[at..].chars().next().unwrap();
                    at += next.len_utf8();
                    if next == '\\'
                        && let Some(escaped) = source[at..].chars().next()
                    {
                        at += escaped.len_utf8();
                    } else if next == '"' {
                        closed = true;
                        break;
                    }
                }
                if !closed {
                    return Err("Zeichenkette nicht geschlossen".into());
                }
            } else {
                while at < source.len() {
                    let tail = &source[at..];
                    let next = tail.chars().next().unwrap();
                    if next.is_whitespace()
                        || matches!(next, '(' | ')')
                        || tail.starts_with(";;")
                        || tail.starts_with("#|")
                    {
                        break;
                    }
                    at += next.len_utf8();
                    if next == '\\'
                        && let Some(escaped) = source[at..].chars().next()
                    {
                        at += escaped.len_utf8();
                    }
                }
            }
            Node {
                start,
                end: at,
                value: source[start..at].into(),
                children: Vec::new(),
                list: false,
            }
        };
        if let Some(parent) = stack.last_mut() {
            parent.children.push(node);
        } else {
            forms.push(node);
        }
    }
    if !stack.is_empty() {
        return Err("Liste nicht geschlossen".into());
    }
    Ok(forms)
}

pub fn expression(source: &str) -> Result<Node, String> {
    let mut nodes = parse(source)?;
    if nodes.len() != 1 {
        return Err("Genau ein Ausdruck erforderlich".into());
    }
    Ok(nodes.remove(0))
}

#[derive(Clone)]
pub struct Config {
    pub forms: Vec<Node>,
    pub src: usize,
    pub layers: Vec<usize>,
}

impl Config {
    pub fn parse(source: &str) -> Result<Self, String> {
        let forms = parse(source)?;
        let mut src = None;
        let mut layers = Vec::new();
        for (i, form) in forms.iter().enumerate() {
            match form.head() {
                "defsrc" => {
                    if src.replace(i).is_some() {
                        return Err("Mehrere defsrc-Blöcke".into());
                    }
                    if form.children[1..].iter().any(|n| n.list) {
                        return Err("defsrc enthält eine unerwartete Liste".into());
                    }
                }
                "deflayer" => {
                    if form.children.len() < 2 || form.children[1].list {
                        return Err("Ein deflayer hat keinen gültigen Namen".into());
                    }
                    layers.push(i);
                }
                _ => {}
            }
        }
        let src = src.ok_or("Es werden defsrc und mindestens ein deflayer benötigt")?;
        if layers.is_empty() {
            return Err("Mindestens ein deflayer benötigt".into());
        }
        let keys = forms[src].children.len() - 1;
        if layers.iter().any(|&i| forms[i].children.len() - 2 != keys) {
            return Err("Ebenen und Quelltasten haben unterschiedlich viele Einträge".into());
        }
        Ok(Self { forms, src, layers })
    }

    pub fn layer(&self, index: usize) -> Result<&Node, String> {
        self.layers
            .get(index)
            .map(|&i| &self.forms[i])
            .ok_or("Ungültige Ebene".into())
    }

    pub fn action(&self, layer: usize, key: usize) -> Result<&Node, String> {
        self.layer(layer)?
            .children
            .get(key.saturating_add(2))
            .ok_or("Bitte eine Taste auswählen".into())
    }
}
