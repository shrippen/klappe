//! The 180° axis (LH-40, LH-41) on the example project: the axis runs between Elisabeth and
//! Burgvogt; cameras A (12.1, 12.2, 12.5) and B (12.3, 12.4) are on opposite sides at
//! the start of their shots.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_services::Session;

fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe")
}

#[test]
fn a_jump_is_against_every_shot_on_the_other_side() {
    let s = Session::open(&example()).unwrap();

    let rows: Vec<_> = s
        .shots()
        .into_iter()
        .map(|r| (r.number, r.axis_jump, r.axis_against))
        .collect();

    // 12.1 is flagged too: the shot before it does not exist, 12.3 is not its neighbour.
    let expected = [
        ("12.1", "12.3, 12.4"),
        ("12.2", "12.3, 12.4"),
        ("12.3", "12.1, 12.2, 12.5"),
        ("12.4", "12.1, 12.2, 12.5"),
        ("12.5", "12.3, 12.4"),
    ];
    for (row, (number, against)) in rows.iter().zip(expected) {
        assert_eq!(
            (row.0.as_str(), row.1, row.2.as_str()),
            (number, true, against)
        );
    }
}
