//! The grips that insert a point are shown only on a side long enough to take them.
#![allow(clippy::unwrap_used)]

use klappe_model::{
    Dash, Drawing, DrawingShape, FigureColor, Fill, FloorPlan, ObjectShape, Point, Project,
    SetObject, SetPlan,
};
use klappe_stellprobe::{handles, ObjectGrip, Source};

/// A free form of 4 m x 2 m: sides of 4 m, 2 m and 0.4 m (the last one from point 2 to 0).
fn form() -> SetPlan {
    let p = Point::new;

    SetPlan {
        objects: vec![SetObject {
            id: "o".into(),
            kind: "form".into(),
            label: "Form".into(),
            at: p(10.0, 6.0),
            width_m: 4.0,
            depth_m: 2.0,
            rotation_deg: 0.0,
            shape: ObjectShape::Polygon {
                points: vec![
                    p(0.0, 0.0),
                    p(4.0, 0.0),
                    p(4.0, 2.0),
                    p(3.6, 2.0),
                    p(0.0, 2.0),
                ],
            },
            height_m: None,
            base_m: None,
            mirrored: false,
        }],
        ..Default::default()
    }
}

fn sides(items: &[klappe_stellprobe::Item]) -> Vec<usize> {
    items
        .iter()
        .filter_map(|i| match i.source {
            Some(Source::ObjectGrip {
                grip: ObjectGrip::Side(n),
                ..
            }) => Some(n),
            Some(Source::DrawingEdge { index, .. }) => Some(index),
            _ => None,
        })
        .collect()
}

#[test]
fn a_form_side_grip_needs_a_side_longer_than_three_grips() {
    let items = handles(
        &Project::new("Test"),
        &form(),
        &FloorPlan::new("a", "Plan A"),
        &Source::Object("o".into()),
    );

    // Side 2 (0.4 m, from (4, 2) to (3.6, 2)) is too short.
    assert_eq!(sides(&items), vec![0, 1, 3, 4]);
}

#[test]
fn a_drawing_edge_grip_needs_an_edge_longer_than_three_grips() {
    let p = Point::new;
    let mut plan = FloorPlan::new("a", "Plan A");
    plan.drawings = vec![Drawing {
        id: "area".into(),
        shape: DrawingShape::Area {
            points: vec![p(0.0, 0.0), p(4.0, 0.0), p(4.0, 0.4)],
        },
        color: FigureColor::Red,
        dash: Dash::Solid,
        fill: Fill::Outline,
        rotation_deg: 0.0,
    }];
    let items = handles(
        &Project::new("Test"),
        &SetPlan::default(),
        &plan,
        &Source::Drawing("area".into()),
    );

    // Edge 1 is 0.4 m long; edge 0 is 4 m, edge 2 is about 4 m.
    assert_eq!(sides(&items), vec![0, 2]);
}
