//! The text, the cursor and undo; no terminal in here.

use crate::buffer::{Buffer, Pos};
use crate::layout::Layout;

/// Undo steps kept; older ones are dropped.
const UNDO_LIMIT: usize = 200;

/// What an edit does, so a run of typing or erasing undoes in one step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Edit {
    Insert,
    Erase,
    Break,
}

#[derive(Debug, Clone)]
struct Snapshot {
    buffer: Buffer,
    cursor: Pos,
}

#[derive(Debug)]
pub(crate) struct Editor {
    buffer: Buffer,
    cursor: Pos,
    /// Column within the row that up/down keep; `None` until a vertical move.
    goal: Option<usize>,
    undo: Vec<Snapshot>,
    redo: Vec<Snapshot>,
    last_edit: Option<Edit>,
    /// Counts every change of the text, undo and redo included.
    revision: u64,
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

impl Editor {
    pub(crate) fn new(text: &str) -> Self {
        Self {
            buffer: Buffer::from_text(text),
            cursor: Pos::default(),
            goal: None,
            undo: Vec::new(),
            redo: Vec::new(),
            last_edit: None,
            revision: 0,
        }
    }

    pub(crate) fn text(&self) -> String {
        self.buffer.text()
    }

    pub(crate) fn lines(&self) -> &[String] {
        self.buffer.lines()
    }

    pub(crate) fn cursor(&self) -> Pos {
        self.cursor
    }

    pub(crate) fn revision(&self) -> u64 {
        self.revision
    }

    /// Puts the cursor at the start of a line.
    pub(crate) fn go_to_line(&mut self, line: usize) {
        self.cursor = self.buffer.clamp(Pos { line, col: 0 });
        self.goal = None;
        self.last_edit = None;
    }

    // --- editing -----------------------------------------------------------------

    /// Saves the state for undo, unless this edit continues the one before.
    fn record(&mut self, edit: Edit) {
        self.revision += 1;
        let joins = edit != Edit::Break && self.last_edit == Some(edit);
        self.last_edit = Some(edit);
        self.goal = None;
        self.redo.clear();
        if joins {
            return;
        }

        self.undo.push(self.snapshot());
        if self.undo.len() > UNDO_LIMIT {
            self.undo.remove(0);
        }
    }

    fn snapshot(&self) -> Snapshot {
        Snapshot {
            buffer: self.buffer.clone(),
            cursor: self.cursor,
        }
    }

    pub(crate) fn type_char(&mut self, c: char) {
        self.record(Edit::Insert);
        self.cursor = self.buffer.insert_char(self.cursor, c);
    }

    pub(crate) fn newline(&mut self) {
        self.record(Edit::Break);
        self.cursor = self.buffer.insert_newline(self.cursor);
    }

    pub(crate) fn backspace(&mut self) {
        if self.cursor == Pos::default() {
            return;
        }
        self.record(Edit::Erase);
        self.cursor = self.buffer.backspace(self.cursor);
    }

    pub(crate) fn delete(&mut self) {
        let at_end = self.cursor.col >= self.buffer.line_len(self.cursor.line)
            && self.cursor.line + 1 >= self.buffer.lines().len();
        if at_end {
            return;
        }
        self.record(Edit::Erase);
        self.buffer.delete(self.cursor);
    }

    pub(crate) fn undo(&mut self) {
        let Some(step) = self.undo.pop() else {
            return;
        };
        self.redo.push(self.snapshot());
        self.restore(step);
    }

    pub(crate) fn redo(&mut self) {
        let Some(step) = self.redo.pop() else {
            return;
        };
        self.undo.push(self.snapshot());
        self.restore(step);
    }

    fn restore(&mut self, step: Snapshot) {
        self.buffer = step.buffer;
        self.cursor = step.cursor;
        self.goal = None;
        self.last_edit = None;
        self.revision += 1;
    }

    // --- movement ----------------------------------------------------------------

    fn horizontal(&mut self, pos: Pos) {
        self.cursor = pos;
        self.goal = None;
        self.last_edit = None;
    }

    pub(crate) fn left(&mut self) {
        let Pos { line, col } = self.cursor;
        if col > 0 {
            self.horizontal(Pos { line, col: col - 1 });
            return;
        }
        if line > 0 {
            let col = self.buffer.line_len(line - 1);
            self.horizontal(Pos {
                line: line - 1,
                col,
            });
        }
    }

    pub(crate) fn right(&mut self) {
        let Pos { line, col } = self.cursor;
        if col < self.buffer.line_len(line) {
            self.horizontal(Pos { line, col: col + 1 });
            return;
        }
        if line + 1 < self.buffer.lines().len() {
            self.horizontal(Pos {
                line: line + 1,
                col: 0,
            });
        }
    }

    /// To the start of the word before the cursor.
    pub(crate) fn word_left(&mut self) {
        let Pos { line, col } = self.cursor;
        if col == 0 {
            self.left();
            return;
        }

        let chars: Vec<char> = self.buffer.line(line).chars().collect();
        let mut at = col;
        while at > 0 && !is_word(chars[at - 1]) {
            at -= 1;
        }
        while at > 0 && is_word(chars[at - 1]) {
            at -= 1;
        }
        self.horizontal(Pos { line, col: at });
    }

    /// To the start of the word after the cursor.
    pub(crate) fn word_right(&mut self) {
        let Pos { line, col } = self.cursor;
        let chars: Vec<char> = self.buffer.line(line).chars().collect();
        if col >= chars.len() {
            self.right();
            return;
        }

        let mut at = col;
        while at < chars.len() && is_word(chars[at]) {
            at += 1;
        }
        while at < chars.len() && !is_word(chars[at]) {
            at += 1;
        }
        self.horizontal(Pos { line, col: at });
    }

