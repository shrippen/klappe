//! The room edit: a rectangle becomes four walls (LH-96).
#![allow(clippy::unwrap_used)]

use std::path::Path;

use klappe_services::{Field, PlanEdit, Point, ServiceError, Session};

fn open() -> Session {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe");

    Session::open(&path).unwrap()
}

fn p(x_m: f64, y_m: f64) -> Point {
    Point::new(x_m, y_m)
}

fn wall_count(s: &Session) -> usize {
    s.project().locations[0].set.walls.len()
}

#[test]
fn a_room_is_four_walls_that_close() {
    let mut s = open();
    let before = wall_count(&s);

    s.edit_plan(PlanEdit::AddRoom {
        from: p(2.0, 2.0),
        to: p(6.0, 5.0),
    })
    .unwrap();

    let walls = &s.project().locations[0].set.walls[before..];
    assert_eq!(walls.len(), 4);
    for (i, wall) in walls.iter().enumerate() {
        assert_eq!(wall.to, walls[(i + 1) % 4].from);
        assert_eq!(wall.thickness_m, 0.2);
    }
    assert_eq!(walls[0].from, p(2.0, 2.0));
    assert_eq!(walls[2].from, p(6.0, 5.0));
}

#[test]
fn a_room_is_one_undo_step() {
    let mut s = open();
    let before = wall_count(&s);

    s.edit_plan(PlanEdit::AddRoom {
        from: p(6.0, 5.0),
        to: p(2.0, 2.0),
    })
    .unwrap();
    s.undo();

    assert_eq!(wall_count(&s), before);
}

#[test]
fn a_flat_room_is_refused() {
    let mut s = open();
    let before = wall_count(&s);

    for to in [p(6.0, 2.0), p(2.0, 5.0), p(2.0, 2.0)] {
        let err = s
            .edit_plan(PlanEdit::AddRoom {
                from: p(2.0, 2.0),
                to,
            })
            .unwrap_err();

        assert!(matches!(
            err,
            ServiceError::BadValue {
                field: Field::Wall,
                ..
            }
        ));
    }
    assert_eq!(wall_count(&s), before);
    assert!(!s.can_undo());
}
