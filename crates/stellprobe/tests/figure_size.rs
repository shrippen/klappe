//! The size of a figure's body on the plan (LH-25).
#![allow(clippy::unwrap_used)]

use klappe_model::{
    Arrival, Character, Departure, FigureColor, FigureTrack, FloorPlan, Mark, Point, Pose, Project,
    SetPlan,
};
use klappe_stellprobe::{hit, render, DisplayList, Item, Role, Shape, FIGURE_RADIUS_M};

const DOT_RADIUS_M: f64 = 0.17;
const GROUP_RADIUS_M: f64 = 0.6;
const HORSE_M: f64 = 4.0;

/// One purple figure at (2, 6), alone on an empty set.
fn list(size_m: Option<f64>, group: Option<u32>) -> DisplayList {
    let mut project = Project::new("Test");
    project.characters = vec![Character {
        id: "e".into(),
        name: "Elisabeth".into(),
        short: "E".into(),
        color: FigureColor::Purple,
        group,
        size_m,
        aliases: Vec::new(),
    }];
    let mut plan = FloorPlan::new("a", "Plan A");
    plan.figures = vec![FigureTrack {
        character: "e".into(),
        marks: vec![Mark {
            at_s: 0.0,
            at: Point::new(2.0, 6.0),
            facing_deg: 0.0,
            pose: Pose::Standing,
            hold_s: 0.0,
            via: Vec::new(),
            curve: Default::default(),
            arrival: Arrival::Walk,
            departure: Departure::Stay,
        }],
    }];
    let set = SetPlan {
        width_m: 20.0,
        depth_m: 13.0,
        walls: Vec::new(),
        objects: Vec::new(),
        background: None,
        notes: Vec::new(),
        ..Default::default()
    };

    render(&project, &set, &plan, 0.0, &|_| None)
}

fn body(list: &DisplayList) -> Option<&Item> {
    list.items.iter().find(|i| {
        i.role == Role::Figure(FigureColor::Purple) && matches!(i.shape, Shape::Circle { .. })
    })
}

fn radius(list: &DisplayList) -> f64 {
    match body(list).unwrap().shape {
        Shape::Circle { radius_m, .. } => radius_m,
        _ => unreachable!(),
    }
}

fn dot_radii(list: &DisplayList) -> Vec<f64> {
    list.items
        .iter()
        .filter(|i| i.role == Role::GroupDot)
        .filter_map(|i| match i.shape {
            Shape::Circle { radius_m, .. } => Some(radius_m),
            _ => None,
        })
        .collect()
}

#[test]
fn a_figure_without_size_is_the_standard() {
    assert_eq!(radius(&list(None, None)), FIGURE_RADIUS_M);
}

/// The size is the diameter of the body.
#[test]
fn a_figure_is_drawn_in_its_size() {
    assert_eq!(radius(&list(Some(HORSE_M), None)), HORSE_M / 2.0);
}

/// A group stays wider than a single figure of the same size, by the same ratio.
#[test]
fn a_group_stays_larger_by_the_same_ratio() {
    let ratio = GROUP_RADIUS_M / FIGURE_RADIUS_M;

    assert_eq!(radius(&list(None, Some(12))), GROUP_RADIUS_M);
    assert_eq!(
        radius(&list(Some(HORSE_M), Some(12))),
        HORSE_M / 2.0 * ratio
    );
}

/// The dots of a group grow with its body.
#[test]
fn the_dots_of_a_group_grow_with_it() {
    let standard = dot_radii(&list(None, Some(12)));
    let large = dot_radii(&list(Some(HORSE_M), Some(12)));

    assert_eq!(standard, vec![DOT_RADIUS_M; 3]);
    assert!(large.iter().all(|r| *r > DOT_RADIUS_M));
}

/// The hit test follows the drawn body.
#[test]
fn a_large_figure_is_hit_across_its_body() {
    let standard = list(None, None);
    let large = list(Some(HORSE_M), None);
    let inside = Point::new(2.0 + 1.5, 6.0);

    assert!(!hit(body(&standard).unwrap(), inside, 0.0));
    assert!(hit(body(&large).unwrap(), inside, 0.0));
}
