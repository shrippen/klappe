//! Reading a written line back: does it say what the change left alone?

use std::ops::Range;

use super::{marks_of, Part, ShownLine, ShownNote};
use crate::Marks;

/// Whether `read` has the text, emphasis and notes of `part` before and behind what was typed
/// (`effective` is the emphasis the writer managed to write). `skip` characters at the start
/// of the read text are not the text: a forcing character that shows.
pub(super) fn reads_right(
    read: &ShownLine,
    skip: usize,
    part: &Part,
    (effective, typed): (&[Marks], &Range<usize>),
) -> bool {
    let all: Vec<char> = read.text.chars().collect();
    if all.len() < skip {
        return false;
    }

    let chars = &all[skip..];
    let marks = &marks_of(&read.runs, all.len())[skip..];
    let notes: Vec<ShownNote> = read
        .notes
        .iter()
        .map(|n| ShownNote {
            at: n.at.saturating_sub(skip),
            text: n.text.clone(),
        })
        .collect();
    let Some(back) = chars.len().checked_sub(part.chars.len() - typed.end) else {
        return false;
    };
    if back < typed.start {
        return false;
    }

    chars[..typed.start] == part.chars[..typed.start]
        && marks[..typed.start] == effective[..typed.start]
        && chars[back..] == part.chars[typed.end..]
        && marks[back..] == effective[typed.end..]
        && notes_right(&notes, part.notes, typed, back)
}

/// Whether the notes before the typed text are the first of `read` and the notes behind it
/// the last, `back` being where the text behind the typed starts in the read text.
fn notes_right(read: &[ShownNote], notes: &[ShownNote], typed: &Range<usize>, back: usize) -> bool {
    let head: Vec<ShownNote> = notes
        .iter()
        .filter(|n| n.at < typed.start)
        .cloned()
        .collect();
    let tail: Vec<ShownNote> = notes
        .iter()
        .filter(|n| n.at >= typed.end)
        .map(|n| ShownNote {
            at: n.at - typed.end + back,
            text: n.text.clone(),
        })
        .collect();
    if read.len() < head.len() + tail.len() {
        return false;
    }

    read[..head.len()] == head[..] && read[read.len() - tail.len()..] == tail[..]
}
