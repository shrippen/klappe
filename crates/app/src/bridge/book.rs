//! The register against the book: answering an open question about a scene or a place
//! name of a heading. A place answer is no undo step: it goes to `orte.toml` and not to the project.

use super::qobject;
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use klappe_services::error_text;

impl qobject::Controller {
    pub(super) fn resolve_scene(mut self: Pin<&mut Self>, id: &QString, index: i32) -> bool {
        let language = self.rust().language;
        let id = id.to_string();
        let Some(s) = self.as_mut().session() else {
            return false;
        };
        let Ok(at) = usize::try_from(index) else {
            return false;
        };

        if let Err(e) = s.answer_book(&id, at) {
            self.say(error_text(language, &e));
            return false;
        }
        self.as_mut().changed();
        self.refresh();
        true
    }

    pub(super) fn resolve_place(mut self: Pin<&mut Self>, name: &QString, index: i32) -> bool {
        let language = self.rust().language;
        let name = name.to_string();
        let Some(s) = self.as_mut().session() else {
            return false;
        };
        let Ok(at) = usize::try_from(index) else {
            return false;
        };

        if let Err(e) = s.answer_place(&name, at) {
            self.say(error_text(language, &e));
            return false;
        }

        // Clears the status line's count; the answer is no project change for "unsaved".
        self.as_mut().changed();
        self.refresh();
        true
    }
}
