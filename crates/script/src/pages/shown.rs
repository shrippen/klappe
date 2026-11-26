//! Text as a page sets it: what a reader sees, without the marks of emphasis, with the
//! emphasis of each character (D-36).

use std::ops::Range;

use crate::inline::{runs, seen_chars};
use crate::{Marks, Run};

/// The visible text of typed text, with per byte its emphasis and its place in the typed text.
#[derive(Debug, Clone)]
pub(super) struct Shown {
    pub text: String,
    pub marks: Vec<Marks>,
    /// The byte of the typed text each byte comes from; ascending.
    pub raw: Vec<usize>,
}

impl Shown {
    /// Reads `typed` line by line: emphasis never runs over a line break.
    pub(super) fn of(typed: &str) -> Self {
        let mut shown = Shown {
            text: String::new(),
            marks: Vec::new(),
            raw: Vec::new(),
        };
        let mut start = 0;

        for (i, line) in typed.split('\n').enumerate() {
            if i > 0 {
                shown.push('\n', Marks::default(), start - 1);
            }
            for seen in seen_chars(line) {
                shown.push(seen.ch, emphasis(seen.marks), start + seen.at);
            }
            start += line.len() + 1;
        }

        shown
    }

    fn push(&mut self, ch: char, marks: Marks, at: usize) {
        self.text.push(ch);
        for k in 0..ch.len_utf8() {
            self.marks.push(marks);
            self.raw.push(at + k);
        }
    }

    /// The part `range` of the bytes.
    pub(super) fn slice(&self, range: Range<usize>) -> Shown {
        Shown {
            text: self.text[range.clone()].to_string(),
            marks: self.marks[range.clone()].to_vec(),
            raw: self.raw[range].to_vec(),
        }
    }

    /// `text` instead, which starts with the present text; the rest has no emphasis.
    pub(super) fn extend_to(&mut self, text: String) {
        let extra = text.len().saturating_sub(self.text.len());
        let last = self.raw.last().copied().unwrap_or_default();

        self.marks
            .extend(std::iter::repeat_n(Marks::default(), extra));
        self.raw.extend(std::iter::repeat_n(last, extra));
        self.text = text;
    }

    /// The runs of the bytes `range`, by character; they cover the text.
    pub(super) fn runs(&self, range: Range<usize>) -> Vec<Run> {
        let marks: Vec<Marks> = self.text[range.clone()]
            .char_indices()
            .map(|(i, _)| self.marks[range.start + i])
            .collect();

        runs(&marks)
    }
}

/// Only the emphasis counts on a page; notes print as typed.
fn emphasis(marks: Marks) -> Marks {
    Marks {
        bold: marks.bold,
        italic: marks.italic,
        underline: marks.underline,
        ..Marks::default()
    }
}

/// One run without emphasis over `text`; none for empty text.
pub(super) fn plain(text: &str) -> Vec<Run> {
    runs(&vec![Marks::default(); text.chars().count()])
}

/// `runs` of a text, for the text `now` that starts with it: the rest has no emphasis.
pub(super) fn padded(runs: &[Run], now: &str) -> Vec<Run> {
    let end = runs.last().map_or(0, |r| r.end);
    let all = now.chars().count();
    let mut out = runs.to_vec();

    match out.last_mut() {
        _ if all <= end => {}
        Some(last) if last.marks == Marks::default() => last.end = all,
        _ => out.push(Run {
            start: end,
            end: all,
            marks: Marks::default(),
        }),
    }

    out
}
