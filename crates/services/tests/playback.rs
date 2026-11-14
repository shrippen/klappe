//! Playback and the jumps between marks on the example project (scene 12, 20 s).
#![allow(clippy::unwrap_used)]

use std::path::Path;

use klappe_services::{Looping, Playing, Session, Step};

fn open() -> Session {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe");
    Session::open(&path).unwrap()
}

#[test]
fn marks_hold_every_mark_and_shot_edge_once() {
    // Figures 0, 6, 14 and 0, 9; cameras 0, 6; shots from 0 to 20.
    assert_eq!(open().marks_s(), vec![0.0, 6.0, 9.0, 14.0, 20.0]);
}

#[test]
fn a_jump_on_goes_to_the_next_mark() {
    let mut s = open();

    s.set_time(6.0);
    s.step_mark(Step::On);
    assert_eq!(s.time_s(), 9.0);

    s.set_time(7.5);
    s.step_mark(Step::On);
    assert_eq!(s.time_s(), 9.0);
}

#[test]
fn a_jump_back_goes_to_the_mark_before() {
    let mut s = open();

    s.set_time(9.0);
    s.step_mark(Step::Back);
    assert_eq!(s.time_s(), 6.0);

    s.set_time(7.5);
    s.step_mark(Step::Back);
    assert_eq!(s.time_s(), 6.0);
}

#[test]
fn the_jumps_stop_at_the_ends() {
    let mut s = open();

    s.set_time(0.0);
    s.step_mark(Step::Back);
    assert_eq!(s.time_s(), 0.0);

    s.set_time(20.0);
    s.step_mark(Step::On);
    assert_eq!(s.time_s(), 20.0);
}

#[test]
fn playing_runs_in_real_time_times_the_speed() {
    let mut s = open();

    assert_eq!(s.advance(1.0, 1.0, Looping::Off), Playing::Running);
    assert_eq!(s.time_s(), 1.0);

    assert_eq!(s.advance(1.0, 0.5, Looping::Off), Playing::Running);
    assert_eq!(s.time_s(), 1.5);
}

#[test]
fn playing_ends_at_the_end_of_the_scene() {
    let mut s = open();
    s.set_time(19.5);

    assert_eq!(s.advance(1.0, 1.0, Looping::Off), Playing::Ended);
    assert_eq!(s.time_s(), 20.0);
}

#[test]
fn a_loop_wraps_at_the_end_of_the_shot() {
    let mut s = open();
    s.select_shot("s12-3").unwrap();
    assert_eq!(s.time_s(), 6.0);

    // Shot 12.3 runs 6 to 9 s: 8.5 + 1 s wraps to 6.5.
    s.set_time(8.5);
    assert_eq!(s.advance(1.0, 1.0, Looping::Shot), Playing::Running);
    assert_eq!(s.time_s(), 6.5);
}

#[test]
fn a_loop_enters_the_shot_from_outside() {
    let mut s = open();
    s.select_shot("s12-3").unwrap();
    s.set_time(15.0);

    s.advance(0.5, 1.0, Looping::Shot);
    assert_eq!(s.time_s(), 6.0);
}

#[test]
fn a_loop_without_a_shot_runs_the_whole_scene() {
    let mut s = open();
    s.set_time(19.5);

    assert_eq!(s.advance(1.0, 1.0, Looping::Shot), Playing::Running);
    assert_eq!(s.time_s(), 0.5);
}

#[test]
fn no_time_or_no_speed_changes_nothing() {
    let mut s = open();
    s.set_time(3.0);

    for (dt_s, speed) in [
        (0.0, 1.0),
        (-1.0, 1.0),
        (1.0, 0.0),
        (f64::NAN, 1.0),
        (1.0, f64::NAN),
    ] {
        assert_eq!(s.advance(dt_s, speed, Looping::Off), Playing::Running);
        assert_eq!(s.time_s(), 3.0);
    }
}
