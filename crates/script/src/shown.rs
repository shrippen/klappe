//! The screenplay as the editor shows it (D-36): per source line the text without the marks
//! of Fountain, and the way back from an edited line to the source.
//!
//! Fountain stays the source; a source line is one editor line, blank ones too. Only a line
//! that is edited is written again, the others stay as they are.
//!
//! ```text
//! source line    @mcClane  says *no* [[Regie]] ^
//! shown text     mcClane  says no                 (the cue, emphasis and note are not text)
//! hidden         `@`, `*` `*`, `[[Regie]]` at 15, ` ^`
//! ```
//!
//! What a line hides is read again from the old source line on every edit, so an edit hands
//! over only the new text and its emphasis. What the user types stays Fountain: a typed
//! `*x*` is emphasis the next time the line is shown.

use std::ops::Range;

use crate::inline::{runs, seen_chars, NOTE_BRACKET, UNDERLINE};
use crate::parse::{
    boneyard_lines, split_number, CENTER_END, DUAL_MARK, ESCAPE, FORCE_ACTION, FORCE_CHARACTER,
    FORCE_HEADING, FORCE_TRANSITION, LYRICS, NOTE_CLOSE, NOTE_OPEN, NUMBER_MARK, STAR,
};
use crate::{text_kinds, Marks, Run, Splice, TextKind};

const SLASH: char = '/';

/// Whether the editor shows the line without its marks or as it is typed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    Shown,
    /// Blank lines, the title page, sections, synopses, notes, page breaks and what a
    /// boneyard touches (D-36, open question: these may show their marks for now).
    Typed,
}

/// Whether the line appears in the editor at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hidden {
    No,
    /// A line of the title page, which only the card shows (D-30).
    TitlePage,
}

/// A note of a line: it stands before the character `at` of the shown text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShownNote {
    pub at: usize,
    /// The text between the brackets.
    pub text: String,
}

/// One source line as the editor shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShownLine {
    pub kind: TextKind,
    pub shape: Shape,
    pub hidden: Hidden,
    pub text: String,
    /// The emphasis of the text, by character; the runs cover the text.
    pub runs: Vec<Run>,
    /// The scene number (`12A`, without the marks) of a heading.
    pub number: Option<String>,
    pub notes: Vec<ShownNote>,
    /// Per character of the text the byte where it stands in the source line, and the
    /// byte where the text ends as a last entry. A character after a backslash (`\*`)
    /// stands at itself. The window counts in UTF-16, the caller converts.
    pub cols: Vec<usize>,
}

/// What a source line has: the part the editor shows (`start..end`, the rest is hidden), and
/// the scene number a heading ends with.
struct Split {
    start: usize,
    end: usize,
    number: Option<String>,
}

/// The shown part of a line, character by character.
#[derive(Default)]
struct Body {
    chars: Vec<char>,
    marks: Vec<Marks>,
    /// Which characters stood after a backslash.
    escaped: Vec<bool>,
    cols: Vec<usize>,
    notes: Vec<ShownNote>,
    boneyard: bool,
}

/// The lines of `source` (split at `\n`) as the editor shows them.
pub fn shown_lines(source: &str) -> Vec<ShownLine> {
    let kinds = text_kinds(source);
    let inside = boneyard_lines(source);

    source
        .split('\n')
        .zip(kinds)
        .enumerate()
        .map(|(at, (line, kind))| {
            let in_boneyard = inside.get(at).copied().unwrap_or(false);

            shown_line(line, kind, in_boneyard)
        })
        .collect()
}

fn shown_line(line: &str, kind: TextKind, in_boneyard: bool) -> ShownLine {
    let hidden = match kind {
        TextKind::Title => Hidden::TitlePage,
        _ => Hidden::No,
    };
    let split = split_line(line, kind).filter(|_| !in_boneyard);
    let body = split.as_ref().map(|s| read_body(line, s.start..s.end));

    let (Some(split), Some(body)) = (split, body.filter(|b| !b.boneyard)) else {
        return typed_line(line, kind, hidden);
    };

    ShownLine {
        kind,
        shape: Shape::Shown,
        hidden,
        text: body.chars.iter().collect(),
        runs: runs(&body.marks),
        number: split.number,
        notes: body.notes,
        cols: body.cols,
    }
}

