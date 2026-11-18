//! Fountain text to elements.

use std::ops::Range;

use crate::{Element, Located, Script};

pub(crate) const FORCE_ACTION: char = '!';
pub(crate) const FORCE_HEADING: char = '.';
pub const FORCE_CHARACTER: char = '@';
pub(crate) const FORCE_TRANSITION: char = '>';
const CENTER_END: char = '<';
const LYRICS: char = '~';
const SECTION: char = '#';
const SYNOPSIS: char = '=';
pub const NUMBER_MARK: char = '#';
pub(crate) const DUAL_MARK: char = '^';
const SHARP_S: char = 'ß';
const PAGE_BREAK_MIN: usize = 3;
const TRANSITION_END: &str = "TO:";
pub(crate) const NOTE_OPEN: &str = "[[";
pub(crate) const NOTE_CLOSE: &str = "]]";
pub(crate) const BONEYARD_OPEN: &str = "/*";
pub(crate) const BONEYARD_CLOSE: &str = "*/";
/// Fountain's escape: `/\*` is the text `/*`, never a boneyard, and other Fountain
/// programs show it the same. A literal `/\*` is written `/\\*`, and so on: each `/`,
/// backslashes, `*` run carries one backslash more on paper than in the text.
pub(crate) const ESCAPE: char = '\\';
pub(crate) const STAR: char = '*';
const SLASH: char = '/';

/// Which way `escape_openers` moves a backslash run.
pub(crate) enum Escape {
    Add,
    Remove,
}

/// Prefixes of scene headings in both languages, longest first; each must be followed by `.` or
/// a space.
pub(crate) const HEADING_PREFIXES: [&str; 13] = [
    "int./ext",
    "int/ext",
    "innen/aussen",
    "innen/außen",
    "aussen/innen",
    "außen/innen",
    "i/e",
    "int",
    "ext",
    "est",
    "innen",
    "aussen",
    "außen",
];

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

/// Per line of `text`: it starts inside a `/* … */` an earlier line opened. Lines count
/// as in [`parse_located`], so an editor can style them without reading the rules again.
pub fn boneyard_lines(text: &str) -> Vec<bool> {
    let (_, origin) = cut_boneyard(text);
    let lines = text.replace("\r\n", "\n").replace('\r', "\n");
    let mut inside = vec![true; lines.matches('\n').count() + 1];

    // A line that starts a line of the reader's text is outside; the rest was cut.
    for line in origin {
        inside[line] = false;
    }

    inside
}

/// Lines of `text` the title page takes, leading blank lines included; 0 without one. A
/// block that ends at the end of the text counts its last line break as a line.
///
/// A line count of the typed text, so an editor can style those lines as the title page.
pub fn title_page_lines(text: &str) -> usize {
    title_page_span(text).map_or(0, |(_, end)| end)
}

/// The lines of `text` from the first key line of the title page to the end of its block
/// (`start..end`), in typed lines, so what stands before it (a boneyard) is not in it.
pub(crate) fn title_page_span(text: &str) -> Option<(usize, usize)> {
    let (text, origin) = cut_boneyard(text);
    let (title_page, body) = split_title_page(&text);
    if title_page.is_empty() {
        return None;
    }

    let skipped = text[..text.len() - body.len()].matches('\n').count();
    let leading = text.len() - text.trim_start_matches(['\n', '\r']).len();
    let first = text[..leading].matches('\n').count();
    let total = origin.last().map_or(0, |last| last + 1);
    let start = origin.get(first).copied().unwrap_or(total);
    // The end is where the next line of the reader's text starts, so a boneyard that ends
    // on the last title line is inside the span.
    let end = origin.get(skipped + 1).copied().unwrap_or(total);

    Some((start, end))
}

/// The text without boneyards and, for each of its lines, the line of the typed text
/// it starts on.
fn cut_boneyard(text: &str) -> (String, Vec<usize>) {
    let text = text.replace("\r\n", "\n").replace('\r', "\n");

    strip_boneyard(&text)
}

/// Cuts `/* ... */` out of the text; an unclosed one runs to the end. An escaped opener
/// (`/\*`) is no opener and reads as `/*`. A `/*` inside a note `[[ ... ]]` on one line
/// is note text, no opener. Also returns the line of `text` each line of the result
/// starts on.
fn strip_boneyard(text: &str) -> (String, Vec<usize>) {
    let mut out = String::with_capacity(text.len());
    let mut origin = vec![0];
    let mut line = 0;
    let mut at = 0;

    for span in boneyard_spans(text) {
        keep(
            &unescape_opener(&text[at..span.start]),
            &mut out,
            &mut origin,
            &mut line,
        );
        // The boneyard's line breaks are gone, but the typed lines still count.
        line += text[span.clone()].matches('\n').count();
        at = span.end;
    }

    keep(
        &unescape_opener(&text[at..]),
        &mut out,
        &mut origin,
        &mut line,
    );
    (escape_openers(&out, Escape::Remove), origin)
}

