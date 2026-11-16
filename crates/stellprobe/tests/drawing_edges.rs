//! The grips that make a point on an edge of an area (LH-19).
#![allow(clippy::unwrap_used)]

use klappe_model::{
    Dash, Drawing, DrawingShape, FigureColor, Fill, FloorPlan, Point, Project, SetPlan,
};
use klappe_stellprobe::{handles, Item, Role, Shape, Source};

fn plan() -> FloorPlan {
    let drawn = |id: &str, shape| Drawing {
        id: id.into(),
        shape,
        color: FigureColor::Red,
        dash: Dash::Solid,
        fill: Fill::Outline,
        rotation_deg: 0.0,
    };
    let p = Point::new;
    let mut plan = FloorPlan::new("a", "Plan A");
    plan.drawings = vec![
        drawn(
            "area",
            DrawingShape::Area {
                points: vec![p(0.0, 0.0), p(4.0, 0.0), p(4.0, 2.0)],
            },
        ),
        drawn(
            "box",
            DrawingShape::Rect {
                from: p(0.0, 0.0),
                to: p(4.0, 2.0),
            },
        ),
    ];

    plan
}

fn grips(selected: &Source) -> Vec<Item> {
    handles(
        &Project::new("Test"),
        &SetPlan::default(),
        &plan(),
        selected,
    )
}

fn edges(items: &[Item]) -> Vec<(Source, Point)> {
    items
        .iter()
        .filter_map(|i| match (&i.source, &i.shape) {
            (Some(s @ Source::DrawingEdge { .. }), Shape::Circle { center, .. }) => {
                Some((s.clone(), *center))
            }
            _ => None,
        })
        .collect()
}

#[test]
fn a_selected_area_has_a_grip_in_the_middle_of_each_edge() {
    let items = grips(&Source::Drawing("area".into()));
    let edge = |index| Source::DrawingEdge {
        drawing: "area".into(),
        index,
    };

    assert_eq!(
        edges(&items),
        [
            (edge(0), Point::new(2.0, 0.0)),
            (edge(1), Point::new(4.0, 1.0)),
            (edge(2), Point::new(2.0, 1.0)),
        ]
    );
    assert!(items.iter().all(|i| i.role == Role::DrawingHandle));
}

/// Smaller than the points', so that it never hides one.
#[test]
fn an_edge_grip_is_smaller_than_a_point_grip() {
    let items = grips(&Source::Drawing("area".into()));
    let radius = |want_edge: bool| match items
        .iter()
        .find(|i| matches!(i.source, Some(Source::DrawingEdge { .. })) == want_edge)
        .map(|i| &i.shape)
    {
        Some(Shape::Circle { radius_m, .. }) => *radius_m,
        other => panic!("{other:?}"),
    };

    assert!(radius(true) < radius(false));
}

#[test]
fn the_grips_of_a_point_selected_keep_showing() {
    let point = Source::DrawingPoint {
        drawing: "area".into(),
        index: 1,
    };

    assert_eq!(edges(&grips(&point)).len(), 3);
}

#[test]
fn a_box_and_an_unselected_area_have_none() {
    assert!(edges(&grips(&Source::Drawing("box".into()))).is_empty());
    assert!(edges(&grips(&Source::Wall(0))).is_empty());
}
