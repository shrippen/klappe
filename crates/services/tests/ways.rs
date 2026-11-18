//! A mark before the first (LH-105), moving a way at once (LH-107) and straight or smooth ways
//! (LH-108) on the example project (`testdata/beispiel.klappe`). Elisabeth: marks (3, 6.5) at
//! 0 s with a 2 s pause, (10, 6.5) at 6 s, (16, 3) at 14 s. Burgvogt: (17, 9.5) at 0 s with a
//! 5 s pause, (12, 6.5) at 9 s. Camera A: (8, 11) at 0 s, 6 s pause, (9, 8) at 6 s.
#![allow(clippy::unwrap_used)]

use std::path::Path;

use klappe_model::LinkTarget;
use klappe_services::{
    Arrival, Bend, Curve, Layer, Lock, PlanEdit, Point, Rig, Role, ServiceError, Session, Shape,
    Snapping, Source,
};

/// Walking speed that times a way, as new marks do (m/s).
const WALK_M_S: f64 = 1.4;

fn open() -> Session {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe");
    let mut s = Session::open(&dir).unwrap();
    s.set_snapping(Snapping::Off);
    s
}

fn p(x_m: f64, y_m: f64) -> Point {
    Point::new(x_m, y_m)
}

fn near(a: Point, b: Point) -> bool {
    a.distance_m(b) < 1e-9
}

/// Equal as the worked numbers are, to a nanometre.
fn near_to(a: Point, b: Point) -> bool {
    a.distance_m(b) < 1e-6
}

fn elisabeth(index: usize) -> Source {
    Source::Mark {
        character: "elisabeth".into(),
        index,
    }
}

/// Elisabeth's marks as place and time.
fn places(s: &Session) -> Vec<(Point, f64)> {
    let track = &s.plan().unwrap().figures[0];
    track.marks.iter().map(|m| (m.at, m.at_s)).collect()
}

/// A bend at (6.5, 4.5) on Elisabeth's way to mark 1: the grip (6.5, 6.5) pulled by (0, -2).
fn bent(s: &mut Session) {
    s.edit_plan(PlanEdit::Shift {
        source: Source::FigureVia {
            character: "elisabeth".into(),
            index: 1,
            bend: Bend::New(0),
        },
        dx_m: 0.0,
        dy_m: -2.0,
    })
    .unwrap();
}

#[test]
fn a_mark_before_the_first_shifts_the_times_later() {
    let mut s = open();
    let before = places(&s);

    s.edit_plan(PlanEdit::AddMarkBefore {
        source: Source::Figure("burgvogt".into()),
        at: p(20.0, 9.5),
    })
    .unwrap();

    let track = &s.plan().unwrap().figures[1];
    let walk_s = 3.0 / WALK_M_S;
    assert_eq!(track.marks.len(), 3);
    assert!(near(track.marks[0].at, p(20.0, 9.5)));
    assert_eq!(track.marks[0].at_s, 0.0);
    assert_eq!(track.marks[0].hold_s, 0.0);
    // It faces the old first mark, which is to its left.
    assert_eq!(track.marks[0].facing_deg, 180.0);
    // The old first mark keeps its place and pause; it, and all later ones, come later.
    assert!(near(track.marks[1].at, p(17.0, 9.5)));
    assert_eq!(track.marks[1].hold_s, 5.0);
    assert!((track.marks[1].at_s - walk_s).abs() < 1e-9);
    assert!((track.marks[2].at_s - (9.0 + walk_s)).abs() < 1e-9);
    // The other figures stay as they were.
    assert_eq!(places(&s), before);

    s.undo();
    assert_eq!(s.plan().unwrap().figures[1].marks.len(), 2);
}

