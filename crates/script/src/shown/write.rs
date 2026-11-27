//! Writing a shown text back as Fountain: the marks of emphasis around every stretch, the
//! notes at their places, the backslashes that were there.

use super::ShownNote;
use crate::inline::{NOTE_BRACKET, UNDERLINE};
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
pub(super) fn write_body(part: &Part) -> (String, Vec<Marks>) {
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
