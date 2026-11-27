//! Emphasis over a selection of the source (D-36, D-37): Ctrl+B, Ctrl+I, Ctrl+U and the menu.
//!
//! The selection is in the source, whichever way the editor shows it: the lines are read with
//! the emphasis hidden, the new emphasis is written back with the canonical marks of
//! [`edit_line`]. With the marks shown, the editor then shows them as they were written.
//!
//! ```text
//! source     Das ist fett und *kursiv*.
//! selection  "fett und"                          Ctrl+B: all of it bold? no, so bold
//! source     Das ist **fett und** *kursiv*.
//! ```

use crate::change::{byte_at, rewrite_lines};
use crate::inline::runs;
use crate::shown::marks_of as marks_by_runs;
use crate::{
    edit_line, shown_lines, text_kinds, Edited, Hide, LineEdit, Marks, Shape, ShownLine, Spot,
    TextKind,
};

/// What the lines are read with: the emphasis of the source, whichever way the editor shows it.
const READ: [Hide; 1] = [Hide::Emphasis];

/// A kind of emphasis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mark {
    Bold,
    Italic,
    Underline,
}

/// The emphasis a selection (or the place of the cursor) has.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EmphasisAt {
    /// Every character of the selection has it; at the cursor: the character before it, at the
    /// start of a line the one behind. A heading is bold throughout.
    pub marks: Marks,
    /// Only headings: bold is theirs already and cannot be toggled.
    pub heading: bool,
}

impl Mark {
    fn of(self, marks: &Marks) -> bool {
        match self {
            Mark::Bold => marks.bold,
            Mark::Italic => marks.italic,
            Mark::Underline => marks.underline,
        }
    }

    fn set(self, marks: &mut Marks, on: bool) {
        match self {
            Mark::Bold => marks.bold = on,
            Mark::Italic => marks.italic = on,
            Mark::Underline => marks.underline = on,
        }
    }
}

/// The shown characters `start..end` of a line that a selection takes.
struct Piece {
    line: usize,
    kind: TextKind,
    shape: Shape,
    start: usize,
    end: usize,
}

/// The first shown character that stands at or behind byte `at` of its source line.
fn first_from(shown: &ShownLine, at: usize) -> usize {
    let len = shown.text.chars().count();

    shown.cols[..len]
        .iter()
        .position(|col| *col >= at)
        .unwrap_or(len)
}

/// How many shown characters stand before byte `at` of their source line.
fn count_before(shown: &ShownLine, at: usize) -> usize {
    let len = shown.text.chars().count();

    shown.cols[..len].iter().filter(|col| **col < at).count()
}

/// The lines of the selection `from` to `to` (columns in UTF-16 units of the source lines) and
/// what of each the selection takes; lines that show their marks or take nothing are left out.
fn pieces(lines: &[&str], shown: &[ShownLine], (from, to): (Spot, Spot)) -> Vec<Piece> {
    let first = from.line.min(lines.len() - 1);
    let last = to.line.clamp(first, lines.len() - 1);
    let mut found = Vec::new();

    for line in first..=last {
        let one = &shown[line];
        let len = one.text.chars().count();
        let start = match line == first {
            true => first_from(one, byte_at(lines[line], from.col)),
            false => 0,
        };
        let end = match line == last {
            true => count_before(one, byte_at(lines[line], to.col)),
            false => len,
        };
        if one.shape == Shape::Typed || start >= end {
            continue;
        }

        found.push(Piece {
            line,
            kind: one.kind,
            shape: one.shape,
            start,
            end,
        });
    }

    found
}

/// The emphasis of each character of a shown line.
fn marks_of(shown: &ShownLine) -> Vec<Marks> {
    marks_by_runs(&shown.runs, shown.text.chars().count())
}

/// As [`marks_of`], and a heading is bold as a whole: its marks never say so, the look does.
fn marks_in(shown: &ShownLine) -> Vec<Marks> {
    let mut marks = marks_of(shown);

    if shown.kind == TextKind::Heading {
        marks.iter_mut().for_each(|m| m.bold = true);
    }

    marks
}

