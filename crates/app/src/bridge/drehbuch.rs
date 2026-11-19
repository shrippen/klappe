//! The screenplay in the window: its text and outline for the editor, and the import.

use super::qobject;
use crate::fileurl;
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use klappe_services::{error_text, outline, OutlineKind};
use serde_json::json;

/// The sections and scenes of `text` for the scene list; synopses are left out.
pub(super) fn outline_json(text: &str) -> String {
    let rows: Vec<_> = outline(text)
        .into_iter()
        .filter_map(|e| {
            let kind = match e.kind {
                OutlineKind::Section(_) => "section",
                OutlineKind::Scene => "scene",
                OutlineKind::Synopsis => return None,
            };

            Some(json!({
                "kind": kind,
                "depth": e.depth,
                "line": e.line,
                "number": e.number.unwrap_or_default(),
                "text": e.text,
            }))
        })
        .collect();

    json!(rows).to_string()
}

impl qobject::Controller {
    pub(super) fn edit_script(mut self: Pin<&mut Self>, text: &QString) {
        let Some(s) = self.as_mut().session() else {
            return;
        };

        s.set_script_text(&text.to_string());
        self.as_mut().changed();
        self.refresh();
    }

    pub(super) fn import_script(mut self: Pin<&mut Self>, path: &QString) -> bool {
        let language = self.rust().language;
        let path = fileurl::to_path(&path.to_string());
        let Some(s) = self.as_mut().session() else {
            return false;
        };

        let result = s.import_script(&path);
        let ok = result.is_ok();
        let message = match result {
            Ok(()) => language.text("script.imported"),
            Err(e) => error_text(language, &e),
        };
        self.as_mut().changed();
        self.as_mut().refresh();
        self.as_mut().say(message);
        ok
    }
}

#[cfg(test)]
mod tests {
    use super::outline_json;

    /// Sections and scenes are listed with their line; a synopsis is not.
    #[test]
    fn the_outline_lists_sections_and_scenes() {
        let text = "# Akt I\n\nINT. SAAL - TAG #1#\n\n= Sie kommt an.\n\nEXT. HOF - NACHT\n";

        let v: serde_json::Value = serde_json::from_str(&outline_json(text)).unwrap();
        let rows = v.as_array().unwrap();

        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0]["kind"], "section");
        assert_eq!(rows[0]["text"], "Akt I");
        assert_eq!(rows[1]["kind"], "scene");
        assert_eq!(rows[1]["number"], "1");
        assert_eq!(rows[1]["line"], 2);
        assert_eq!(rows[2]["number"], "");
        assert_eq!(rows[2]["line"], 6);
    }

    #[test]
    fn no_text_is_an_empty_list() {
        assert_eq!(outline_json(""), "[]");
    }
}
