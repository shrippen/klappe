//! Undo and redo, saving, and the backup of unsaved changes.

use super::q;
use super::qobject;
use crate::notice::backup_view;
use crate::notice::error_text;
use core::pin::Pin;
use cxx_qt::CxxQtType;
use klappe_services::Autosave;

impl qobject::Controller {
    pub(super) fn undo(mut self: Pin<&mut Self>) {
        if let Some(s) = self.as_mut().session() {
            s.undo();
        }
        self.as_mut().changed();
        self.refresh();
    }

    pub(super) fn redo(mut self: Pin<&mut Self>) {
        if let Some(s) = self.as_mut().session() {
            s.redo();
        }
        self.as_mut().changed();
        self.refresh();
    }

    pub(super) fn save(mut self: Pin<&mut Self>) -> bool {
        let language = self.rust().language;
        let result = match self.as_mut().session() {
            Some(s) => s.save().map_err(|e| error_text(language, &e)),
            None => return false,
        };

        let message = match &result {
            Ok(()) => language.text("menu.saved"),
            Err(e) => e.clone(),
        };
        self.as_mut().say(message);
        self.refresh();
        result.is_ok()
    }

    pub(super) fn autosave(mut self: Pin<&mut Self>) {
        let language = self.rust().language;
        let Some(s) = self.as_mut().session() else {
            return;
        };

        // Written: nothing to show. Clean and Held: nothing to write.
        match s.autosave() {
            Ok(Autosave::Written | Autosave::Clean | Autosave::Held) => {}
            Err(e) => self.say(error_text(language, &e)),
        }
    }

    pub(super) fn restore_backup(mut self: Pin<&mut Self>) {
        let language = self.rust().language;
        let Some(s) = self.as_mut().session() else {
            return;
        };

        if let Err(e) = s.restore_backup() {
            self.say(error_text(language, &e));
            return;
        }
        self.as_mut().changed();
        self.as_mut().refresh();
        self.show_backup();
    }

    pub(super) fn discard_backup(mut self: Pin<&mut Self>) {
        let language = self.rust().language;
        let Some(s) = self.as_mut().session() else {
            return;
        };

        if let Err(e) = s.discard_backup() {
            self.say(error_text(language, &e));
            return;
        }
        self.show_backup();
    }

    /// Pushes the session's backup state to QML.
    pub(super) fn show_backup(mut self: Pin<&mut Self>) {
        let view = match self.as_mut().session() {
            Some(s) => backup_view(s.backup()),
            None => return,
        };
        self.as_mut().set_backup_state(q(view.0));
        self.set_backup_message(q(&view.1));
    }
}
