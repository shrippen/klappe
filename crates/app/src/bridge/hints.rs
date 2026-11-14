//! Data for the tooltips on the plan.

use super::q;
use super::qobject;
use crate::plan;
use crate::plan::ElementKind;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use klappe_services::{Hint, Session};
use serde_json::json;

impl qobject::Controller {
    pub(super) fn describe(&self, key: &QString) -> QString {
        let Some(s) = self.rust().session.as_ref() else {
            return QString::default();
        };
        q(&describe(s, &key.to_string()).to_string())
    }
}

/// Data for a tooltip; QML puts it into words with the catalogue.
fn describe(s: &Session, key: &str) -> serde_json::Value {
    let Some(plan::ParsedKey { kind, id, index }) = plan::parse_key(key) else {
        return json!({});
    };
    let hint = match kind {
        ElementKind::Camera => s.camera_hint(id),
        ElementKind::Figure => Some(s.figure_hint(id, index)),
        ElementKind::Mark => Some(s.mark_hint(id, index)),
        ElementKind::FigureVia | ElementKind::CameraVia => Some(Hint::Bend {
            index: index.map(|i| i + 1),
        }),
        ElementKind::Light => s.light_hint(id),
        ElementKind::Object => s.object_hint(id),
        ElementKind::Note => Some(s.note_hint(id)),
        ElementKind::Wall | ElementKind::Drawing => None,
    };

    // Without a plan nothing is described, a note included.
    match s.plan().and(hint) {
        Some(hint) => serde_json::to_value(hint).unwrap_or_default(),
        None => json!({}),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge::tests::example;
    use crate::bridge::view::choices_json;
    use klappe_services::{FigureColor, ProjectEdit, Source};

    /// A group's tooltip, and the camera that follows it, name it with its head count.
    #[test]
    fn describe_names_a_group_with_its_count() {
        let mut s = example();
        s.edit_project(ProjectEdit::EditCharacter {
            id: "burgvogt".into(),
            name: "Burgvogt".into(),
            short: "B".into(),
            color: FigureColor::Blue,
            group: Some(12),
        })
        .unwrap();

        let figure = describe(&s, "figure:burgvogt");
        let camera = describe(&s, "camera:b");

        assert_eq!(figure["name"], "Burgvogt (12)");
        assert_eq!(camera["follows"], "Burgvogt (12)");
    }

    /// A tooltip reads back the key the plan wrote: the kind, the id and the index.
    #[test]
    fn describe_reads_the_keys_the_plan_writes() {
        let s = example();
        let described = |source: Source| describe(&s, &plan::key(&source));
        let mark = |index| Source::Mark {
            character: "elisabeth".into(),
            index,
        };

        assert_eq!(described(Source::Camera("a".into()))["kind"], "camera");
        assert_eq!(
            described(Source::Figure("elisabeth".into()))["kind"],
            "figure"
        );
        assert_eq!(described(mark(0))["kind"], "mark");
        assert_eq!(described(mark(0))["index"], 1);
        assert_eq!(described(mark(2))["index"], 3);

        let via = described(Source::FigureVia {
            character: "elisabeth".into(),
            index: 1,
        });
        assert_eq!(via, json!({ "kind": "bend", "index": 2 }));
        let via = described(Source::CameraVia {
            camera: "a".into(),
            index: 0,
        });
        assert_eq!(via, json!({ "kind": "bend", "index": 1 }));

        assert_eq!(described(Source::Wall(0)), json!({}));
        assert_eq!(described(Source::Drawing("d".into())), json!({}));
        assert_eq!(describe(&s, "nonsense:x"), json!({}));
    }

    /// The size fields show what the selected object measures now.
    #[test]
    fn describe_gives_the_size_of_an_object() {
        let s = example();

        let table = describe(&s, "object:tisch");

        assert_eq!(table["kind"], "object");
        assert_eq!(table["library"], "table");
        assert_eq!(
            (table["width"].as_f64(), table["depth"].as_f64()),
            (Some(6.0), Some(1.2))
        );
        // The example's table has no height: `null`, the field stays empty.
        assert_eq!(table["height"], serde_json::Value::Null);
        assert_eq!(describe(&s, "object:o99"), json!({}));
    }

    /// The camera card starts from what the selected camera has now, and offers the presets.
    #[test]
    fn describe_gives_the_cameras_body_and_frame() {
        let s = example();

        let a = describe(&s, "camera:a");

        assert_eq!(a["body"], "super35");
        assert_eq!(a["sensor"], 24.89);
        assert_eq!(a["aspect"], 2.39);
        assert_eq!(a["ownLens"], 35.0);

        let choices: serde_json::Value = serde_json::from_str(&choices_json(&s)).unwrap();
        assert_eq!(choices["camera"]["bodies"][0]["id"], "super35");
        assert!(!choices["camera"]["lens_sets"]
            .as_array()
            .unwrap()
            .is_empty());
    }

    /// Every kind of element a tooltip describes, as QML reads it.
    #[test]
    fn describe_gives_every_kind_its_fields() {
        let s = example();

        assert_eq!(
            describe(&s, "camera:b"),
            json!({
                "kind": "camera", "label": "B", "lens": 50.0, "ownLens": 50.0, "sensor": 24.89,
                "aspect": 2.39, "body": "super35", "fov": 28.0, "height": 1.6,
                "shots": "12.3, 12.4", "follows": "Burgvogt",
            })
        );
        assert_eq!(describe(&s, "camera:a")["shots"], "12.1, 12.2, 12.5");
        assert_eq!(describe(&s, "camera:x"), json!({}));
        assert_eq!(
            describe(&s, "light:led-1"),
            json!({ "kind": "light", "label": "LED-Panel", "temp": 5600.0, "power": 200.0 })
        );
        assert_eq!(describe(&s, "light:x"), json!({}));
        assert_eq!(
            describe(&s, "note:n1"),
            json!({ "kind": "note", "text": "Vorhang schließen" })
        );
        assert_eq!(
            describe(&s, "note:x"),
            json!({ "kind": "note", "text": "" })
        );
        assert_eq!(
            describe(&s, "figure:elisabeth"),
            json!({ "kind": "figure", "name": "Elisabeth", "index": null, "time": null })
        );
        assert_eq!(
            describe(&s, "mark:elisabeth:1"),
            json!({ "kind": "mark", "name": "Elisabeth", "index": 2, "time": 6.0 })
        );
        assert_eq!(
            describe(&s, "object:tisch"),
            json!({
                "kind": "object", "label": "Tafel", "library": "table", "width": 6.0,
                "depth": 1.2, "height": null,
            })
        );
    }

    /// The scene's set decides the background and the notes the tooltips read.
    #[test]
    fn the_set_of_the_scene_gives_background_and_notes() {
        let s = example();

        assert!(!s.has_background());
        assert_eq!(s.note_text("n1").as_deref(), Some("Vorhang schließen"));
        assert_eq!(s.note_text("x"), None);
    }
}
