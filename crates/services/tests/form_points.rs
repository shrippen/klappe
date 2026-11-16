//! Dragging the points of a free form (LH-92): a form of the example project's set, drawn as
//! a triangle with its corners at (4, 5), (8, 5) and (4, 9) on the plan.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_services::{
    Layer, Lock, ObjectGrip, ObjectShape, PlanEdit, Point, Role, ServiceError, Session, Shape,
    Snapping, Source,
};

fn open() -> Session {
    let dir: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe");
    let mut s = Session::open(&dir).unwrap();
    s.set_snapping(Snapping::Off);
    s.edit_plan(PlanEdit::AddShape {
        points: vec![
            Point::new(4.0, 5.0),
            Point::new(8.0, 5.0),
            Point::new(4.0, 9.0),
        ],
    })
    .unwrap();

    s
}

fn near(a: Point, b: Point) -> bool {
    (a.x_m - b.x_m).abs() < 1e-9 && (a.y_m - b.y_m).abs() < 1e-9
}

fn form_id(s: &Session) -> String {
    let objects = &s.project().locations[0].set.objects;

    objects.last().unwrap().id.clone()
}

fn form(s: &Session) -> Source {
    Source::Object(form_id(s))
}

fn point(s: &Session, n: usize) -> Source {
    Source::ObjectGrip {
        object: form_id(s),
        grip: ObjectGrip::Point(n),
    }
}

/// Where the form's points lie on the plan: the shown grips.
fn places(s: &Session) -> Vec<Point> {
    let list = s.display_list_for(Some(&form(s)));

    list.items
        .iter()
        .filter(|i| {
            matches!(
                i.source,
                Some(Source::ObjectGrip {
                    grip: ObjectGrip::Point(_),
                    ..
                })
            )
        })
        .filter_map(|i| match &i.shape {
            Shape::Circle { center, .. } => Some(*center),
            _ => None,
        })
        .collect()
}

/// (centre, width, depth, point count) of the form.
fn state(s: &Session) -> (Point, f64, f64, usize) {
    let o = s.project().locations[0].set.objects.last().unwrap();
    let count = match &o.shape {
        ObjectShape::Polygon { points } => points.len(),
        _ => 0,
    };

    (o.at, o.width_m, o.depth_m, count)
}

fn turn(s: &mut Session, facing_deg: f64) {
    let source = form(s);

    s.edit_plan(PlanEdit::Turn { source, facing_deg }).unwrap();
}

fn drag(s: &mut Session, n: usize, dx_m: f64, dy_m: f64) -> Result<(), ServiceError> {
    let source = point(s, n);

    s.edit_plan(PlanEdit::Shift { source, dx_m, dy_m })
}

fn side(s: &Session, n: usize) -> Source {
    Source::ObjectGrip {
        object: form_id(s),
        grip: ObjectGrip::Side(n),
    }
}

fn pull(s: &mut Session, n: usize, dx_m: f64, dy_m: f64) -> Result<(), ServiceError> {
    let source = side(s, n);

    s.edit_plan(PlanEdit::Shift { source, dx_m, dy_m })
}

fn assert_others_stay(before: &[Point], after: &[Point], moved: usize) {
    assert_eq!(before.len(), after.len());
    for (n, (a, b)) in before.iter().zip(after).enumerate() {
        assert!(n == moved || near(*a, *b), "point {n}: {a:?} -> {b:?}");
    }
}

#[test]
fn a_selected_form_shows_a_grip_on_each_point_and_picks_it() {
    let s = open();
    let found = places(&s);

    assert_eq!(found.len(), 3);
    assert!(near(found[0], Point::new(4.0, 5.0)));
    assert!(near(found[2], Point::new(4.0, 9.0)));
    assert_eq!(s.pick_for(found[1], Some(&form(&s))), Some(point(&s, 1)));
    assert!(s
        .display_list()
        .items
        .iter()
        .all(|i| i.role != Role::ObjectHandle));
}

