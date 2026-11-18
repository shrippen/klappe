//! The text, the cursor and undo; no terminal in here.

use crate::buffer::{Buffer, Pos};
use crate::history::{Before, History};
use crate::layout::Layout;

/// What an edit does, so a run of typing or erasing undoes in one step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Edit {
    Insert,
    Erase,
    Break,
}

/// Whether a typed character starts a new undo step (the first letter of a word).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    Join,
    New,
}

#[derive(Debug)]
pub(crate) struct Editor {
    buffer: Buffer,
    cursor: Pos,
    /// Display column within the row that up/down keep; `None` until a vertical move.
    goal: Option<usize>,
    history: History,
    last_edit: Option<Edit>,
    /// Counts every change of the text, undo and redo included.
    revision: u64,
    /// New edits made (not undo or redo): a new edit drops what redo could bring back.
    edits: u64,
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
            history: History::default(),
            last_edit: None,
            revision: 0,
            edits: 0,
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

    /// Notes the point between two edits where something else happened (a book answer): the
    /// next edit starts a step of its own. Returns how many steps lie before the point.
    pub(crate) fn mark_step(&mut self) -> usize {
        self.last_edit = None;

        self.undo_depth()
    }

    /// How many new edits were made, undo and redo not counted.
    pub(crate) fn edits(&self) -> u64 {
        self.edits
    }

    /// How many steps undo can take back.
    pub(crate) fn undo_depth(&self) -> usize {
        self.history.depth()
    }

    /// Puts the cursor at a position.
    pub(crate) fn go_to(&mut self, pos: Pos) {
        self.cursor = self.buffer.clamp(pos);
        self.goal = None;
        self.last_edit = None;
    }

    /// Puts the cursor at the start of a line.
    pub(crate) fn go_to_line(&mut self, line: usize) {
        self.cursor = self.buffer.clamp(Pos { line, col: 0 });
        self.goal = None;
        self.last_edit = None;
    }

    // --- editing -----------------------------------------------------------------

    /// Runs `change` on the buffer and records what it did for undo. Everything an edit can
    /// touch lies within the line of the cursor and its neighbours; `replace_line` may
    /// rewrite the cursor line afterwards, as part of the same step.
    fn edit(&mut self, kind: Edit, step: Step, change: impl FnOnce(&mut Buffer, Pos) -> Pos) {
        let window = self.buffer.window(self.cursor.line);
        let before = Before::of(&self.buffer, window.start, window.end);
        let from = self.cursor;

        self.cursor = change(&mut self.buffer, from);
        let joins = step == Step::Join && kind != Edit::Break && self.last_edit == Some(kind);
        self.last_edit = Some(kind);
        self.finish(before, from, joins);
    }

    fn finish(&mut self, before: Before, from: Pos, joins: bool) {
        self.revision += 1;
        self.edits += 1;
        self.goal = None;
        self.history
            .record(before, &self.buffer, (from, self.cursor), joins);
    }

    pub(crate) fn type_char(&mut self, c: char) {
        let at = self.cursor;
        let after_space = at.col > 0
            && self
                .buffer
                .line(at.line)
                .chars()
                .nth(at.col - 1)
                .is_some_and(char::is_whitespace);
        let step = match after_space && !c.is_whitespace() {
            true => Step::New,
            false => Step::Join,
        };

        self.edit(Edit::Insert, step, |b, p| b.insert_char(p, c));
    }

    /// Rewrites the cursor line as part of the edit just made, so undo takes both back.
    pub(crate) fn replace_line(&mut self, text: String) {
        self.buffer.set_line(self.cursor.line, text);
    }

    pub(crate) fn newline(&mut self) {
        self.edit(Edit::Break, Step::New, |b, p| b.insert_newline(p));
    }

    pub(crate) fn backspace(&mut self) {
        if self.cursor == Pos::default() {
            return;
        }
        self.edit(Edit::Erase, Step::Join, |b, p| b.backspace(p));
    }

