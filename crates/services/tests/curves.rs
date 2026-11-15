//! Bending the ways of figures and cameras (LH-21) on the example project
//! (`testdata/beispiel.klappe`), numbers worked out by hand: Burgvogt walks from (17, 9.5) to
//! (12, 6.5), leaving at 5 s and arriving at 9 s.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_services::{
    Bend, Layer, Lock, PlanEdit, Point, Role, ServiceError, Session, Shape, Snapping, Source,
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

fn grip_of(index: usize, bend: Bend) -> Source {
    Source::FigureVia {
        character: "burgvogt".into(),
        index,
        bend,
    }
}

/// The grip that pulls a new bend out of piece `n` of Burgvogt's way to mark 1.
fn grip(n: usize) -> Source {
    grip_of(1, Bend::New(n))
}

/// Bend `n` of Burgvogt's way to mark 1.
fn bend(n: usize) -> Source {
    grip_of(1, Bend::At(n))
}

fn via(s: &Session) -> Vec<Point> {
    s.plan().unwrap().figures[1].marks[1].via.clone()
}

/// Burgvogt's way with one bend at (14.5, 4): the grip (14.5, 8) pulled by (0, -4).
fn bent(s: &mut Session) {
    s.edit_plan(PlanEdit::Shift {
        source: grip(0),
        dx_m: 0.0,
        dy_m: -4.0,
    })
    .unwrap();
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

/// The grip sits halfway along the straight leg, (14.5, 8). Pulled by (1.2, -2) it makes a
/// bend at (15.7, 6), in one undo step.
#[test]
fn dragging_the_grip_makes_a_bend() {
    let mut s = open();

    s.edit_plan(PlanEdit::Shift {
        source: grip(0),
        dx_m: 1.2,
        dy_m: -2.0,
    })
    .unwrap();

    assert_eq!(via(&s).len(), 1);
    assert!(near(via(&s)[0], p(15.7, 6.0)), "{:?}", via(&s));

    s.undo();
    assert!(via(&s).is_empty());
}

/// The way passes its bend: Burgvogt's drawn path goes through it, and his walk takes the
/// longer way, so he is not where he was at the middle of the time.
#[test]
fn the_figure_walks_through_the_bend() {
    let mut s = open();
    let straight = {
        s.set_time(7.0);
        body(&s)
    };

    bent(&mut s);
    s.set_time(9.0);

    assert!(near(body(&s), p(12.0, 6.5)), "{:?}", body(&s));
    s.set_time(7.0);
    assert!(!near(body(&s), straight), "{:?}", body(&s));
}

/// A drag that leaves the grip where it is makes no bend; one that moves it does, and a
/// second grip makes a second bend, in order along the way.
#[test]
fn bends_are_made_in_order_along_the_way() {
    let mut s = open();
    let drag = |s: &mut Session, source: Source, dy_m| {
        s.edit_plan(PlanEdit::Shift {
            source,
            dx_m: 0.0,
            dy_m,
        })
        .unwrap();
    };

    drag(&mut s, grip(0), 0.0);
    assert!(via(&s).is_empty());

    drag(&mut s, grip(0), -4.0);
    let first = via(&s)[0];

    // Pulled out of the piece after the bend: the new one comes second.
    drag(&mut s, grip(1), 1.0);
    assert_eq!(via(&s).len(), 2);
    assert!(near(via(&s)[0], first));

    // Out of the piece before the bend: the new one comes first.
    drag(&mut s, grip(0), 1.0);
    assert_eq!(via(&s).len(), 3);
    assert!(near(via(&s)[1], first));
}

/// A second drag, on a bend, starts from where the bend is.
#[test]
fn a_second_drag_moves_the_bend() {
    let mut s = open();
    bent(&mut s);

    s.edit_plan(PlanEdit::Shift {
        source: bend(0),
        dx_m: 0.0,
        dy_m: 1.0,
    })
    .unwrap();

    assert_eq!(via(&s).len(), 1);
    assert!(near(via(&s)[0], p(14.5, 5.0)), "{:?}", via(&s));
}

#[test]
fn a_grip_of_a_camera_bends_its_path() {
    let mut s = open();

    s.edit_plan(PlanEdit::Shift {
        source: Source::CameraVia {
            camera: "a".into(),
            index: 1,
            bend: Bend::New(0),
        },
        dx_m: 0.0,
        dy_m: 1.0,
    })
    .unwrap();

    // The grip was halfway between (8, 11) and (9, 8), at (8.5, 9.5).
    let via = &s.plan().unwrap().cameras[0].marks[1].via;
    assert_eq!(via.len(), 1);
    assert!(near(via[0], p(8.5, 10.5)), "{via:?}");
}

#[test]
fn straighten_takes_all_bends_out() {
    let mut s = open();
    bent(&mut s);
    s.edit_plan(PlanEdit::Shift {
        source: grip(1),
        dx_m: 0.0,
        dy_m: 1.0,
    })
    .unwrap();
    assert_eq!(via(&s).len(), 2);

    s.edit_plan(PlanEdit::Straighten { source: bend(0) })
        .unwrap();
    assert!(via(&s).is_empty());

    s.undo();
    assert_eq!(via(&s).len(), 2);
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
                bend: Bend::New(0),
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
            .map(|m| !m.via.is_empty())
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
/// (17, 9.5)-(12, 6.5) with a bend at (14.5, 4) becomes (17, 9.5)-(11, 7), bend (14 - 11/17,
/// 8.25 - 75/17).
#[test]
fn moving_a_mark_carries_the_bend() {
    let mut s = open();
    bent(&mut s);

    s.edit_plan(PlanEdit::Move {
        source: Source::Mark {
            character: "burgvogt".into(),
            index: 1,
        },
        to: p(11.0, 7.0),
    })
    .unwrap();

    let expected = p(14.0 - 11.0 / 17.0, 8.25 - 75.0 / 17.0);
    assert_eq!(via(&s).len(), 1);
    assert!(near(via(&s)[0], expected), "{:?}", via(&s));
}

/// How far bend `n` of the way to mark `index` of Elisabeth lies from the leg's middle, in
/// legs: what a move must not change.
fn bulge(s: &Session, index: usize, n: usize) -> f64 {
    let marks = &s.plan().unwrap().figures[0].marks;
    let (a, b) = (marks[index - 1].at, marks[index].at);

    marks[index].via[n].distance_m(a.lerp(b, 0.5)) / a.distance_m(b)
}

/// A mark has a way to it and a way from it: all their bends go along.
#[test]
fn moving_a_mark_carries_both_ways() {
    let mut s = open();
    for (index, bend) in [(1, Bend::New(0)), (2, Bend::New(0)), (2, Bend::New(1))] {
        s.edit_plan(PlanEdit::Shift {
            source: Source::FigureVia {
                character: "elisabeth".into(),
                index,
                bend,
            },
            dx_m: 0.0,
            dy_m: 1.0,
        })
        .unwrap();
    }
    let before = (bulge(&s, 1, 0), bulge(&s, 2, 0), bulge(&s, 2, 1));
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
    assert!((bulge(&s, 1, 0) - before.0).abs() < 1e-9);
    assert!((bulge(&s, 2, 0) - before.1).abs() < 1e-9);
    assert!((bulge(&s, 2, 1) - before.2).abs() < 1e-9);
}

/// Marks and grips moved together all land `d` away: bends included, whatever the order.
#[test]
fn a_group_moves_its_bends_along() {
    let mut s = open();
    bent(&mut s);
    let before = via(&s)[0];
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

    assert!(near(via(&s)[0], p(before.x_m + 1.5, before.y_m + 0.5)));
}

/// A group that turns turns the bends with it.
#[test]
fn a_group_turns_its_bends() {
    let mut s = open();
    bent(&mut s);
    let before = via(&s)[0];
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
    assert!(near(via(&s)[0], expected), "{:?}", via(&s));
}

/// Removing the first mark makes the second the first, which has no way to it.
#[test]
fn the_new_first_mark_has_no_bend() {
    let mut s = open();
    bent(&mut s);

    s.edit_plan(PlanEdit::Remove {
        source: Source::Mark {
            character: "burgvogt".into(),
            index: 0,
        },
    })
    .unwrap();

    assert!(s.plan().unwrap().figures[1].marks[0].via.is_empty());
}

/// A grip has no leg before the first mark, nor beyond the last, and no bend or piece that is
/// not there.
#[test]
fn a_grip_that_is_not_there_is_gone() {
    let mut s = open();

    for source in [
        grip_of(0, Bend::New(0)),
        grip_of(2, Bend::New(0)),
        grip(1),
        bend(0),
    ] {
        let edit = PlanEdit::Shift {
            source,
            dx_m: 1.0,
            dy_m: 0.0,
        };
        assert!(matches!(s.edit_plan(edit), Err(ServiceError::Gone)));
    }
}

/// Removing a bend takes that one out, the others stay; a grip for a new bend has nothing to
/// remove.
#[test]
fn removing_a_bend_takes_it_out() {
    let mut s = open();
    bent(&mut s);
    s.edit_plan(PlanEdit::Shift {
        source: grip(1),
        dx_m: 0.0,
        dy_m: 1.0,
    })
    .unwrap();
    let second = via(&s)[1];

    s.edit_plan(PlanEdit::Remove { source: grip(0) }).unwrap();
    assert_eq!(via(&s).len(), 2);

    s.edit_plan(PlanEdit::Remove { source: bend(0) }).unwrap();

    assert_eq!(via(&s), [second]);
    assert_eq!(s.plan().unwrap().figures[1].marks.len(), 2);

    s.undo();
    assert_eq!(via(&s).len(), 2);
}

#[test]
fn a_locked_layer_keeps_its_bends() {
    let mut s = open();
    s.set_lock(Layer::Figures, Lock::Locked);

    for edit in [
        PlanEdit::Shift {
            source: grip(0),
            dx_m: 0.0,
            dy_m: -2.0,
        },
        PlanEdit::Straighten { source: grip(0) },
        PlanEdit::Remove { source: bend(0) },
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
    let mut s = open();
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
    assert_eq!(s.pick_for(p(14.5, 8.0), Some(&selected)), Some(grip(0)));
    assert_eq!(s.pick_for(p(14.5, 8.0), None), None);

    // A bend gets a grip of its own, and a grip for a new bend on each side of it.
    s.edit_plan(PlanEdit::Shift {
        source: grip(0),
        dx_m: 0.0,
        dy_m: -4.0,
    })
    .unwrap();
    assert_eq!(grips(&s.display_list_for(Some(&selected))), 3);
    assert_eq!(s.pick_for(p(14.5, 4.0), Some(&selected)), Some(bend(0)));
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

/// While a grip is dragged, the preview shows the curve and the new bend where it is dragged.
#[test]
fn the_preview_moves_the_grip_too() {
    let s = open();
    let selected = grip(0);
    let edit = PlanEdit::Shift {
        source: selected.clone(),
        dx_m: 0.0,
        dy_m: -2.0,
    };

    let list = s.preview_for(&edit, Some(&selected)).unwrap();
    let at: Vec<Point> = list
        .items
        .iter()
        .filter(|i| i.source == Some(bend(0)))
        .filter_map(|i| match i.shape {
            Shape::Circle { center, .. } => Some(center),
            _ => None,
        })
        .collect();

    assert_eq!(at.len(), 1);
    assert!(near(at[0], p(14.5, 6.0)), "{at:?}");
}

/// A drag that goes nowhere makes no bend, also with snapping on: the grip is off the grid,
/// and the spot it would snap to is not where the pointer went.
#[test]
fn a_drag_without_movement_makes_no_bend_when_snapping() {
    let mut s = open();
    bent(&mut s);
    s.set_snapping(Snapping::On);

    s.edit_plan(PlanEdit::Shift {
        source: grip(0),
        dx_m: 0.0,
        dy_m: 0.0,
    })
    .unwrap();

    assert_eq!(via(&s).len(), 1, "{:?}", via(&s));
}
