//! The caption of a set object's height on the plan (LH-93).

use klappe_services::{look, Item, Language, Shape};
use serde_json::{json, Value};

use super::{flat, hue_name, SOLID};
use crate::tools::length_text;

/// The caption colour of an object without a height.
const WARNING: &str = "warning";
const CM_PER_M: f64 = 100.0;

/// The words of a height caption, in the language on screen.
#[derive(Debug, Default)]
pub struct HeightTexts {
    decimal: String,
    unit: String,
    missing: String,
}

impl HeightTexts {
    pub fn of(language: Language) -> Self {
        Self {
            decimal: language.text("unit.decimal"),
            unit: language.text("unit.m"),
            missing: language.text("height.missing"),
        }
    }
}

/// The caption of a height item: the number in muted ink, or the missing text as a warning.
/// A circle without radius carries it, so the canvas draws the text and no shape.
pub(super) fn height_shape(item: &Item, cm: Option<u32>, texts: &HeightTexts) -> Option<Value> {
    let Shape::Circle { center, .. } = &item.shape else {
        return None;
    };

    let (label, text) = match cm {
        Some(cm) => {
            let metres = f64::from(cm) / CM_PER_M;
            (
                hue_name(look(item.role).hue),
                length_text(metres, &texts.decimal, &texts.unit),
            )
        }
        None => (WARNING, texts.missing.clone()),
    };

    Some(json!({
        "kind": "circle",
        "points": flat(&[*center]),
        "radius": 0.0,
        "width": 0.0,
        "label": label,
        "stroke": SOLID,
        "fill": "none",
        "text": text,
        "key": "",
        "selected": false,
    }))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use klappe_services::{DisplayList, Point, Role};

    use super::super::shapes_json;
    use super::*;

    fn caption(cm: Option<u32>) -> Item {
        Item {
            role: Role::Height { cm },
            shape: Shape::Circle {
                center: Point::new(2.0, 3.0),
                radius_m: 0.0,
            },
            label: None,
            source: None,
        }
    }

    /// A height is a muted caption in the language's number format; a missing one the
    /// missing text as a warning. Neither is a shape of its own: no radius, no key.
    #[test]
    fn a_height_is_a_caption_and_a_missing_one_a_warning() {
        let list = DisplayList {
            lamps: Vec::new(),
            items: vec![caption(Some(125)), caption(None)],
            width_m: 20.0,
            depth_m: 13.0,
        };
        let texts = HeightTexts {
            decimal: ",".into(),
            unit: "m".into(),
            missing: "Höhe ?".into(),
        };

        let json = shapes_json(&list, Path::new("/p"), &|_| false, &[], None, None, &texts);
        let v: Value = serde_json::from_str(&json).unwrap_or_default();

        assert_eq!(v["shapes"][0]["text"], "1,25 m");
        assert_eq!(v["shapes"][0]["label"], "muted");
        assert_eq!(v["shapes"][0]["points"], json!([2.0, 3.0]));
        assert_eq!(v["shapes"][1]["text"], "Höhe ?");
        assert_eq!(v["shapes"][1]["label"], "warning");
        assert_eq!(v["shapes"][1]["key"], "");
        assert_eq!(v["shapes"][1]["radius"], 0.0);
    }
}
