//! A small XML reader and writer, enough for Final Draft files: elements, attributes and
//! text with the five named entities and numeric ones. No namespaces, no DTD (a `DOCTYPE` is
//! skipped, so no entity can be declared).

use std::fmt;

/// Deepest element nesting read; FDX stays under ten, a file beyond this is not one.
const MAX_DEPTH: usize = 64;
const INDENT: &str = "\t";

/// Why a file is no XML this reader takes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FdxError(String);

impl FdxError {
    pub(crate) fn new(message: &str) -> FdxError {
        FdxError(message.to_owned())
    }
}

impl fmt::Display for FdxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for FdxError {}

/// An element with its attributes in file order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Node {
    pub name: String,
    pub attrs: Vec<(String, String)>,
    pub kids: Vec<Kid>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Kid {
    Node(Node),
    Text(String),
}

impl Node {
    pub fn new(name: &str) -> Node {
        Node {
            name: name.to_owned(),
            attrs: Vec::new(),
            kids: Vec::new(),
        }
    }

    pub fn with_attr(mut self, name: &str, value: &str) -> Node {
        self.attrs.push((name.to_owned(), value.to_owned()));

        self
    }

    pub fn with_node(mut self, child: Node) -> Node {
        self.kids.push(Kid::Node(child));

        self
    }

    pub fn with_text(mut self, text: &str) -> Node {
        self.kids.push(Kid::Text(text.to_owned()));

        self
    }

    pub fn attr(&self, name: &str) -> Option<&str> {
        let found = self.attrs.iter().find(|(k, _)| k == name);

        found.map(|(_, v)| v.as_str())
    }

    /// The child elements in order.
    pub fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.kids.iter().filter_map(|k| match k {
            Kid::Node(n) => Some(n),
            Kid::Text(_) => None,
        })
    }

    /// The first child element of this name.
    pub fn child(&self, name: &str) -> Option<&Node> {
        self.nodes().find(|n| n.name == name)
    }

    /// The text directly inside, without the children's.
    pub fn text(&self) -> String {
        self.kids
            .iter()
            .filter_map(|k| match k {
                Kid::Text(t) => Some(t.as_str()),
                Kid::Node(_) => None,
            })
            .collect()
    }

    /// Writes the element and what it holds; an element with text is kept on one line, so
    /// no indentation becomes part of the text.
    pub fn write(&self, out: &mut String, depth: usize) {
        let pad = INDENT.repeat(depth);
        out.push_str(&pad);
        out.push('<');
        out.push_str(&self.name);
        for (key, value) in &self.attrs {
            out.push_str(&format!(" {key}=\"{}\"", escape(value, Escape::Attribute)));
        }

        if self.kids.is_empty() {
            out.push_str("/>\n");
            return;
        }
        out.push('>');

        let has_text = self.kids.iter().any(|k| matches!(k, Kid::Text(_)));
        if has_text {
            for kid in &self.kids {
                match kid {
                    Kid::Text(t) => out.push_str(&escape(t, Escape::Text)),
                    Kid::Node(n) => n.write_inline(out),
                }
            }
            out.push_str(&format!("</{}>\n", self.name));
            return;
        }

        out.push('\n');
        for node in self.nodes() {
            node.write(out, depth + 1);
        }
        out.push_str(&format!("{pad}</{}>\n", self.name));
    }

    fn write_inline(&self, out: &mut String) {
        let mut line = String::new();
        self.write(&mut line, 0);
        out.push_str(line.trim_end());
    }
}

#[derive(Clone, Copy)]
enum Escape {
    Text,
    Attribute,
}

/// Entities for what XML reserves; characters XML 1.0 cannot hold are left out.
fn escape(text: &str, kind: Escape) -> String {
    let mut out = String::with_capacity(text.len());

    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' if matches!(kind, Escape::Attribute) => out.push_str("&quot;"),
            '\n' | '\t' => out.push(c),
            c if c < ' ' || c == '\u{fffe}' || c == '\u{ffff}' => {}
            c => out.push(c),
        }
    }

    out
}

/// Reads a document and gives its root element.
pub(crate) fn parse_xml(text: &str) -> Result<Node, FdxError> {
    let text = text.trim_start_matches('\u{feff}').replace("\r\n", "\n");
    let mut reader = Reader { s: &text, at: 0 };

    reader.skip_misc()?;
    let root = reader.element(0)?;
    reader.skip_misc()?;
    if reader.at < text.len() {
        return Err(reader.fail("text after the root element"));
    }

    Ok(root)
}

struct Reader<'a> {
    s: &'a str,
    at: usize,
}