    pub(crate) fn delete(&mut self) {
        let at_end = self.cursor.col >= self.buffer.line_len(self.cursor.line)
            && self.cursor.line + 1 >= self.buffer.lines().len();
        if at_end {
            return;
        }
        self.edit(Edit::Erase, Step::Join, |b, p| {
            b.delete(p);
            p
        });
    }

    /// Replaces lines `from..to` by `lines` and puts the cursor at `cursor`; one undo step.
    /// For what is more than a keystroke: a suggestion taken, a replacement.
    pub(crate) fn set_lines(&mut self, from: usize, to: usize, lines: Vec<String>, cursor: Pos) {
        let before = Before::of(&self.buffer, from, to);
        let start = self.cursor;
        if before.is(&lines) {
            // Nothing to take back: only the cursor moves.
            self.go_to(cursor);
            return;
        }

        self.buffer.splice(from, to, lines);
        self.cursor = self.buffer.clamp(cursor);
        self.last_edit = None;
        self.finish(before, start, false);
    }

    /// Replaces the whole text, as one undo step that spans only the lines that differ.
    pub(crate) fn set_text(&mut self, text: &str) {
        self.set_text_at(text, self.cursor);
    }

    /// Like [`Self::set_text`], with the cursor at `cursor` after the step (and after a redo).
    pub(crate) fn set_text_at(&mut self, text: &str, cursor: Pos) {
        let new = Buffer::from_text(text);
        let old = self.buffer.lines();
        let (new_lines, old_len) = (new.lines(), old.len());

        let same_head = old
            .iter()
            .zip(new_lines)
            .take_while(|(a, b)| a == b)
            .count();
        let room = old_len.min(new_lines.len()) - same_head;
        let same_tail = old
            .iter()
            .rev()
            .zip(new_lines.iter().rev())
            .take(room)
            .take_while(|(a, b)| a == b)
            .count();
        if same_head == old_len && old_len == new_lines.len() {
            self.go_to(cursor);
            return;
        }

        let changed = new_lines[same_head..new_lines.len() - same_tail].to_vec();
        self.set_lines(same_head, old_len - same_tail, changed, cursor);
    }

    pub(crate) fn undo(&mut self) {
        let Some(cursor) = self.history.undo(&mut self.buffer) else {
            return;
        };
        self.after_history(cursor);
    }

    pub(crate) fn redo(&mut self) {
        let Some(cursor) = self.history.redo(&mut self.buffer) else {
            return;
        };
        self.after_history(cursor);
    }

    fn after_history(&mut self, cursor: Pos) {
        self.cursor = self.buffer.clamp(cursor);
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
        let at = layout.row_of(self.cursor);
        if layout.row(at).is_none() {
            return;
        }

        self.horizontal(layout.pos_at(at, layout.row_cols(at)));
    }

