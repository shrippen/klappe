//! The grips that set the width of a door or window (LH-10).
#![allow(clippy::unwrap_used)]

use klappe_model::{FloorPlan, Opening, OpeningKind, Point, Project, SetPlan, Wall};
use klappe_stellprobe::{handles, OpeningEnd, Role, Shape, Source};

/// One wall from (0, 2) to (10, 2) with a door 1 m wide at 4 m.
fn set() -> SetPlan {
    let mut wall = Wall::new(Point::new(0.0, 2.0), Point::new(10.0, 2.0), 0.3);
    wall.openings
        .push(Opening::new(OpeningKind::Door, 4.0, 1.0));

    SetPlan {
        walls: vec![wall],
        ..Default::default()
    }
}

fn grips(selected: &Source) -> Vec<(Role, Option<Source>, Point)> {
    let list = handles(
        &Project::new("Test"),
        &set(),
        &FloorPlan::new("a", "Plan A"),
        selected,
    );

    list.into_iter()
        .filter_map(|i| match i.shape {
            Shape::Circle { center, .. } => Some((i.role, i.source, center)),
            _ => None,
        })
        .collect()
}

fn edge(end: OpeningEnd) -> Option<Source> {
    Some(Source::OpeningEdge {
        wall: 0,
        index: 0,
        end,
    })
}

/// A selected opening has a grip on each end, on the wall's middle line.
#[test]
fn a_selected_opening_has_a_grip_on_each_end_of_the_wall_line() {
    let door = Source::Opening { wall: 0, index: 0 };

    assert_eq!(
        grips(&door),
        vec![
            (
                Role::OpeningHandle,
                edge(OpeningEnd::Start),
                Point::new(3.5, 2.0)
            ),
            (
                Role::OpeningHandle,
                edge(OpeningEnd::End),
                Point::new(4.5, 2.0)
            ),
        ]
    );
}

/// A grip stays while it is dragged; other elements, and openings that are gone, have none.
#[test]
fn the_grips_stay_while_dragged_and_others_have_none() {
    let grip = Source::OpeningEdge {
        wall: 0,
        index: 0,
        end: OpeningEnd::End,
    };

    assert_eq!(grips(&grip).len(), 2);
    assert!(grips(&Source::Wall(0)).is_empty());
    assert!(grips(&Source::Opening { wall: 0, index: 1 }).is_empty());
    assert!(grips(&Source::Opening { wall: 3, index: 0 }).is_empty());
}
