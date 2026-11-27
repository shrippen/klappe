//! Writing a shown text back as Fountain: the marks of emphasis around every stretch, the
//! notes at their places, the backslashes that were there.

use super::ShownNote;
use crate::inline::{seen_chars, NOTE_BRACKET, UNDERLINE};
use crate::parse::{ESCAPE, NOTE_CLOSE, NOTE_OPEN, STAR};
use crate::Marks;

const SLASH: char = '/';

/// What a shown text is written from.
pub(super) struct Part<'a> {
    pub(super) chars: &'a [char],
    pub(super) marks: &'a [Marks],
    pub(super) escaped: &'a [bool],
    pub(super) notes: &'a [ShownNote],
}

/// The emphasis wanted for the stretch being written, and the part of it that stands open in
/// the written text. They differ where stars were left out (see [`open`]).
#[derive(Clone, Copy, PartialEq, Eq)]
struct Emphasis {
    wanted: Marks,
    open: Marks,
}

const PLAIN: Marks = Marks {
    bold: false,
    italic: false,
    underline: false,
    note: false,
    boneyard: false,
};

/// Whether a mark that goes on over the next stretch stays open.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Nest {
    Yes,
    No,
}

/// The text of `part` as Fountain, with a mark around every stretch of emphasis. A mark that
/// goes on over the next stretch stays open: `**a *b* c**` is bold with italic inside, unless
/// a reader would not read that back; then every stretch has its own marks.
/// The emphasis it wrote comes with it: a stretch that could not be marked is plain there.
pub(super) fn write_body(part: &Part) -> (String, Vec<Marks>) {
    // Without emphasis there is nothing to nest, and nothing to read back.
    if part.marks.iter().all(|m| *m == PLAIN) {
        return write_with(part, Nest::No);
    }

    let nested = write_with(part, Nest::Yes);
    if reads_back(&nested, part) {
        return nested;
    }

    write_with(part, Nest::No)
}

/// Whether the written text has the characters of `part` with the emphasis it says it wrote.
fn reads_back((text, written): &(String, Vec<Marks>), part: &Part) -> bool {
    let read: Vec<(char, Marks)> = seen_chars(text)
        .into_iter()
        .filter(|seen| !seen.marks.note)
        .map(|seen| (seen.ch, seen.marks))
        .collect();

    read.len() == part.chars.len()
        && read
            .iter()
            .zip(part.chars.iter().zip(written))
            .all(|((got, marks), (ch, want))| got == ch && marks == want)
}

fn write_with(part: &Part, nest: Nest) -> (String, Vec<Marks>) {
    let mut out = String::new();
    let mut written = part.marks.to_vec();
    let mut tail = Tail::Free;
    let mut now = Emphasis {
        wanted: PLAIN,
        open: PLAIN,
    };

    for i in 0..=part.chars.len() {
        let want = part.marks.get(i).copied().unwrap_or(PLAIN);
        let notes: Vec<&ShownNote> = part
            .notes
            .iter()
            .filter(|n| n.at.min(part.chars.len()) == i)
            .collect();

        if want != now.wanted {
            now = restyle(&mut out, (now, tail), (want, nest), &notes);
        } else {
            put_notes(&mut out, &notes);
        }

        let Some(&ch) = part.chars.get(i) else {
            break;
        };
        written[i] = now.open;
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

/// Goes on from the emphasis `now` to `want`: what ends closes, the notes that stand here come
/// next (a note between two stretches stands outside what ends and what starts there), then
/// what starts opens.
fn restyle(
    out: &mut String,
    (now, tail): (Emphasis, Tail),
    (want, nest): (Marks, Nest),
    notes: &[&ShownNote],
) -> Emphasis {
    let (shut, start) = moves(now.open, want, nest);
    let mut tail = tail;
    if !stars(shut).is_empty() || shut.underline {
        tail = tail_after_close(shut);
    }
    close(out, shut);

    if !notes.is_empty() {
        put_notes(out, notes);
        tail = Tail::Free;
    }
    let kept = Marks {
        bold: now.open.bold && !shut.bold,
        italic: now.open.italic && !shut.italic,
        underline: now.open.underline && !shut.underline,
        ..PLAIN
    };
    let open = open(out, start, kept, tail);

    Emphasis { wanted: want, open }
}

fn put_notes(out: &mut String, notes: &[&ShownNote]) {
    for note in notes {
        out.push_str(&format!("{NOTE_OPEN}{}{NOTE_CLOSE}", note.text));
    }
}

/// What closes and what opens to go from the emphasis `open` to `want`: what both have goes
/// on. Stars that close next to stars that open would touch (`*` and `**` read as `***`), so
/// then everything closes and opens again.
fn moves(open: Marks, want: Marks, nest: Nest) -> (Marks, Marks) {
    if nest == Nest::No {
        return (open, want);
    }

    let shut = Marks {
        bold: open.bold && !want.bold,
        italic: open.italic && !want.italic,
        underline: open.underline && !want.underline,
        ..PLAIN
    };
    let start = Marks {
        bold: want.bold && !open.bold,
        italic: want.italic && !open.italic,
        underline: want.underline && !open.underline,
        ..PLAIN
    };
    if !stars(shut).is_empty() && !stars(start).is_empty() {
        return (open, want);
    }

    (shut, start)
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

/// What the text ends with after what `shut` closes: stars, unless an underline follows them.
fn tail_after_close(shut: Marks) -> Tail {
    match stars(shut).is_empty() || shut.underline {
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

/// Opens `start`, which goes on to what `kept` has open; the emphasis open then. Stars that
/// would touch the stars before them, or follow a `/`, are left out: that stretch stays
/// without them.
fn open(out: &mut String, start: Marks, kept: Marks, tail: Tail) -> Marks {
    let blocked = !stars(start).is_empty() && !start.underline && tail != Tail::Free;
    let start = match blocked {
        true => Marks {
            bold: false,
            italic: false,
            ..start
        },
        false => start,
    };

    if start.underline {
        out.push(UNDERLINE);
    }
    out.push_str(stars(start));

    Marks {
        bold: kept.bold || start.bold,
        italic: kept.italic || start.italic,
        underline: kept.underline || start.underline,
        ..PLAIN
    }
}

/// Closes what `shut` names: the stars, then the underline.
fn close(out: &mut String, shut: Marks) {
    out.push_str(stars(shut));
    if shut.underline {
        out.push(UNDERLINE);
    }
}
