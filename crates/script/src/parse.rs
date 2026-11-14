//! Fountain text to elements.

use std::ops::Range;

use crate::{Element, Located, Script};

pub(crate) const FORCE_ACTION: char = '!';
pub(crate) const FORCE_HEADING: char = '.';
pub(crate) const FORCE_CHARACTER: char = '@';
pub(crate) const FORCE_TRANSITION: char = '>';
const CENTER_END: char = '<';
const LYRICS: char = '~';
const SECTION: char = '#';
const SYNOPSIS: char = '=';
const NUMBER_MARK: char = '#';
const DUAL_MARK: char = '^';
const SHARP_S: char = 'ß';
const PAGE_BREAK_MIN: usize = 3;
const TRANSITION_END: &str = "TO:";
const NOTE_OPEN: &str = "[[";
const NOTE_CLOSE: &str = "]]";
pub(crate) const BONEYARD_OPEN: &str = "/*";
const BONEYARD_CLOSE: &str = "*/";
/// Fountain's escape: `/\*` is the text `/*`, never a boneyard, and other Fountain
/// programs show it the same. A literal `/\*` is written `/\\*`, and so on: each `/`,
/// backslashes, `*` run carries one backslash more on paper than in the text.
const ESCAPE: char = '\\';
const STAR: char = '*';
const SLASH: char = '/';

/// Which way `escape_openers` moves a backslash run.
pub(crate) enum Escape {
    Add,
    Remove,
}

/// Prefixes of scene headings, longest first; each must be followed by `.` or a space.
const HEADING_PREFIXES: [&str; 6] = ["int./ext", "int/ext", "i/e", "int", "ext", "est"];

/// Parses Fountain text. Never fails; unknown input becomes `Element::Action`.
pub fn parse(text: &str) -> Script {
    let (text, _) = cut_boneyard(text);
    let (title_page, body) = split_title_page(&text);

    Script {
        title_page,
        elements: parse_elements(body),
    }
}

/// Like [`parse`], but each element carries the lines of `text` it stands on, so a
/// caller that shows the text can tell what each line is without redoing the rules.
///
/// Lines count from 0, in the text with CR and CRLF read as one line break. A boneyard
/// swallows its line breaks: `foo /*⏎⏎*/ bar` is one line of the parser's text and
/// stands on the line where `foo` is. The title page has no element.
pub fn parse_located(text: &str) -> Vec<Located> {
    let (text, origin) = cut_boneyard(text);
    let (_, body) = split_title_page(&text);
    let skipped = text[..text.len() - body.len()].matches('\n').count();
    let (elements, spans) = parse_with_lines(body);

    elements
        .into_iter()
        .zip(spans)
        .map(|(element, span)| Located {
            element,
            lines: span
                .filter_map(|l| origin.get(skipped + l).copied())
                .collect(),
        })
        .collect()
}

/// The text without boneyards and, for each of its lines, the line of the typed text
/// it starts on.
fn cut_boneyard(text: &str) -> (String, Vec<usize>) {
    let text = text.replace("\r\n", "\n").replace('\r', "\n");

    strip_boneyard(&text)
}

/// Cuts `/* ... */` out of the text; an unclosed one runs to the end. An escaped opener
/// (`/\*`) is no opener and reads as `/*`. Also returns the line of `text` each line of
/// the result starts on.
fn strip_boneyard(text: &str) -> (String, Vec<usize>) {
    let mut out = String::with_capacity(text.len());
    let mut origin = vec![0];
    let mut line = 0;
    let mut rest = text;

    while let Some(start) = rest.find(BONEYARD_OPEN) {
        keep(&rest[..start], &mut out, &mut origin, &mut line);
        let after = &rest[start + BONEYARD_OPEN.len()..];
        let Some(end) = after.find(BONEYARD_CLOSE) else {
            return (escape_openers(&out, Escape::Remove), origin);
        };
        // The boneyard's line breaks are gone, but the typed lines still count.
        line += after[..end].matches('\n').count();
        rest = &after[end + BONEYARD_CLOSE.len()..];
    }

    keep(rest, &mut out, &mut origin, &mut line);
    (escape_openers(&out, Escape::Remove), origin)
}

/// Appends text that stays; every line break in it starts a line of the result.
fn keep(text: &str, out: &mut String, origin: &mut Vec<usize>, line: &mut usize) {
    out.push_str(text);

    for _ in text.matches('\n') {
        *line += 1;
        origin.push(*line);
    }
}

/// Adds or removes one backslash in every `/`, backslashes, `*` run (`/*` has none), so
/// `/*` and `/\*` in the text survive a write and read apart from a real opener.
///
/// Example: `Add` turns `a /\* b` into `a /\\* b`; `Remove` turns it back.
pub(crate) fn escape_openers(text: &str, way: Escape) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;

    while let Some(slash) = rest.find(SLASH) {
        let after = &rest[slash + 1..];
        let run = after.len() - after.trim_start_matches(ESCAPE).len();
        let is_opener = after[run..].starts_with(STAR);
        let keep = match (is_opener, run, &way) {
            (false, _, _) | (_, 0, Escape::Remove) => run,
            (_, _, Escape::Add) => run + 1,
            (_, _, Escape::Remove) => run - 1,
        };

        out.push_str(&rest[..=slash]);
        out.extend(std::iter::repeat_n(ESCAPE, keep));
        rest = &after[run..];
    }

    out.push_str(rest);
    out
}

