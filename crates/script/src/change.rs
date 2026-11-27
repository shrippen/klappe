//! One change of the shown text, across lines, as a change of the source (D-36).
//!
//! The editor hands over what the user did to the text it shows: these characters of these
//! lines replaced by that. A change inside one line is one [`LineEdit`]; over lines the first
//! and the last line are edited, the lines between are typed text, and what stands behind the
//! change in the last line goes along with its source.
//!
//! ```text
//! shown   Ein langes| Wort        Enter in the middle of a bold word
//! source  Ein **lan**  +  **ges** Wort
//! ```

use std::ops::Range;

use crate::inline::{runs, NOTE_BRACKET, UNDERLINE};
use crate::parse::{boneyard_lines, ESCAPE, NOTE_CLOSE, NOTE_OPEN, STAR};
use crate::shown::{edit_notes, marks_of, shown_line, Notes, Untouched};
use crate::{
    shown_lines, splice_lines, text_kinds, Hide, LineEdit, Marks, Run, ShownLine, Splice, TextKind,
};

/// A place in the shown text: the line and the column in it, counted in UTF-16 units as the
/// window counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Spot {
    pub line: usize,
    pub col: usize,
}

/// The shown text from `from` to `to` replaced by `typed` (which may hold line breaks).
/// `runs` is the emphasis of `typed`, in UTF-16 units of it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    pub from: Spot,
    pub to: Spot,
    pub typed: String,
    pub runs: Vec<Run>,
}

/// The source after a change. The lines `from..to` of the old source are the lines
/// `from..from + shown.len()` of the new one; `shown` is how the editor shows them. A line
/// is part of it when the change touched it, or when what it is (its kind, a boneyard over
/// it) differs now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edited {
    pub source: String,
    pub from: usize,
    pub to: usize,
    pub shown: Vec<ShownLine>,
}

/// The source lines a change is written from.
struct Source<'a> {
    lines: &'a [&'a str],
    kinds: &'a [TextKind],
    /// Per line: a boneyard that an earlier line opened covers it.
    inside: &'a [bool],
    hide: &'a [Hide],
}

/// The old lines a change starts and ends in, shown.
struct Ends {
    first: usize,
    last: usize,
    head: ShownLine,
    tail: ShownLine,
}

/// `source` with `change` made, and the lines to show again.
pub fn edit_shown(source: &str, hide: &[Hide], change: &Change) -> Edited {
    let lines: Vec<&str> = source.split('\n').collect();
    let kinds = text_kinds(source);
    let inside = boneyard_lines(source);
    let show = |at: usize| {
        let in_boneyard = inside.get(at).copied().unwrap_or(false);
        shown_line(lines[at], kinds[at], in_boneyard, hide)
    };

    let first = change.from.line.min(lines.len() - 1);
    let last = change.to.line.clamp(first, lines.len() - 1);
    let ends = Ends {
        first,
        last,
        head: show(first),
        tail: show(last),
    };
    let from_source = Source {
        lines: &lines,
        kinds: &kinds,
        inside: &inside,
        hide,
    };

    let change = in_chars(change, &ends);
    let widened = pair_marks(&from_source, &ends, &change);
    let written = write_lines(&from_source, &ends, widened.as_ref().unwrap_or(&change));
    let splice = splice_lines(source, first..last + 1, &written);

    reshown(&from_source, &splice, (first, last))
}

/// `source` with the text from `from` to `to` replaced by `text`, which is Fountain as it
/// stands: for a caller that works on the source (search and replace, a suggestion, a moved
/// scene). The columns are UTF-16 units of the source lines.
pub fn replace_source(source: &str, hide: &[Hide], from: Spot, to: Spot, text: &str) -> Edited {
    let lines: Vec<&str> = source.split('\n').collect();
    let first = from.line.min(lines.len() - 1);
    let last = to.line.clamp(first, lines.len() - 1);
    let (start, end) = (
        byte_at(lines[first], from.col),
        byte_at(lines[last], to.col),
    );
    let (start, end) = match first == last {
        true => (start, end.max(start)),
        false => (start, end),
    };
    let joined = format!("{}{}{}", &lines[first][..start], text, &lines[last][end..]);
    let new: Vec<String> = joined.split('\n').map(str::to_owned).collect();

    rewrite_lines(source, hide, (first, last), &new)
}

