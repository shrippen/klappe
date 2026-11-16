//! Word wrap in columns (Courier: one character, one column).

use std::ops::Range;

/// The byte ranges of the rows `text` breaks into at `width` columns, at spaces; a word
/// longer than a row is cut. Empty text has no row.
pub(super) fn wrap(text: &str, width: usize) -> Vec<Range<usize>> {
    let width = width.max(1);
    let mut rows = Vec::new();
    // The open row: where it starts, where its last word ends and how many characters.
    let mut open: Option<(usize, usize, usize)> = None;

    for (start, end) in words(text) {
        let len = text[start..end].chars().count();

        if let Some((from, to, used)) = open {
            if used + 1 + len <= width {
                open = Some((from, end, used + 1 + len));
                continue;
            }
            rows.push(from..to);
        }

        open = Some(chunks(text, start, end, len, width, &mut rows));
    }

    if let Some((from, to, _)) = open {
        rows.push(from..to);
    }

    rows
}

/// The words of `text` as byte ranges.
fn words(text: &str) -> impl Iterator<Item = (usize, usize)> + '_ {
    let mut rest = 0;

    std::iter::from_fn(move || {
        let tail = &text[rest..];
        let skipped = tail.len() - tail.trim_start().len();
        let start = rest + skipped;
        let word = &text[start..];
        if word.is_empty() {
            return None;
        }

        let end = start + word.find(char::is_whitespace).unwrap_or(word.len());
        rest = end;

        Some((start, end))
    })
}

/// A word on an empty row; one longer than `width` fills rows of its own, and the last
/// piece is returned as the open row.
fn chunks(
    text: &str,
    start: usize,
    end: usize,
    len: usize,
    width: usize,
    rows: &mut Vec<Range<usize>>,
) -> (usize, usize, usize) {
    if len <= width {
        return (start, end, len);
    }

    let mut from = start;
    let mut left = len;
    while left > width {
        let cut = text[from..]
            .char_indices()
            .nth(width)
            .map_or(end, |(i, _)| from + i);
        rows.push(from..cut);
        from = cut;
        left -= width;
    }

    (from, end, left)
}
