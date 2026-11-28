//! The measuring tool snaps to the corners and edges of walls and objects (LH-14, Q1-15).
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_services::{Grid, PlanEdit, Point, Role, Session, Shape, Snapping};

fn p(x: f64, y: f64) -> Point {
    Point::new(x, y)
}

fn near(a: Point, b: Point) -> bool {
    a.distance_m(b) < 1e-9
}

/// The example project with a wall from (1, 1) to (4, 1) on a 1 m grid, and the wall's
/// thickness.
fn walled() -> (Session, f64) {
    let dir: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe");
    let mut s = Session::open(&dir).unwrap();
    s.set_grid(Grid::M1);
    s.edit_plan(PlanEdit::AddWall {
        from: p(1.0, 1.0),
        to: p(4.0, 1.0),
    })
    .unwrap();

    let thickness = s
        .display_list()
        .items
        .iter()
        .find_map(|i| match (&i.role, &i.shape) {
            (Role::Wall, Shape::Line { from, width_m, .. }) if near(*from, p(1.0, 1.0)) => {
                Some(*width_m)
            }
            _ => None,
        })
        .unwrap();

    (s, thickness)
}

/// Near a side of a wall the point lands on the side, not on the grid.
#[test]
fn the_measure_point_snaps_to_a_wall_edge() {
    let (s, t) = walled();
    let side = 1.0 + t / 2.0;

    assert!(near(s.snap_measure(p(2.4, side + 0.04)), p(2.4, side)));
}

/// Near a corner of a wall the point lands on the corner.
#[test]
fn the_measure_point_snaps_to_a_wall_corner() {
    let (s, t) = walled();

    assert!(near(
        s.snap_measure(p(4.03, 1.0 + t / 2.0 + 0.03)),
        p(4.0, 1.0 + t / 2.0)
    ));
}

/// Away from every wall and object, and with snapping off, the old rules hold.
#[test]
fn the_measure_point_falls_back_to_the_grid() {
    let (mut s, _) = walled();

    assert!(near(s.snap_measure(p(6.3, 9.2)), p(6.0, 9.0)));

    s.set_snapping(Snapping::Off);
    assert!(near(s.snap_measure(p(2.4, 1.14)), p(2.4, 1.14)));
}
