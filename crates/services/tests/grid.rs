//! The grid of snapping (LH-14), on the example project (`testdata/beispiel.klappe`).
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_services::{Grid, PlanEdit, Point, Session, Source};

fn open() -> Session {
    let dir: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe");
    Session::open(&dir).unwrap()
}

fn p(x: f64, y: f64) -> Point {
    Point::new(x, y)
}

fn near(a: Point, b: Point) -> bool {
    a.distance_m(b) < 1e-9
}

/// Each grid rounds to its own spacing: (1.13, 2.62) on 5 cm -> (1.15, 2.60), 10 cm ->
/// (1.1, 2.6), 25 cm -> (1.25, 2.5), 50 cm -> (1.0, 2.5), 1 m -> (1.0, 3.0).
#[test]
fn the_grid_sets_the_spacing_of_snapping() {
    let mut s = open();
    assert_eq!(s.grid(), Grid::Cm25);
    let want = [
        (Grid::Cm5, p(1.15, 2.6)),
        (Grid::Cm10, p(1.1, 2.6)),
        (Grid::Cm25, p(1.25, 2.5)),
        (Grid::Cm50, p(1.0, 2.5)),
        (Grid::M1, p(1.0, 3.0)),
    ];

    for (grid, at) in want {
        s.set_grid(grid);
        assert!(near(s.snap(p(1.13, 2.62)), at), "{grid:?}");
    }
}

/// Mark 2 of Elisabeth stands at (10, 6.5); by (0.3, 0.1) it lands on the 10 cm grid at
/// (10.3, 6.6).
#[test]
fn a_shift_lands_on_the_chosen_grid() {
    let mut s = open();
    s.set_grid(Grid::Cm10);

    s.edit_plan(PlanEdit::Shift {
        source: Source::Mark {
            character: "elisabeth".into(),
            index: 1,
        },
        dx_m: 0.3,
        dy_m: 0.1,
    })
    .unwrap();

    let at = s.plan().unwrap().figures[0].marks[1].at;
    assert!(near(at, p(10.3, 6.6)), "{at:?}");
}

#[test]
fn grids_are_found_by_name() {
    for grid in Grid::ALL {
        assert_eq!(Grid::from_name(grid.name()), Some(grid));
    }
    assert_eq!(Grid::from_name("km"), None);
}
