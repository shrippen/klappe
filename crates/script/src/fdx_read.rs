//! FDX to a [`Script`].

use std::collections::HashMap;

use crate::fdx::{
    self, paragraphs, FdxLoss, ALIGNMENT, CENTER, CONTENT, DUAL, KEY, NEW_PAGE, NUMBER, RIGHT,
    ROOT, TEXT, TITLE_PAGE, TYPE, YES,
};
use crate::fdx_marks::{marked_text, Reading};
use crate::xml::{parse_xml, FdxError, Node};
use crate::{Element, Script};

/// Title page keys, in the order of the sheet: what the centered paragraphs of a file
/// without our key attribute are called.
const CENTERED_KEYS: [&str; 4] = ["Title", "Credit", "Author", "Source"];
const DRAFT_DATE: &str = "Draft date";
const CONTACT: &str = "Contact";

/// Reads an FDX file. Paragraph types FDX has beyond the screenplay's (`Shot`, `General`,
/// `Cast List`) become action; empty paragraphs are dropped. The loss says what Fountain could
/// not keep (`styles`: runs that lost their emphasis).
pub fn read_fdx(text: &str) -> Result<(Script, FdxLoss), FdxError> {
    let root = parse_xml(text)?;
    if root.name != ROOT {
        return Err(FdxError::new(&format!(
            "the root element is <{}>, not <{ROOT}>",
            root.name
        )));
    }

    let mut out = Out {
        elements: Vec::new(),
        styles: 0,
    };
    if let Some(content) = root.child(CONTENT) {
        read_content(content, &mut out);
    }

    let title = root.child(TITLE_PAGE).and_then(|t| t.child(CONTENT));
    let script = Script {
        title_page: title.map(read_title).unwrap_or_default(),
        elements: out.elements,
    };
    let loss = FdxLoss {
        styles: out.styles,
        ..FdxLoss::default()
    };

    Ok((script, loss))
}

/// What reading has collected so far.
struct Out {
    elements: Vec<Element>,
    styles: usize,
}

fn read_content(content: &Node, out: &mut Out) {
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
fn read_dual(node: &Node, out: &mut Out) {
    let mut cues = 0;

    for par in paragraphs(node) {
        let is_cue = par.attr(TYPE) == Some(fdx::CHARACTER);
        read_paragraph(par, is_cue && cues > 0, out);
        if is_cue {
            cues += 1;
        }
    }
}

fn read_paragraph(par: &Node, dual: bool, out: &mut Out) {
    if par.attr(NEW_PAGE) == Some(YES) {
        out.elements.push(Element::PageBreak);
    }

    let kind = par.attr(TYPE).unwrap_or_default();
    let reading = match kind {
        fdx::HEADING | fdx::CHARACTER | fdx::TRANSITION => Reading::Literal,
        _ => Reading::Inline,
    };
    let text = marked_text(par, reading, &mut out.styles);
    let text = text.trim();
    if text.is_empty() {
        return;
    }

    let element = match kind {
        fdx::HEADING => Element::SceneHeading {
            text: text.to_owned(),
            number: par
                .attr(NUMBER)
                .filter(|n| !n.is_empty())
                .map(str::to_owned),
        },
        fdx::CHARACTER => cue(text, dual),
        fdx::PARENTHETICAL => Element::Parenthetical(unwrap_parens(text)),
        fdx::DIALOGUE => Element::Dialogue(no_blank_lines(text)),
        fdx::TRANSITION => Element::Transition(text.to_owned()),
        _ if par.attr(ALIGNMENT) == Some(CENTER) => Element::Centered(text.to_owned()),
        _ => Element::Action(text.to_owned()),
    };
    out.elements.push(element);
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

/// A blank line ends a dialogue in Fountain, so a dialogue with one would split; the blank
/// lines are dropped.
fn no_blank_lines(text: &str) -> String {
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();

    lines.join("\n")
}

fn unwrap_parens(text: &str) -> String {
    let inner = text.strip_prefix('(').unwrap_or(text);

    inner.strip_suffix(')').unwrap_or(inner).trim().to_owned()
}

/// The title page: our paragraphs name their key; in a foreign file the centered ones are
/// title, credit, author and source in order, a right-aligned one is the draft date and the
/// rest the contact.
fn read_title(content: &Node) -> Vec<(String, String)> {
    let mut pairs: Vec<(String, String)> = Vec::new();
    let mut index: HashMap<String, usize> = HashMap::new();
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

        match index.get(&key) {
            Some(&at) => {
                pairs[at].1.push('\n');
                pairs[at].1.push_str(&text);
            }
            None => {
                index.insert(key.clone(), pairs.len());
                pairs.push((key, text));
            }
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