/// The byte of `line` where UTF-16 unit `col` stands; the end of the line past it.
pub(crate) fn byte_at(line: &str, col: usize) -> usize {
    let mut units = 0;

    line.char_indices()
        .find(|(_, ch)| {
            units += ch.len_utf16();
            units > col
        })
        .map_or(line.len(), |(index, _)| index)
}

/// `source` with the lines `first..=last` replaced by `new`, and the lines to show again.
pub(crate) fn rewrite_lines(
    source: &str,
    hide: &[Hide],
    (first, last): (usize, usize),
    new: &[String],
) -> Edited {
    let lines: Vec<&str> = source.split('\n').collect();
    let kinds = text_kinds(source);
    let inside = boneyard_lines(source);
    let splice = splice_lines(source, first..last + 1, new);
    let from_source = Source {
        lines: &lines,
        kinds: &kinds,
        inside: &inside,
        hide,
    };

    reshown(&from_source, &splice, (first, last))
}

/// The source after `splice`, and the lines to show again: those the change touched
/// (`touched`, first and last line), and those it made another kind or put in a boneyard.
fn reshown(from: &Source, splice: &Splice, touched: (usize, usize)) -> Edited {
    let Source {
        lines,
        kinds,
        inside,
        hide,
    } = *from;
    let (first, last) = touched;
    let mut all: Vec<&str> = lines[..splice.from].to_vec();
    all.extend(splice.lines.iter().map(String::as_str));
    all.extend(&lines[splice.to..]);
    let new_source = all.join("\n");
    let new_kinds = text_kinds(&new_source);
    let new_inside = boneyard_lines(&new_source);

    let (old_len, new_len) = (splice.to - splice.from, splice.lines.len());
    let moved = |at: usize| at + new_len - old_len;
    let same = |old: usize, new: usize| {
        kinds[old] == new_kinds[new] && inside.get(old) == new_inside.get(new)
    };
    let top = first.min(splice.from);
    let bottom = (last + 1).max(splice.to);
    let from = (0..top).find(|&at| !same(at, at)).unwrap_or(top);
    let to = (bottom..lines.len())
        .rev()
        .find(|&at| !same(at, moved(at)))
        .map_or(bottom, |at| at + 1);

    // `all` holds the new lines of the old lines `from..to` where `moved` puts them.
    let shown = (from..moved(to))
        .map(|at| {
            let in_boneyard = new_inside.get(at).copied().unwrap_or(false);
            shown_line(all[at], new_kinds[at], in_boneyard, hide)
        })
        .collect();

    Edited {
        source: new_source,
        from,
        to,
        shown,
    }
}

/// The characters of `text` before UTF-16 unit `units`.
fn chars_before(text: &str, units: usize) -> usize {
    let mut seen = 0;

    text.chars()
        .take_while(|c| {
            let before = seen;
            seen += c.len_utf16();
            before < units
        })
        .count()
}

/// `change` with its columns and runs counted in characters.
fn in_chars(change: &Change, ends: &Ends) -> Change {
    let at = |spot: Spot, text: &str| Spot {
        line: spot.line,
        col: chars_before(text, spot.col),
    };
    let runs = change
        .runs
        .iter()
        .map(|run| Run {
            start: chars_before(&change.typed, run.start),
            end: chars_before(&change.typed, run.end),
            marks: run.marks,
        })
        .collect();

    Change {
        from: at(change.from, &ends.head.text),
        to: at(change.to, &ends.tail.text),
        typed: change.typed.clone(),
        runs,
    }
}

