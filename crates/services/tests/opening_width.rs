//! The grips that set the width of a door or window (LH-10), on the example project's set:
//! wall 2 runs from (20, 13) to (0, 13) with a door 0.9 m wide at 5 m, so the door's start
//! is at x = 15.45 and its end at x = 14.55.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_services::{
    Layer, Lock, OpeningEnd, OpeningKind, PlanEdit, Point, ServiceError, Session, Snapping, Source,
};

const WALL: usize = 2;

fn open() -> Session {
    let dir: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe");
    let mut s = Session::open(&dir).unwrap();
    s.set_snapping(Snapping::Off);

    s
}

fn grip(end: OpeningEnd) -> Source {
    Source::OpeningEdge {
        wall: WALL,
        index: 0,
        end,
    }
}

fn drag(end: OpeningEnd, dx_m: f64) -> PlanEdit {
    PlanEdit::Shift {
        source: grip(end),
        dx_m,
        dy_m: 0.0,
    }
}

/// The door's `(start, end)` along the wall.
fn span_m(s: &Session) -> (f64, f64) {
    s.project().locations[0].set.walls[WALL].openings[0].span_m()
}

fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

/// The wall runs towards -x, so dragging the end towards -x widens the door; the start stays.
#[test]
fn dragging_the_end_widens_the_door_and_moves_its_middle() {
    let mut s = open();

    s.edit_plan(drag(OpeningEnd::End, -1.0)).unwrap();

    let (start_m, end_m) = span_m(&s);
    assert!(
        near(start_m, 4.55) && near(end_m, 6.45),
        "{start_m} {end_m}"
    );
    let door = &s.project().locations[0].set.walls[WALL].openings[0];
    assert!(near(door.at_m, 5.5) && near(door.width_m, 1.9));
}

#[test]
fn dragging_the_start_keeps_the_end() {
    let mut s = open();

    s.edit_plan(drag(OpeningEnd::Start, 0.5)).unwrap();

    let (start_m, end_m) = span_m(&s);
    assert!(
        near(start_m, 4.05) && near(end_m, 5.45),
        "{start_m} {end_m}"
    );
}

#[test]
fn a_drag_is_one_undo_step() {
    let mut s = open();

    s.edit_plan(drag(OpeningEnd::End, -1.0)).unwrap();
    s.undo();

    let (start_m, end_m) = span_m(&s);
    assert!(near(start_m, 4.55) && near(end_m, 5.45));
    assert!(!s.can_undo());
}

#[test]
fn an_end_dragged_over_the_other_leaves_the_minimum_width() {
    let mut s = open();

    s.edit_plan(drag(OpeningEnd::End, 5.0)).unwrap();

    let (start_m, end_m) = span_m(&s);
    assert!(
        near(start_m, 4.55) && near(end_m - start_m, 0.3),
        "{start_m} {end_m}"
    );
}

#[test]
fn an_opening_in_the_way_refuses_the_drag() {
    let mut s = open();
    let window_at = Point::new(12.0, 13.0);
    s.edit_plan(PlanEdit::AddOpening {
        kind: OpeningKind::Window,
        at: window_at,
    })
    .unwrap();

    // The window covers 7.4 m to 8.6 m along the wall; the door's end would reach 9 m.
    let result = s.edit_plan(drag(OpeningEnd::End, -3.5));

    assert!(result.is_err());
    let (start_m, end_m) = span_m(&s);
    assert!(near(start_m, 4.55) && near(end_m, 5.45));
}

#[test]
fn past_the_end_of_the_wall_is_refused() {
    let mut s = open();

    let result = s.edit_plan(drag(OpeningEnd::Start, 5.0));

    assert!(matches!(result, Err(ServiceError::BadValue { .. })));
    let (start_m, end_m) = span_m(&s);
    assert!(near(start_m, 4.55) && near(end_m, 5.45));
}

#[test]
fn a_locked_set_layer_refuses_the_grip() {
    let mut s = open();
    s.set_lock(Layer::Set, Lock::Locked);

    let result = s.edit_plan(drag(OpeningEnd::End, -1.0));

    assert!(matches!(result, Err(ServiceError::Locked(Layer::Set))));
    let (start_m, end_m) = span_m(&s);
    assert!(near(start_m, 4.55) && near(end_m, 5.45));
}

#[test]
fn removing_the_grip_removes_nothing() {
    let mut s = open();

    let result = s.edit_plan(PlanEdit::Remove {
        source: grip(OpeningEnd::End),
    });

    assert!(result.is_err());
    assert_eq!(s.project().locations[0].set.walls[WALL].openings.len(), 1);
}
