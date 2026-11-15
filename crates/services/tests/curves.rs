//! Bending the ways of figures and cameras (LH-21) on the example project
//! (`testdata/beispiel.klappe`), numbers worked out by hand: Burgvogt walks from (17, 9.5) to
//! (12, 6.5), leaving at 5 s and arriving at 9 s.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_services::{
    Layer, Lock, PlanEdit, Point, Role, ServiceError, Session, Shape, Snapping, Source,
};

fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe")
}

fn open() -> Session {
    let mut s = Session::open(&example()).unwrap();
    s.set_snapping(Snapping::Off);
    s
}

fn p(x_m: f64, y_m: f64) -> Point {
    Point::new(x_m, y_m)
}

fn near(a: Point, b: Point) -> bool {
    a.distance_m(b) < 1e-9
}

fn grip(index: usize) -> Source {
    Source::FigureVia {
        character: "burgvogt".into(),
        index,
    }
}

fn via(s: &Session) -> Option<Point> {
    s.plan().unwrap().figures[1].marks[1].via
}

/// Where the list draws Burgvogt's body.
fn body(s: &Session) -> Point {
    let list = s.display_list();
    let item = list
        .items
        .iter()
        .find(|i| matches!(i.role, Role::Figure(_)) && i.label.as_deref() == Some("Burgvogt"))
        .unwrap();
    let Shape::Circle { center, .. } = item.shape else {
        unreachable!("a figure is a circle");
    };
    center
}

/// The grip sits halfway along the straight leg, (14.5, 8). Pulled by (1.2, -2) at right angles
/// to the leg, the way passes (15.7, 6) halfway: `via` is (16.9, 4), and the curve being
/// symmetric, Burgvogt is at (15.7, 6) at 7 s, the middle of his walk.
#[test]
fn dragging_the_grip_bends_the_way() {
    let mut s = open();

    s.edit_plan(PlanEdit::Shift {
        source: grip(1),
        dx_m: 1.2,
        dy_m: -2.0,
    })
    .unwrap();

    assert!(near(via(&s).unwrap(), p(16.9, 4.0)), "{:?}", via(&s));
    s.set_time(7.0);
    assert!(near(body(&s), p(15.7, 6.0)), "{:?}", body(&s));

    s.undo();
    assert_eq!(via(&s), None);
}

/// A second drag starts from where the grip is now: on the curve, not on `via`.
#[test]
fn a_second_drag_moves_from_the_grip() {
    let mut s = open();
    let shift = |dy_m| PlanEdit::Shift {
        source: grip(1),
        dx_m: 0.0,
        dy_m,
    };

    s.edit_plan(shift(-2.0)).unwrap();
    s.edit_plan(shift(1.0)).unwrap();

    // The grip went to (14.5, 6), then to (14.5, 7): via (14.5, 6).
    assert!(near(via(&s).unwrap(), p(14.5, 6.0)), "{:?}", via(&s));
}

#[test]
fn a_grip_of_a_camera_bends_its_path() {
    let mut s = open();

    s.edit_plan(PlanEdit::Shift {
        source: Source::CameraVia {
            camera: "a".into(),
            index: 1,
        },
        dx_m: 0.0,
        dy_m: 1.0,
    })
    .unwrap();

    // The grip was halfway between (8, 11) and (9, 8), at (8.5, 9.5).
    let via = s.plan().unwrap().cameras[0].marks[1].via.unwrap();
    assert!(near(via, p(8.5, 11.5)), "{via:?}");
}

#[test]
fn straighten_takes_the_bend_out() {
    let mut s = open();
    s.edit_plan(PlanEdit::Shift {
        source: grip(1),
        dx_m: 0.0,
        dy_m: -2.0,
    })
    .unwrap();

    s.edit_plan(PlanEdit::Straighten { source: grip(1) })
        .unwrap();
    assert_eq!(via(&s), None);

    s.undo();
    assert!(via(&s).is_some());
}