impl Reader<'_> {
    fn fail(&self, what: &str) -> FdxError {
        let line = self.s[..self.at].matches('\n').count() + 1;

        FdxError(format!("line {line}: {what}"))
    }

    fn rest(&self) -> &str {
        &self.s[self.at..]
    }

    fn starts(&self, token: &str) -> bool {
        self.rest().starts_with(token)
    }

    fn skip_space(&mut self) {
        let rest = self.rest();
        self.at += rest.len() - rest.trim_start().len();
    }

    /// Moves past `end`, which must come later; returns what stood before it.
    fn until(&mut self, end: &str) -> Result<&str, FdxError> {
        let Some(len) = self.rest().find(end) else {
            return Err(self.fail(&format!("missing {end}")));
        };
        let from = self.at;
        self.at += len + end.len();

        Ok(&self.s[from..from + len])
    }

    /// Blanks, the declaration, comments and the `DOCTYPE` before and after the root.
    fn skip_misc(&mut self) -> Result<(), FdxError> {
        loop {
            self.skip_space();
            if self.starts("<?") {
                self.until("?>")?;
            } else if self.starts("<!--") {
                self.until("-->")?;
            } else if self.starts("<!DOCTYPE") {
                self.until(">")?;
            } else {
                return Ok(());
            }
        }
    }

    fn name(&mut self) -> Result<String, FdxError> {
        let rest = self.rest();
        let len = rest
            .find(|c: char| c.is_whitespace() || matches!(c, '/' | '>' | '='))
            .unwrap_or(rest.len());
        if len == 0 {
            return Err(self.fail("a name is missing"));
        }
        let name = rest[..len].to_owned();
        self.at += len;

        Ok(name)
    }

    fn element(&mut self, depth: usize) -> Result<Node, FdxError> {
        if depth > MAX_DEPTH {
            return Err(self.fail("elements nested too deep"));
        }
        if !self.starts("<") {
            return Err(self.fail("an element is missing"));
        }
        self.at += 1;

        let mut node = Node::new(&self.name()?);
        loop {
            self.skip_space();
            if self.starts("/>") {
                self.at += 2;
                return Ok(node);
            }
            if self.starts(">") {
                self.at += 1;
                break;
            }
            let (key, value) = self.attribute()?;
            node.attrs.push((key, value));
        }

        self.content(&mut node, depth)?;

        Ok(node)
    }

    fn attribute(&mut self) -> Result<(String, String), FdxError> {
        let key = self.name()?;
        self.skip_space();
        if !self.starts("=") {
            return Err(self.fail("= is missing after an attribute name"));
        }
        self.at += 1;
        self.skip_space();

        let quote = self.rest().chars().next();
        let Some(quote @ ('"' | '\'')) = quote else {
            return Err(self.fail("an attribute value needs quotes"));
        };
        self.at += 1;
        let raw = self.until(&quote.to_string())?.to_owned();

        Ok((key, self.decode(&raw)?))
    }

    /// The children and text up to the closing tag of `node`.
    fn content(&mut self, node: &mut Node, depth: usize) -> Result<(), FdxError> {
        loop {
            if self.rest().is_empty() {
                return Err(self.fail(&format!("<{}> is never closed", node.name)));
            }
            if self.starts("</") {
                self.at += 2;
                let name = self.name()?;
                self.skip_space();
                if name != node.name || !self.starts(">") {
                    return Err(self.fail(&format!("</{name}> closes <{}>", node.name)));
                }
                self.at += 1;

                return Ok(());
            }
            if self.starts("<!--") {
                self.until("-->")?;
            } else if self.starts("<?") {
                self.until("?>")?;
            } else if self.starts("<![CDATA[") {
                self.at += "<![CDATA[".len();
                let text = self.until("]]>")?.to_owned();
                node.kids.push(Kid::Text(text));
            } else if self.starts("<") {
                let child = self.element(depth + 1)?;
                node.kids.push(Kid::Node(child));
            } else {
                let len = self.rest().find('<').unwrap_or(self.rest().len());
                let raw = self.rest()[..len].to_owned();
                self.at += len;
                node.kids.push(Kid::Text(self.decode(&raw)?));
            }
        }
    }

    /// Replaces the entities in `raw`.
    fn decode(&self, raw: &str) -> Result<String, FdxError> {
        let mut out = String::with_capacity(raw.len());
        let mut rest = raw;

        while let Some(start) = rest.find('&') {
            out.push_str(&rest[..start]);
            let after = &rest[start + 1..];
            let Some(end) = after.find(';') else {
                return Err(self.fail("an & without ;"));
            };
            let Some(c) = entity(&after[..end]) else {
                return Err(self.fail(&format!("unknown entity &{};", &after[..end])));
            };
            out.push(c);
            rest = &after[end + 1..];
        }
        out.push_str(rest);

        Ok(out)
    }
}

fn entity(name: &str) -> Option<char> {
    match name {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        _ => {
            let digits = name.strip_prefix('#')?;
            let code = match digits.strip_prefix(['x', 'X']) {
                Some(hex) => u32::from_str_radix(hex, 16).ok()?,
                None => digits.parse().ok()?,
            };

            char::from_u32(code)
        }
    }
}
