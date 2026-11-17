//! The backup of unsaved work (LH-05, DN-06): a timer writes it, a start asks about it.
//! The question blocks editing until it is answered (restore or discard), as the window's
//! does; Ctrl+Q leaves with the backup kept.
//! The files are the session's; this only calls it.

use klappe_services::BackupState;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

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
    /// Whether the session still holds a backup nobody has answered: it writes no other one
    /// meanwhile, so the question must stay open then.
    pub(crate) fn backup_held(&self) -> bool {
        *self.session.backup() != BackupState::None
    }

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
            // Leaving keeps the backup: no editing happens while it waits, so nothing is lost.
            KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                return Flow::Quit;
            }
            _ => {}
        }

        Flow::Continue
    }

    /// Restores the backup; on an error the question stays open with the message.
    fn restore(&mut self) {
        if let Err(e) = self.session.restore_backup() {
            self.notice = Some(Notice::SaveFailed(e.to_string()));
            return;
        }

        let text = self.session.script_text().to_owned();
        self.editor.set_text(&text);
        self.links = Links::read(&mut self.session);
        self.answered();
    }

    fn discard(&mut self) {
        if let Err(e) = self.session.discard_backup() {
            self.notice = Some(Notice::SaveFailed(e.to_string()));
            return;
        }

        self.answered();
    }

    fn answered(&mut self) {
        self.backup = None;
        self.notice = None;
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
