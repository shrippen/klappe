//! The questions the register leaves open about the book (`Ctrl+K`): which scene of the
//! book an entry is, then which play location a name of a heading is (DB-51), one question
//! at a time.

use klappe_services::{error_text, Answer, PendingPlace, PendingScene, PlaceAnswer};
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

    /// The location of the headings asked about once no scene is: the first open one.
    pub(crate) fn place_question(&self) -> Option<&PendingPlace> {
        self.session.pending_places().first()
    }

    /// The lines of the place question being asked.
    pub(crate) fn place_answers(&self) -> Vec<PlaceAnswer> {
        let Some(question) = self.place_question() else {
            return Vec::new();
        };

        self.session.place_answers(&question.name)
    }

    /// Whether any question about the book is open.
    fn book_asks(&self) -> bool {
        self.book_question().is_some() || self.place_question().is_some()
    }

    pub(super) fn open_book(&mut self) -> Flow {
        if !self.book_asks() {
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
        if !self.book_asks() {
            self.mode = Mode::Normal;
            return Flow::Continue;
        }
        let options = match self.book_question() {
            Some(_) => self.book_answers().len(),
            None => self.place_answers().len(),
        };

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
            return self.answer_place();
        };
        let id = question.id.clone();

        // The answer is written at once: the TUI saves no plans, so quitting asks nothing.
        let answered = self.session.answer_book(&id, self.book_sel);
        if let Err(e) = answered.and_then(|()| self.session.save_register()) {
            self.notice = Some(Notice::SaveFailed(error_text(self.lang, &e)));
            self.mode = Mode::Normal;
            return;
        }

        self.answered.push(self.editor.mark_step());
        self.unanswered.clear();
        self.book_sel = 0;
        self.refresh_book_note();
        if !self.book_asks() {
            self.mode = Mode::Normal;
        }
    }

    /// Gives the chosen answer about a play location to the session. It writes only
    /// `orte.toml`, from the file as it is on disk, and is no step of Ctrl+Z.
    fn answer_place(&mut self) {
        let Some(question) = self.place_question() else {
            return;
        };
        let name = question.name.clone();

        if let Err(e) = self.session.answer_place(&name, self.book_sel) {
            self.notice = Some(Notice::SaveFailed(error_text(self.lang, &e)));
            self.mode = Mode::Normal;
            return;
        }

        self.book_sel = 0;
        self.refresh_book_note();
        if !self.book_asks() {
            self.mode = Mode::Normal;
        }
    }

    /// Ctrl+Z: takes back the latest of editor edit and book answer. The answer is the
    /// latest when the editor has no step since it.
    pub(super) fn undo(&mut self) -> Flow {
        // The session's top step is an answer, not another kind of step (audit 16, finding 2).
        let answer = self.session.undo_is_answer();
        if !answer || self.answered.last() != Some(&self.editor.undo_depth()) {
            return self.edit_with(Editor::undo);
        }

        self.answered.pop();
        self.session.undo();
        self.unanswered
            .push((self.editor.undo_depth(), self.editor.edits()));

        self.settle_answer()
    }

    /// Ctrl+Y: does a taken-back answer again when the editor is back at the step it was at
    /// then (its undo depth, not its revision, which every undo and redo raises) and made no
    /// new edit since, which drops the answer from redo like any undone step.
    pub(super) fn redo(&mut self) -> Flow {
        let now = (self.editor.undo_depth(), self.editor.edits());
        if self.unanswered.last().is_some_and(|then| then.1 != now.1) {
            self.unanswered.clear();
        }
        if self.unanswered.last() != Some(&now) {
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
            self.notice = Some(Notice::SaveFailed(error_text(self.lang, &e)));
        }
        self.book_sel = 0;
        self.refresh_book_note();

        Flow::Continue
    }
}