/// The box grows to (4, 5)-(10, 9): its centre moves to (7, 7), width 6, depth 4.
#[test]
fn dragging_a_point_moves_only_it_and_refits_the_box() {
    let mut s = open();
    let before = places(&s);

    drag(&mut s, 1, 2.0, 0.0).unwrap();

    let after = places(&s);
    assert_others_stay(&before, &after, 1);
    assert!(near(after[1], Point::new(10.0, 5.0)));
    let (at, width_m, depth_m, count) = state(&s);
    assert!(near(at, Point::new(7.0, 7.0)), "{at:?}");
    assert!((width_m - 6.0).abs() < 1e-9 && (depth_m - 4.0).abs() < 1e-9);
    assert_eq!(count, 3);
}

/// Turned by 90 degrees, the form's own frame is turned: the others still keep their place.
#[test]
fn a_turned_form_keeps_the_other_points_in_place() {
    let mut s = open();
    turn(&mut s, 90.0);
    let before = places(&s);

    drag(&mut s, 2, 3.0, 1.0).unwrap();

    let after = places(&s);
    assert_others_stay(&before, &after, 2);
    assert!(near(
        after[2],
        Point::new(before[2].x_m + 3.0, before[2].y_m + 1.0)
    ));
    let (_, width_m, depth_m, _) = state(&s);
    assert!(width_m > 0.0 && depth_m > 0.0);
}

/// Point 1 onto the line through the others leaves no area.
#[test]
fn a_move_that_leaves_no_outline_is_refused() {
    let mut s = open();
    let before = state(&s);

    let result = drag(&mut s, 1, -4.0, 4.0);

    assert!(matches!(result, Err(ServiceError::BadValue { .. })));
    assert_eq!(state(&s), before);
    assert!(near(places(&s)[1], Point::new(8.0, 5.0)));
}

#[test]
fn a_drag_is_one_undo_step() {
    let mut s = open();
    let before = (state(&s), places(&s));

    drag(&mut s, 1, 2.0, 0.0).unwrap();
    s.undo();

    assert_eq!(state(&s), before.0);
    assert_eq!(places(&s), before.1);
    s.undo();
    assert!(s.project().locations[0]
        .set
        .objects
        .iter()
        .all(|o| !matches!(o.shape, ObjectShape::Polygon { .. })));
}

#[test]
fn a_locked_set_layer_refuses_the_drag() {
    let mut s = open();
    let before = state(&s);
    s.set_lock(Layer::Set, Lock::Locked);

    let result = drag(&mut s, 1, 2.0, 0.0);

    assert!(matches!(result, Err(ServiceError::Locked(Layer::Set))));
    assert_eq!(state(&s), before);
}

/// Like an area's, a form keeps three points: Delete is refused with the reason for the
/// status line.
#[test]
fn delete_keeps_three_points() {
    let mut s = open();
    let before = state(&s);

    let result = s.edit_plan(PlanEdit::Remove {
        source: point(&s, 0),
    });

    assert!(matches!(result, Err(ServiceError::FewPoints)));
    assert_eq!(state(&s), before);
}

/// With four points, one goes; the rest stay where they are and the box shrinks to them.
#[test]
fn delete_takes_a_point_out_of_four() {
    let mut s = open();
    s.edit_plan(PlanEdit::AddShape {
        points: vec![
            Point::new(4.0, 5.0),
            Point::new(8.0, 5.0),
            Point::new(8.0, 9.0),
            Point::new(5.0, 9.0),
        ],
    })
    .unwrap();
    let before = places(&s);

    s.edit_plan(PlanEdit::Remove {
        source: point(&s, 1),
    })
    .unwrap();

    let after = places(&s);
    assert_eq!(after.len(), 3);
    assert!(near(after[0], before[0]) && near(after[1], before[2]) && near(after[2], before[3]));
    let (at, width_m, depth_m, _) = state(&s);
    assert!(near(at, Point::new(6.0, 7.0)), "{at:?}");
    assert!((width_m - 4.0).abs() < 1e-9 && (depth_m - 4.0).abs() < 1e-9);
    s.undo();
    assert_eq!(places(&s), before);
}

