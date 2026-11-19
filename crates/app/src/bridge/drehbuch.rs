//! The screenplay in the window: its text and outline for the editor, and the import.

use super::{q, qobject};
use crate::fileurl;
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use klappe_services::{
    error_text, outline, text_kinds, Align, ImportReport, OutlineKind, TextKind, TEXT_COLUMNS,
};
use serde_json::{json, Map, Value};

/// The letter of each line kind for `scriptKinds` and its name in `scriptLooks`.
const KINDS: [(TextKind, char, &str); 14] = [
    (TextKind::Blank, ' ', "blank"),
    (TextKind::Title, 't', "title"),
    (TextKind::Heading, 'h', "heading"),
    (TextKind::Action, 'a', "action"),
    (TextKind::Character, 'c', "character"),
    (TextKind::Parenthetical, 'p', "parenthetical"),
    (TextKind::Dialogue, 'd', "dialogue"),
    (TextKind::Transition, 'r', "transition"),
    (TextKind::Centered, 'm', "centered"),
    (TextKind::Lyrics, 'l', "lyrics"),
    (TextKind::Section, 's', "section"),
    (TextKind::Synopsis, 'y', "synopsis"),
    (TextKind::Note, 'n', "note"),
    (TextKind::PageBreak, 'b', "pageBreak"),
];

fn letter(kind: TextKind) -> char {
    KINDS
        .iter()
        .find(|(k, _, _)| *k == kind)
        .map_or(' ', |(_, letter, _)| *letter)
}

/// One letter per line of `text`, as `scriptLooks` names them.
pub(super) fn kinds_text(text: &str) -> String {
    text_kinds(text).into_iter().map(letter).collect()
}

/// Every letter of `kinds_text` with its kind and the columns the page sets it in.
pub(super) fn looks_json() -> String {
    let looks: Map<String, Value> = KINDS
        .iter()
        .map(|(kind, letter, name)| {
            let mut look = json!({ "kind": name });
            if let Some(margin) = kind.page_kind().map(|k| k.margin()) {
                look["x"] = json!(margin.x);
                look["width"] = json!(margin.width);
                look["align"] = json!(match margin.align {
                    Align::Left => "left",
                    Align::Right => "right",
                    Align::Center => "center",
                });
            }
            (letter.to_string(), look)
        })
        .collect();

    json!({ "columns": TEXT_COLUMNS, "looks": looks }).to_string()
}

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

/// The counts of an import for the report dialog.
fn report_json(report: &ImportReport) -> String {
    json!({
        "scenes": report.scenes,
        "created": report.created,
        "pending": report.pending,
        "notInBook": report.not_in_book,
    })
    .to_string()
}

impl qobject::Controller {
    pub(super) fn script_kinds(&self, text: &QString) -> QString {
        q(&kinds_text(&text.to_string()))
    }

    pub(super) fn script_looks(&self) -> QString {
        q(&looks_json())
    }

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
            Ok(report) => {
                self.as_mut().set_import_report(q(&report_json(&report)));
                language.text("script.imported")
            }
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
    use super::{kinds_text, looks_json, outline_json};

    /// A cue and its line get their own letter, and the looks place the cue as the page does.
    #[test]
    fn a_cue_is_set_in_the_columns_of_the_page() {
        assert_eq!(kinds_text("INT. SAAL\n\nANNA\nHallo.\n\nCUT TO:"), "h cd r");

        let v: serde_json::Value = serde_json::from_str(&looks_json()).unwrap();
        assert_eq!(v["columns"], 60);
        assert_eq!(v["looks"]["c"]["kind"], "character");
        assert_eq!(v["looks"]["c"]["x"], 22);
        assert_eq!(v["looks"]["r"]["align"], "right");
        assert!(v["looks"]["s"].get("x").is_none());
    }

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