/// The lines that stand where the lines of `ends` stood.
fn write_lines(from: &Source, ends: &Ends, change: &Change) -> Vec<String> {
    let Source {
        lines, kinds, hide, ..
    } = *from;
    let (first, last) = (ends.first, ends.last);
    let typed: Vec<char> = change.typed.chars().collect();
    let head_text: Vec<char> = ends.head.text.chars().collect();
    let tail_text: Vec<char> = ends.tail.text.chars().collect();
    let typed_marks = marks_of(&change.runs, typed.len());
    let head_marks = marks_of(&ends.head.runs, head_text.len());
    let tail_marks = marks_of(&ends.tail.runs, tail_text.len());

    // What the change cuts: the first line from `keep` on, the last line up to `cut`.
    let keep = change.from.col.min(head_text.len());
    let cut = change.to.col.min(tail_text.len());
    let cut = match first == last {
        true => cut.max(keep),
        false => cut,
    };

    let on = |at: usize, shown: &ShownLine, edit: &LineEdit, notes: Notes| {
        let how = (Untouched::Keep, notes);
        edit_notes(lines[at], kinds[at], shown.shape, hide, edit, how)
    };

    let pieces = line_pieces(&typed);
    let one = &pieces[0];

    // One piece: the start of the first line, the piece and the rest of the last line make
    // one line. The rest keeps its source; the start is typed again, with the backslashes and
    // notes it had.
    if pieces.len() == 1 {
        let piece = &typed[one.clone()];
        if first == last {
            let marks = [
                &head_marks[..keep],
                &typed_marks[one.clone()],
                &tail_marks[cut..],
            ]
            .concat();
            let edit = line_edit(keep..cut, piece, &marks);

            return vec![on(last, &ends.tail, &edit, Notes::All)];
        }

        let head = (&head_marks[..], lines[first]);
        let (text, marks) = joined(&ends.head, head, keep, (piece, &typed_marks[one.clone()]));
        let marks = [&marks[..], &tail_marks[cut..]].concat();

        return vec![on(
            last,
            &ends.tail,
            &line_edit(0..cut, &text, &marks),
            Notes::From(cut),
        )];
    }

    let mut out = Vec::new();
    let marks = [&head_marks[..keep], &typed_marks[one.clone()]].concat();
    let edit = line_edit(keep..head_text.len(), &typed[one.clone()], &marks);
    out.push(on(first, &ends.head, &edit, Notes::Before(keep)));

    // Lines between are typed: Fountain as it stands.
    for piece in &pieces[1..pieces.len() - 1] {
        out.push(typed[piece.clone()].iter().collect());
    }

    let rest = &pieces[pieces.len() - 1];
    let marks = [&typed_marks[rest.clone()], &tail_marks[cut..]].concat();
    out.push(on(
        last,
        &ends.tail,
        &line_edit(0..cut, &typed[rest.clone()], &marks),
        Notes::From(cut),
    ));

    out
}

/// The edit that cuts `removed` and types `text`, the emphasis of the new line being `marks`.
fn line_edit(removed: Range<usize>, text: &[char], marks: &[Marks]) -> LineEdit {
    LineEdit {
        removed,
        typed: text.iter().collect(),
        runs: runs(marks),
    }
}

/// The start of the first line (to `keep`) as it is typed again, then the piece typed, then the
/// notes the first line has from `keep` on, as the characters and their emphasis.
fn joined(
    head: &ShownLine,
    (marks, source): (&[Marks], &str),
    keep: usize,
    (piece, piece_marks): (&[char], &[Marks]),
) -> (Vec<char>, Vec<Marks>) {
    let (mut text, mut out) = retyped(head, source, marks, 0..keep);
    text.extend(piece);
    out.extend(piece_marks);

    for note in head.notes.iter().filter(|n| n.at >= keep) {
        let note = note_chars(&note.text);
        out.extend(vec![out.last().copied().unwrap_or_default(); note.len()]);
        text.extend(note);
    }

    (text, out)
}

/// The characters of a note as they stand in the source.
fn note_chars(text: &str) -> Vec<char> {
    format!("{NOTE_OPEN}{text}{NOTE_CLOSE}").chars().collect()
}