/// Splits `Key: value` lines at the top of the text from the body.
///
/// The block counts as a title page only if every line is a key line or an indented
/// continuation and at least one key has a value; otherwise `CUT TO:` would be taken
/// for a key.
fn split_title_page(text: &str) -> (Vec<(String, String)>, &str) {
    let text = text.trim_start_matches(['\n', '\r']);
    let block_end = text.find("\n\n").unwrap_or(text.len());
    let block = &text[..block_end];

    let mut pairs: Vec<(String, Vec<&str>)> = Vec::new();
    for line in block.lines() {
        if line.starts_with([' ', '\t']) {
            let Some(last) = pairs.last_mut() else {
                return (Vec::new(), text);
            };
            last.1.push(line.trim());
            continue;
        }

        let Some((key, value)) = line.split_once(':') else {
            return (Vec::new(), text);
        };
        if !is_key(key) {
            return (Vec::new(), text);
        }
        let first = value.trim();
        pairs.push((key.trim().to_string(), vec![first]));
    }

    let has_value = pairs.iter().any(|(_, v)| v.iter().any(|l| !l.is_empty()));
    if !has_value {
        return (Vec::new(), text);
    }

    let pairs = pairs
        .into_iter()
        .map(|(k, v)| {
            let lines: Vec<&str> = v.into_iter().filter(|l| !l.is_empty()).collect();
            (k, lines.join("\n"))
        })
        .collect();

    (pairs, &text[block_end..])
}

fn is_key(key: &str) -> bool {
    let key = key.trim();
    let mut chars = key.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    first.is_alphabetic() && chars.all(|c| c.is_alphanumeric() || c == ' ' || c == '_' || c == '-')
}

/// Whether the cue line is taken as written or was forced with `@`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Cue {
    Auto,
    Forced,
}

/// Blank-line context of a line; most rules need both neighbours blank.
struct Around {
    prev_blank: bool,
    next_blank: bool,
}

/// Parses text without title page and boneyard.
pub(crate) fn parse_elements(text: &str) -> Vec<Element> {
    parse_with_lines(text).0
}

/// The elements of the text and, for each, the lines it stands on.
fn parse_with_lines(text: &str) -> (Vec<Element>, Vec<Range<usize>>) {
    let lines: Vec<&str> = text.lines().map(str::trim_end).collect();
    let mut out = Vec::new();
    let mut spans = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        if is_blank(lines[i]) {
            i += 1;
            continue;
        }

        let around = Around {
            prev_blank: i == 0 || is_blank(lines[i - 1]),
            next_blank: i + 1 >= lines.len() || is_blank(lines[i + 1]),
        };
        let used = parse_at(&lines, i, &around, &mut out);

        // The elements the line took follow each other: a cue, its parentheticals and
        // its dialogue stand on consecutive lines.
        let mut line = i;
        for el in &out[spans.len()..] {
            let end = line + line_count(el);
            spans.push(line..end);
            line = end;
        }
        i += used;
    }

    (out, spans)
}

/// Number of text lines an element takes.
fn line_count(el: &Element) -> usize {
    match el {
        Element::Action(t) | Element::Dialogue(t) => t.matches('\n').count() + 1,
        _ => 1,
    }
}

fn is_blank(line: &str) -> bool {
    line.trim().is_empty()
}

/// Parses the element starting at line `i`; returns the number of lines it used.
fn parse_at(lines: &[&str], i: usize, around: &Around, out: &mut Vec<Element>) -> usize {
    let line = lines[i];
    let t = line.trim();

    if is_page_break(t) {
        out.push(Element::PageBreak);
        return 1;
    }

    if let Some(note) = note_inner(t) {
        out.push(Element::Note(note.to_string()));
        return 1;
    }

    // A forced action takes its whole paragraph, like an unforced one.
    if let Some(first) = t.strip_prefix(FORCE_ACTION) {
        return action(lines, i, first, out);
    }

    if let Some(element) = marked(t) {
        let is_cue = matches!(element, Element::Character { .. });
        out.push(element);
        if is_cue {
            return 1 + dialogue(lines, i + 1, out);
        }
        return 1;
    }

    let blank_both = around.prev_blank && around.next_blank;

    if blank_both && has_heading_prefix(t) {
        out.push(heading(t));
        return 1;
    }

    if blank_both && is_upper(t) && t.ends_with(TRANSITION_END) {
        out.push(Element::Transition(t.to_string()));
        return 1;
    }

    if around.prev_blank && !around.next_blank {
        if let Some(cue) = cue(t, Cue::Auto) {
            out.push(cue);
            return 1 + dialogue(lines, i + 1, out);
        }
    }

    action(lines, i, line, out)
}

