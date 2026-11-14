//! The camera card: what a body, sensor width, lens or frame typed for the selected camera
//! means as a plan edit, and what the card lists to choose from (LH-32, LH-38). No optics here
//! (agent.md): the session sets the values, Stellprobe works out the angle of view.

use klappe_services::{
    read_number, CameraPresets, Field, PlanEdit, Range, ServiceError, Source, Unit,
};

use crate::tools::Typed;
use serde_json::{json, Value};

/// A field of the camera card, as QML names it to `setCameraField`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraField {
    /// A body of the presets, by id: sets sensor and frame together.
    Body,
    /// Sensor width in millimetres.
    Sensor,
    /// Focal length in millimetres.
    Lens,
    /// Frame, width to height: `2.39` or `16:9`.
    Aspect,
}

impl CameraField {
    pub fn named(name: &str) -> Option<CameraField> {
        match name {
            "body" => Some(CameraField::Body),
            "sensor" => Some(CameraField::Sensor),
            "lens" => Some(CameraField::Lens),
            "aspect" => Some(CameraField::Aspect),
            _ => None,
        }
    }
}

/// A size as typed, more than zero. Absurd sizes are left to the session.
fn positive(field: Field, text: &str, unit: Unit) -> Result<f64, ServiceError> {
    read_number(field, text, unit, Range::Positive)
}

/// A frame as typed: a number (`2,39`) or a ratio (`16:9`).
fn parse_aspect(text: &str) -> Result<f64, ServiceError> {
    let part = |text| positive(Field::Aspect, text, Unit::None);
    let Some((width, height)) = text.split_once(':') else {
        return part(text);
    };
    let aspect = part(width).and_then(|width| Ok(width / part(height)?));

    aspect
        .ok()
        .filter(|aspect| aspect.is_finite())
        .ok_or_else(|| ServiceError::BadValue {
            field: Field::Aspect,
            value: text.trim().to_owned(),
        })
}

/// The value typed or chosen for `field` of the selected camera, as an edit. `None` when
/// something other than a camera is selected or the body is not in the presets.
pub fn camera_edit(
    selected: Option<&Source>,
    presets: &CameraPresets,
    field: CameraField,
    value: &str,
) -> Typed {
    let Some(Source::Camera(id)) = selected else {
        return None;
    };
    let camera = id.clone();

    match field {
        CameraField::Body => {
            let body = presets.bodies.iter().find(|b| b.id == value)?;

            Some(Ok(PlanEdit::SetSensor {
                camera,
                width_mm: body.width_mm,
                aspect: Some(body.aspect),
            }))
        }
        CameraField::Sensor => {
            let width = positive(Field::Sensor, value, Unit::Millimetre);

            Some(width.map(|width_mm| PlanEdit::SetSensor {
                camera,
                width_mm,
                aspect: None,
            }))
        }
        CameraField::Lens => {
            let lens = positive(Field::Lens, value, Unit::Millimetre);

            Some(lens.map(|lens_mm| PlanEdit::SetLens { camera, lens_mm }))
        }
        CameraField::Aspect => {
            Some(parse_aspect(value).map(|aspect| PlanEdit::SetAspect { camera, aspect }))
        }
    }
}

/// The id of the body whose sensor is `width_mm` wide, else empty: a sensor of another width
/// is a custom one.
pub fn body_id(presets: &CameraPresets, width_mm: f64) -> &str {
    let body = presets.bodies.iter().find(|b| b.width_mm == width_mm);

    body.map(|b| b.id.as_str()).unwrap_or_default()
}

/// What the card lists: bodies, and lens sets with the focal lengths each offers.
pub fn presets_json(presets: &CameraPresets) -> Value {
    let sets: Vec<Value> = presets
        .lens_sets
        .iter()
        .map(|set| json!({ "id": set.id, "stops_mm": set.stops_mm(), "zoom": set.zoom_mm.is_some() }))
        .collect();

    json!({ "bodies": presets.bodies, "lens_sets": sets })
}

#[cfg(test)]
mod tests {
    use super::*;
    use klappe_services::Session;

    /// The presets outlive the session that hands them out.
    fn presets() -> &'static CameraPresets {
        let dir =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe");