    /// Moves by screen rows, keeping the display column; stops at the first and last row.
    pub(crate) fn rows(&mut self, delta: isize, layout: &Layout) {
        let from = layout.row_of(self.cursor);
        if layout.row(from).is_none() {
            return;
        }

        let last = layout.len().saturating_sub(1);
        let to = from.saturating_add_signed(delta).min(last);
        // A page break row is no place for the cursor; go on past it, or stay.
        let Some(to) = layout.text_row(to, delta).filter(|&to| to != from) else {
            return;
        };

        let x = self
            .goal
            .unwrap_or(layout.col_in_row(from, self.cursor.col));
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
        Layout::build(ed.lines(), &kinds, width, &[])
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
    fn replace_line_joins_the_typing_step() {
        let mut ed = Editor::new("");
        ed.type_char('a');
        ed.replace_line("A".into());
        ed.type_char('b');
        ed.replace_line("AB".into());
        assert_eq!(ed.text(), "AB");

        ed.undo();
        assert_eq!(ed.text(), "");
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
    fn typing_undoes_by_word() {
        let mut ed = Editor::new("");
        typed(&mut ed, "ab cd  ef");

        ed.undo();
        assert_eq!(ed.text(), "ab cd  ");
        ed.undo();
        assert_eq!(ed.text(), "ab ");
        ed.undo();
        assert_eq!(ed.text(), "");
    }

    #[test]
    fn undo_has_no_fixed_limit() {
        let mut ed = Editor::new("");
        for _ in 0..1000 {
            ed.newline();
        }
        assert_eq!(ed.history.depth(), 1000);

        for _ in 0..1000 {
            ed.undo();
        }
        assert_eq!(ed.text(), "");
        for _ in 0..1000 {
            ed.redo();
        }
        assert_eq!(ed.lines().len(), 1001);
    }

    #[test]
    fn a_step_keeps_lines_not_the_text() {
        let text = "x\n".repeat(5000);
        let mut ed = Editor::new(&text);
        ed.go_to_line(2500);
        typed(&mut ed, "ab");
        ed.newline();

        // Steps hold the lines around the cursor, however long the text is.
        let kept: usize = ed.history.lines_kept();
        assert!(kept < 20, "{kept}");
        ed.undo();
        ed.undo();
        assert_eq!(ed.text(), text);
    }

    #[test]
    fn erasing_across_lines_is_one_step_and_undoes_exactly() {
        let mut ed = Editor::new("one\ntwo\nthree\nfour");
        ed.go_to_line(3);
        ed.backspace();
        ed.backspace();
        ed.backspace();
        assert_eq!(ed.text(), "one\ntwo\nthrfour");

        ed.undo();
        assert_eq!(ed.text(), "one\ntwo\nthree\nfour");
        assert_eq!(ed.cursor(), at(3, 0));
        ed.redo();
        assert_eq!(ed.text(), "one\ntwo\nthrfour");
        assert_eq!(ed.cursor(), at(2, 3));
    }

    #[test]
    fn set_lines_is_one_step_and_set_text_spans_the_difference() {
        let mut ed = Editor::new("a\nb\nc\nd\ne");
        ed.set_text("a\nB\nB2\nd\ne");
        assert_eq!(ed.text(), "a\nB\nB2\nd\ne");
        assert_eq!(ed.history.lines_kept(), 2 + 2);

        ed.undo();
        assert_eq!(ed.text(), "a\nb\nc\nd\ne");
        ed.redo();
        assert_eq!(ed.text(), "a\nB\nB2\nd\ne");

        let r = ed.revision();
        ed.set_text("a\nB\nB2\nd\ne");
        assert_eq!(ed.revision(), r);
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
    fn vertical_moves_keep_the_display_column_of_wide_chars() {
        let mut ed = Editor::new("日本語\nabcdefgh");
        let l = layout_of(&ed, 20);

        ed.end(&l);
        ed.rows(1, &l);
        assert_eq!(ed.cursor(), at(1, 6));

        ed.rows(-1, &l);
        assert_eq!(ed.cursor(), at(0, 3));
    }

    #[test]
    fn wide_char_covering_the_column_is_stepped_before() {
        let mut ed = Editor::new("abcde\na日本");
        let l = layout_of(&ed, 20);
        for _ in 0..2 {
            ed.right();
        }

        // Display column 2 is the middle of 日 (columns 1..3).
        ed.rows(1, &l);
        assert_eq!(ed.cursor(), at(1, 1));
    }

    #[test]
    fn vertical_moves_keep_the_display_column_of_tabs() {
        let mut ed = Editor::new("\tx\nabcdefgh");
        let l = layout_of(&ed, 20);
        ed.right();

        ed.rows(1, &l);
        assert_eq!(ed.cursor(), at(1, 4));

        ed.rows(-1, &l);
        assert_eq!(ed.cursor(), at(0, 1));
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
