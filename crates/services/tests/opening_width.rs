//! The grips that set the width of a door or window (LH-10), on the example project's set:
//! wall 2 runs from (20, 13) to (0, 13) with a door 0.9 m wide at 5 m, so the door's start
//! is at x = 15.45 and its end at x = 14.55.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_model::MIN_OPENING_WIDTH_M;
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
        near(start_m, 4.55) && near(end_m - start_m, MIN_OPENING_WIDTH_M),
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

/// A door of the least width at the start of its wall: widening it would push the start past
/// the wall, so the drag is refused and the other end does not move.
#[test]
fn a_least_door_at_the_wall_start_keeps_its_end() {
    let mut s = open();
    let door = Source::Opening {
        wall: WALL,
        index: 0,
    };
    s.edit_plan(PlanEdit::SetOpeningWidth {
        source: door.clone(),
        width_m: MIN_OPENING_WIDTH_M,
    })
    .unwrap();
    s.edit_plan(PlanEdit::Shift {
        source: door,
        dx_m: 20.0,
        dy_m: 0.0,
    })
    .unwrap();
    let (start_m, end_m) = span_m(&s);
    assert!(
        near(start_m, 0.0) && near(end_m, MIN_OPENING_WIDTH_M),
        "{start_m} {end_m}"
    );

    let _ = s.edit_plan(drag(OpeningEnd::Start, -0.05));

    let (start_m, end_m) = span_m(&s);
    assert!(
        near(start_m, 0.0) && near(end_m, MIN_OPENING_WIDTH_M),
        "{start_m} {end_m}"
    );
}

/// Puts the door, `width_m` wide, flush at the far end of its wall.
fn door_at_wall_end(s: &mut Session, width_m: f64) {
    let door = Source::Opening {
        wall: WALL,
        index: 0,
    };
    s.edit_plan(PlanEdit::SetOpeningWidth {
        source: door.clone(),
        width_m,
    })
    .unwrap();
    s.edit_plan(PlanEdit::Shift {
        source: door,
        dx_m: -20.0,
        dy_m: 0.0,
    })
    .unwrap();
}

/// A door flush at the wall's end, its start dragged outwards: its end lies a rounding error
/// past the wall, which must not refuse the drag.
#[test]
fn a_door_at_the_wall_end_widens_from_its_start() {
    for width_m in [0.45, 0.7, 1.7] {
        let mut s = open();
        door_at_wall_end(&mut s, width_m);

        s.edit_plan(drag(OpeningEnd::Start, 0.1)).unwrap();

        let (start_m, end_m) = span_m(&s);
        assert!(
            near(start_m, 20.0 - width_m - 0.1) && (end_m - 20.0).abs() < 1e-6,
            "{width_m}: {start_m} {end_m}"
        );
    }
}

/// The mirror of the least door at the wall start: its end grip at the wall's end.
#[test]
fn a_least_door_at_the_wall_end_keeps_its_start() {
    let mut s = open();
    door_at_wall_end(&mut s, MIN_OPENING_WIDTH_M);
    let (start_before, _) = span_m(&s);

    let _ = s.edit_plan(drag(OpeningEnd::End, 0.05));

    let (start_m, end_m) = span_m(&s);
    assert!(near(start_m, start_before), "{start_m} {end_m}");
}

/// Typing a width follows the same least as dragging an end: the least itself is taken, a
/// hair under it is refused.
#[test]
fn a_typed_width_follows_the_least_a_drag_leaves() {
    let mut s = open();
    let typed = |width_m| PlanEdit::SetOpeningWidth {
        source: Source::Opening {
            wall: WALL,
            index: 0,
        },
        width_m,
    };

    assert!(s.edit_plan(typed(MIN_OPENING_WIDTH_M - 0.001)).is_err());
    s.edit_plan(typed(MIN_OPENING_WIDTH_M)).unwrap();

    let (start_m, end_m) = span_m(&s);
    assert!(near(end_m - start_m, MIN_OPENING_WIDTH_M));
}

/// A door dragged down to the least width is saved with a width that loads again: the width
/// is the limit itself, not the difference of two rounded ends.
#[test]
fn a_door_dragged_to_the_least_width_reopens() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("opening-least-width");
    let _ = std::fs::remove_dir_all(&dir);
    copy_dir(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe"),
        &dir,
    );
    let mut s = Session::open(&dir).unwrap();
    s.set_snapping(Snapping::Off);
    s.edit_plan(drag(OpeningEnd::End, 5.0)).unwrap();

    s.save().unwrap();

    let reopened = Session::open(&dir).unwrap();
    let door = &reopened.project().locations[0].set.walls[WALL].openings[0];
    assert!(door.width_m >= MIN_OPENING_WIDTH_M, "{}", door.width_m);
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        match entry.file_type().unwrap().is_dir() {
            true => copy_dir(&entry.path(), &target),
            false => {
                std::fs::copy(entry.path(), target).unwrap();
            }
        }
    }
}
