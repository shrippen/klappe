//! Set objects that figures move (LH-24): where they are, and what the plan draws.

use super::*;
use crate::prop::object_at;
use klappe_model::{ObjectMark, ObjectTrack};

const TRACK_COLOR: FigureColor = FigureColor::Cyan;

fn object_mark(at_s: f64, x_m: f64, carried_by: Option<&str>) -> ObjectMark {
    ObjectMark {
        at_s,
        at: Point::new(x_m, 3.0),
        rotation_deg: 0.0,
        hold_s: 0.0,
        carried_by: carried_by.map(str::to_owned),
    }
}

/// A table at (5, 3) in a set with one object, and a plan where figure `e` walks from (2, 6)
/// at 0 s to (7, 6) at 8 s.
fn with_table(marks: Vec<ObjectMark>) -> (SetPlan, FloorPlan) {
    let mut set = set();
    set.objects = vec![SetObject {
        id: "t".into(),
        kind: "table".into(),
        label: "Tisch".into(),
        at: Point::new(5.0, 3.0),
        width_m: 1.0,
        depth_m: 1.0,
        rotation_deg: 0.0,
        shape: ObjectShape::Rect,
        height_m: Some(0.8),
        base_m: None,
        mirrored: false,
    }];
    let mut plan = plan();
    plan.object_tracks = vec![ObjectTrack {
        object: "t".into(),
        marks,
    }];

    (set, plan)
}

fn centre(list: &DisplayList) -> Point {
    let body = list
        .items
        .iter()
        .find(|i| i.role == Role::Object && i.source == Some(Source::Object("t".into())))
        .unwrap();
    let Shape::Polygon { points } = &body.shape else {
        return Point::new(f64::NAN, f64::NAN);
    };
    let n = points.len() as f64;

    Point::new(
        points.iter().map(|p| p.x_m).sum::<f64>() / n,
        points.iter().map(|p| p.y_m).sum::<f64>() / n,
    )
}

fn near(a: Point, b: Point) -> bool {
    a.distance_m(b) < 1e-9
}

#[test]
fn an_object_without_marks_stays_where_the_set_puts_it() {
    let (set, mut plan) = with_table(Vec::new());
    plan.object_tracks.clear();

    let list = render(&project(), &set, &plan, 4.0, &|_| None);

    assert!(near(centre(&list), Point::new(5.0, 3.0)));
    assert_eq!(count(&list, Role::Mark(TRACK_COLOR)), 0);
}

#[test]
fn an_object_walks_its_marks() {
    let (set, plan) = with_table(vec![
        object_mark(0.0, 5.0, None),
        object_mark(10.0, 15.0, None),
    ]);
    let at = |t_s| centre(&render(&project(), &set, &plan, t_s, &|_| None));

    assert!(near(at(0.0), Point::new(5.0, 3.0)));
    assert!(near(at(5.0), Point::new(10.0, 3.0)));
    assert!(near(at(12.0), Point::new(15.0, 3.0)));
}

/// The marks are circles, the way a line, both owned by the object.
#[test]
fn the_marks_and_the_way_are_drawn() {
    let (set, plan) = with_table(vec![
        object_mark(0.0, 5.0, None),
        object_mark(10.0, 15.0, None),
    ]);
    let list = render(&project(), &set, &plan, 0.0, &|_| None);

    assert_eq!(count(&list, Role::Mark(TRACK_COLOR)), 2);
    assert_eq!(count(&list, Role::FigurePath(TRACK_COLOR)), 1);
    let numbered: Vec<_> = list
        .items
        .iter()
        .filter(|i| i.role == Role::Mark(TRACK_COLOR))
        .map(|i| (i.label.clone().unwrap(), i.source.clone().unwrap()))
        .collect();
    assert_eq!(
        numbered[1],
        (
            "2".to_owned(),
            Source::ObjectMark {
                object: "t".into(),
                index: 1
            }
        )
    );
}

/// From a mark that names a carrier the object keeps its offset to the figure until the next
/// mark; the figure's own path is its way, so none is drawn to that mark.
#[test]
fn a_carried_object_keeps_its_offset_to_the_figure() {
    let (set, plan) = with_table(vec![
        object_mark(0.0, 5.0, Some("e")),
        object_mark(20.0, 30.0, None),
    ]);
    let walker = plan.figure("e").unwrap();
    let list = |t_s| render(&project(), &set, &plan, t_s, &|_| None);

    // The figure stands at (2, 6) and the object at (5, 3): offset (3, -3).
    assert!(near(centre(&list(0.0)), Point::new(5.0, 3.0)));
    let figure = crate::motion::figure_at(walker, 8.0).unwrap().at;
    assert!(near(
        centre(&list(8.0)),
        Point::new(figure.x_m + 3.0, figure.y_m - 3.0)
    ));
    assert_eq!(count(&list(0.0), Role::FigurePath(TRACK_COLOR)), 0);

    // At the next mark it is set down.
    assert!(near(centre(&list(20.0)), Point::new(30.0, 3.0)));
}

/// A carrier that is not in the scene (or not there yet) leaves the marks to say.
#[test]
fn a_carrier_the_plan_does_not_have_changes_nothing() {
    let (_, plan) = with_table(vec![
        object_mark(0.0, 5.0, Some("nobody")),
        object_mark(10.0, 15.0, None),
    ]);
    let track = plan.object_track("t").unwrap();

    let place = object_at(&plan, track, 5.0).unwrap();

    assert!(near(place.at, Point::new(10.0, 3.0)));
}

#[test]
fn a_track_of_an_object_the_set_lacks_draws_nothing() {
    let (mut set, plan) = with_table(vec![object_mark(0.0, 5.0, None)]);
    set.objects.clear();

    let list = render(&project(), &set, &plan, 0.0, &|_| None);

    assert_eq!(count(&list, Role::Mark(TRACK_COLOR)), 0);
}
