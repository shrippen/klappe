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

use std::collections::{BTreeMap, BTreeSet};

use crate::change::{byte_at, rewrite_lines};
use crate::inline::runs;
use crate::parse::boneyard_lines;
use crate::shown::{marks_of as marks_by_runs, shown_line};
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

    fn set(self, marks: &mut Marks, switch: Switch) {
        let on = switch == Switch::On;

        match self {
            Mark::Bold => marks.bold = on,
            Mark::Italic => marks.italic = on,
            Mark::Underline => marks.underline = on,
        }
    }
}

/// Whether a toggle sets the emphasis or takes it away.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Switch {
    On,
    Off,
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
fn pieces(
    lines: &[&str],
    show: &dyn Fn(usize) -> ShownLine,
    (from, to): (Spot, Spot),
) -> Vec<Piece> {
    let first = from.line.min(lines.len() - 1);
    let last = to.line.clamp(first, lines.len() - 1);
    let mut found = Vec::new();

    for (line, text) in lines.iter().enumerate().take(last + 1).skip(first) {
        let one = &show(line);
        let len = one.text.chars().count();
        let start = match line == first {
            true => first_from(one, byte_at(text, from.col)),
            false => 0,
        };
        let end = match line == last {
            true => count_before(one, byte_at(text, to.col)),
            false => len,
        };
        // What forces the element is no text to mark: `~eins` is marked from its `e`.
        let (lead, trail) = forcing(one);
        let (start, end) = (start.max(lead), end.min(len.saturating_sub(trail)));
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

/// How many characters at the start and at the end of a shown line only force its element
/// (`!` action, `@` cue, `~` lyrics, `>` transition or centered, `.` heading, `<` closing a
/// centered line).
fn forcing(shown: &ShownLine) -> (usize, usize) {
    let mark = match shown.kind {
        TextKind::Action => '!',
        TextKind::Character => '@',
        TextKind::Lyrics => '~',
        TextKind::Transition | TextKind::Centered => '>',
        TextKind::Heading => '.',
        _ => return (0, 0),
    };
    let text = &shown.text;
    // `..INT.` is a heading with a dot of its own.
    let lead = text.starts_with(mark) && !(mark == '.' && text.starts_with(".."));
    let trail = shown.kind == TextKind::Centered && text.len() > 1 && text.ends_with('<');

    (usize::from(lead), usize::from(trail))
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

    let found = pieces(&lines, &|at| shown[at].clone(), (from, to));
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
pub fn toggle_mark(source: &str, hide: &[Hide], range: (Spot, Spot), mark: Mark) -> Edited {
    toggle_rounds(source, hide, range, mark).0
}

/// [`toggle_mark`], and how many times the new book was read before it held.
pub(crate) fn toggle_rounds(
    source: &str,
    hide: &[Hide],
    (from, to): (Spot, Spot),
    mark: Mark,
) -> (Edited, usize) {
    let lines: Vec<&str> = source.split('\n').collect();
    let book = Book {
        lines: &lines,
        kinds: text_kinds(source),
        inside: boneyard_lines(source),
    };
    let first = from.line.min(lines.len() - 1);
    let last = to.line.clamp(first, lines.len() - 1);
    let mut found = pieces(&lines, &|at| book.shown(at), (from, to));
    found.retain(|p| !(mark == Mark::Bold && p.kind == TextKind::Heading));

    let has = |p: &Piece| {
        marks_of(&book.shown(p.line))[p.start..p.end]
            .iter()
            .all(|m| mark.of(m))
    };
    let switch = match found.iter().all(has) {
        true => Switch::Off,
        false => Switch::On,
    };
    let mut changes: BTreeMap<usize, Change> = BTreeMap::new();

    for piece in &found {
        let mut marks = marks_of(&book.shown(piece.line));
        marks[piece.start..piece.end]
            .iter_mut()
            .for_each(|m| mark.set(m, switch));
        let edit = LineEdit {
            removed: 0..0,
            typed: String::new(),
            runs: runs(&marks),
        };
        let line = edit_line(lines[piece.line], piece.kind, piece.shape, &READ, &edit);

        // A line that comes back as it was has nothing to take.
        if line != lines[piece.line] {
            changes.insert(piece.line, Change { line, marks });
        }
    }

    // The new book is read once: what a line shows, and what every line of the book is, must be
    // as before, but for the emphasis asked for. A line that spoils it goes back; so does what
    // may have caused it. Again, until nothing is spoilt (at the latest when nothing is left).
    let mut rounds = 0;
    loop {
        rounds += 1;
        let bad = book.spoilt(&changes);
        if bad.is_empty() {
            break;
        }

        book.shed(&mut changes, &bad);
    }

    let new: Vec<String> = (first..=last)
        .map(|at| match changes.get(&at) {
            Some(change) => change.line.clone(),
            None => lines[at].to_owned(),
        })
        .collect();

    (rewrite_lines(source, hide, (first, last), &new), rounds)
}

/// A line written again, and the emphasis it is to show.
struct Change {
    line: String,
    marks: Vec<Marks>,
}

/// A line of the new book that is not as it should be, and whether the change of that line itself
/// is the cause (its kind or its emphasis, read with the lines around it as they were).
struct Spoilt {
    at: usize,
    own: bool,
}

/// The book a toggle starts from.
struct Book<'a> {
    lines: &'a [&'a str],
    kinds: Vec<TextKind>,
    /// Per line: a boneyard that an earlier line opened covers it.
    inside: Vec<bool>,
}

impl Book<'_> {
    /// Line `at` as the editor shows it.
    fn shown(&self, at: usize) -> ShownLine {
        shown_line(self.lines[at], self.kinds[at], self.inside[at], &READ)
    }

    /// The lines of the book with `changes` made that are not what they were (another kind,
    /// other text, notes or number) or do not show the emphasis they should. A line that is
    /// not changed, and has the kind and the boneyard it had, shows what it showed.
    fn spoilt(&self, changes: &BTreeMap<usize, Change>) -> Vec<Spoilt> {
        let all: Vec<&str> = (0..self.lines.len())
            .map(|at| match changes.get(&at) {
                Some(change) => change.line.as_str(),
                None => self.lines[at],
            })
            .collect();
        let source = all.join("\n");
        let (kinds, inside) = (text_kinds(&source), boneyard_lines(&source));
        let mut found = Vec::new();

        for at in 0..self.lines.len() {
            let in_boneyard = inside.get(at).copied().unwrap_or(false);
            let changed = changes.get(&at);
            let kind_moved = kinds[at] != self.kinds[at];
            let same = !kind_moved && in_boneyard == self.inside[at];
            if !kind_moved && same && changed.is_none() {
                continue;
            }

            let own = match (kind_moved, changed) {
                (true, Some(change)) => self.alone(at, &change.line),
                (true, None) => false,
                (false, _) => {
                    let was = self.shown(at);
                    let now = shown_line(all[at], kinds[at], in_boneyard, &READ);
                    let wanted = match changed {
                        Some(change) => change.marks.clone(),
                        None => marks_of(&was),
                    };
                    let spoilt = was.text != now.text
                        || was.notes != now.notes
                        || was.number != now.number
                        || marks_of(&now) != wanted;
                    if !spoilt {
                        continue;
                    }

                    changed.is_some() && same
                }
            };
            found.push(Spoilt { at, own });
        }

        found
    }

    /// Whether `line` in place of line `at` is another kind by itself: read with two lines
    /// either side as they were. A line that is only another kind because of a neighbour that
    /// changed is not.
    fn alone(&self, at: usize, line: &str) -> bool {
        let mut from = at.saturating_sub(2);
        while from > 0 && self.inside[from] {
            from -= 1;
        }
        let to = (at + 3).min(self.lines.len());
        let mut changed: Vec<&str> = self.lines[from..to].to_vec();
        changed[at - from] = line;
        let old = text_kinds(&self.lines[from..to].join("\n"));
        let new = text_kinds(&changed.join("\n"));

        old[at - from] != new[at - from]
    }

    /// Takes back changes that spoil the book. Those whose own change is the cause all go at
    /// once. If there are none, in every block (up to a blank line) with a spoilt line the first
    /// change that may be the cause: the first changed line that is spoilt, else the first
    /// changed line; what is left is read again, so a line spoilt by its neighbour comes back
    /// to stay. When no block has a change to take back, all of them go.
    fn shed(&self, changes: &mut BTreeMap<usize, Change>, bad: &[Spoilt]) {
        let own: Vec<usize> = bad
            .iter()
            .filter(|s| s.own && changes.contains_key(&s.at))
            .map(|s| s.at)
            .collect();
        if !own.is_empty() {
            own.iter().for_each(|at| drop(changes.remove(at)));
            return;
        }

        let blocks: BTreeSet<(usize, usize)> = bad
            .iter()
            .filter_map(|spoilt| self.block_of(spoilt.at))
            .collect();
        let mut gone = Vec::new();

        for (from, to) in blocks {
            let mut here = changes.range(from..to).map(|(line, _)| *line);
            let first = here
                .clone()
                .find(|line| bad.iter().any(|s| s.at == *line))
                .or(here.next());
            gone.extend(first);
        }
        if gone.is_empty() {
            changes.clear();
        }
        for line in gone {
            changes.remove(&line);
        }
    }

    /// The lines from `from` to `to` (excluding), without a blank line, that line `at` is in
    /// (a blank line is in the block above it); none above a blank start.
    fn block_of(&self, at: usize) -> Option<(usize, usize)> {
        let blank = |n: usize| self.lines[n].trim().is_empty();
        let at = (0..=at).rev().find(|&n| !blank(n))?;
        let from = (0..at).rev().find(|&n| blank(n)).map_or(0, |n| n + 1);
        let to = (at..self.lines.len())
            .find(|&n| blank(n))
            .unwrap_or(self.lines.len());

        Some((from, to))
    }
}
