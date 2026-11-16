//! The grips on a lamp's beam (LH-50), on the example project's lamp `led-1`: at (1, 10),
//! facing 330°, 60° beam, drawn 4 m long. So the direction grip is at (4.4641, 8) and the
//! beam grip, at 360° and halfway out the beam, at (3, 10).
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_model::{is_beam_deg, Light, MIN_BEAM_DEG};
use klappe_services::{
    Layer, LightGrip, Lock, PlanEdit, Point, Role, ServiceError, Session, Shape, Snapping, Source,
};

const REACH_M: f64 = 4.0;
const EPS: f64 = 1e-9;

fn open() -> Session {
    let dir: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe");
    let mut s = Session::open(&dir).unwrap();
    s.set_snapping(Snapping::Off);

    s
}

fn lamp() -> Source {
    Source::Light("led-1".into())
}

fn grip(grip: LightGrip) -> Source {
    Source::LightGrip {
        light: "led-1".into(),
        grip,
    }
}

fn light(s: &Session) -> Light {
    s.plan()
        .unwrap()
        .lights
        .iter()
        .find(|l| l.id == "led-1")
        .unwrap()
        .clone()
}

/// Where the grip is shown while the lamp is selected.
fn grip_at(s: &Session, which: LightGrip) -> Option<Point> {
    let list = s.display_list_for(Some(&lamp()));

    list.items
        .iter()
        .filter(|i| i.role == Role::LightHandle)
        .find_map(|i| match (&i.source, &i.shape) {
            (Some(source), Shape::Circle { center, .. }) if *source == grip(which) => Some(*center),
            _ => None,
        })
}

/// A point `REACH_M` from the lamp at `deg`.
fn pointer(s: &Session, deg: f64) -> Point {
    let at = light(s).at;
    let (sin, cos) = deg.to_radians().sin_cos();

    Point::new(at.x_m + REACH_M * cos, at.y_m + REACH_M * sin)
}

/// Drags `which` from where it sits to `to`.
fn drag(s: &Session, which: LightGrip, to: Point) -> PlanEdit {
    let from = grip_at(s, which).unwrap();

    PlanEdit::Shift {
        source: grip(which),
        dx_m: to.x_m - from.x_m,
        dy_m: to.y_m - from.y_m,
    }
}

#[test]
fn a_selected_lamp_has_a_grip_on_its_facing_and_one_on_its_beam_edge() {
    let s = open();
    let direction = grip_at(&s, LightGrip::Direction).unwrap();
    let beam = grip_at(&s, LightGrip::Beam).unwrap();

    assert!(direction.distance_m(Point::new(1.0 + 4.0 * 0.866_025_403_784_438_6, 8.0)) < 1e-6);
    // Halfway out on the beam's edge at 0°.
    assert!(beam.distance_m(Point::new(3.0, 10.0)) < 1e-6);
    assert!(s
        .display_list()
        .items
        .iter()
        .all(|i| i.role != Role::LightHandle));
}

#[test]
fn the_grips_are_picked_only_while_the_lamp_is_selected() {
    let s = open();
    let beam = grip_at(&s, LightGrip::Beam).unwrap();

    assert_eq!(s.pick_for(beam, Some(&lamp())), Some(grip(LightGrip::Beam)));
    assert_ne!(
        s.pick_for(beam, None),
        Some(grip(LightGrip::Beam)),
        "no grip without the selection"
    );
}

#[test]
fn dragging_the_direction_grip_turns_the_lamp_to_the_pointer() {
    let mut s = open();
    let edit = drag(&s, LightGrip::Direction, Point::new(1.0, 14.0));

    s.edit_plan(edit).unwrap();

    let after = light(&s);
    assert!(
        (after.facing_deg - 90.0).abs() < EPS,
        "{}",
        after.facing_deg
    );
    assert!((after.beam_deg - 60.0).abs() < EPS);
    assert_eq!(after.at, Point::new(1.0, 10.0));
}

#[test]
fn dragging_the_beam_grip_narrows_and_widens_the_beam() {
    let mut s = open();

    let edit = drag(&s, LightGrip::Beam, pointer(&s, 345.0));
    s.edit_plan(edit).unwrap();
    assert!((light(&s).beam_deg - 30.0).abs() < 1e-6);

    // The grip moved with the beam; the other side of the facing counts as well.
    let edit = drag(&s, LightGrip::Beam, pointer(&s, 230.0));
    s.edit_plan(edit).unwrap();

    let after = light(&s);
    assert!((after.beam_deg - 200.0).abs() < 1e-6, "{}", after.beam_deg);
    assert!((after.facing_deg - 330.0).abs() < EPS);
    assert_eq!(after.at, Point::new(1.0, 10.0));
}

#[test]
fn the_beam_stays_within_what_a_lamp_may_have() {
    let mut s = open();

    let edit = drag(&s, LightGrip::Beam, pointer(&s, 330.0));
    s.edit_plan(edit).unwrap();
    let narrow = light(&s).beam_deg;
    assert!(
        is_beam_deg(narrow) && narrow <= MIN_BEAM_DEG + EPS,
        "{narrow}"
    );

    let edit = drag(&s, LightGrip::Beam, pointer(&s, 150.0));
    s.edit_plan(edit).unwrap();
    let wide = light(&s).beam_deg;
    assert!(is_beam_deg(wide) && wide > 359.0, "{wide}");
}

/// One edit, one undo step.
#[test]
fn a_drag_undoes_in_one_step() {
    let mut s = open();
    let edit = drag(&s, LightGrip::Beam, pointer(&s, 345.0));

    s.edit_plan(edit).unwrap();
    s.undo();

    assert!((light(&s).beam_deg - 60.0).abs() < EPS);
    assert!(!s.can_undo());
}

#[test]
fn a_locked_lights_layer_refuses_the_grips() {
    let mut s = open();
    let edit = drag(&s, LightGrip::Direction, Point::new(1.0, 14.0));
    s.set_lock(Layer::Lights, Lock::Locked);

    let result = s.edit_plan(edit);

    assert!(matches!(result, Err(ServiceError::Locked(Layer::Lights))));
    assert!((light(&s).facing_deg - 330.0).abs() < EPS);
}

#[test]
fn removing_a_grip_removes_nothing() {
    let mut s = open();

    let result = s.edit_plan(PlanEdit::Remove {
        source: grip(LightGrip::Beam),
    });

    assert!(result.is_err());
    assert_eq!(s.plan().unwrap().lights.len(), 1);
}

/// A beam closed to its narrowest: the beam grip is still the one a click on it takes, so the
/// beam can be opened again.
#[test]
fn the_beam_grip_stays_apart_from_the_direction_grip() {
    let mut s = open();
    let edit = drag(&s, LightGrip::Beam, pointer(&s, 330.0));
    s.edit_plan(edit).unwrap();
    assert!((light(&s).beam_deg - 1.0).abs() < 1e-6);

    let beam = grip_at(&s, LightGrip::Beam).unwrap();

    assert_eq!(s.pick_for(beam, Some(&lamp())), Some(grip(LightGrip::Beam)));
}
