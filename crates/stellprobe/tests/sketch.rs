//! The sketch pen's strokes in the display list (LH-120).
#![allow(clippy::unwrap_used)]

use klappe_model::{FloorPlan, Point, Project, SetPlan, Sketch};
use klappe_stellprobe::{look, render, Hue, Role, Shape};

#[test]
fn a_stroke_is_the_last_item_a_polyline_that_belongs_to_nothing() {
    let mut plan = FloorPlan::new("a", "Plan A");
    let points = vec![
        Point::new(1.0, 1.0),
        Point::new(2.0, 1.5),
        Point::new(3.0, 1.0),
    ];
    plan.sketches.push(Sketch {
        points: points.clone(),
    });
    plan.sketches.push(Sketch {
        points: vec![Point::new(5.0, 5.0), Point::new(6.0, 6.0)],
    });
    let set = SetPlan::default();

    let list = render(&Project::new("F"), &set, &plan, 0.0, &|_| None);

    let strokes: Vec<_> = list
        .items
        .iter()
        .filter(|i| i.role == Role::Sketch)
        .collect();
    assert_eq!(strokes.len(), 2);
    assert_eq!(list.items.last().unwrap().role, Role::Sketch);
    assert_eq!(strokes[0].shape, Shape::Polyline { points });
    assert!(strokes.iter().all(|s| s.source.is_none()));
}

#[test]
fn the_pen_is_red_and_solid() {
    let pen = look(Role::Sketch);

    assert_eq!(pen.hue, Hue::Color(klappe_model::FigureColor::Red));
}