/// A line the editor shows as typed: all of it is the text.
fn typed_line(line: &str, kind: TextKind, hidden: Hidden) -> ShownLine {
    let mut cols: Vec<usize> = line.char_indices().map(|(at, _)| at).collect();
    cols.push(line.len());
    let marks = vec![Marks::default(); line.chars().count()];

    ShownLine {
        kind,
        shape: Shape::Typed,
        hidden,
        text: line.to_owned(),
        runs: runs(&marks),
        number: None,
        notes: Vec::new(),
        cols,
    }
}

/// Where the shown part of `line` lies; `None` for a kind the editor shows as typed.
///
/// A forcing character and a mark at the end with the one blank before it are hidden; every
/// other blank is text, so a blank the user types is not lost when the line is shown again.
///
/// ```text
/// forced heading   .INT. KÜCHE - TAG #12A#      hidden: `.` and ` #12A#`
/// forced cue       @mcClane ^                   hidden: `@` and ` ^`
/// centered         >ENDE<                       hidden: `>` and `<`
/// ```
fn split_line(line: &str, kind: TextKind) -> Option<Split> {
    let end = line.trim_end().len();
    let lead = (line.len() - line.trim_start().len()).min(end);
    let text = &line[lead..end];
    // The byte behind `mark` when it opens the line, else 0.
    let after = |mark: char| match text.starts_with(mark) {
        true => lead + mark.len_utf8(),
        false => 0,
    };
    let mut split = Split {
        start: 0,
        end: line.len(),
        number: None,
    };

    match kind {
        TextKind::Heading => {
            if text.starts_with(FORCE_HEADING) && !text.starts_with("..") {
                split.start = after(FORCE_HEADING);
            }
            if let (_, Some(number)) = split_number(&line[split.start..end]) {
                let token = number.len() + 2 * NUMBER_MARK.len_utf8();
                split.end = gap_start(line, split.start, end - token);
                split.number = Some(number.to_owned());
            }
        }
        TextKind::Action => split.start = after(FORCE_ACTION),
        TextKind::Character => {
            split.start = after(FORCE_CHARACTER);
            split.end = before_mark(line, split.start, end, DUAL_MARK);
        }
        TextKind::Transition => split.start = after(FORCE_TRANSITION),
        TextKind::Centered => {
            split.start = after(FORCE_TRANSITION);
            split.end = before_mark(line, split.start, end, CENTER_END);
        }
        TextKind::Lyrics => split.start = after(LYRICS),
        TextKind::Parenthetical | TextKind::Dialogue => {}
        TextKind::Blank
        | TextKind::Title
        | TextKind::Section
        | TextKind::Synopsis
        | TextKind::Note
        | TextKind::PageBreak => return None,
    }

    Some(split)
}

/// Where the text ends before something that stands at byte `at` of the line: one blank
/// that leads up to it belongs to it.
fn gap_start(line: &str, start: usize, at: usize) -> usize {
    match line[start..at].chars().next_back() {
        Some(blank) if blank.is_whitespace() => at - blank.len_utf8(),
        _ => at,
    }
}

/// Where the text ends when `mark` closes the line (`ANNA ^`); the end of the line without it.
fn before_mark(line: &str, start: usize, end: usize, mark: char) -> usize {
    match line[start..end].ends_with(mark) {
        true => gap_start(line, start, end - mark.len_utf8()),
        false => line.len(),
    }
}

/// The characters of `line[range]` as a reader sees them, notes taken out.
fn read_body(line: &str, range: Range<usize>) -> Body {
    let base = range.start;
    let mut body = Body::default();
    let mut note = String::new();

    for seen in seen_chars(&line[range.clone()]) {
        if seen.marks.boneyard {
            body.boneyard = true;
        }
        if seen.marks.note {
            note.push(seen.ch);
            continue;
        }
        take_notes(&mut note, body.chars.len(), &mut body.notes);

        body.chars.push(seen.ch);
        body.marks.push(emphasis(seen.marks));
        body.escaped.push(seen.escaped);
        body.cols.push(base + seen.at);
    }
    take_notes(&mut note, body.chars.len(), &mut body.notes);
    body.cols.push(range.end);

    body
}