/// A mark straightens its own leg; the figure all of its legs.
#[test]
fn straighten_by_mark_or_by_figure() {
    let mut s = open();
    let bend = |s: &mut Session, index| {
        s.edit_plan(PlanEdit::Shift {
            source: Source::FigureVia {
                character: "elisabeth".into(),
                index,
            },
            dx_m: 0.0,
            dy_m: 1.0,
        })
        .unwrap();
    };
    let vias = |s: &Session| -> Vec<bool> {
        s.plan().unwrap().figures[0]
            .marks
            .iter()
            .map(|m| m.via.is_some())
            .collect()
    };

    bend(&mut s, 1);
    bend(&mut s, 2);
    assert_eq!(vias(&s), [false, true, true]);

    s.edit_plan(PlanEdit::Straighten {
        source: Source::Mark {
            character: "elisabeth".into(),
            index: 1,
        },
    })
    .unwrap();
    assert_eq!(vias(&s), [false, false, true]);

    bend(&mut s, 1);
    s.edit_plan(PlanEdit::Straighten {
        source: Source::Figure("elisabeth".into()),
    })
    .unwrap();
    assert_eq!(vias(&s), [false, false, false]);
}

/// Moving a mark carries the bend: it keeps its place relative to the leg. Hand-worked: leg
/// (17, 9.5)-(12, 6.5) with `via` (14.5, 4) becomes (17, 9.5)-(11, 7), `via` (14 - 11/17,
/// 8.25 - 75/17).
#[test]
fn moving_a_mark_carries_the_bend() {
    let mut s = open();
    s.edit_plan(PlanEdit::Shift {
        source: grip(1),
        dx_m: 0.0,
        dy_m: -2.0,
    })
    .unwrap();

    s.edit_plan(PlanEdit::Move {
        source: Source::Mark {
            character: "burgvogt".into(),
            index: 1,
        },
        to: p(11.0, 7.0),
    })
    .unwrap();

    let expected = p(14.0 - 11.0 / 17.0, 8.25 - 75.0 / 17.0);
    assert!(near(via(&s).unwrap(), expected), "{:?}", via(&s));
}

/// How far the bend of the way to mark `index` of Elisabeth lies from the leg's middle, in
/// legs: what a move must not change.
fn bulge(s: &Session, index: usize) -> f64 {
    let marks = &s.plan().unwrap().figures[0].marks;
    let (a, b) = (marks[index - 1].at, marks[index].at);

    marks[index].via.unwrap().distance_m(a.lerp(b, 0.5)) / a.distance_m(b)
}

/// A mark has a way to it and a way from it: both bends go along.
#[test]
fn moving_a_mark_carries_both_ways() {
    let mut s = open();
    for index in [1, 2] {
        s.edit_plan(PlanEdit::Shift {
            source: Source::FigureVia {
                character: "elisabeth".into(),
                index,
            },
            dx_m: 0.0,
            dy_m: 1.0,
        })
        .unwrap();
    }
    let before = (bulge(&s, 1), bulge(&s, 2));
    let at = s.plan().unwrap().figures[0].marks[1].at;

    s.edit_plan(PlanEdit::Shift {
        source: Source::Mark {
            character: "elisabeth".into(),
            index: 1,
        },
        dx_m: 0.7,
        dy_m: -0.4,
    })
    .unwrap();

    let moved = s.plan().unwrap().figures[0].marks[1].at;
    assert!(moved.distance_m(at) > 0.1);
    assert!((bulge(&s, 1) - before.0).abs() < 1e-9);
    assert!((bulge(&s, 2) - before.1).abs() < 1e-9);
}

/// Marks and grips moved together all land `d` away: bends included, whatever the order.
#[test]
fn a_group_moves_its_bends_along() {
    let mut s = open();
    s.edit_plan(PlanEdit::Shift {
        source: grip(1),
        dx_m: 0.0,
        dy_m: -2.0,
    })
    .unwrap();
    let before = via(&s).unwrap();
    let mark = |index| Source::Mark {
        character: "burgvogt".into(),
        index,
    };

    s.edit_plan(PlanEdit::ShiftGroup {
        sources: vec![mark(0), mark(1)],
        dx_m: 1.5,
        dy_m: 0.5,
    })
    .unwrap();

    assert!(near(
        via(&s).unwrap(),
        p(before.x_m + 1.5, before.y_m + 0.5)
    ));
}

/// A group that turns turns the bends with it.
#[test]
fn a_group_turns_its_bends() {
    let mut s = open();
    s.edit_plan(PlanEdit::Shift {
        source: grip(1),
        dx_m: 0.0,
        dy_m: -2.0,
    })
    .unwrap();
    let before = via(&s).unwrap();
    let marks = s.plan().unwrap().figures[1].marks.clone();
    let mid = marks[0].at.lerp(marks[1].at, 0.5);
    let offset = p(before.x_m - mid.x_m, before.y_m - mid.y_m);
    let mark = |index| Source::Mark {
        character: "burgvogt".into(),
        index,
    };

    s.edit_plan(PlanEdit::SpinGroup {
        sources: vec![mark(0), mark(1)],
        by_deg: 90.0,
    })
    .unwrap();

    // A quarter turn clockwise on the plan sends (x, y) to (-y, x).
    let marks = s.plan().unwrap().figures[1].marks.clone();
    let mid = marks[0].at.lerp(marks[1].at, 0.5);
    let expected = p(mid.x_m - offset.y_m, mid.y_m + offset.x_m);
    assert!(near(via(&s).unwrap(), expected), "{:?}", via(&s));
}

