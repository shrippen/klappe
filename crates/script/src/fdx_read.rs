//! FDX to a [`Script`].

use crate::fdx::{
    self, paragraphs, Style, ALIGNMENT, CENTER, CONTENT, DUAL, KEY, NEW_PAGE, NUMBER, RIGHT, ROOT,
    STYLE, TEXT, TITLE_PAGE, TYPE, YES,
};
use crate::xml::{parse_xml, FdxError, Node};
use crate::{Element, Script};

/// Title page keys, in the order of the sheet: what the centered paragraphs of a file
/// without our key attribute are called.
const CENTERED_KEYS: [&str; 4] = ["Title", "Credit", "Author", "Source"];
const DRAFT_DATE: &str = "Draft date";
const CONTACT: &str = "Contact";

/// Reads an FDX file. Paragraph types FDX has beyond the screenplay's (`Shot`, `General`,
/// `Cast List`) become action; empty paragraphs are dropped.
pub fn read_fdx(text: &str) -> Result<Script, FdxError> {
    let root = parse_xml(text)?;
    if root.name != ROOT {
        return Err(FdxError::new(&format!(
            "the root element is <{}>, not <{ROOT}>",
            root.name
        )));
    }

    let mut elements = Vec::new();
    if let Some(content) = root.child(CONTENT) {
        read_content(content, &mut elements);
    }

    let title = root.child(TITLE_PAGE).and_then(|t| t.child(CONTENT));

    Ok(Script {
        title_page: title.map(read_title).unwrap_or_default(),
        elements,
    })
}

fn read_content(content: &Node, out: &mut Vec<Element>) {
    for node in content.nodes() {
        if node.name == DUAL {
            read_dual(node, out);
            continue;
        }
        if node.name == fdx::PARAGRAPH {
            read_paragraph(node, false, out);
        }
    }
}

/// The voices of a dual dialogue: the second cue carries the mark.
fn read_dual(node: &Node, out: &mut Vec<Element>) {
    let mut cues = 0;

    for par in paragraphs(node) {
        let is_cue = par.attr(TYPE) == Some(fdx::CHARACTER);
        read_paragraph(par, is_cue && cues > 0, out);
        if is_cue {
            cues += 1;
        }
    }
}

fn read_paragraph(par: &Node, dual: bool, out: &mut Vec<Element>) {
    if par.attr(NEW_PAGE) == Some(YES) {
        out.push(Element::PageBreak);
    }

    let text = marked_text(par);
    let text = text.trim();
    if text.is_empty() {
        return;
    }

    let element = match par.attr(TYPE).unwrap_or_default() {
        fdx::HEADING => Element::SceneHeading {
            text: text.to_owned(),
            number: par
                .attr(NUMBER)
                .filter(|n| !n.is_empty())
                .map(str::to_owned),
        },
        fdx::CHARACTER => cue(text, dual),
        fdx::PARENTHETICAL => Element::Parenthetical(unwrap_parens(text)),
        fdx::DIALOGUE => Element::Dialogue(text.to_owned()),
        fdx::TRANSITION => Element::Transition(text.to_owned()),
        _ if par.attr(ALIGNMENT) == Some(CENTER) => Element::Centered(text.to_owned()),
        _ => Element::Action(text.to_owned()),
    };
    out.push(element);
}

/// `ELISABETH (V.O.)` is the name and its extension.
fn cue(text: &str, dual: bool) -> Element {
    let split = text
        .strip_suffix(')')
        .and_then(|head| head.rsplit_once('('))
        .map(|(name, ext)| (name.trim(), ext.trim()))
        .filter(|(name, ext)| !name.is_empty() && !ext.is_empty());

    let (name, extension) = match split {
        Some((name, ext)) => (name, Some(ext.to_owned())),
        None => (text, None),
    };

    Element::Character {
        name: name.to_owned(),
        extension,
        dual,
    }
}

fn unwrap_parens(text: &str) -> String {
    let inner = text.strip_prefix('(').unwrap_or(text);

    inner.strip_suffix(')').unwrap_or(inner).trim().to_owned()
}

/// A text run and how it is set.
struct Piece {
    style: Style,
    text: String,
}

/// The paragraph's text as Fountain: runs with a style get its marks, and a literal `*` or
/// `_` gets a backslash so it stays text.
fn marked_text(par: &Node) -> String {
    let mut pieces: Vec<Piece> = Vec::new();

    for run in par.nodes().filter(|n| n.name == TEXT) {
        let style = Style::parse(run.attr(STYLE).unwrap_or_default());
        let text = run.text();

        // Neighbours of one style join, so `**a****b**` never comes out.
        match pieces.last_mut() {
            Some(last) if last.style == style => last.text.push_str(&text),
            _ => pieces.push(Piece { style, text }),
        }
    }

    pieces.iter().map(|p| mark(&p.text, p.style)).collect()
}

/// `text` with the marks of `style` around each line's words; blanks stay outside, and a
/// line is marked alone because Fountain closes emphasis on the line it opened.
fn mark(text: &str, style: Style) -> String {
    let lines: Vec<String> = text
        .split('\n')
        .map(|line| mark_line(&escape(line), style))
        .collect();

    lines.join("\n")
}

fn mark_line(line: &str, style: Style) -> String {
    let core = line.trim();
    let marks = (
        if style.underline { "_" } else { "" },
        match (style.bold, style.italic) {
            (true, true) => "***",
            (true, false) => "**",
            (false, true) => "*",
            (false, false) => "",
        },
    );
    if core.is_empty() || (marks.0.is_empty() && marks.1.is_empty()) {
        return line.to_owned();
    }

    let lead = &line[..line.len() - line.trim_start().len()];
    let trail = &line[line.trim_end().len()..];
    let (under, stars) = marks;

    format!("{lead}{under}{stars}{core}{stars}{under}{trail}")
}

fn escape(line: &str) -> String {
    let mut out = String::with_capacity(line.len());

    for c in line.chars() {
        if c == '*' || c == '_' {
            out.push('\\');
        }
        out.push(c);
    }

    out
}

/// The title page: our paragraphs name their key; in a foreign file the centered ones are
/// title, credit, author and source in order, a right-aligned one is the draft date and the
/// rest the contact.
fn read_title(content: &Node) -> Vec<(String, String)> {
    let mut pairs: Vec<(String, String)> = Vec::new();
    let mut centered = 0;

    for par in paragraphs(content) {
        let text: String = par
            .nodes()
            .filter(|n| n.name == TEXT)
            .map(Node::text)
            .collect();
        let key = match par.attr(KEY) {
            Some(key) => key.to_owned(),
            None if text.trim().is_empty() => continue,
            None => foreign_key(par.attr(ALIGNMENT), &mut centered),
        };

        match pairs.iter_mut().find(|(k, _)| *k == key) {
            Some((_, value)) => {
                value.push('\n');
                value.push_str(&text);
            }
            None => pairs.push((key, text)),
        }
    }

    pairs
}

fn foreign_key(alignment: Option<&str>, centered: &mut usize) -> String {
    match alignment {
        Some(CENTER) => {
            let key = CENTERED_KEYS[(*centered).min(CENTERED_KEYS.len() - 1)];
            *centered += 1;

            key.to_owned()
        }
        Some(RIGHT) => DRAFT_DATE.to_owned(),
        _ => CONTACT.to_owned(),
    }
}
