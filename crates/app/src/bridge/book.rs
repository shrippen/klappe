//! The register against the book: answering an open question about a scene, a name of a cue
//! (a figure) or a place name of a heading. An answer is an undo step, unless the question is
//! gone (another program answered it): that one is none. A figure answer also goes to
//! `figuren.toml` at once, a place answer to `orte.toml`.

use super::qobject;
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use klappe_services::error_text;

use crate::manage::question_key;

impl qobject::Controller {
    pub(super) fn answer_question(
        mut self: Pin<&mut Self>,
        kind: &QString,
        key: &QString,
        index: i32,
    ) -> bool {
        let language = self.rust().language;
        let Some(key) = question_key(&kind.to_string(), key.to_string()) else {
            return false;
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

        self.as_mut().changed();
        self.refresh();
        true
    }
}
