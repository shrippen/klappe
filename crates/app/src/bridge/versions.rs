//! The locked versions of the screenplay in the window (DB-40 to DB-42, DB-44): the list for
//! the menu, locking and deleting, the lines the revision marks, and the comparison of two
//! states. What a version is lives in the session; this only words it for QML.

use super::{q, qobject};
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use klappe_services::{
    color_text, error_text, Comparison, Draft, Edit, Language, OnOutside, Saved, ScenePlace,
    SceneStatus, Version,
};
use serde_json::json;

/// The versions in the order they were locked, and the colour the revision since the last one
/// has (`next`, null before the first version): `{ versions: [{ id, name, date, color,
/// colorName }], next: { color, colorName } | null }`.
pub(super) fn versions_json(versions: &[Version], language: Language) -> String {
    let versions_view: Vec<_> = versions
        .iter()
        .map(|v| {
            json!({
                "id": v.id,
                "name": v.name,
                "date": v.date,
                "color": v.color.name(),
                "colorName": color_text(language, v.color),
            })
        })
        .collect();
    let next = versions.last().map(|v| {
        let color = v.color.next();

        json!({ "color": color.name(), "colorName": color_text(language, color) })
    });

    json!({ "versions": versions_view, "next": next }).to_string()
}

/// The comparison as `[{ number, heading, status, place, lines: [{ change, line, text }] }]`;
/// `status` is `new`, `removed`, `changed` or `same`, `place` `kept` or `moved`, `change`
/// `added` or `removed`.
pub(super) fn compare_json(comparison: &Comparison) -> String {
    let scenes: Vec<_> = comparison
        .scenes
        .iter()
        .map(|scene| {
            let lines: Vec<_> = scene
                .lines
                .iter()
                .map(|l| {
                    let change = match l.change {
                        Edit::Added => "added",
                        Edit::Removed => "removed",
                    };

                    json!({ "change": change, "line": l.line, "text": l.text })
                })
                .collect();
            let status = match scene.status {
                SceneStatus::New => "new",
                SceneStatus::Removed => "removed",
                SceneStatus::Changed => "changed",
                SceneStatus::Same => "same",
            };
            let place = match scene.place {
                ScenePlace::Kept => "kept",
                ScenePlace::Moved => "moved",
            };

            json!({
                "number": scene.number,
                "heading": scene.heading,
                "status": status,
                "place": place,
                "lines": lines,
            })
        })
        .collect();

    json!(scenes).to_string()
}

/// A state of the book as QML names it: a version's id, empty for the book now.
fn draft_of(id: &QString) -> Draft {
    let id = id.to_string();
    if id.is_empty() {
        return Draft::Current;
    }

    Draft::Version(id)
}

impl qobject::Controller {
    pub(super) fn versions_json(mut self: Pin<&mut Self>) -> QString {
        let language = self.rust().language;
        let Some(s) = self.as_mut().session() else {
            return q(&versions_json(&[], language));
        };

        q(&versions_json(s.versions(), language))
    }

    pub(super) fn lock_version(mut self: Pin<&mut Self>, name: &QString) -> QString {
        let language = self.rust().language;
        let name = name.to_string();

        // As a save: the text the lock keeps is the one on disk.
        self.as_mut().check_outside();
        let result = match self.as_mut().session() {
            Some(s) => s
                .lock_version(&name, language, OnOutside::Ask)
                .map_err(|e| error_text(language, &e)),
            None => return QString::default(),
        };
        self.as_mut().show_outside();

        match result {
            Err(text) => return q(&text),
            Ok(Saved::Conflict) => {
                self.as_mut().outside_asked();
                return q(&language.text("cli.lockConflict"));
            }
            Ok(Saved::Done) => {}
        }

        self.as_mut().changed();
        self.as_mut().refresh();
        let text = language
            .text("versions.locked")
            .replace("{name}", name.trim());
        self.say(text);

        QString::default()
    }

    pub(super) fn remove_version(mut self: Pin<&mut Self>, id: &QString) -> bool {
        let language = self.rust().language;
        let result = match self.as_mut().session() {
            Some(s) => s.remove_version(&id.to_string()),
            None => return false,
        };

        let ok = result.is_ok();
        let text = match result {
            Ok(()) => language.text("versions.removed"),
            Err(e) => error_text(language, &e),
        };
        self.say(text);

        ok
    }

    pub(super) fn revision_json(mut self: Pin<&mut Self>) -> QString {
        let Some(s) = self.as_mut().session() else {
            return q("[]");
        };
        let lines = s.revision().map(|c| c.marked_lines()).unwrap_or_default();

        q(&json!(lines).to_string())
    }

    pub(super) fn compare_json(mut self: Pin<&mut Self>, from: &QString, to: &QString) -> QString {
        let (from, to) = (draft_of(from), draft_of(to));
        let Some(s) = self.as_mut().session() else {
            return q("[]");
        };
        let comparison = s.compare_drafts(&from, &to).unwrap_or_default();

        q(&compare_json(&comparison))
    }
}

#[cfg(test)]
mod tests {
    use super::{compare_json, versions_json};
    use klappe_services::{
        Comparison, Edit, Language, LineChange, RevisionColor, SceneDiff, ScenePlace, SceneStatus,
        Version,
    };

    fn version(id: &str, color: RevisionColor) -> Version {
        Version {
            id: id.into(),
            name: format!("Fassung {id}"),
            date: "2026-11-25".into(),
            color,
        }
    }

    /// The list names each colour in the language, and the revision after the last version
    /// has the next colour; before the first version there is none.
    #[test]
    fn the_list_names_the_colours_and_the_next() {
        let list = [
            version("a", RevisionColor::White),
            version("b", RevisionColor::Blue),
        ];

        let v: serde_json::Value =
            serde_json::from_str(&versions_json(&list, Language::German)).unwrap();
        assert_eq!(v["versions"][1]["color"], "blue");
        assert_eq!(v["versions"][1]["colorName"], "Blau");
        assert_eq!(v["next"]["color"], "pink");
        assert_eq!(v["next"]["colorName"], "Rosa");

        let none: serde_json::Value =
            serde_json::from_str(&versions_json(&[], Language::German)).unwrap();
        assert_eq!(none["versions"].as_array().map(Vec::len), Some(0));
        assert!(none["next"].is_null());
    }

    /// A scene's status, its place and its changed lines reach QML by name.
    #[test]
    fn a_scene_tells_status_place_and_lines() {
        let comparison = Comparison {
            scenes: vec![SceneDiff {
                number: Some("2".into()),
                heading: Some("EXT. HOF".into()),
                status: SceneStatus::Changed,
                place: ScenePlace::Moved,
                lines: vec![
                    LineChange {
                        change: Edit::Removed,
                        line: 4,
                        text: "alt".into(),
                    },
                    LineChange {
                        change: Edit::Added,
                        line: 5,
                        text: "neu".into(),
                    },
                ],
            }],
        };

        let v: serde_json::Value = serde_json::from_str(&compare_json(&comparison)).unwrap();
        assert_eq!(v[0]["status"], "changed");
        assert_eq!(v[0]["place"], "moved");
        assert_eq!(v[0]["number"], "2");
        assert_eq!(v[0]["lines"][0]["change"], "removed");
        assert_eq!(v[0]["lines"][1]["text"], "neu");
        assert_eq!(v[0]["lines"][1]["line"], 5);
    }
}
