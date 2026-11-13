//! Elements to Fountain text.

use crate::parse::{
    parse, parse_elements, FORCE_ACTION, FORCE_CHARACTER, FORCE_HEADING, FORCE_TRANSITION,
};
use crate::{Element, Script};

const TITLE_INDENT: &str = "   ";

/// Writes the script as Fountain text that parses back to the same elements.
///
/// Elements whose plain form would be read as something else get the forcing marker
/// of their kind (`!`, `.`, `@`, `>`).
pub fn write(script: &Script) -> String {
    let mut out = String::new();

    for (key, value) in &script.title_page {
        out.push_str(&title_entry(key, value));
    }
    if !script.title_page.is_empty() {
        out.push('\n');
    }

    let body_start = out.len();
    for (i, el) in script.elements.iter().enumerate() {
        if i > 0 && !same_block(&script.elements[i - 1], el) {
            out.push('\n');
        }
        out.push_str(&render(el, script.elements.get(i + 1)));
        out.push('\n');
    }

    // Without a title page, a first line like `Ort: Burghof` would become one.
    let reads_as_title = script.title_page.is_empty() && !parse(&out).title_page.is_empty();
    if reads_as_title {
        if let Some(marker) = script.elements.first().and_then(marker) {
            out.insert(body_start, marker);
        }
    }

    out
}

/// The marker that forces an element's plain form to stay what it is.
fn marker(el: &Element) -> Option<char> {
    match el {
        Element::SceneHeading { .. } => Some(FORCE_HEADING),
        Element::Action(_) => Some(FORCE_ACTION),
        Element::Character { .. } => Some(FORCE_CHARACTER),
        Element::Transition(_) => Some(FORCE_TRANSITION),
        _ => None,
    }
}

fn title_entry(key: &str, value: &str) -> String {
    if !value.contains('\n') {
        return format!("{key}: {value}\n");
    }

    let mut s = format!("{key}:\n");
    for line in value.lines() {
        s.push_str(TITLE_INDENT);
        s.push_str(line);
        s.push('\n');
    }
    s
}

/// Cue, parentheticals and dialogue sit on consecutive lines; all else is separated.
fn same_block(prev: &Element, next: &Element) -> bool {
    let in_dialogue = matches!(next, Element::Parenthetical(_) | Element::Dialogue(_));
    let after_cue = matches!(
        prev,
        Element::Character { .. } | Element::Parenthetical(_) | Element::Dialogue(_)
    );

    in_dialogue && after_cue
}

/// The text of one element, forced where the plain form would be misread.
fn render(el: &Element, next: Option<&Element>) -> String {
    match el {
        Element::SceneHeading { text, number } => {
            let plain = match number {
                Some(n) => format!("{text} #{n}#"),
                None => text.clone(),
            };
            forced(el, &plain, &plain, FORCE_HEADING)
        }
        Element::Action(text) => forced(el, text, text, FORCE_ACTION),
        Element::Character {
            name,
            extension,
            dual,
        } => {
            let mut plain = name.clone();
            if let Some(ext) = extension {
                plain.push_str(&format!(" ({ext})"));
            }
            if *dual {
                plain.push('^');
            }
            // A cue only counts with a line below it: the dialogue that follows, or none.
            let below = match next {
                Some(Element::Parenthetical(_) | Element::Dialogue(_)) => "\nx",
                _ => "\n\n",
            };
            let probe = format!("{plain}{below}");
            forced(el, &plain, &probe, FORCE_CHARACTER)
        }
        Element::Transition(text) => forced(el, text, text, FORCE_TRANSITION),
        Element::Parenthetical(text) => format!("({text})"),
        Element::Dialogue(text) => text.clone(),
        Element::Centered(text) => format!("> {text} <"),
        Element::Section { depth, text } => format!("{} {text}", "#".repeat(*depth)),
        Element::Synopsis(text) => format!("= {text}"),
        Element::Note(text) => format!("[[{text}]]"),
        Element::PageBreak => "===".to_string(),
        Element::Lyrics(text) => format!("~{text}"),
    }
}

/// `plain` if parsing `probe` gives back `el`, else `plain` behind `marker`.
fn forced(el: &Element, plain: &str, probe: &str, marker: char) -> String {
    if parse_elements(probe).first() == Some(el) {
        return plain.to_string();
    }

    format!("{marker}{plain}")
}
