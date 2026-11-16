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
        }],
        ..Default::default()
    }
}

fn grips(set: &SetPlan, selected: &Source) -> Vec<(Source, Point)> {
    let project = Project::new("Test");

    handles(&project, set, &FloorPlan::new("a", "Plan A"), selected)
        .into_iter()
        .filter_map(|item| {
            assert_eq!(item.role, Role::ObjectHandle);

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

/// A free form is reshaped by its points, which are a separate matter.
#[test]
fn a_free_form_has_no_size_grips() {
    let points = vec![
        Point::new(-1.0, -1.0),
        Point::new(1.0, -1.0),
        Point::new(0.0, 1.0),
    ];
    let form = set(0.0, ObjectShape::Polygon { points });

    assert!(grips(&form, &Source::Object("o".into())).is_empty());
}
