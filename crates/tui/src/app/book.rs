//! The questions the register leaves open about the book (`Ctrl+K`): which scene of the
//! book an entry is, one question at a time.

use klappe_services::{Answer, PendingScene};
use ratatui::crossterm::event::{KeyCode, KeyEvent};

use super::{App, Editor, Flow, Mode, Notice};

impl App {
    /// The question being asked: the first open one.
    pub(crate) fn book_question(&self) -> Option<&PendingScene> {
        self.session.pending_scenes().first()
    }

    /// The lines of the question being asked, the last one "not in the book".
    pub(crate) fn book_answers(&self) -> Vec<Answer> {
        let Some(question) = self.book_question() else {
            return Vec::new();
        };

        self.session.book_answers(&question.id)
    }

    pub(super) fn open_book(&mut self) -> Flow {
        if self.book_question().is_none() {
            return Flow::Continue;
        }
        // An answer is written at once: against the saved book only, never against text the
        // user may still discard (the session holds the autosaved text).
        if self.is_modified() {
            self.notice = Some(Notice::SaveFirst);
            return Flow::Continue;
        }
        self.book_sel = 0;
        self.mode = Mode::Book;
        self.suggest = None;

        Flow::Continue
    }

    pub(super) fn book_key(&mut self, key: KeyEvent) -> Flow {
        if self.book_question().is_none() {
            self.mode = Mode::Normal;
            return Flow::Continue;
        }
        let options = self.book_answers().len();

        match key.code {
            KeyCode::Esc => self.mode = Mode::Normal,
            KeyCode::Down => self.book_sel = (self.book_sel + 1) % options,
            KeyCode::Up => self.book_sel = (self.book_sel + options - 1) % options,
            KeyCode::Enter => self.answer_book(),
            _ => {}
        }

        Flow::Continue
    }

    /// Gives the chosen answer to the session and goes on with the next question.
    fn answer_book(&mut self) {
        let Some(question) = self.book_question() else {
            return;
        };
        let id = question.id.clone();

        // The answer is written at once: the TUI saves no plans, so quitting asks nothing.
        let answered = self.session.answer_book(&id, self.book_sel);
        if let Err(e) = answered.and_then(|()| self.session.save_register()) {
            self.notice = Some(Notice::SaveFailed(e.to_string()));
            self.mode = Mode::Normal;
            return;
        }

        self.answered.push(self.editor.mark_step());
        self.unanswered.clear();
        self.book_sel = 0;
        self.refresh_book_note();
        if self.book_question().is_none() {
            self.mode = Mode::Normal;
        }
    }

    /// Ctrl+Z: takes back the latest of editor edit and book answer. The answer is the
    /// latest when the editor has no step since it.
    pub(super) fn undo(&mut self) -> Flow {
        if self.answered.last() != Some(&self.editor.undo_depth()) {
            return self.edit_with(Editor::undo);
        }

        self.answered.pop();
        self.session.undo();
        self.unanswered.push(self.editor.undo_depth());

        self.settle_answer()
    }

    /// Ctrl+Y: does a taken-back answer again when the editor is back at the step it was at
    /// then (its undo depth, not its revision, which every undo and redo raises).
    pub(super) fn redo(&mut self) -> Flow {
        if self.unanswered.last() != Some(&self.editor.undo_depth()) {
            return self.edit_with(Editor::redo);
        }

        self.unanswered.pop();
        self.session.redo();
        self.answered.push(self.editor.mark_step());

        self.settle_answer()
    }

    /// Answers are no longer taken back (the register moved on with a saved change).
    pub(super) fn forget_answers(&mut self) {
        self.answered.clear();
        self.unanswered.clear();
    }

    /// Writes the register as undo or redo left it and shows its questions.
    fn settle_answer(&mut self) -> Flow {
        if let Err(e) = self.session.save_register() {
            self.notice = Some(Notice::SaveFailed(e.to_string()));
        }
        self.book_sel = 0;
        self.refresh_book_note();

        Flow::Continue
    }
}
