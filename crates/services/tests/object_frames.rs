//! Review of #214, round 2: the way of an object in its own frame, free-form points of a
//! moving object, and a scene that changes its location.
#![allow(clippy::unwrap_used)]

use std::path::Path;

use klappe_services::{
    ObjectGrip, PlanEdit, Point, ProjectEdit, Role, Session, Shape, Snapping, Source,
};

fn open() -> Session {
    Session::open(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe"))
        .unwrap()
}

fn p(x_m: f64, y_m: f64) -> Point {
    Point::new(x_m, y_m)
}

fn near(a: Point, b: Point) -> bool {
    a.distance_m(b) < 1e-6
}

fn body(s: &Session, id: &str) -> Vec<Point> {
    let list = s.display_list();
    let item = list
        .items
        .iter()
        .find(|i| i.role == Role::Object && i.source == Some(Source::Object(id.into())))
        .unwrap();
    match &item.shape {
        Shape::Polygon { points } => points.clone(),
        _ => Vec::new(),
    }
}

fn new_object(s: &mut Session, scene: &str, at: Point) -> String {
    s.edit_plan(PlanEdit::AddObject {
        kind: "table".into(),
        at,
    })
    .unwrap();

    let location = &s.project().scene(scene).unwrap().location;

    s.project()
        .location(location)
        .unwrap()
        .set
        .objects
        .last()
        .unwrap()
        .id
        .clone()
}

/// A pillar that walks from (6, 4) at 0° to (16, 4) turned 90°: a width grip dragged by 1 m
/// at the last mark moves the middle by 0.5 m along the object's own x; each mark takes that
/// shift in its own frame, so the first moves along x, the turned one along y.
#[test]
fn a_resize_shifts_each_mark_in_the_frame_of_the_object() {
    let mut s = open();
    s.set_snapping(Snapping::Off);
    let id = "saeule-1";
    s.edit_plan(PlanEdit::AddObjectMark {
        object: id.into(),
        at: p(16.0, 4.0),
    })
    .unwrap();
    s.edit_plan(PlanEdit::Turn {
        source: Source::ObjectMark {
            object: id.into(),
            index: 1,
        },
        facing_deg: 90.0,
    })
    .unwrap();
    s.set_time(20.0);
    let grip = Source::ObjectGrip {
        object: id.into(),
        grip: ObjectGrip::Width,
    };
    let at = s
        .display_list_for(Some(&Source::Object(id.into())))
        .items
        .iter()
        .find(|i| i.source == Some(grip.clone()))
        .and_then(|i| match &i.shape {
            Shape::Circle { center, .. } => Some(*center),
            _ => None,
        })
        .unwrap();

    s.edit_plan(PlanEdit::Move {
        source: grip,
        to: p(at.x_m, at.y_m + 1.0),
    })
    .unwrap();

    let marks = &s.plan().unwrap().object_track(id).unwrap().marks;
    assert!(near(marks[0].at, p(6.5, 4.0)), "{:?}", marks[0].at);
    assert!(near(marks[1].at, p(16.0, 4.5)), "{:?}", marks[1].at);
}

/// Taking a point out of a free form that has walked: the other points stay where they are
/// drawn.
#[test]
fn removing_a_form_point_of_a_moving_object_keeps_the_others() {
    let mut s = open();
    s.edit_plan(PlanEdit::AddShape {
        points: vec![p(2.0, 2.0), p(4.0, 2.0), p(4.0, 3.0), p(2.0, 5.0 / 3.0)],
    })
    .unwrap();
    let id = s.project().locations[0]
        .set
        .objects
        .last()
        .unwrap()
        .id
        .clone();
    s.edit_plan(PlanEdit::AddObjectMark {
        object: id.clone(),
        at: p(8.0, 2.5),
    })
    .unwrap();
    s.set_time(30.0);
    let before = body(&s, &id);

    s.edit_plan(PlanEdit::Remove {
        source: Source::ObjectGrip {
            object: id.clone(),
            grip: ObjectGrip::Point(2),
        },
    })
    .unwrap();

    let after = body(&s, &id);
    assert_eq!(after.len(), 3);
    for point in &after {
        assert!(
            before.iter().any(|b| near(*b, *point)),
            "{point:?} {before:?}"
        );
    }
}

/// A scene that moves to another location must not bind its object ways to objects of the
/// new set that happen to share an id; one undo step brings them back.
#[test]
fn changing_the_location_of_a_scene_drops_its_object_ways() {
    let mut s = open();
    let id = new_object(&mut s, "szene-012", p(3.0, 3.0));
    s.edit_plan(PlanEdit::AddObjectMark {
        object: id.clone(),
        at: p(5.0, 3.0),
    })
    .unwrap();
    s.edit_project(ProjectEdit::AddLocation {
        name: "Halle".into(),
        width_m: 8.0,
        depth_m: 5.5,
        wall_height_m: 2.6,
    })
    .unwrap();
    let halle = s
        .project()
        .locations
        .iter()
        .find(|l| l.name == "Halle")
        .unwrap()
        .id
        .clone();
    s.edit_project(ProjectEdit::AddScene {
        number: "99".into(),
        heading: "x".into(),
        location: halle.clone(),
    })
    .unwrap();
    let other = s
        .project()
        .scenes
        .iter()
        .find(|c| c.number == "99")
        .unwrap()
        .id
        .clone();
    s.select_scene(&other).unwrap();
    assert_eq!(new_object(&mut s, &other, p(1.0, 1.0)), id);
    s.select_scene("szene-012").unwrap();
    let number = s.project().scene("szene-012").unwrap().number.clone();

    s.edit_project(ProjectEdit::EditScene {
        id: "szene-012".into(),
        number,
        heading: "x".into(),
        location: halle,
    })
    .unwrap();

    assert!(s.plan().unwrap().object_tracks.is_empty());
    s.undo();
    assert_eq!(s.plan().unwrap().object_tracks.len(), 1);
}
