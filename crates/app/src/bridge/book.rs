//! The register against the book: answering an open question about a scene.

use super::qobject;
use crate::manage;
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
        let Some(choice) = manage::pending_choice(s, &id, index) else {
            return false;
        };

        if let Err(e) = s.resolve_scene(&id, choice) {
            self.say(error_text(language, &e));
            return false;
        }
        self.as_mut().changed();
        self.refresh();
        true
    }
}
