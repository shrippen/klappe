//! The camera card: what a body, sensor width, lens or frame typed for the selected camera
//! means as a plan edit, and what the card lists to choose from (LH-32, LH-38). No optics here
//! (agent.md): the session sets the values, Stellprobe works out the angle of view.

use klappe_services::{CameraPresets, PlanEdit, Source};
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

/// A number as typed: a comma or a point as decimal separator, more than zero. Absurd sizes
/// are left to the session.
fn parse_positive(text: &str) -> Option<f64> {
    let number: f64 = text.trim().replace(',', ".").parse().ok()?;

    (number.is_finite() && number > 0.0).then_some(number)
}

/// A frame as typed: a number (`2,39`) or a ratio (`16:9`).
fn parse_aspect(text: &str) -> Option<f64> {
    let Some((width, height)) = text.split_once(':') else {
        return parse_positive(text);
    };
    let aspect = parse_positive(width)? / parse_positive(height)?;

    aspect.is_finite().then_some(aspect)
}

/// The value typed or chosen for `field` of the selected camera, as an edit. `None` when
/// something other than a camera is selected, the body is not in the presets, or the text is
/// no number.
pub fn camera_edit(
    selected: Option<&Source>,
    presets: &CameraPresets,
    field: CameraField,
    value: &str,
) -> Option<PlanEdit> {
    let Some(Source::Camera(id)) = selected else {
        return None;
    };
    let camera = id.clone();

    match field {
        CameraField::Body => {
            let body = presets.bodies.iter().find(|b| b.id == value)?;

            Some(PlanEdit::SetSensor {
                camera,
                width_mm: body.width_mm,
                aspect: Some(body.aspect),
            })
        }
        CameraField::Sensor => Some(PlanEdit::SetSensor {
            camera,
            width_mm: parse_positive(value)?,
            aspect: None,
        }),
        CameraField::Lens => Some(PlanEdit::SetLens {
            camera,
            lens_mm: parse_positive(value)?,
        }),
        CameraField::Aspect => Some(PlanEdit::SetAspect {
            camera,
            aspect: parse_aspect(value)?,
        }),
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

    fn camera() -> Source {
        Source::Camera("a".into())
    }

    #[test]
    fn a_body_sets_sensor_and_frame() {
        let edit = camera_edit(Some(&camera()), presets(), CameraField::Body, "full-frame");

        assert_eq!(
            edit,
            Some(PlanEdit::SetSensor {
                camera: "a".into(),
                width_mm: 36.0,
                aspect: Some(1.78),
            })
        );
        assert_eq!(
            camera_edit(Some(&camera()), presets(), CameraField::Body, "kodak"),
            None
        );
    }

    #[test]
    fn typed_numbers_take_comma_or_point() {
        let pick = |field, text| camera_edit(Some(&camera()), presets(), field, text);

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

    /// Characterisation: the camera card refuses the unit the shot list takes.
    #[test]
    fn typed_units_are_refused_by_the_camera_card() {
        let pick = |field, text| camera_edit(Some(&camera()), presets(), field, text);

        assert_eq!(pick(CameraField::Lens, "85 mm"), None);
        assert_eq!(pick(CameraField::Sensor, "36 mm"), None);
    }

    #[test]
    fn a_ratio_is_a_frame() {
        let pick = |text| camera_edit(Some(&camera()), presets(), CameraField::Aspect, text);

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
                let edit = camera_edit(Some(&camera()), presets(), field, text);

                assert_eq!(edit, None, "{field:?} {text:?}");
            }
        }
    }

    #[test]
    fn only_a_camera_has_a_card() {
        for selected in [None, Some(Source::Object("o1".into()))] {
            let edit = camera_edit(selected.as_ref(), presets(), CameraField::Lens, "50");

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
