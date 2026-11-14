//! The object library: save to, import and export.

use super::qobject;
use crate::fileurl;
use crate::notice::error_text;
use crate::tools;
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;

impl qobject::Controller {
    pub(super) fn save_to_library(mut self: Pin<&mut Self>, name: &QString) -> bool {
        let language = self.rust().language;
        let name = name.to_string();
        let location = {
            let this = self.as_ref();
            let scene = this.rust().session.as_ref().and_then(|s| s.scene());
            scene.map(|scene| scene.location.clone())
        };
        let selected = self.rust().selected.only();
        let Some(edit) = tools::library_edit(selected, location.as_deref(), &name) else {
            return false;
        };

        if !self.as_mut().commit_project(Ok(edit)) {
            return false;
        }

        // The edit cleared the status line; the result goes there.
        let saved = language
            .text("library.saved")
            .replace("{name}", name.trim());
        self.as_mut().say(saved);
        true
    }

    pub(super) fn import_library(mut self: Pin<&mut Self>, path: &QString) -> bool {
        let language = self.rust().language;
        let path = fileurl::to_path(&path.to_string());
        let Some(s) = self.as_mut().session() else {
            return false;
        };

        let (message, ok) = match s.import_library(&path) {
            Ok(0) => (language.text("library.importedNone"), true),
            Ok(count) => {
                let text = language.text("library.imported");
                (text.replace("{count}", &count.to_string()), true)
            }
            Err(e) => (error_text(language, &e), false),
        };
        self.as_mut().changed();
        self.as_mut().refresh();
        self.as_mut().say(message);
        ok
    }

    pub(super) fn export_library(mut self: Pin<&mut Self>, path: &QString) -> bool {
        let language = self.rust().language;
        let path = fileurl::to_path(&path.to_string());
        let Some(s) = self.as_mut().session() else {
            return false;
        };

        let (message, ok) = match s.export_library(&path) {
            Ok(count) => {
                let text = language.text("library.exported");
                let text = text.replace("{count}", &count.to_string());
                (text.replace("{path}", &path.to_string_lossy()), true)
            }
            Err(e) => (error_text(language, &e), false),
        };
        self.as_mut().say(message);
        ok
    }
}