/// Removing the first mark makes the second the first, which has no way to it.
#[test]
fn the_new_first_mark_has_no_bend() {
    let mut s = open();
    s.edit_plan(PlanEdit::Shift {
        source: grip(1),
        dx_m: 0.0,
        dy_m: -2.0,
    })
    .unwrap();

    s.edit_plan(PlanEdit::Remove {
        source: Source::Mark {
            character: "burgvogt".into(),
            index: 0,
        },
    })
    .unwrap();

    assert_eq!(s.plan().unwrap().figures[1].marks[0].via, None);
}

/// A grip has no leg before the first mark, nor beyond the last.
#[test]
fn a_grip_that_is_not_there_is_gone() {
    let mut s = open();

    for index in [0, 2] {
        let edit = PlanEdit::Shift {
            source: grip(index),
            dx_m: 1.0,
            dy_m: 0.0,
        };
        assert!(matches!(s.edit_plan(edit), Err(ServiceError::Gone)));
    }
}

#[test]
fn removing_a_grip_straightens() {
    let mut s = open();
    s.edit_plan(PlanEdit::Shift {
        source: grip(1),
        dx_m: 0.0,
        dy_m: -2.0,
    })
    .unwrap();

    s.edit_plan(PlanEdit::Remove { source: grip(1) }).unwrap();

    assert_eq!(via(&s), None);
    assert_eq!(s.plan().unwrap().figures[1].marks.len(), 2);
}

#[test]
fn a_locked_layer_keeps_its_bends() {
    let mut s = open();
    s.set_lock(Layer::Figures, Lock::Locked);

    for edit in [
        PlanEdit::Shift {
            source: grip(1),
            dx_m: 0.0,
            dy_m: -2.0,
        },
        PlanEdit::Straighten { source: grip(1) },
    ] {
        assert!(matches!(
            s.edit_plan(edit),
            Err(ServiceError::Locked(Layer::Figures))
        ));
    }
}

/// Grips show with the selection only, can be picked then, and are not in what is exported.
#[test]
fn grips_show_for_the_selected_way() {
    let s = open();
    let grips = |list: &klappe_services::DisplayList| {
        list.items
            .iter()
            .filter(|i| matches!(i.role, Role::FigureHandle(_)))
            .count()
    };
    let selected = Source::Figure("burgvogt".into());

    assert_eq!(grips(&s.display_list()), 0);
    assert_eq!(grips(&s.display_list_for(None)), 0);
    assert_eq!(grips(&s.display_list_for(Some(&selected))), 1);

    // At (14.5, 8) halfway between the marks; unselected it is not there to take.
    assert_eq!(s.pick_for(p(14.5, 8.0), Some(&selected)), Some(grip(1)));
    assert_eq!(s.pick_for(p(14.5, 8.0), None), None);
}

/// With the figures layer hidden, its grips are hidden too.
#[test]
fn hidden_layers_hide_their_grips() {
    let mut s = open();
    s.set_layer(Layer::Figures, klappe_services::Visibility::Hidden);
    let selected = Source::Figure("burgvogt".into());

    let list = s.display_list_for(Some(&selected));

    assert!(list
        .items
        .iter()
        .all(|i| !matches!(i.role, Role::FigureHandle(_))));
}

/// While the grip is dragged, the preview shows the curve and the grip where it is dragged.
#[test]
fn the_preview_moves_the_grip_too() {
    let s = open();
    let selected = grip(1);
    let edit = PlanEdit::Shift {
        source: selected.clone(),
        dx_m: 0.0,
        dy_m: -2.0,
    };

    let list = s.preview_for(&edit, Some(&selected)).unwrap();
    let at = list
        .items
        .iter()
        .find_map(|i| match (&i.role, &i.shape) {
            (Role::FigureHandle(_), Shape::Circle { center, .. }) => Some(*center),
            _ => None,
        })
        .unwrap();

    assert!(near(at, p(14.5, 6.0)), "{at:?}");
}
