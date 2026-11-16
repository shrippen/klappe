//! The backup of unsaved work (LH-05, DN-06): a timer writes it, a start asks about it.
//! The files are the session's; this only calls it.

use klappe_services::BackupState;
use ratatui::crossterm::event::{KeyCode, KeyEvent};

use super::{App, Flow, Mode, Notice};
use crate::links::Links;

/// What waits for an answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Backup {
    /// Unsaved work of an earlier run: restore or discard.
    Found,
    /// A backup that cannot be read: discard it.
    Unreadable,
}

impl App {
    /// Asks about a backup the session found at open.
    pub(super) fn check_backup(&mut self) {
        self.backup = match self.session.backup() {
            BackupState::None => None,
            BackupState::Found => Some(Backup::Found),
            BackupState::Unreadable(_) => Some(Backup::Unreadable),
        };
        if self.backup.is_some() {
            self.mode = Mode::Backup;
        }
    }

    pub(super) fn backup_key(&mut self, key: KeyEvent) -> Flow {
        let Some(open) = self.backup else {
            self.mode = Mode::Normal;
            return Flow::Continue;
        };

        match key.code {
            KeyCode::Char('r' | 'R') if open == Backup::Found => self.restore(),
            KeyCode::Char('d' | 'D') => self.discard(),
            // Later: the backup stays, and nothing is written over it meanwhile.
            KeyCode::Esc => self.mode = Mode::Normal,
            _ => {}
        }

        Flow::Continue
    }

    fn restore(&mut self) {
        match self.session.restore_backup() {
            Ok(()) => {
                let text = self.session.script_text().to_owned();
                self.editor.set_text(&text);
                self.links = Links::read(&mut self.session);
                self.backup = None;
            }
            Err(e) => self.notice = Some(Notice::SaveFailed(e.to_string())),
        }
        self.mode = Mode::Normal;
    }

    fn discard(&mut self) {
        match self.session.discard_backup() {
            Ok(()) => self.backup = None,
            Err(e) => self.notice = Some(Notice::SaveFailed(e.to_string())),
        }
        self.mode = Mode::Normal;
    }

    /// Writes the unsaved text into the backup; called on a timer and when the program ends.
    /// Does nothing when the text is as the last backup or save left it.
    pub fn autosave(&mut self) {
        let revision = self.editor.revision();
        if !self.is_modified() || revision == self.autosaved {
            return;
        }

        self.session.set_script_text(&self.editor.text());
        match self.session.autosave() {
            Ok(_) => self.autosaved = revision,
            Err(e) => self.notice = Some(Notice::SaveFailed(e.to_string())),
        }
    }
}