        Session::open(&dir).unwrap().camera_presets()
    }

    /// The edit a typed field made, if it made one.
    fn edit_of(typed: Typed) -> Option<PlanEdit> {
        typed?.ok()
    }

    fn camera() -> Source {
        Source::Camera("a".into())
    }

    #[test]
    fn a_body_sets_sensor_and_frame() {
        let edit = edit_of(camera_edit(
            Some(&camera()),
            presets(),
            CameraField::Body,
            "full-frame",
        ));

        assert_eq!(
            edit,
            Some(PlanEdit::SetSensor {
                camera: "a".into(),
                width_mm: 36.0,
                aspect: Some(1.78),
            })
        );
        assert_eq!(
            edit_of(camera_edit(
                Some(&camera()),
                presets(),
                CameraField::Body,
                "kodak"
            )),
            None
        );
    }

    #[test]
    fn typed_numbers_take_comma_or_point() {
        let pick = |field, text| edit_of(camera_edit(Some(&camera()), presets(), field, text));

        assert_eq!(
            pick(CameraField::Sensor, " 23,5 "),
            Some(PlanEdit::SetSensor {
                camera: "a".into(),
                width_mm: 23.5,
                aspect: None,
            })
        );
        assert_eq!(
            pick(CameraField::Lens, "35.5"),
            Some(PlanEdit::SetLens {
                camera: "a".into(),
                lens_mm: 35.5,
            })
        );
        assert_eq!(
            pick(CameraField::Aspect, "2,39"),
            Some(PlanEdit::SetAspect {
                camera: "a".into(),
                aspect: 2.39,
            })
        );
    }

    /// Audit, Fund 2: the camera card takes the unit the shot list takes.
    #[test]
    fn typed_units_are_taken_by_the_camera_card() {
        let pick = |field, text| edit_of(camera_edit(Some(&camera()), presets(), field, text));

        assert!(pick(CameraField::Lens, "85 mm").is_some());
        assert!(pick(CameraField::Sensor, "36 mm").is_some());
        assert!(pick(CameraField::Lens, "85 m").is_none());
    }

    #[test]
    fn a_ratio_is_a_frame() {
        let pick = |text| {
            edit_of(camera_edit(
                Some(&camera()),
                presets(),
                CameraField::Aspect,
                text,
            ))
        };

        assert_eq!(
            pick("16:9"),
            Some(PlanEdit::SetAspect {
                camera: "a".into(),
                aspect: 16.0 / 9.0,
            })
        );
        assert_eq!(pick("16:0"), None);
        assert_eq!(pick("16:"), None);
        assert_eq!(pick(":9"), None);
    }

    #[test]
    fn what_is_no_length_is_no_edit() {
        for field in [CameraField::Sensor, CameraField::Lens, CameraField::Aspect] {
            for text in ["", "0", "-5", "breit", "NaN", "inf", "1e999"] {
                let edit = edit_of(camera_edit(Some(&camera()), presets(), field, text));

                assert_eq!(edit, None, "{field:?} {text:?}");
            }
        }
    }

    #[test]
    fn only_a_camera_has_a_card() {
        for selected in [None, Some(Source::Object("o1".into()))] {
            let edit = edit_of(camera_edit(
                selected.as_ref(),
                presets(),
                CameraField::Lens,
                "50",
            ));

            assert_eq!(edit, None);
        }
    }

    #[test]
    fn a_width_finds_its_body() {
        assert_eq!(body_id(presets(), 24.89), "super35");
        assert_eq!(body_id(presets(), 36.0), "full-frame");
        assert_eq!(body_id(presets(), 23.4), "");
    }

    #[test]
    fn the_card_lists_bodies_and_stops() {
        let json = presets_json(presets());
        let primes = json["lens_sets"]
            .as_array()
            .and_then(|sets| sets.iter().find(|s| s["id"] == "primes"));
        let zoom = json["lens_sets"]
            .as_array()
            .and_then(|sets| sets.iter().find(|s| s["id"] == "zoom-standard"));

        assert_eq!(json["bodies"][0]["id"], "super35");
        assert_eq!(json["bodies"][0]["width_mm"], 24.89);
        assert_eq!(primes.map(|s| s["stops_mm"][0].clone()), Some(json!(18.0)));
        assert_eq!(
            zoom.map(|s| s["stops_mm"].clone()),
            Some(json!([24.0, 70.0]))
        );
        assert_eq!(zoom.map(|s| s["zoom"].clone()), Some(json!(true)));
    }
}