/// Only what shows on the page is a mark of the text.
fn emphasis(marks: Marks) -> Marks {
    Marks {
        note: false,
        boneyard: false,
        ..marks
    }
}

/// The notes in `raw` (`[[a]][[b]]`), put before character `at`; empties `raw`.
fn take_notes(raw: &mut String, at: usize, notes: &mut Vec<ShownNote>) {
    let mut rest = raw.as_str();

    while let Some(inner) = rest.strip_prefix(NOTE_OPEN) {
        let Some(end) = inner.find(NOTE_CLOSE) else {
            break;
        };
        notes.push(ShownNote {
            at,
            text: inner[..end].to_owned(),
        });
        rest = &inner[end + NOTE_CLOSE.len()..];
    }
    raw.clear();
}

/// The source line for the edited text of a line that was `old`, of `kind` and `shape` when
/// it was shown.
///
/// What the line hid stays, the number, the forcing character and the notes at their place
/// (a note moves with the text typed before it); the emphasis `runs` becomes Fountain marks
/// in the canonical order (`_` outside, then `***`, `**`, `*`). Characters the user typed
/// stay as typed, so `*x*` becomes italic the next time. A character that stood after a
/// backslash and was not touched keeps it. Where two marks cannot stand side by side
/// (`**a***b***` reads as stars) or a `/` stands before one (`/*` opens a boneyard), the
/// later run is written without emphasis. Clearing the whole text clears the line, so an
/// emptied heading does not keep its number; to remove a line, use [`splice_lines`].
///
/// `text` is one line.
pub fn edit_line(old: &str, kind: TextKind, shape: Shape, text: &str, runs: &[Run]) -> String {
    let split = split_line(old, kind).filter(|_| shape == Shape::Shown);
    let Some(split) = split else {
        return text.to_owned();
    };

    let before = read_body(old, split.start..split.end);
    let after: Vec<char> = text.chars().collect();
    let marks = marks_of(runs, after.len());

    // The line is not touched: it stays as it is, even if it is not canonical.
    if after == before.chars && marks == before.marks {
        return old.to_owned();
    }
    if after.is_empty() && before.notes.is_empty() {
        return String::new();
    }

    rewrite(old, &split, &before, &after, &marks)
}

/// The line `old` written again for the text `after` with `marks`.
fn rewrite(old: &str, split: &Split, before: &Body, after: &[char], marks: &[Marks]) -> String {
    let (kept_start, kept_end) = common_ends(&before.chars, after);
    let tail = after.len() - kept_end;
    let escaped: Vec<bool> = (0..after.len())
        .map(|i| match i {
            _ if i < kept_start => before.escaped[i],
            _ if i >= tail => before.escaped[before.chars.len() - (after.len() - i)],
            _ => false,
        })
        .collect();
    let same_text = kept_start + kept_end >= before.chars.len().max(after.len());

    let part = Part {
        chars: after,
        marks,
        escaped: &escaped,
        notes: &moved_notes(before, after.len(), kept_start, kept_end),
    };
    let mut body = write_body(&part);

    // Text that is not new reads as it is shown, whatever the marks do to each other.
    if same_text && !reads_as(&body, &part) {
        body = write_body(&Part {
            marks: &vec![Marks::default(); after.len()],
            escaped: &vec![true; after.len()],
            ..part
        });
    }

    format!("{}{body}{}", &old[..split.start], &old[split.end..])
}

/// How many characters at the start and at the end `a` and `b` share; they do not overlap.
fn common_ends(a: &[char], b: &[char]) -> (usize, usize) {
    let most = a.len().min(b.len());
    let start = a.iter().zip(b).take_while(|(x, y)| x == y).count();
    let end = a
        .iter()
        .rev()
        .zip(b.iter().rev())
        .take(most - start)
        .take_while(|(x, y)| x == y)
        .count();

    (start, end)
}

