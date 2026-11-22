//! A change to the project folder from outside (the window beside the terminal, a Git pull):
//! the editor follows it while nobody typed, and a save asks when the screenplay changed both
//! here and outside. The files are the session's; this only calls it.

use klappe_services::{error_text, outside_files_text, FileKind, OnOutside, OutsideFile};
use ratatui::crossterm::event::{KeyCode, KeyEvent};

use super::{App, Flow, Mode, Notice};
use crate::editor::Editor;
use crate::links::Links;

impl App {
    /// What the question says: the files changed outside and here, as words.
    pub(crate) fn outside_text(&self) -> String {
        let files = self.script_files();

        self.lang
            .text("outside.both")
            .replace("{files}", &outside_files_text(self.lang, &files))
    }

    /// The open conflicts a screenplay save is asked about.
    fn script_files(&self) -> Vec<OutsideFile> {
        let asks = |f: &&OutsideFile| matches!(f.kind, FileKind::Script | FileKind::Register);

        self.session
            .outside()
            .iter()
            .filter(asks)
            .cloned()
            .collect()
    }

    /// Takes over a screenplay changed outside while the editor is untouched: the editor
    /// shows it, its history starts over, the status line says so. Whether it did.
    pub(super) fn follow_outside(&mut self) -> bool {
        let Ok(found) = self.session.check() else {
            return false;
        };
        if !found.taken.iter().any(|f| f.kind == FileKind::Script) {
            return false;
        }

        self.reset_editor();
        self.notice = Some(Notice::Outside(self.taken_text(&found.taken)));

        true
    }

    /// Before a writer of one file of the project: the session must not take over a screenplay
    /// behind the editor's back. An untouched editor follows a change outside now; typed text
    /// goes to the session first, so a change outside is a conflict the next save asks about.
    pub(super) fn hand_over(&mut self) {
        if self.is_modified() {
            self.session.set_script_text(&self.editor.text());
            return;
        }

        self.follow_outside();
    }

    fn taken_text(&self, files: &[OutsideFile]) -> String {
        self.lang
            .text("outside.taken")
            .replace("{files}", &outside_files_text(self.lang, files))
    }

    /// The editor holds the session's text with a history of its own again.
    fn reset_editor(&mut self) {
        let line = self.editor.cursor().line;
        self.editor = Editor::new(self.session.script_text());
        self.editor.go_to_line(line);
        self.saved_revision = self.editor.revision();
        self.autosaved = self.editor.revision();
        self.forget_answers();
        self.built_for = None;
        self.links = Links::read(&mut self.session);
        self.refresh_book_note();
    }

    pub(super) fn outside_key(&mut self, key: KeyEvent) -> Flow {
        match key.code {
            KeyCode::Char('r' | 'R') => self.reload_outside(),
            KeyCode::Char('o' | 'O') => self.overwrite_outside(),
            KeyCode::Esc => self.mode = Mode::Normal,
            _ => {}
        }

        Flow::Continue
    }

    /// Drops what was typed and takes the folder as it is.
    fn reload_outside(&mut self) {
        let files = self.script_files();
        if let Err(e) = self.session.reload() {
            self.notice = Some(Notice::SaveFailed(error_text(self.lang, &e)));
            self.mode = Mode::Normal;
            return;
        }

        self.reset_editor();
        self.notice = Some(Notice::Outside(self.taken_text(&files)));
        self.mode = Mode::Normal;
    }

    /// Writes the editor's text over the file.
    fn overwrite_outside(&mut self) {
        self.mode = Mode::Normal;
        match self.session.save_script(OnOutside::Overwrite) {
            Ok(_) => self.saved(),
            Err(e) => self.notice = Some(Notice::SaveFailed(error_text(self.lang, &e))),
        }
    }
}
