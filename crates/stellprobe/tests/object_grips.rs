//! The grips that set the width and depth of a set object (LH-25).
#![allow(clippy::unwrap_used)]

use klappe_model::{FloorPlan, ObjectShape, Point, Project, SetObject, SetPlan};
use klappe_stellprobe::{handles, ObjectGrip, Role, Shape, Source};

fn set(rotation_deg: f64, shape: ObjectShape) -> SetPlan {
    SetPlan {
        objects: vec![SetObject {
            id: "o".into(),
            kind: "table".into(),
            label: "Tafel".into(),
            at: Point::new(10.0, 6.0),
            width_m: 4.0,
            depth_m: 2.0,
            rotation_deg,
            shape,
            height_m: None,
            base_m: None,
            mirrored: false,
            element: None,
        }],
        ..Default::default()
    }
}

/// The grips on the points of a free form, in order.
fn points(set: &SetPlan, selected: &Source) -> Vec<(Source, Point)> {
    grips(set, selected)
        .into_iter()
        .filter(|(source, _)| {
            matches!(
                source,
                Source::ObjectGrip {
                    grip: ObjectGrip::Point(_),
                    ..
                }
            )
        })
        .collect()
}

fn grips(set: &SetPlan, selected: &Source) -> Vec<(Source, Point)> {
    let project = Project::new("Test");

    handles(&project, set, &FloorPlan::new("a", "Plan A"), selected)
        .into_iter()
        .filter_map(|item| {
            assert_eq!(item.role, Role::Handle);

            match item.shape {
                Shape::Circle { center, .. } => Some((item.source.unwrap(), center)),
                _ => None,
            }
        })
        .collect()
}

fn grip(grip: ObjectGrip) -> Source {
    Source::ObjectGrip {
        object: "o".into(),
        grip,
    }
}

fn near(a: Point, b: Point) -> bool {
    (a.x_m - b.x_m).abs() < 1e-9 && (a.y_m - b.y_m).abs() < 1e-9
}

/// Unturned: the width grip 0.45 m outside the middle of the right edge, the depth grip as far
/// below the middle of the bottom edge.
#[test]
fn an_object_has_a_grip_outside_its_right_and_bottom_edge() {
    let found = grips(&set(0.0, ObjectShape::Rect), &Source::Object("o".into()));

    assert_eq!(found.len(), 2);
    assert_eq!(found[0].0, grip(ObjectGrip::Width));
    assert!(near(found[0].1, Point::new(12.45, 6.0)));
    assert_eq!(found[1].0, grip(ObjectGrip::Depth));
    assert!(near(found[1].1, Point::new(10.0, 7.45)));
}

/// Turned by 90 degrees clockwise, the right edge is below the centre and the bottom edge to
/// its left.
#[test]
fn the_grips_turn_with_the_object() {
    let found = grips(&set(90.0, ObjectShape::Rect), &Source::Object("o".into()));

    assert!(near(found[0].1, Point::new(10.0, 8.45)));
    assert!(near(found[1].1, Point::new(8.55, 6.0)));
}

/// An ellipse is sized by width and depth like a rectangle; the grips follow the grip itself
/// being selected, too.
#[test]
fn an_ellipse_has_grips_and_a_selected_grip_keeps_them() {
    let ellipse = set(0.0, ObjectShape::Ellipse);

    assert_eq!(grips(&ellipse, &Source::Object("o".into())).len(), 2);
    assert_eq!(grips(&ellipse, &grip(ObjectGrip::Width)).len(), 2);
    assert!(grips(&ellipse, &Source::Object("x".into())).is_empty());
}

fn triangle(rotation_deg: f64) -> SetPlan {
    let points = vec![
        Point::new(-1.0, -1.0),
        Point::new(1.0, -1.0),
        Point::new(0.0, 1.0),
    ];

    set(rotation_deg, ObjectShape::Polygon { points })
}

/// A free form has a grip on each point, not on the edges (LH-92). The outline's own box is
/// 2 m x 2 m and the object 4 m x 2 m, so the points lie stretched twice as wide.
#[test]
fn a_free_form_has_a_grip_on_each_point() {
    let found = points(&triangle(0.0), &Source::Object("o".into()));

    assert_eq!(found.len(), 3);
    assert_eq!(found[0].0, grip(ObjectGrip::Point(0)));
    assert!(near(found[0].1, Point::new(8.0, 5.0)));
    assert!(near(found[1].1, Point::new(12.0, 5.0)));
    assert_eq!(found[2].0, grip(ObjectGrip::Point(2)));
    assert!(near(found[2].1, Point::new(10.0, 7.0)));
}

/// Turned by 90 degrees clockwise, (x, y) around the centre (10, 6) becomes (-y, x); a grip
/// that is selected keeps them all.
#[test]
fn the_point_grips_turn_with_the_form() {
    let turned = triangle(90.0);
    let found = points(&turned, &grip(ObjectGrip::Point(1)));

    assert_eq!(found.len(), 3);
    assert_eq!(grips(&turned, &grip(ObjectGrip::Point(1))).len(), 6);
    assert!(near(found[0].1, Point::new(11.0, 4.0)));
    assert!(near(found[1].1, Point::new(11.0, 8.0)));
    assert!(near(found[2].1, Point::new(9.0, 6.0)));
}

/// A free form has a grip halfway along each side as well, the last on the closing side
/// (LH-92). The points are (8, 5), (12, 5) and (10, 7) on the plan.
#[test]
fn a_free_form_has_a_grip_halfway_along_each_side() {
    let found = grips(&triangle(0.0), &Source::Object("o".into()));
    let sides: Vec<_> = found
        .iter()
        .filter(|(source, _)| {
            matches!(
                source,
                Source::ObjectGrip {
                    grip: ObjectGrip::Side(_),
                    ..
                }
            )
        })
        .collect();

    assert_eq!(sides.len(), 3);
    assert_eq!(sides[0].0, grip(ObjectGrip::Side(0)));
    assert!(near(sides[0].1, Point::new(10.0, 5.0)));
    assert!(near(sides[1].1, Point::new(11.0, 6.0)));
    assert_eq!(sides[2].0, grip(ObjectGrip::Side(2)));
    assert!(near(sides[2].1, Point::new(9.0, 6.0)));
}

/// A selected side grip keeps them all; a rectangle has no side grips.
#[test]
fn a_selected_side_grip_keeps_the_grips() {
    let found = grips(&triangle(0.0), &grip(ObjectGrip::Side(1)));

    assert_eq!(found.len(), 6);
    assert_eq!(
        grips(&set(0.0, ObjectShape::Rect), &Source::Object("o".into())).len(),
        2
    );
}
