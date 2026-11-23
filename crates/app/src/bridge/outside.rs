//! A change to the project folder from outside (the terminal beside the window, a Git pull):
//! what the window takes over, and the files it has to ask about.

use super::q;
use super::qobject;
use crate::notice::outside_taken_text;
use core::pin::Pin;
use cxx_qt::CxxQtType;
use klappe_services::{error_text, outside_files_text};

impl qobject::Controller {
    /// What the project folder changed outside: the window takes over files it did not change
    /// itself and tells which (`outsideTaken`); a file both changed stays and is shown
    /// (`outsideFiles`) for the user to settle.
    pub(super) fn check_outside(mut self: Pin<&mut Self>) {
        let language = self.rust().language;
        let found = match self.as_mut().session() {
            Some(s) => s.check().map(|outside| (outside, s.is_dirty())),
            None => return,
        };
        let (outside, unsaved) = match found {
            Ok(found) => found,
            Err(e) => {
                self.say(error_text(language, &e));
                return;
            }
        };

        self.as_mut().show_outside();
        if outside.taken.is_empty() {
            return;
        }

        self.as_mut().changed();
        self.as_mut().refresh();
        let text = outside_taken_text(language, &outside.taken, unsaved);
        self.outside_taken(&q(&text));
    }

    /// Drops the window's changes and reads the folder again.
    pub(super) fn reload_project(mut self: Pin<&mut Self>) {
        let language = self.rust().language;
        let result = match self.as_mut().session() {
            Some(s) => {
                let files = s.outside().to_vec();
                s.reload().map(|()| files)
            }
            None => return,
        };
        let files = match result {
            Ok(files) => files,
            Err(e) => {
                self.say(error_text(language, &e));
                return;
            }
        };

        self.as_mut().changed();
        self.as_mut().refresh();
        self.as_mut().show_backup();
        self.as_mut().show_outside();

        // Nothing was in conflict any more (settled since the dialog opened): no file to name.
        if files.is_empty() {
            return;
        }

        let text = outside_taken_text(language, &files, false);
        self.outside_taken(&q(&text));
    }

    /// Pushes the files in conflict, worded, to QML.
    pub(super) fn show_outside(mut self: Pin<&mut Self>) {
        let language = self.rust().language;
        let words = match self.as_mut().session() {
            Some(s) => outside_files_text(language, s.outside()),
            None => String::new(),
        };

        self.set_outside_files(q(&words));
    }
}