/// The notes of `before` in the text that has `len` characters now: one in the unchanged end
/// goes with it, one in the unchanged start stays, one in what changed goes to its start.
fn moved_notes(before: &Body, len: usize, kept_start: usize, kept_end: usize) -> Vec<ShownNote> {
    let old_len = before.chars.len();

    before
        .notes
        .iter()
        .map(|note| {
            let at = match note.at {
                at if at >= old_len - kept_end => len - (old_len - at),
                at if at <= kept_start => at,
                _ => kept_start,
            };

            ShownNote {
                at,
                text: note.text.clone(),
            }
        })
        .collect()
}

/// The emphasis of each of `len` characters; what no run covers is plain.
fn marks_of(runs: &[Run], len: usize) -> Vec<Marks> {
    let mut marks = vec![Marks::default(); len];

    for run in runs {
        for mark in marks.iter_mut().take(run.end).skip(run.start) {
            *mark = emphasis(run.marks);
        }
    }

    marks
}

/// What a shown text is written from.
struct Part<'a> {
    chars: &'a [char],
    marks: &'a [Marks],
    escaped: &'a [bool],
    notes: &'a [ShownNote],
}

/// Whether `body` read again gives the text, emphasis and notes of `part`.
fn reads_as(body: &str, part: &Part) -> bool {
    let again = read_body(body, 0..body.len());

    !again.boneyard
        && again.chars == part.chars
        && again.marks == part.marks
        && again.notes == part.notes
}

/// The emphasis that one marked stretch has, as a segment of the written text.
#[derive(Clone, Copy, PartialEq, Eq)]
struct Emphasis {
    marks: Marks,
    applied: bool,
}

const PLAIN: Marks = Marks {
    bold: false,
    italic: false,
    underline: false,
    note: false,
    boneyard: false,
};

/// The text of `part` as Fountain, with a mark around every stretch of emphasis.
fn write_body(part: &Part) -> String {
    let mut out = String::new();
    let mut tail = Tail::Free;
    let mut now = Emphasis {
        marks: PLAIN,
        applied: false,
    };

    for i in 0..=part.chars.len() {
        let want = part.marks.get(i).copied().unwrap_or(PLAIN);
        let changed = want != now.marks;
        if changed {
            if now.applied {
                tail = tail_after_close(now.marks);
            }
            close(&mut out, now);
        }

        // A note between two stretches stands outside both.
        for note in part
            .notes
            .iter()
            .filter(|n| n.at.min(part.chars.len()) == i)
        {
            out.push_str(&format!("{NOTE_OPEN}{}{NOTE_CLOSE}", note.text));
            tail = Tail::Free;
        }
        if changed {
            now = open(&mut out, want, tail);
        }

        let Some(&ch) = part.chars.get(i) else {
            break;
        };
        let protected = part.escaped[i] && is_protected(ch);
        if protected {
            out.push(ESCAPE);
        }
        out.push(ch);
        tail = match ch {
            STAR if !protected => Tail::Star,
            SLASH => Tail::Slash,
            _ => Tail::Free,
        };
    }

    out
}

/// What the written text ends with, as far as a mark that follows cares.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Tail {
    Free,
    /// A `*` that is no escaped one: stars after it would join it.
    Star,
    /// A `/`: `/*` opens a boneyard.
    Slash,
}

/// What the text ends with after the stars of a closed stretch.
fn tail_after_close(marks: Marks) -> Tail {
    match stars(marks).is_empty() || marks.underline {
        true => Tail::Free,
        false => Tail::Star,
    }
}

/// A character a backslash can protect.
fn is_protected(ch: char) -> bool {
    matches!(ch, STAR | UNDERLINE | ESCAPE | NOTE_BRACKET)
}

fn stars(marks: Marks) -> &'static str {
    match (marks.bold, marks.italic) {
        (true, true) => "***",
        (true, false) => "**",
        (false, true) => "*",
        (false, false) => "",
    }
}