/// The emphasis `from` to `to` has; `from == to` asks for the place of a cursor.
pub fn emphasis_at(source: &str, from: Spot, to: Spot) -> EmphasisAt {
    let lines: Vec<&str> = source.split('\n').collect();
    let shown = shown_lines(source, &READ);

    if from == to {
        return at_cursor(&lines, &shown, from);
    }

    let found = pieces(&lines, &shown, (from, to));
    let marks: Vec<Marks> = found
        .iter()
        .flat_map(|p| marks_in(&shown[p.line])[p.start..p.end].to_vec())
        .collect();
    let all = |has: fn(&Marks) -> bool| !marks.is_empty() && marks.iter().all(has);

    EmphasisAt {
        marks: Marks {
            bold: all(|m| m.bold),
            italic: all(|m| m.italic),
            underline: all(|m| m.underline),
            ..Marks::default()
        },
        heading: !found.is_empty() && found.iter().all(|p| p.kind == TextKind::Heading),
    }
}

fn at_cursor(lines: &[&str], shown: &[ShownLine], spot: Spot) -> EmphasisAt {
    let line = spot.line.min(lines.len() - 1);
    let one = &shown[line];
    let len = one.text.chars().count();
    let before = count_before(one, byte_at(lines[line], spot.col));
    // The character typed next takes after the one before, at a line's start after the one behind.
    let at = before.saturating_sub(1);
    let marks = marks_in(one).get(at).copied().filter(|_| len > 0);

    EmphasisAt {
        marks: Marks {
            note: false,
            boneyard: false,
            ..marks.unwrap_or_default()
        },
        heading: one.kind == TextKind::Heading,
    }
}

/// `source` with `mark` toggled over `from` to `to`: off when the whole selection has it,
/// else on for all of it. The lines to show again are shown with `hide`. A selection without
/// characters, and bold on headings, change nothing; neither does a line that could not take
/// the emphasis without a change of its text or of what it is.
pub fn toggle_mark(source: &str, hide: &[Hide], (from, to): (Spot, Spot), mark: Mark) -> Edited {
    let lines: Vec<&str> = source.split('\n').collect();
    let shown = shown_lines(source, &READ);
    let first = from.line.min(lines.len() - 1);
    let last = to.line.clamp(first, lines.len() - 1);
    let mut found = pieces(&lines, &shown, (from, to));
    found.retain(|p| !(mark == Mark::Bold && p.kind == TextKind::Heading));

    let has = |p: &Piece| {
        marks_of(&shown[p.line])[p.start..p.end]
            .iter()
            .all(|m| mark.of(m))
    };
    let on = !found.iter().all(has);
    let old: Vec<String> = lines[first..=last]
        .iter()
        .map(|l| (*l).to_owned())
        .collect();
    let mut new = old.clone();

    for piece in &found {
        let mut marks = marks_of(&shown[piece.line]);
        marks[piece.start..piece.end]
            .iter_mut()
            .for_each(|m| mark.set(m, on));
        let edit = LineEdit {
            removed: 0..0,
            typed: String::new(),
            runs: runs(&marks),
        };

        new[piece.line - first] =
            edit_line(lines[piece.line], piece.kind, piece.shape, &READ, &edit);
    }

    let same = Same::of(source, &lines, first);
    if !same.holds(&new) {
        // Line by line: the lines that cannot take it stay as they are.
        for piece in &found {
            let at = piece.line - first;
            let mut one = old.clone();
            one[at].clone_from(&new[at]);
            if !same.holds(&one) {
                new[at].clone_from(&old[at]);
            }
        }
    }

    rewrite_lines(source, hide, (first, last), &new)
}

/// What a toggle leaves as it is: the text each line shows and what each line is.
struct Same<'a> {
    lines: &'a [&'a str],
    first: usize,
    kinds: Vec<TextKind>,
    texts: Vec<String>,
}

impl<'a> Same<'a> {
    fn of(source: &str, lines: &'a [&'a str], first: usize) -> Same<'a> {
        Same {
            lines,
            first,
            kinds: text_kinds(source),
            texts: shown_lines(source, &READ)
                .into_iter()
                .map(|l| l.text)
                .collect(),
        }
    }

    /// Whether the source with `new` in place of the lines from `first` has them still.
    fn holds(&self, new: &[String]) -> bool {
        let end = self.first + new.len();
        let mut all: Vec<&str> = self.lines[..self.first].to_vec();
        all.extend(new.iter().map(String::as_str));
        all.extend(&self.lines[end..]);
        let source = all.join("\n");
        let shown = shown_lines(&source, &READ);

        text_kinds(&source) == self.kinds
            && shown.len() == self.texts.len()
            && shown
                .iter()
                .zip(&self.texts)
                .all(|(l, text)| l.text == *text)
    }
}
