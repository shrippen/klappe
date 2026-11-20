//! The questions the register leaves open about the book (`Ctrl+K`): which scene of the
//! book an entry is, then which scripted location a place name of a heading is (DB-51), one
//! question at a time; the session words them.

use klappe_services::{error_text, BookQuestion, QuestionKey};
use ratatui::crossterm::event::{KeyCode, KeyEvent};

use super::{App, Editor, Flow, Mode, Notice};

impl App {
    /// The question being asked, worded by the session: a scene first, then a place name of
    /// the headings.
    pub(crate) fn question(&self) -> Option<BookQuestion> {
        self.session.book_questions(self.lang).into_iter().next()
    }

    /// Whether any question about the book is open.
    fn book_asks(&self) -> bool {
        self.question().is_some()
    }

    /// The number of the entry asked about, if the question is about a scene (for the tests).
    #[cfg(test)]
    pub(crate) fn asked_entry(&self) -> Option<String> {
        let QuestionKey::Scene(id) = self.question()?.key else {
            return None;
        };

        self.session
            .pending_scenes()
            .iter()
            .find(|p| p.id == id)
            .map(|p| p.number.clone())
    }

    /// Whether the question is about a place name of the headings (for the tests).
    #[cfg(test)]
    pub(crate) fn asks_place(&self) -> bool {
        self.question()
            .is_some_and(|q| matches!(q.key, QuestionKey::Place(_)))
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
        let options = self.question().map_or(0, |q| q.answers.len()).max(1);

        match key.code {
            KeyCode::Esc => self.mode = Mode::Normal,
            KeyCode::Down => self.book_sel = (self.book_sel + 1) % options,
            KeyCode::Up => self.book_sel = (self.book_sel + options - 1) % options,
            KeyCode::Enter => self.answer_book(),
            _ => {}
        }

        Flow::Continue
    }

    /// Gives the chosen answer to the session and goes on with the next question. An answer
    /// about a scene is written to the register at once and is a step of Ctrl+Z; one about a
    /// place name writes only `orte.toml`, from the file as it is on disk, and is none.
    fn answer_book(&mut self) {
        let Some(question) = self.question() else {
            return;
        };
        let scene = matches!(question.key, QuestionKey::Scene(_));

        // The answer is written at once: the TUI saves no plans, so quitting asks nothing.
        let mut answered = self.session.answer_question(&question.key, self.book_sel);
        if scene {
            answered = answered.and_then(|()| self.session.save_register());
        }
        if let Err(e) = answered {
            self.notice = Some(Notice::SaveFailed(error_text(self.lang, &e)));
            self.mode = Mode::Normal;
            return;
        }

        if scene {
            self.answered.push(self.editor.mark_step());
            self.unanswered.clear();
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
        // A new edit, or another step of the session, drops the answers from redo.
        let stale = self.unanswered.last().is_some_and(|then| then.1 != now.1);
        if stale || !self.session.redo_is_answer() {
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
