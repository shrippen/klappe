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

use crate::inline::{all_chars, runs, NOTE_BRACKET, UNDERLINE};
use crate::lcs::unpaired;
use crate::parse::{
    boneyard_lines, split_number, CENTER_END, DUAL_MARK, ESCAPE, FORCE_ACTION, FORCE_CHARACTER,
    FORCE_HEADING, FORCE_TRANSITION, LYRICS, NOTE_CLOSE, NOTE_OPEN, NUMBER_MARK, STAR,
};
use crate::{text_kinds, Marks, Run, Splice, TextKind};

const SLASH: char = '/';

/// What the editor leaves out of the text of a line; what a line does not hide stays in the
/// text as typed. Step by step the editor hides more.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hide {
    /// The marks of emphasis (`*`, `_`) and the backslash of `\*`.
    Emphasis,
    /// The character that forces an element (`.`, `!`, `@`, `~`, `>`) and the `^` or `<`
    /// that ends a line.
    Forcing,
    /// The scene number `#12A#` of a heading.
    Number,
    /// Notes `[[…]]` inside a line.
    Notes,
    /// The lines of the title page.
    TitlePage,
}

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

/// The lines of `source` (split at `\n`) as the editor shows them, with what `hide` names
/// left out of the text.
pub fn shown_lines(source: &str, hide: &[Hide]) -> Vec<ShownLine> {
    let kinds = text_kinds(source);
    let inside = boneyard_lines(source);

    source
        .split('\n')
        .zip(kinds)
        .enumerate()
        .map(|(at, (line, kind))| {
            let in_boneyard = inside.get(at).copied().unwrap_or(false);

            shown_line(line, kind, in_boneyard, hide)
        })
        .collect()
}