    /// Start of the screen row.
    pub(crate) fn home(&mut self, layout: &Layout) {
        let Some(row) = layout.row(layout.row_of(self.cursor)) else {
            return;
        };
        self.horizontal(Pos {
            line: row.line,
            col: row.start,
        });
    }

    /// End of the screen row.
    pub(crate) fn end(&mut self, layout: &Layout) {
        let Some(row) = layout.row(layout.row_of(self.cursor)) else {
            return;
        };
        self.horizontal(layout.pos_at(layout.row_of(self.cursor), row.end - row.start));
    }

    /// Moves by screen rows, keeping the column; stops at the first and last row.
    pub(crate) fn rows(&mut self, delta: isize, layout: &Layout) {
        let Some(row) = layout.row(layout.row_of(self.cursor)) else {
            return;
        };
        let from = layout.row_of(self.cursor);
        let last = layout.len().saturating_sub(1);
        let to = from.saturating_add_signed(delta).min(last);
        if to == from {
            return;
        }

        let x = self.goal.unwrap_or(self.cursor.col - row.start);
        self.cursor = layout.pos_at(to, x);
        self.goal = Some(x);
        self.last_edit = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fountain::LineKind;

    fn at(line: usize, col: usize) -> Pos {
        Pos { line, col }
    }

    fn layout_of(ed: &Editor, width: usize) -> Layout {
        let kinds = vec![LineKind::Action; ed.lines().len()];
        Layout::build(ed.lines(), &kinds, width)
    }

    fn typed(ed: &mut Editor, text: &str) {
        for c in text.chars() {
            ed.type_char(c);
        }
    }

    #[test]
    fn typing_umlauts_moves_cursor_by_characters() {
        let mut ed = Editor::new("");
        typed(&mut ed, "ÄÖÜ – ß");

        assert_eq!(ed.text(), "ÄÖÜ – ß");
        assert_eq!(ed.cursor(), at(0, 7));

        ed.left();
        ed.left();
        ed.backspace();
        assert_eq!(ed.text(), "ÄÖÜ  ß");
    }

    #[test]
    fn left_and_right_cross_lines() {
        let mut ed = Editor::new("äb\ncd");
        ed.go_to_line(1);
        ed.left();
        assert_eq!(ed.cursor(), at(0, 2));

        ed.right();
        assert_eq!(ed.cursor(), at(1, 0));
    }

    #[test]
    fn words_skip_punctuation_and_umlauts() {
        let mut ed = Editor::new("Übel – schön, genug");
        ed.word_right();
        assert_eq!(ed.cursor(), at(0, 7));

        ed.word_right();
        assert_eq!(ed.cursor(), at(0, 14));

        ed.word_left();
        assert_eq!(ed.cursor(), at(0, 7));

        ed.word_left();
        assert_eq!(ed.cursor(), at(0, 0));
    }

    #[test]
    fn word_move_wraps_to_neighbour_line() {
        let mut ed = Editor::new("ab\ncd");
        ed.word_right();
        ed.word_right();
        assert_eq!(ed.cursor(), at(1, 0));

        ed.word_left();
        assert_eq!(ed.cursor(), at(0, 2));
    }

    #[test]
    fn undo_redo_groups_typing() {
        let mut ed = Editor::new("");
        typed(&mut ed, "abc");
        ed.newline();
        typed(&mut ed, "de");
        assert_eq!(ed.text(), "abc\nde");

        ed.undo();
        assert_eq!(ed.text(), "abc\n");

        ed.undo();
        assert_eq!(ed.text(), "abc");

        ed.undo();
        assert_eq!(ed.text(), "");

        ed.redo();
        ed.redo();
        ed.redo();
        assert_eq!(ed.text(), "abc\nde");
        assert_eq!(ed.cursor(), at(1, 2));
    }

    #[test]
    fn edit_after_undo_drops_redo() {
        let mut ed = Editor::new("");
        typed(&mut ed, "a");
        ed.undo();
        typed(&mut ed, "b");
        ed.redo();

        assert_eq!(ed.text(), "b");
    }

    #[test]
    fn undo_and_redo_count_as_changes() {
        let mut ed = Editor::new("");
        let r0 = ed.revision();
        typed(&mut ed, "a");
        let r1 = ed.revision();
        ed.undo();

        assert!(r1 > r0);
        assert!(ed.revision() > r1);
    }

    #[test]
    fn delete_and_backspace_at_the_ends_change_nothing() {
        let mut ed = Editor::new("a");
        ed.backspace();
        ed.go_to_line(0);
        ed.right();
        ed.delete();

        assert_eq!(ed.text(), "a");
        assert_eq!(ed.revision(), 0);
    }

    #[test]
    fn vertical_moves_follow_wrapped_rows_and_keep_column() {
        let mut ed = Editor::new("aaaaaaaa bbbbbbbb\nxy");
        let l = layout_of(&ed, 10);
        ed.right();
        ed.right();
        ed.right();

        ed.rows(1, &l);
        assert_eq!(ed.cursor(), at(0, 12));

        ed.rows(1, &l);
        assert_eq!(ed.cursor(), at(1, 2));

        ed.rows(-1, &l);
        assert_eq!(ed.cursor(), at(0, 12));

        ed.rows(-1, &l);
        assert_eq!(ed.cursor(), at(0, 3));
    }

    #[test]
    fn home_and_end_work_on_the_screen_row() {
        let mut ed = Editor::new("aaaaaaaa bbbbbbbb");
        let l = layout_of(&ed, 10);
        ed.rows(1, &l);
        ed.right();
        ed.right();

        ed.home(&l);
        assert_eq!(ed.cursor(), at(0, 9));

        ed.end(&l);
        assert_eq!(ed.cursor(), at(0, 17));
    }
}
