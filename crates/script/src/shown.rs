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
//! over only the change: what was removed, what was typed and the emphasis of the new text.
//! What the user types stays Fountain: a typed `*x*` is emphasis the next time the line is
//! shown.

use std::ops::Range;

use crate::inline::{all_chars, runs};
use crate::parse::{
    boneyard_lines, split_number, CENTER_END, DUAL_MARK, FORCE_ACTION, FORCE_CHARACTER,
    FORCE_HEADING, FORCE_TRANSITION, LYRICS, NOTE_CLOSE, NOTE_OPEN, NUMBER_MARK,
};
use crate::{text_kinds, Marks, Run, Splice, TextKind};

mod lead;
mod read;
mod write;

use lead::{behind, leads, End, Lead};
use read::reads_right;
use write::{write_body, Part};

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

pub(crate) fn shown_line(
    line: &str,
    kind: TextKind,
    in_boneyard: bool,
    hide: &[Hide],
) -> ShownLine {
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

    // A forcing character with nothing behind it would be a typed character that shows
    // nowhere: it stays text until a character follows (`.` and `.` and `.` typed is `...`,
    // `@` and `x` typed is the cue `x`).
    let alone = body.chars.is_empty() && body.notes.is_empty() && split.start > 0;
    if alone || is_forcing_alone(line) {
        return typed_line(line, kind, hidden);
    }

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

/// Whether `line` is one forcing character and nothing else.
fn is_forcing_alone(line: &str) -> bool {
    let mut chars = line.trim_end().chars();

    match (chars.next(), chars.next()) {
        (Some(ch), None) => lead_chars().contains(&ch),
        _ => false,
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

/// The characters that force an element when they open a line.
fn lead_chars() -> [char; 5] {
    [
        FORCE_HEADING,
        FORCE_ACTION,
        FORCE_CHARACTER,
        FORCE_TRANSITION,
        LYRICS,
    ]
}

/// The character that forces an element of `kind` when it opens a line, as `split_line` hides it.
pub(crate) fn lead_mark(kind: TextKind) -> Option<char> {
    match kind {
        TextKind::Heading => Some(FORCE_HEADING),
        TextKind::Action => Some(FORCE_ACTION),
        TextKind::Character => Some(FORCE_CHARACTER),
        TextKind::Transition | TextKind::Centered => Some(FORCE_TRANSITION),
        TextKind::Lyrics => Some(LYRICS),
        _ => None,
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

/// One change of a shown line: the characters `removed` of its text are replaced by what
/// was `typed`, and `runs` is the emphasis of the whole new text. A change of the emphasis
/// alone removes and types nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineEdit {
    pub removed: Range<usize>,
    pub typed: String,
    pub runs: Vec<Run>,
}

/// The source line for one change of a line that was `old`, of `kind` and `shape` when it
/// was shown with `hide`.
///
/// - What the change does not touch stays: its backslashes (`\*`), the notes (a note stands
///   before the character it stood before; where that is gone, behind what was typed) and
///   the number.
/// - Only what was typed is read as Fountain: `*x*` is italic, `INT.` a heading.
/// - The kind follows the text: unchanged unless the visible text now reads as another kind
///   than before (or nothing old is left). If the line would read otherwise, the forcing
///   character is left out or the one of the kind put in front (without `Hide::Forcing` it
///   is visible text).
/// - The written line is read again; typed text that damages its neighbours is written
///   literally.
///
/// `typed` is one line. To remove a line, use [`splice_lines`].
pub fn edit_line(
    old: &str,
    kind: TextKind,
    shape: Shape,
    hide: &[Hide],
    change: &LineEdit,
) -> String {
    edit(old, kind, shape, hide, change, Untouched::Keep)
}

/// Whether an edit that changes nothing gives the line back as it is, or writes it again.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Untouched {
    Keep,
    /// Only the tests write a line that is not touched again.
    #[cfg_attr(not(test), allow(dead_code))]
    Rewrite,
}

/// Which of the notes of a line an edit keeps; a change over a line break shares them out.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Notes {
    All,
    /// Those that stand before the character with this index.
    Before(usize),
    /// Those that stand before this character or later.
    From(usize),
}

pub(crate) fn edit(
    old: &str,
    kind: TextKind,
    shape: Shape,
    hide: &[Hide],
    change: &LineEdit,
    untouched: Untouched,
) -> String {
    edit_notes(old, kind, shape, hide, change, (untouched, Notes::All))
}

/// `edit`, keeping only the notes `keep` names.
pub(crate) fn edit_notes(
    old: &str,
    kind: TextKind,
    shape: Shape,
    hide: &[Hide],
    change: &LineEdit,
    (untouched, keep): (Untouched, Notes),
) -> String {
    let split = split_line(old, kind, hide).filter(|_| shape == Shape::Shown);
    let Some(split) = split else {
        return typed_edit(old, change);
    };

    let mut before = read_body(old, split.start..split.end, hide);
    let all = before.notes.len();
    before.notes.retain(|note| match keep {
        Notes::All => true,
        Notes::Before(at) => note.at < at,
        Notes::From(at) => note.at >= at,
    });
    let shared_out = before.notes.len() != all;
    let plan = plan(&before, change, hide.contains(&Hide::Emphasis));

    // The line is not touched: it stays as it is, even if it is not canonical.
    let same = !shared_out
        && plan.chars == before.chars
        && plan.marks == before.marks
        && plan.notes == before.notes;
    if untouched == Untouched::Keep && same {
        return old.to_owned();
    }
    if plan.chars.is_empty() && before.notes.is_empty() {
        // Marks around nothing (`__`) and a forcing mark alone have no text to write.
        return match before.chars.is_empty() {
            true => old.to_owned(),
            false => String::new(),
        };
    }

    rewrite(old, &split, &before, &plan, (kind, hide))
}

/// A line the editor shows as typed: the change applies to the line itself.
fn typed_edit(old: &str, change: &LineEdit) -> String {
    let chars: Vec<char> = old.chars().collect();
    let start = change.removed.start.min(chars.len());
    let end = change.removed.end.clamp(start, chars.len());
    let part = |range: Range<usize>| chars[range].iter().collect::<String>();

    format!(
        "{}{}{}",
        part(0..start),
        change.typed,
        part(end..chars.len())
    )
}

/// The new shown text of a line, what each character brings along, and where the change is.
struct Plan {
    chars: Vec<char>,
    marks: Vec<Marks>,
    escaped: Vec<bool>,
    notes: Vec<ShownNote>,
    /// Where the typed characters stand in `chars`.
    typed: Range<usize>,
    /// How many characters of the old text are left.
    kept: usize,
}

impl Plan {
    /// The same, with what was typed protected by backslashes.
    fn literal(&self) -> Plan {
        let mut escaped = self.escaped.clone();
        escaped[self.typed.clone()].fill(true);

        Plan {
            escaped,
            ..self.copy()
        }
    }

    /// The same with every character protected (a typed `_` can turn an earlier one into
    /// emphasis).
    fn guarded(&self) -> Plan {
        Plan {
            escaped: vec![true; self.chars.len()],
            ..self.copy()
        }
    }

    /// The same without emphasis and with every character protected.
    fn plain(&self) -> Plan {
        Plan {
            marks: vec![Marks::default(); self.chars.len()],
            escaped: vec![true; self.chars.len()],
            ..self.copy()
        }
    }

    fn copy(&self) -> Plan {
        Plan {
            chars: self.chars.clone(),
            marks: self.marks.clone(),
            escaped: self.escaped.clone(),
            notes: self.notes.clone(),
            typed: self.typed.clone(),
            kept: self.kept,
        }
    }
}

/// `old` with `cut` replaced by `new`.
fn spliced<T: Clone>(old: &[T], cut: &Range<usize>, new: &[T]) -> Vec<T> {
    [&old[..cut.start], new, &old[cut.end..]].concat()
}

/// The text after `change`, with the notes and backslashes of the old text carried along.
fn plan(before: &Body, change: &LineEdit, emphasis_hidden: bool) -> Plan {
    let len = before.chars.len();
    let start = change.removed.start.min(len);
    let cut = start..change.removed.end.clamp(start, len);
    let typed: Vec<char> = change.typed.chars().collect();

    let chars = spliced(&before.chars, &cut, &typed);
    let marks = match emphasis_hidden {
        true => marks_of(&change.runs, chars.len()),
        false => vec![Marks::default(); chars.len()],
    };
    // A note stays before its character; where that is gone, it moves behind the typed text.
    let notes = before
        .notes
        .iter()
        .map(|note| ShownNote {
            at: match note.at < cut.start {
                true => note.at,
                false => note.at.max(cut.end) - cut.len() + typed.len(),
            },
            text: note.text.clone(),
        })
        .collect();

    Plan {
        escaped: spliced(&before.escaped, &cut, &vec![false; typed.len()]),
        typed: start..start + typed.len(),
        kept: len - cut.len(),
        chars,
        marks,
        notes,
    }
}

/// What the line is written from.
struct Target<'a> {
    old: &'a str,
    split: &'a Split,
    kind: TextKind,
    hide: &'a [Hide],
    /// The kind the written line has to read as.
    reads: TextKind,
    above: Above,
}

/// The line `old` written again for `plan`.
fn rewrite(
    old: &str,
    split: &Split,
    before: &Body,
    plan: &Plan,
    (kind, hide): (TextKind, &[Hide]),
) -> String {
    let (reads, above) = target_kind(old, (kind, hide), before, plan);
    let target = Target {
        old,
        split,
        kind,
        hide,
        reads,
        above,
    };

    // Typed text may damage its neighbours (`*` behind italic text): then it is literal; then
    // every character is protected; when even that fails, nothing is emphasis.
    let mut result = written(&target, plan);
    if result.is_err() && !plan.typed.is_empty() {
        result = written(&target, &plan.literal());
    }
    if result.is_err() {
        result = written(&target, &plan.guarded());
    }
    if result.is_err() {
        result = written(&target, &plan.plain());
    }

    result.unwrap_or_else(|line| line)
}

/// What stands above a line: a blank, or text (which makes `INT. KÜCHE` action).
#[derive(Clone, Copy, PartialEq, Eq)]
enum Above {
    Blank,
    Text,
}

/// A shown text as a source line, as `kind` reads it: a cue has speech below it, speech a
/// cue above it, anything else stands alone.
fn read_in(line: &str, kind: TextKind, above: Above, hide: &[Hide]) -> ShownLine {
    let (source, at) = match (kind, above) {
        (TextKind::Character, _) => (format!("{line}\nx"), 0),
        (TextKind::Parenthetical | TextKind::Dialogue, _) => (format!("ANNA\n{line}"), 1),
        (TextKind::Action, Above::Text) => (format!("x\n{line}"), 1),
        _ => (line.to_owned(), 0),
    };

    shown_lines(&source, hide).swap_remove(at)
}

/// The kind the new line has to read as, and what stands above it when it is read: the old
/// kind while the visible text reads as before (the hidden marks held it), else the kind
/// the new text reads as. One rule, also when the whole text is replaced: a cue renamed
/// stays a cue.
///
/// A line that does not read as its kind by itself is action because of the line above it
/// (`>Text` under a line of action); it stays so.
///
/// Example: `*INT.* KÜCHE` is action; without the emphasis it stays action.
fn target_kind(
    old: &str,
    (kind, hide): (TextKind, &[Hide]),
    before: &Body,
    plan: &Plan,
) -> (TextKind, Above) {
    let reads =
        |chars: &[char]| read_in(&chars.iter().collect::<String>(), kind, Above::Blank, &[]).kind;
    if read_in(old, kind, Above::Blank, hide).kind != kind {
        return (kind, Above::Text);
    }

    let new = reads(&plan.chars);
    // A note line shows its marks: with emphasis in the text it cannot be one.
    let emphasized = plan.marks.iter().any(|m| m.bold || m.italic || m.underline);
    match new != reads(&before.chars) && !(new == TextKind::Note && emphasized) {
        true => (new, Above::Blank),
        false => (kind, Above::Blank),
    }
}

/// The line for `plan` that reads as the target kind, with the text and emphasis the change
/// left alone as written; the first line it tried if none does.
fn written(target: &Target, plan: &Plan) -> Result<String, String> {
    let part = Part {
        chars: &plan.chars,
        marks: &plan.marks,
        escaped: &plan.escaped,
        notes: &plan.notes,
    };
    let (body, effective) = write_body(&part);
    let old_tail = &target.old[target.split.end..];
    let mut first = None;
    let mut kind_held = false;
    let forcing_hidden = target.hide.contains(&Hide::Forcing);

    for (lead, how, end) in leads(target, old_tail) {
        // A character put in front is only for a kind that was read wrong, or, when it hides,
        // for text that it would show.
        if how == Lead::Added && kind_held && !forcing_hidden {
            continue;
        }

        let tail = behind(old_tail, end, target.kind);
        // A blank typed at the end must not become the blank that leads up to a mark behind it.
        let apart = end != End::Guarded
            && body.ends_with(char::is_whitespace)
            && tail.starts_with(|c: char| !c.is_whitespace());
        let gap = if apart { " " } else { "" };
        let line = format!("{lead}{body}{gap}{tail}");
        let read = read_in(&line, target.kind, target.above, target.hide);
        kind_held |= read.kind == target.reads;

        // A forcing character that shows is not part of the text. A line that shows its marks
        // (a section) has no hidden text to compare.
        let skip = usize::from(how == Lead::Added && !forcing_hidden);
        let right = read.kind == target.reads
            && (is_typed_kind(read.kind)
                || reads_right(&read, skip, &part, (&effective, &plan.typed)));
        if right {
            return Ok(line);
        }

        first.get_or_insert(line);
    }

    Err(first.unwrap_or_default())
}

/// The kinds the editor shows with their marks.
fn is_typed_kind(kind: TextKind) -> bool {
    matches!(
        kind,
        TextKind::Blank
            | TextKind::Title
            | TextKind::Section
            | TextKind::Synopsis
            | TextKind::Note
            | TextKind::PageBreak
    )
}

/// The emphasis of each of `len` characters; what no run covers is plain.
pub(crate) fn marks_of(runs: &[Run], len: usize) -> Vec<Marks> {
    let mut marks = vec![Marks::default(); len];

    for run in runs {
        for mark in marks.iter_mut().take(run.end).skip(run.start) {
            *mark = emphasis(run.marks);
        }
    }

    marks
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