fn shown_line(line: &str, kind: TextKind, in_boneyard: bool, hide: &[Hide]) -> ShownLine {
    let hidden = match kind == TextKind::Title && hide.contains(&Hide::TitlePage) {
        true => Hidden::TitlePage,
        false => Hidden::No,
    };
    let split = split_line(line, kind, hide).filter(|_| !in_boneyard);
    let body = split
        .as_ref()
        .map(|s| read_body(line, s.start..s.end, hide));

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
fn split_line(line: &str, kind: TextKind, hide: &[Hide]) -> Option<Split> {
    let end = line.trim_end().len();
    let lead = (line.len() - line.trim_start().len()).min(end);
    let text = &line[lead..end];
    // The byte behind `mark` when it opens the line, else 0.
    let forcing = hide.contains(&Hide::Forcing);
    let after = |mark: char| match forcing && text.starts_with(mark) {
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
            if !text.starts_with("..") {
                split.start = after(FORCE_HEADING);
            }
            if let (_, Some(number)) = split_number(&line[split.start..end]) {
                let token = number.len() + 2 * NUMBER_MARK.len_utf8();
                if hide.contains(&Hide::Number) {
                    split.end = gap_start(line, split.start, end - token);
                }
                split.number = Some(number.to_owned());
            }
        }
        TextKind::Action => split.start = after(FORCE_ACTION),
        TextKind::Character => {
            split.start = after(FORCE_CHARACTER);
            split.end = before_mark(line, split.start, end, DUAL_MARK, forcing);
        }
        TextKind::Transition => split.start = after(FORCE_TRANSITION),
        TextKind::Centered => {
            split.start = after(FORCE_TRANSITION);
            split.end = before_mark(line, split.start, end, CENTER_END, forcing);
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
fn before_mark(line: &str, start: usize, end: usize, mark: char, forcing: bool) -> usize {
    match forcing && line[start..end].ends_with(mark) {
        true => gap_start(line, start, end - mark.len_utf8()),
        false => line.len(),
    }
}

/// The characters of `line[range]` as a reader sees them, with what `hide` names taken out.
fn read_body(line: &str, range: Range<usize>, hide: &[Hide]) -> Body {
    let base = range.start;
    let (emphasis_hidden, notes_hidden) =
        (hide.contains(&Hide::Emphasis), hide.contains(&Hide::Notes));
    let mut body = Body::default();
    let mut note = String::new();

    for seen in all_chars(&line[range.clone()]) {
        if seen.marks.boneyard {
            body.boneyard = true;
        }
        if seen.marks.note && notes_hidden {
            note.push(seen.ch);
            continue;
        }
        take_notes(&mut note, body.chars.len(), &mut body.notes);
        if seen.is_mark && emphasis_hidden {
            continue;
        }

        body.chars.push(seen.ch);
        body.marks.push(match emphasis_hidden {
            true => emphasis(seen.marks),
            false => Marks::default(),
        });
        body.escaped.push(emphasis_hidden && seen.escaped);
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
/// `hide` is the set the line was shown with. Notes stay at their characters when text is
/// added or removed anywhere in the line, and a character that is not new keeps its backslash.
///
/// `text` is one line.
pub fn edit_line(
    old: &str,
    kind: TextKind,
    shape: Shape,
    hide: &[Hide],
    text: &str,
    runs: &[Run],
) -> String {
    edit(old, kind, shape, hide, text, runs, Untouched::Keep)
}

/// Whether an edit that changes nothing gives the line back as it is, or writes it again.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Untouched {
    Keep,
    /// Only the tests write a line that is not touched again.
    #[cfg_attr(not(test), allow(dead_code))]
    Rewrite,
}

pub(crate) fn edit(
    old: &str,
    kind: TextKind,
    shape: Shape,
    hide: &[Hide],
    text: &str,
    runs: &[Run],
    untouched: Untouched,
) -> String {
    let split = split_line(old, kind, hide).filter(|_| shape == Shape::Shown);
    let Some(split) = split else {
        return text.to_owned();
    };

    let before = read_body(old, split.start..split.end, hide);
    let after: Vec<char> = text.chars().collect();
    let marks = match hide.contains(&Hide::Emphasis) {
        true => marks_of(runs, after.len()),
        false => vec![Marks::default(); after.len()],
    };

    // The line is not touched: it stays as it is, even if it is not canonical.
    if untouched == Untouched::Keep && after == before.chars && marks == before.marks {
        return old.to_owned();
    }
    if after.is_empty() && before.notes.is_empty() {
        // Marks around nothing (`__`) and a forcing mark alone have no text to write.
        return match before.chars.is_empty() {
            true => old.to_owned(),
            false => String::new(),
        };
    }

    rewrite(old, &split, &before, &after, &marks, hide)
}

/// The line `old` written again for the text `after` with `marks`.
fn rewrite(
    old: &str,
    split: &Split,
    before: &Body,
    after: &[char],
    marks: &[Marks],
    hide: &[Hide],
) -> String {
    let pairs = pairs(&before.chars, after);
    let escaped: Vec<bool> = pairs
        .iter()
        .map(|old| old.is_some_and(|at| before.escaped[at]))
        .collect();
    let part = Part {
        chars: after,
        marks,
        escaped: &escaped,
        notes: &moved_notes(before, &pairs, after.len()),
    };
    let (mut body, mut effective) = write_body(&part);

    // Text that is not new reads as it is shown, whatever the marks do to each other.
    let same_text = after == before.chars;
    let emphasis_hidden = hide.contains(&Hide::Emphasis);
    if same_text && emphasis_hidden && !reads_as(&body, &part, &effective, hide) {
        effective = vec![Marks::default(); after.len()];
        (body, _) = write_body(&Part {
            marks: &effective,
            escaped: &vec![true; after.len()],
            ..part
        });
    }

    // A blank typed at the end must not become the blank that leads up to a mark behind it.
    let tail = &old[split.end..];
    let apart =
        body.ends_with(char::is_whitespace) && tail.starts_with(|c: char| !c.is_whitespace());
    let gap = if apart { " " } else { "" };

    format!("{}{body}{gap}{tail}", &old[..split.start])
}

/// For each character of `new` the character of `old` it is, or none for a new one: the
/// characters both texts share in a longest common subsequence, so a change in two places
/// keeps what lies between.
fn pairs(old: &[char], new: &[char]) -> Vec<Option<usize>> {
    let keys = |text: &[char]| text.iter().map(char::to_string).collect::<Vec<_>>();
    let (removed, added) = unpaired(&keys(old), &keys(new));
    let mut kept = (0..old.len()).filter(|at| !removed.contains(at));

    (0..new.len())
        .map(|at| match added.contains(&at) {
            true => None,
            false => kept.next(),
        })
        .collect()
}

/// The notes of `before` in the new text of `len` characters: a note stays before the
/// character it stood before; where that is gone, behind the character before it; one at the
/// end stays at the end.
fn moved_notes(before: &Body, pairs: &[Option<usize>], len: usize) -> Vec<ShownNote> {
    let old_len = before.chars.len();

    before
        .notes
        .iter()
        .map(|note| {
            let next = pairs
                .iter()
                .position(|old| old.is_some_and(|at| at >= note.at));
            let last = pairs
                .iter()
                .rposition(|old| old.is_some_and(|at| at < note.at));
            let at = match (next, last) {
                _ if note.at >= old_len => len,
                (Some(next), _) => next,
                (None, Some(last)) => last + 1,
                (None, None) => 0,
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

/// Whether `body` read again gives the text and notes of `part` and `marks`, the emphasis
/// that was written.
fn reads_as(body: &str, part: &Part, marks: &[Marks], hide: &[Hide]) -> bool {
    let again = read_body(body, 0..body.len(), hide);

    !again.boneyard
        && again.chars == part.chars
        && again.marks == marks
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
/// The emphasis it wrote comes with it: a stretch that could not be marked is plain there.
fn write_body(part: &Part) -> (String, Vec<Marks>) {
    let mut out = String::new();
    let mut written = part.marks.to_vec();
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
        if now.marks != PLAIN && !now.applied {
            written[i] = PLAIN;
        }
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

    (out, written)
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