/// Links name a mark by its index: the old marks move on by one, their links with them.
#[test]
fn a_mark_before_the_first_keeps_the_links() {
    let mut s = open();
    let burgvogt_links = |s: &Session| {
        s.plan()
            .unwrap()
            .links
            .iter()
            .filter_map(|l| match &l.target {
                LinkTarget::Mark { character, index } if character == "burgvogt" => Some(*index),
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(burgvogt_links(&s), vec![1]);

    s.edit_plan(PlanEdit::AddMarkBefore {
        source: Source::Figure("burgvogt".into()),
        at: p(20.0, 9.5),
    })
    .unwrap();
    assert_eq!(burgvogt_links(&s), vec![2]);

    s.undo();
    assert_eq!(burgvogt_links(&s), vec![1]);
}

#[test]
fn a_figure_that_appeared_appears_at_the_new_first_mark() {
    let mut s = open();
    s.edit_plan(PlanEdit::SetArrival {
        source: elisabeth(0),
        arrival: Arrival::Appear,
    })
    .unwrap();

    s.edit_plan(PlanEdit::AddMarkBefore {
        source: elisabeth(0),
        at: p(0.0, 6.5),
    })
    .unwrap();

    let marks = &s.plan().unwrap().figures[0].marks;
    assert_eq!(marks[0].arrival, Arrival::Appear);
    assert_eq!(marks[1].arrival, Arrival::Walk);
}

#[test]
fn a_camera_mark_before_the_first_rolls_by_dolly() {
    let mut s = open();

    s.edit_plan(PlanEdit::AddMarkBefore {
        source: Source::Camera("a".into()),
        at: p(8.0, 14.0),
    })
    .unwrap();

    let marks = &s.plan().unwrap().cameras[0].marks;
    assert_eq!(marks.len(), 3);
    assert!(near(marks[0].at, p(8.0, 14.0)));
    assert_eq!(marks[0].at_s, 0.0);
    assert_eq!(marks[0].rig, Rig::Dolly);
    assert_eq!(marks[1].rig, Rig::Dolly);
    assert!((marks[1].at_s - 3.0 / WALK_M_S).abs() < 1e-9);
    assert_eq!(marks[1].hold_s, 6.0);
    assert_eq!(marks[0].height_m, marks[1].height_m);
}

#[test]
fn a_way_moves_with_its_marks_and_bends() {
    let mut s = open();
    bent(&mut s);

    s.edit_plan(PlanEdit::ShiftWay {
        source: Source::FigureVia {
            character: "elisabeth".into(),
            index: 1,
            bend: Bend::At(0),
        },
        dx_m: 1.0,
        dy_m: 2.0,
    })
    .unwrap();

    let track = &s.plan().unwrap().figures[0];
    assert!(near(track.marks[0].at, p(4.0, 8.5)));
    assert!(near(track.marks[1].at, p(11.0, 8.5)));
    assert!(near(track.marks[1].via[0], p(7.5, 6.5)));
    // The time and the mark beyond stay.
    assert_eq!(track.marks[1].at_s, 6.0);
    assert!(near(track.marks[2].at, p(16.0, 3.0)));

    s.undo();
    let track = &s.plan().unwrap().figures[0];
    assert!(near(track.marks[0].at, p(3.0, 6.5)));
    assert!(near(track.marks[1].via[0], p(6.5, 4.5)));
}

/// Named by its mark, the way to it moves; the first mark has none; a camera's way is the one
/// to its mark at the playhead.
#[test]
fn a_way_is_named_by_its_mark_or_camera() {
    let mut s = open();
    let shift = |source| PlanEdit::ShiftWay {
        source,
        dx_m: 0.0,
        dy_m: 1.0,
    };

    assert!(s.edit_plan(shift(elisabeth(0))).is_err());
    assert!(s.edit_plan(shift(Source::Camera("a".into()))).is_err());

    s.set_time(7.0);
    s.edit_plan(shift(Source::Camera("a".into()))).unwrap();
    let marks = &s.plan().unwrap().cameras[0].marks;
    assert!(near(marks[0].at, p(8.0, 12.0)));
    assert!(near(marks[1].at, p(9.0, 9.0)));

    s.edit_plan(shift(elisabeth(1))).unwrap();
    assert!(near(places(&s)[0].0, p(3.0, 7.5)));
}

#[test]
fn the_ways_beyond_keep_their_shape() {
    let mut s = open();
    // The example's way to mark 2 has a bend; straight, then a bend beside the middle of the leg.
    s.edit_plan(PlanEdit::Straighten {
        source: elisabeth(2),
    })
    .unwrap();
    s.edit_plan(PlanEdit::Shift {
        source: Source::FigureVia {
            character: "elisabeth".into(),
            index: 2,
            bend: Bend::New(0),
        },
        dx_m: 0.0,
        dy_m: 2.0,
    })
    .unwrap();
    let bend = s.plan().unwrap().figures[0].marks[2].via[0];
    // The grip sat in the middle of the leg, (13, 4.75), and was pulled by (0, 2).
    assert!(near(bend, p(13.0, 6.75)), "{bend:?}");

    s.edit_plan(PlanEdit::ShiftWay {
        source: elisabeth(1),
        dx_m: 0.0,
        dy_m: 1.0,
    })
    .unwrap();

    // The leg (10, 6.5)-(16, 3) became (10, 7.5)-(16, 3). By hand, as complex numbers: the
    // offset of the bend from the old middle (13, 4.75) is z = 2i; the legs are e = 6 - 3.5i
    // and f = 6 - 4.5i, so z * f / e = 2i * (51.75 - 6i) / 48.25 = (12 + 103.5i) / 48.25, added
    // to the new middle (13, 5.25): the bend lies at (13 + 12 / 48.25, 5.25 + 103.5 / 48.25).
    let moved = s.plan().unwrap().figures[0].marks[2].via[0];
    assert!(
        near_to(moved, p(13.0 + 12.0 / 48.25, 5.25 + 103.5 / 48.25)),
        "{moved:?}"
    );
}

#[test]
fn a_locked_layer_keeps_its_ways() {
    let mut s = open();
    s.set_lock(Layer::Figures, Lock::Locked);

    for edit in [
        PlanEdit::AddMarkBefore {
            source: elisabeth(0),
            at: p(0.0, 0.0),
        },
        PlanEdit::ShiftWay {
            source: elisabeth(1),
            dx_m: 1.0,
            dy_m: 0.0,
        },
        PlanEdit::SetCurve {
            source: elisabeth(1),
            curve: Curve::Straight,
        },
    ] {
        assert!(matches!(
            s.edit_plan(edit),
            Err(ServiceError::Locked(Layer::Figures))
        ));
    }

    s.set_lock(Layer::Cameras, Lock::Locked);
    assert!(matches!(
        s.edit_plan(PlanEdit::AddMarkBefore {
            source: Source::Camera("a".into()),
            at: p(0.0, 0.0),
        }),
        Err(ServiceError::Locked(Layer::Cameras))
    ));
}

/// The drawn way of Elisabeth to mark 1, the points of its polyline.
fn way_points(s: &Session) -> usize {
    let list = s.display_list();
    let item = list
        .items
        .iter()
        .find(|i| matches!(i.role, Role::FigurePath(_)) && i.label.is_none());
    let Some(Shape::Polyline { points }) = item.map(|i| i.shape.clone()) else {
        unreachable!("a figure's way is a polyline");
    };

    points.len()
}

#[test]
fn a_straight_way_is_drawn_with_corners() {
    let mut s = open();
    bent(&mut s);
    let smooth = way_points(&s);

    s.edit_plan(PlanEdit::SetCurve {
        source: elisabeth(1),
        curve: Curve::Straight,
    })
    .unwrap();

    assert_eq!(s.plan().unwrap().figures[0].marks[1].curve, Curve::Straight);
    assert!(way_points(&s) < smooth);

    // The first mark has no way; undo gives the curve back.
    assert!(s
        .edit_plan(PlanEdit::SetCurve {
            source: elisabeth(0),
            curve: Curve::Straight,
        })
        .is_err());
    s.undo();
    assert_eq!(way_points(&s), smooth);
}

#[test]
fn the_curve_shows_in_the_mark_fields() {
    let mut s = open();
    let info = |s: &Session, index| s.mark_info(&elisabeth(index)).unwrap().curve;

    assert_eq!(info(&s, 0), None);
    assert_eq!(info(&s, 1), Some(Curve::Smooth));

    s.edit_plan(PlanEdit::SetCurve {
        source: Source::CameraVia {
            camera: "a".into(),
            index: 1,
            bend: Bend::New(0),
        },
        curve: Curve::Straight,
    })
    .unwrap();
    s.set_time(7.0);
    assert_eq!(
        s.mark_info(&Source::Camera("a".into())).unwrap().curve,
        Some(Curve::Straight)
    );
}