/// Whether the character `at` of a shown line stands behind a backslash in its source line.
fn is_escaped(line: &ShownLine, source: &str, at: usize) -> bool {
    let col = line.cols[at];
    let protected = matches!(
        line.text.chars().nth(at),
        Some(STAR | UNDERLINE | ESCAPE | NOTE_BRACKET)
    );

    protected && col > 0 && source.as_bytes()[col - 1] == b'\\'
}

/// The characters `range` of a shown line as they are typed again: a backslash before each
/// that its source protects, and, where the notes are hidden, a note before the character it
/// stands before. What the user did not touch keeps what made it text.
fn retyped(
    line: &ShownLine,
    source: &str,
    marks: &[Marks],
    range: Range<usize>,
) -> (Vec<char>, Vec<Marks>) {
    let chars: Vec<char> = line.text.chars().collect();
    let (mut out, mut out_marks) = (Vec::new(), Vec::new());

    for at in range {
        for note in line.notes.iter().filter(|n| n.at == at) {
            let note = note_chars(&note.text);
            out_marks.extend(vec![
                out_marks.last().copied().unwrap_or_default();
                note.len()
            ]);
            out.extend(note);
        }
        if is_escaped(line, source, at) {
            out.push(ESCAPE);
            out_marks.push(marks[at]);
        }
        out.push(chars[at]);
        out_marks.push(marks[at]);
    }

    (out, out_marks)
}

/// The characters of `typed` each of its lines spans.
fn line_pieces(typed: &[char]) -> Vec<Range<usize>> {
    let mut pieces = Vec::new();
    let mut start = 0;

    for (at, ch) in typed.iter().enumerate() {
        if *ch == '\n' {
            pieces.push(start..at);
            start = at + 1;
        }
    }
    pieces.push(start..typed.len());

    pieces
}

/// How many earlier marks a typed one is tried with.
const PAIRS_TRIED: usize = 4;

/// A mark typed to close what an earlier one in the line opened (`*Wort` and then `*`): the
/// change is widened back to that mark, so the text between is typed again and the pair
/// is read as emphasis. `None` when no earlier mark pairs with it.
fn pair_marks(from: &Source, ends: &Ends, change: &Change) -> Option<Change> {
    let is_mark = |c: char| c == STAR || c == UNDERLINE;
    let single = ends.first == ends.last && !change.typed.contains('\n');
    if !single || !change.typed.chars().any(is_mark) {
        return None;
    }

    let text: Vec<char> = ends.head.text.chars().collect();
    let keep = change.from.col.min(text.len());
    let cut = change.to.col.clamp(keep, text.len());
    let marks = marks_of(&ends.head.runs, text.len());
    let typed_marks = marks_of(&change.runs, change.typed.chars().count());
    let escaped = |at: usize| is_escaped(&ends.head, from.lines[ends.first], at);
    let literal = text.len() - (cut - keep) + change.typed.chars().count();

    // The marks before the change, nearest first, each from the start of its run of marks.
    let mut starts: Vec<usize> = (0..keep)
        .rev()
        .filter(|&at| is_mark(text[at]) && !escaped(at))
        .map(|at| {
            (0..=at)
                .rev()
                .take_while(|&i| text[i] == text[at])
                .last()
                .unwrap_or(at)
        })
        .collect();
    starts.dedup();

    for start in starts.into_iter().take(PAIRS_TRIED) {
        // The notes between stay where the change moves them; the backslashes go along.
        let mut head = ends.head.clone();
        head.notes.clear();
        let (again, again_marks) = retyped(&head, from.lines[ends.first], &marks, start..keep);
        let widened = Change {
            from: Spot {
                line: ends.first,
                col: start,
            },
            to: change.to,
            typed: again.iter().collect::<String>() + &change.typed,
            runs: runs(&[&again_marks[..], &typed_marks[..]].concat()),
        };
        let line = write_lines(from, ends, &widened).swap_remove(0);
        let read = shown_lines(&line, from.hide).swap_remove(0);
        if read.text.chars().count() < literal {
            return Some(widened);
        }
    }

    None
}
