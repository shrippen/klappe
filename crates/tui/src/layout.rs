//! Soft wrapping: which part of which line is on which screen row.

use crate::buffer::Pos;
use crate::fountain::LineKind;

/// Narrowest text width a line is wrapped at, however small the window.
const MIN_WRAP: usize = 8;
/// An indent takes at most this share of the width: 1/3.
const INDENT_SHARE: usize = 3;

/// One screen row: characters `start..end` of `line`, after `indent` blank columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Row {
    pub line: usize,
    pub start: usize,
    pub end: usize,
    pub indent: usize,
    /// The last row of its line; only there the cursor may stand behind the last character.
    pub last: bool,
}

#[derive(Debug, Default)]
pub(crate) struct Layout {
    rows: Vec<Row>,
    first_row: Vec<usize>,
}

/// Splits a line into `(start, end)` character ranges of at most `width`, at spaces.
///
/// The space stays at the end of the row it ends, so ranges are gapless and the cursor
/// has a place for every column. An empty line has one empty range.
pub(crate) fn wrap(line: &str, width: usize) -> Vec<(usize, usize)> {
    let chars: Vec<char> = line.chars().collect();
    let width = width.max(1);
    let mut out = Vec::new();
    let mut start = 0;

    while chars.len() - start > width {
        let limit = start + width;
        let space = chars[start..limit].iter().rposition(|&c| c == ' ');
        let end = match space {
            Some(at) => start + at + 1,
            None => limit,
        };
        out.push((start, end));
        start = end;
    }

    out.push((start, chars.len()));
    out
}

impl Layout {
    pub(crate) fn build(lines: &[String], kinds: &[LineKind], width: usize) -> Self {
        let mut rows = Vec::new();
        let mut first_row = Vec::with_capacity(lines.len());

        for (i, line) in lines.iter().enumerate() {
            let kind = kinds.get(i).copied().unwrap_or(LineKind::Action);
            let indent = kind.indent().min(width / INDENT_SHARE);
            // One column stays free for the cursor behind the last character.
            let room = width.saturating_sub(indent + 1).max(MIN_WRAP);
            let parts = wrap(line, room);
            let count = parts.len();

            first_row.push(rows.len());
            for (n, (start, end)) in parts.into_iter().enumerate() {
                rows.push(Row {
                    line: i,
                    start,
                    end,
                    indent,
                    last: n + 1 == count,
                });
            }
        }

        Self { rows, first_row }
    }

    pub(crate) fn len(&self) -> usize {
        self.rows.len()
    }

    pub(crate) fn row(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }

    /// Screen row of a position; at a wrap point the cursor is at the start of the next row.
    pub(crate) fn row_of(&self, pos: Pos) -> usize {
        let mut at = self.first_row.get(pos.line).copied().unwrap_or(0);

        while let Some(next) = self.rows.get(at + 1) {
            if next.line != pos.line || next.start > pos.col {
                break;
            }
            at += 1;
        }

        at
    }

    /// Position at column `x` of a row, kept on that row.
    pub(crate) fn pos_at(&self, row: usize, x: usize) -> Pos {
        let Some(row) = self.rows.get(row).or(self.rows.last()) else {
            return Pos::default();
        };
        let len = row.end - row.start;
        let max = if row.last { len } else { len.saturating_sub(1) };

        Pos {
            line: row.line,
            col: row.start + x.min(max),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(line: usize, col: usize) -> Pos {
        Pos { line, col }
    }

    #[test]
    fn short_and_empty_lines_are_one_row() {
        assert_eq!(wrap("", 10), [(0, 0)]);
        assert_eq!(wrap("abc", 10), [(0, 3)]);
        assert_eq!(wrap("abcdefghij", 10), [(0, 10)]);
    }

    #[test]
    fn breaks_after_the_last_space_that_fits() {
        assert_eq!(wrap("aaa bbb ccc", 8), [(0, 8), (8, 11)]);
    }

    #[test]
    fn long_word_is_cut_at_the_width() {
        assert_eq!(wrap("abcdefghijkl", 5), [(0, 5), (5, 10), (10, 12)]);
    }

    #[test]
    fn ranges_count_characters_not_bytes() {
        assert_eq!(wrap("ÄÖÜ ÄÖÜ ÄÖÜ", 8), [(0, 8), (8, 11)]);
    }

    fn layout(lines: &[&str], kinds: &[LineKind], width: usize) -> Layout {
        let lines: Vec<String> = lines.iter().map(|s| (*s).to_owned()).collect();
        Layout::build(&lines, kinds, width)
    }

    #[test]
    fn rows_follow_lines_and_wrapping() {
        let l = layout(&["aaaaaaaa bbbbbbbb", "", "x"], &[LineKind::Action; 3], 10);

        // room is 9: "aaaaaaaa " / "bbbbbbbb"
        assert_eq!(l.len(), 4);
        assert_eq!(l.row_of(at(0, 0)), 0);
        assert_eq!(l.row_of(at(0, 8)), 0);
        assert_eq!(l.row_of(at(0, 9)), 1);
        assert_eq!(l.row_of(at(0, 17)), 1);
        assert_eq!(l.row_of(at(1, 0)), 2);
        assert_eq!(l.row_of(at(2, 1)), 3);
    }

    #[test]
    fn pos_at_stays_on_the_row() {
        let l = layout(&["aaaaaaaa bbbbbbbb"], &[LineKind::Action], 10);

        assert_eq!(l.pos_at(0, 99), at(0, 8));
        assert_eq!(l.pos_at(1, 99), at(0, 17));
        assert_eq!(l.pos_at(1, 2), at(0, 11));
    }

    #[test]
    fn indent_narrows_the_row() {
        let l = layout(&["x"], &[LineKind::Character], 60);

        assert_eq!(l.row(0).map(|r| r.indent), Some(20));
    }

    #[test]
    fn indent_is_capped_in_a_narrow_window() {
        let l = layout(&["x"], &[LineKind::Character], 30);

        assert_eq!(l.row(0).map(|r| r.indent), Some(10));
    }
}