/// Byte ranges of the `/* ... */` comments in `text`, openers and closers included; an
/// unclosed one runs to the end. The one place that says what a boneyard is.
pub(crate) fn boneyard_spans(text: &str) -> Vec<Range<usize>> {
    let mut spans = Vec::new();
    let mut at = 0;

    while let Some(found) = text[at..].find(BONEYARD_OPEN) {
        let rest = &text[at..];
        if let Some(end) = note_end(rest, found) {
            at += end;
            continue;
        }

        let start = at + found;
        if is_escaped(text, start) {
            at = start + BONEYARD_OPEN.len();
            continue;
        }

        let after = start + BONEYARD_OPEN.len();
        let end = text[after..]
            .find(BONEYARD_CLOSE)
            .map_or(text.len(), |close| after + close + BONEYARD_CLOSE.len());
        spans.push(start..end);
        at = end;
    }

    spans
}

/// Whether the character at `at` follows an odd run of backslashes: `\/*` escapes the
/// slash, `\\/*` is a backslash and an opener. Matches how `inline_runs` skips escapes.
fn is_escaped(text: &str, at: usize) -> bool {
    let run = text[..at]
        .chars()
        .rev()
        .take_while(|&c| c == ESCAPE)
        .count();

    run % 2 == 1
}

/// Turns the escaped opener `\/*` into the text `/*`, as the escape says.
fn unescape_opener(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut at = 0;

    while let Some(found) = text[at..].find(BONEYARD_OPEN) {
        let start = at + found;
        let cut = if is_escaped(text, start) {
            ESCAPE.len_utf8()
        } else {
            0
        };

        out.push_str(&text[at..start - cut]);
        out.push_str(BONEYARD_OPEN);
        at = start + BONEYARD_OPEN.len();
    }

    out.push_str(&text[at..]);
    out
}

/// Where the note ends that holds the opener at `start` of `rest`, if one does. A note
/// closes on its own line: `[[a /* b]]` is a note, `[[a /* b` is not.
fn note_end(rest: &str, start: usize) -> Option<usize> {
    let mut at = 0;

    while let Some(found) = rest[at..start].find(NOTE_OPEN) {
        let body_at = at + found + NOTE_OPEN.len();
        let body = &rest[body_at..];
        let line = body.find('\n').unwrap_or(body.len());
        let Some(close) = body[..line].find(NOTE_CLOSE) else {
            at = body_at;
            continue;
        };

        let end = body_at + close + NOTE_CLOSE.len();
        if end > start {
            return Some(end);
        }
        at = end;
    }

    None
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
    heading_prefix(t).is_some()
}

/// The heading prefix `t` starts with, followed by `.` or a space. The German prefixes are
/// words of prose too ("Innen ist es kalt."): before a space they count only in capitals,
/// e.g. "INNEN KÜCHE" or "innen. küche".
pub(crate) fn heading_prefix(t: &str) -> Option<&'static str> {
    let lower = t.to_lowercase();

    HEADING_PREFIXES.iter().copied().find(|p| {
        let Some(rest) = lower.strip_prefix(p) else {
            return false;
        };
        if rest.starts_with('.') {
            return true;
        }
        if !rest.starts_with(' ') {
            return false;
        }

        !is_german(p) || in_capitals(t.split(' ').next().unwrap_or(t))
    })
}

fn is_german(prefix: &str) -> bool {
    ["innen", "aussen", "außen"]
        .iter()
        .any(|w| prefix.starts_with(w))
}

/// Whether a word is written in capitals; `ß` has no common capital and is allowed.
fn in_capitals(word: &str) -> bool {
    !word.chars().any(|c| c.is_lowercase() && c != 'ß')
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

    // `ANNA (V.O.) (FORTS.)`: every trailing group is an extension, kept as `V.O.) (FORTS.`
    // so the writer's `({extension})` gives the line back.
    let mut name = t;
    let mut extensions: Vec<&str> = Vec::new();
    while let Some((head, ext)) = name.strip_suffix(')').and_then(|b| b.rsplit_once('(')) {
        extensions.insert(0, ext.trim());
        name = head.trim_end();
    }
    let extension = (!extensions.is_empty()).then(|| extensions.join(") ("));

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
