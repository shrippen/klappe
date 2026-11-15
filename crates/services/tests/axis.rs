//! The 180° axis (LH-40, LH-41) on the example project: the axis runs between Elisabeth and
//! Burgvogt; cameras A (12.1, 12.2, 12.5) and B (12.3, 12.4) are on opposite sides at
//! the start of their shots.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_services::{Layer, Lock, ServiceError, Session};

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

#[test]
fn the_axis_pair_is_chosen_and_undone() {
    let mut s = Session::open(&example()).unwrap();
    let before = s.axis_choice().unwrap();
    assert!(before.own);

    s.set_axis(Some(("burgvogt", "elisabeth"))).unwrap();

    let after = s.axis_choice().unwrap();
    assert_eq!(
        (after.a.as_str(), after.b.as_str()),
        ("burgvogt", "elisabeth")
    );

    s.undo();

    assert_eq!(s.axis_choice().unwrap(), before);
}

#[test]
fn swapping_the_pair_keeps_the_split() {
    let mut s = Session::open(&example()).unwrap();
    let sides = |s: &Session| -> Vec<bool> { s.shots().iter().map(|r| r.axis_jump).collect() };

    s.set_axis(Some(("burgvogt", "elisabeth"))).unwrap();

    // Both cameras swap sides, so the split stays: every shot still crosses another.
    assert_eq!(sides(&s), vec![true; 5]);
}

#[test]
fn a_bad_pair_is_refused() {
    let mut s = Session::open(&example()).unwrap();

    // A figure against itself, or against one without a track.
    let same = s.set_axis(Some(("elisabeth", "elisabeth")));
    assert!(matches!(same, Err(ServiceError::BadValue { .. })));

    let gone = s.set_axis(Some(("elisabeth", "nobody")));
    assert!(matches!(gone, Err(ServiceError::BadValue { .. })));
}

#[test]
fn no_choice_takes_the_first_two_figures() {
    let mut s = Session::open(&example()).unwrap();

    s.set_axis(None).unwrap();

    let axis = s.axis_choice().unwrap();
    assert!(!axis.own);
    assert_eq!(
        (axis.a.as_str(), axis.b.as_str()),
        ("elisabeth", "burgvogt")
    );
}

#[test]
fn a_locked_figure_layer_refuses_the_choice() {
    let mut s = Session::open(&example()).unwrap();
    s.set_lock(Layer::Figures, Lock::Locked);

    let result = s.set_axis(Some(("burgvogt", "elisabeth")));

    assert!(result.is_err());
}
