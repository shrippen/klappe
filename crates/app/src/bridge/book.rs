//! The register against the book: answering an open question about a scene or a place
//! name of a heading. A place answer is no undo step: it goes to `orte.toml` and not to the project.

use super::qobject;
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use klappe_services::{error_text, QuestionKey};

/// The question kinds of `projectJson.questions` (`manage::project_json`).
const PLACE_KIND: &str = "place";

impl qobject::Controller {
    pub(super) fn answer_question(
        mut self: Pin<&mut Self>,
        kind: &QString,
        key: &QString,
        index: i32,
    ) -> bool {
        let language = self.rust().language;
        let key = match kind.to_string().as_str() {
            PLACE_KIND => QuestionKey::Place(key.to_string()),
            _ => QuestionKey::Scene(key.to_string()),
        };
        let Some(s) = self.as_mut().session() else {
            return false;
        };
        let Ok(at) = usize::try_from(index) else {
            return false;
        };

        if let Err(e) = s.answer_question(&key, at) {
            self.say(error_text(language, &e));
            return false;
        }

        // A place answer is no project change for "unsaved", but clears the status line's count.
        self.as_mut().changed();
        self.refresh();
        true
    }
}
