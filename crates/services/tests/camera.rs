//! Camera bodies, lenses and frames (LH-32, LH-38), on the example project: camera `a` has
//! Super 35, 35 mm and 2.39, camera `b` the same with its own marks.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_model::{Camera, Sensor};
use klappe_services::{Field, Layer, Lock, PlanEdit, ServiceError, Session};

fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe")
}

fn open() -> Session {
    Session::open(&example()).unwrap()
}

fn camera(s: &Session, id: &str) -> Camera {
    s.plan().unwrap().camera(id).unwrap().clone()
}

fn set_sensor(camera: &str, width_mm: f64, aspect: Option<f64>) -> PlanEdit {
    PlanEdit::SetSensor {
        camera: camera.into(),
        width_mm,
        aspect,
    }
}

fn bad(result: Result<(), ServiceError>, field: Field) -> bool {
    matches!(result, Err(ServiceError::BadValue { field: f, .. }) if f == field)
}

/// The session offers the shipped bodies and lens sets.
#[test]
fn the_session_offers_the_presets() {
    let s = open();

    let presets = s.camera_presets();

    assert!(presets.bodies.iter().any(|b| b.id == "super35"));
    assert!(presets.lens_sets.iter().any(|l| l.id == "primes"));
}

/// A body's known width gives the named sensor, its aspect the frame; undo restores both.
#[test]
fn a_known_width_sets_the_named_sensor_and_the_frame() {
    let mut s = open();

    s.edit_plan(set_sensor("a", 36.0, Some(1.78))).unwrap();

    let a = camera(&s, "a");
    assert_eq!((a.sensor, a.aspect), (Sensor::FullFrame, 1.78));
    assert_eq!(a.lens_mm, 35.0);

    s.undo();

    let a = camera(&s, "a");
    assert_eq!((a.sensor, a.aspect), (Sensor::Super35, 2.39));
}

/// Another width is a custom sensor; without an aspect the frame stays.
#[test]
fn another_width_is_custom_and_keeps_the_frame() {
    let mut s = open();

    s.edit_plan(set_sensor("a", 23.5, None)).unwrap();

    let a = camera(&s, "a");
    assert_eq!(a.sensor, Sensor::Custom { width_mm: 23.5 });
    assert_eq!(a.aspect, 2.39);
}

/// Only the chosen camera changes.
#[test]
fn the_other_camera_stays() {
    let mut s = open();
    let before = camera(&s, "b");

    s.edit_plan(set_sensor("a", 17.3, Some(1.78))).unwrap();
    s.edit_plan(PlanEdit::SetLens {
        camera: "a".into(),
        lens_mm: 50.0,
    })
    .unwrap();

    assert_eq!(camera(&s, "b"), before);
}

#[test]
fn a_lens_is_set_and_undone() {
    let mut s = open();

    s.edit_plan(PlanEdit::SetLens {
        camera: "a".into(),
        lens_mm: 50.0,
    })
    .unwrap();
    assert_eq!(camera(&s, "a").lens_mm, 50.0);

    s.undo();
    assert_eq!(camera(&s, "a").lens_mm, 35.0);
}

#[test]
fn a_frame_is_set_and_undone() {
    let mut s = open();

    s.edit_plan(PlanEdit::SetAspect {
        camera: "a".into(),
        aspect: 1.85,
    })
    .unwrap();
    assert_eq!(camera(&s, "a").aspect, 1.85);

    s.undo();
    assert_eq!(camera(&s, "a").aspect, 2.39);
}

/// Every camera edit is on the camera layer: locked, it is refused and nothing changes.
#[test]
fn a_locked_camera_layer_refuses_all_three() {
    let mut s = open();
    s.set_lock(Layer::Cameras, Lock::Locked);
    let before = camera(&s, "a");

    let edits = [
        set_sensor("a", 36.0, Some(1.78)),
        PlanEdit::SetLens {
            camera: "a".into(),
            lens_mm: 50.0,
        },
        PlanEdit::SetAspect {
            camera: "a".into(),
            aspect: 1.85,
        },
    ];
    for edit in edits {
        let result = s.edit_plan(edit);

        assert!(
            matches!(result, Err(ServiceError::Locked(Layer::Cameras))),
            "{result:?}"
        );
    }
    assert_eq!(camera(&s, "a"), before);
}

/// A lock on another layer does not stop them.
#[test]
fn another_locked_layer_lets_them_through() {
    let mut s = open();
    s.set_lock(Layer::Set, Lock::Locked);

    s.edit_plan(PlanEdit::SetLens {
        camera: "a".into(),
        lens_mm: 50.0,
    })
    .unwrap();

    assert_eq!(camera(&s, "a").lens_mm, 50.0);
}

/// NaN, infinity, zero, negative and absurd values change nothing.
#[test]
fn broken_values_are_refused() {
    let mut s = open();
    let before = camera(&s, "a");

    for value in [f64::NAN, f64::INFINITY, 0.0, -1.0, 1e300] {
        assert!(bad(
            s.edit_plan(set_sensor("a", value, None)),
            Field::Sensor
        ));
        assert!(bad(
            s.edit_plan(set_sensor("a", 36.0, Some(value))),
            Field::Aspect
        ));
        let lens = PlanEdit::SetLens {
            camera: "a".into(),
            lens_mm: value,
        };
        assert!(bad(s.edit_plan(lens), Field::Lens));
        let aspect = PlanEdit::SetAspect {
            camera: "a".into(),
            aspect: value,
        };
        assert!(bad(s.edit_plan(aspect), Field::Aspect));
    }

    assert_eq!(camera(&s, "a"), before);
}

/// A camera that is not in the plan is gone.
#[test]
fn an_unknown_camera_is_gone() {
    let mut s = open();

    let result = s.edit_plan(PlanEdit::SetLens {
        camera: "z".into(),
        lens_mm: 50.0,
    });

    assert!(matches!(result, Err(ServiceError::Gone)), "{result:?}");
}

/// The camera's own lens changes, not a shot's: shot 12.2 of camera `a` keeps its 40 mm.
#[test]
fn a_shot_keeps_its_own_lens() {
    let mut s = open();

    s.edit_plan(PlanEdit::SetLens {
        camera: "a".into(),
        lens_mm: 50.0,
    })
    .unwrap();

    let rows = s.shots();
    let own = rows.iter().find(|r| r.number == "12.2").unwrap();
    let follows = rows.iter().find(|r| r.number == "12.1").unwrap();
    assert_eq!(own.lens_mm, 40.0);
    assert_eq!(follows.lens_mm, 50.0);
}