/// Opens `want`. Stars that would touch the stars before them, or follow a `/`, are left out
/// and the stretch stays plain.
fn open(out: &mut String, want: Marks, tail: Tail) -> Emphasis {
    let blocked = !stars(want).is_empty() && !want.underline && tail != Tail::Free;
    if want == PLAIN || blocked {
        return Emphasis {
            marks: want,
            applied: false,
        };
    }

    if want.underline {
        out.push(UNDERLINE);
    }
    out.push_str(stars(want));

    Emphasis {
        marks: want,
        applied: true,
    }
}

fn close(out: &mut String, now: Emphasis) {
    if !now.applied {
        return;
    }

    out.push_str(stars(now.marks));
    if now.marks.underline {
        out.push(UNDERLINE);
    }
}

/// `source` with the lines `lines` replaced by `new`, as the lines that change: the lines
/// both sides share at the start and the end are left out, so a line that stays is never
/// part of the splice. An empty range inserts, an empty `new` deletes. A range past the end
/// is cut to the lines there are.
pub fn splice_lines(source: &str, lines: Range<usize>, new: &[String]) -> Splice {
    let old: Vec<&str> = source.split('\n').collect();
    let from = lines.start.min(old.len());
    let to = lines.end.clamp(from, old.len());
    let old = &old[from..to];

    let head = old
        .iter()
        .zip(new)
        .take_while(|(a, b)| **a == b.as_str())
        .count();
    let tail = old[head..]
        .iter()
        .rev()
        .zip(new[head..].iter().rev())
        .take_while(|(a, b)| **a == b.as_str())
        .count();

    Splice {
        from: from + head,
        to: to - tail,
        lines: new[head..new.len() - tail].to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LCG_MUL: u64 = 6_364_136_223_846_793_005;
    const LCG_ADD: u64 = 1_442_695_040_888_963_407;
    const CASES: u64 = 3000;
    const PARTS: [&str; 20] = [
        "Anna", "geht", "ÄÖÜ", "🎬", "ANNA", "INT.", " ", " ", "*", "**", "_", "\\*", "\\", "#1#",
        "[[n]]", ".", "@", "^", "/", ">",
    ];

    fn line(seed: &mut u64) -> String {
        let mut next = |below: usize| {
            *seed = seed.wrapping_mul(LCG_MUL).wrapping_add(LCG_ADD);

            ((*seed >> 33) as usize) % below
        };
        let count = 1 + next(7);

        (0..count).map(|_| PARTS[next(PARTS.len())]).collect()
    }

    /// The writer without the early return for a line that is not touched: every line it
    /// writes again says what it said, as far as the line alone shows.
    #[test]
    fn the_writer_keeps_what_a_line_says() {
        let mut seed = 7;
        let mut lines = 0;
        let mut differs = 0;

        for _ in 0..CASES {
            let line = line(&mut seed);
            let kind = text_kinds(&line)[0];
            let shown = shown_line(&line, kind, false);
            let Some(split) = split_line(&line, kind).filter(|_| shown.shape == Shape::Shown)
            else {
                continue;
            };

            let before = read_body(&line, split.start..split.end);
            let chars: Vec<char> = shown.text.chars().collect();
            let marks = marks_of(&shown.runs, chars.len());
            let again = rewrite(&line, &split, &before, &chars, &marks);
            let reread = shown_line(&again, kind, false);

            lines += 1;
            assert_eq!(reread.notes, shown.notes, "{line:?} -> {again:?}");
            if reread.kind == kind && reread.shape == Shape::Shown {
                assert_eq!(reread.text, shown.text, "{line:?} -> {again:?}");
            }
            if reread.runs != shown.runs || reread.kind != kind {
                differs += 1;
            }
        }

        // A mark around nothing, or marks that touch, are written another way.
        assert!(lines > CASES as usize / 2, "{lines} lines");
        assert!(differs * 20 < lines, "{differs} of {lines} differ");
    }
}