/// Elements that announce themselves with a leading character.
fn marked(t: &str) -> Option<Element> {
    let first = t.chars().next()?;
    let rest = &t[first.len_utf8()..];

    match first {
        SECTION => {
            let depth = t.chars().take_while(|&c| c == SECTION).count();
            Some(Element::Section {
                depth,
                text: t[depth..].trim().to_string(),
            })
        }
        SYNOPSIS => Some(Element::Synopsis(rest.trim().to_string())),
        // `...` is an ellipsis, not a heading.
        FORCE_HEADING if !rest.starts_with(FORCE_HEADING) => Some(heading(rest.trim())),
        FORCE_CHARACTER => cue(rest.trim(), Cue::Forced),
        LYRICS => Some(Element::Lyrics(rest.trim().to_string())),
        FORCE_TRANSITION => Some(forced_transition(rest)),
        _ => None,
    }
}

/// `>text<` is centered, any other `>text` a transition.
fn forced_transition(rest: &str) -> Element {
    match rest.strip_suffix(CENTER_END) {
        Some(inner) => Element::Centered(inner.trim().to_string()),
        None => Element::Transition(rest.trim().to_string()),
    }
}

fn is_page_break(t: &str) -> bool {
    t.len() >= PAGE_BREAK_MIN && t.chars().all(|c| c == SYNOPSIS)
}

/// The text of a `[[note]]` that fills the whole line.
fn note_inner(t: &str) -> Option<&str> {
    let inner = t.strip_prefix(NOTE_OPEN)?.strip_suffix(NOTE_CLOSE)?;
    if inner.contains(NOTE_OPEN) || inner.contains(NOTE_CLOSE) {
        return None;
    }

    Some(inner)
}

fn has_heading_prefix(t: &str) -> bool {
    let lower = t.to_lowercase();

    HEADING_PREFIXES.iter().any(|p| {
        lower
            .strip_prefix(p)
            .is_some_and(|rest| rest.starts_with(['.', ' ']))
    })
}

/// Splits a trailing `#12A#` off the heading.
fn heading(t: &str) -> Element {
    let split = t.strip_suffix(NUMBER_MARK).and_then(|body| {
        let at = body.rfind(NUMBER_MARK)?;
        let number = &body[at + 1..];
        if number.is_empty() || number.contains(char::is_whitespace) {
            return None;
        }
        Some((body[..at].trim_end(), number))
    });

    match split {
        Some((text, number)) => Element::SceneHeading {
            text: text.to_string(),
            number: Some(number.to_string()),
        },
        None => Element::SceneHeading {
            text: t.to_string(),
            number: None,
        },
    }
}

/// No lowercase letter and at least one letter.
fn is_upper(s: &str) -> bool {
    // `ß` has no uppercase form in everyday use, so it counts as uppercase.
    let lower = |c: char| c.is_lowercase() && c != SHARP_S;

    s.chars().any(char::is_alphabetic) && !s.chars().any(lower)
}

/// Parses `NAME (EXT)^`; `Auto` also demands an all-uppercase name.
fn cue(t: &str, mode: Cue) -> Option<Element> {
    let (t, dual) = match t.strip_suffix(DUAL_MARK) {
        Some(rest) => (rest.trim_end(), true),
        None => (t, false),
    };

    let (name, extension) = match t.strip_suffix(')').and_then(|b| b.rsplit_once('(')) {
        Some((name, ext)) => (name.trim_end(), Some(ext.trim().to_string())),
        None => (t, None),
    };

    if name.is_empty() {
        return None;
    }
    // An unforced cue is not a heading that lacks the blank line below.
    if mode == Cue::Auto && (!is_upper(name) || has_heading_prefix(name)) {
        return None;
    }

    Some(Element::Character {
        name: name.to_string(),
        extension,
        dual,
    })
}

/// Parses the lines after a cue up to the next blank line; returns the lines used.
fn dialogue(lines: &[&str], from: usize, out: &mut Vec<Element>) -> usize {
    let mut used = 0;
    let mut text: Vec<&str> = Vec::new();

    for line in &lines[from..] {
        if is_blank(line) {
            break;
        }
        used += 1;

        let t = line.trim();
        let inner = t.strip_prefix('(').and_then(|r| r.strip_suffix(')'));
        let Some(inner) = inner else {
            text.push(t);
            continue;
        };

        flush_dialogue(&mut text, out);
        out.push(Element::Parenthetical(inner.trim().to_string()));
    }

    flush_dialogue(&mut text, out);
    used
}

fn flush_dialogue(text: &mut Vec<&str>, out: &mut Vec<Element>) {
    if text.is_empty() {
        return;
    }

    out.push(Element::Dialogue(text.join("\n")));
    text.clear();
}

/// Takes the paragraph from line `i` up to the next blank line as one action.
///
/// `first` replaces line `i`, so a forced action can drop its `!`.
fn action(lines: &[&str], i: usize, first: &str, out: &mut Vec<Element>) -> usize {
    let used = lines[i..].iter().take_while(|l| !is_blank(l)).count();
    let mut text = vec![first];
    text.extend_from_slice(&lines[i + 1..i + used]);
    out.push(Element::Action(text.join("\n")));
    used
}