/// The side grips lie halfway along each edge; the last one on the closing edge.
#[test]
fn a_selected_form_shows_a_grip_halfway_along_each_side() {
    let s = open();
    let list = s.display_list_for(Some(&form(&s)));
    let sides: Vec<_> = list
        .items
        .iter()
        .filter(|i| {
            matches!(
                i.source,
                Some(Source::ObjectGrip {
                    grip: ObjectGrip::Side(_),
                    ..
                })
            )
        })
        .map(|i| (i.source.clone().unwrap(), i.shape.clone()))
        .collect();

    assert_eq!(sides.len(), 3);
    let centre = |n: usize| match &sides[n].1 {
        Shape::Circle { center, .. } => *center,
        _ => panic!("no circle"),
    };
    assert_eq!(sides[1].0, side(&s, 1));
    assert!(near(centre(0), Point::new(6.0, 5.0)));
    assert!(near(centre(1), Point::new(6.0, 7.0)));
    assert!(near(centre(2), Point::new(4.0, 7.0)));
    assert_eq!(s.pick_for(centre(0), Some(&form(&s))), Some(side(&s, 0)));
}

/// Pulling the middle of the first side down makes a point there, after point 0.
#[test]
fn pulling_a_side_makes_a_point_in_one_step() {
    let mut s = open();
    let before = (state(&s), places(&s));

    pull(&mut s, 0, 0.0, -2.0).unwrap();

    let after = places(&s);
    assert_eq!(after.len(), 4);
    assert!(near(after[0], before.1[0]) && near(after[2], before.1[1]));
    assert!(near(after[3], before.1[2]));
    assert!(near(after[1], Point::new(6.0, 3.0)));
    assert_eq!(state(&s).3, 4);
    s.undo();
    assert_eq!(state(&s), before.0);
}

/// The closing side puts its point last; a turned form keeps the others in place.
#[test]
fn a_turned_form_keeps_the_points_when_a_side_is_pulled() {
    let mut s = open();
    turn(&mut s, 90.0);
    let before = places(&s);

    pull(&mut s, 2, 2.0, 1.0).unwrap();

    let after = places(&s);
    assert_eq!(after.len(), 4);
    for n in 0..3 {
        assert!(near(before[n], after[n]), "point {n}");
    }
}

/// A side not pulled at all makes no point.
#[test]
fn a_side_not_pulled_makes_no_point() {
    let mut s = open();
    let before = state(&s);

    pull(&mut s, 0, 0.0, 0.0).unwrap();

    assert_eq!(state(&s), before);
}

/// A point far beyond the size of an object leaves no outline: refused, the form stays.
#[test]
fn a_pull_that_leaves_no_outline_is_refused() {
    let mut s = open();
    let before = state(&s);

    let result = pull(&mut s, 0, 5000.0, 0.0);

    assert!(matches!(result, Err(ServiceError::BadValue { .. })));
    assert_eq!(state(&s), before);
}

#[test]
fn a_locked_set_layer_refuses_a_side_pull() {
    let mut s = open();
    let before = state(&s);
    s.set_lock(Layer::Set, Lock::Locked);

    let result = pull(&mut s, 0, 0.0, -2.0);

    assert!(matches!(result, Err(ServiceError::Locked(Layer::Set))));
    assert_eq!(state(&s), before);
}

/// A side is no element: Delete refuses.
#[test]
fn delete_on_a_side_does_nothing() {
    let mut s = open();
    let before = state(&s);

    let result = s.edit_plan(PlanEdit::Remove {
        source: side(&s, 0),
    });

    assert!(result.is_err());
    assert_eq!(state(&s), before);
}
