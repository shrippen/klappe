//! The undo tooltip names the step (LH-103), on the example project (note `n1`).
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_services::{PlanEdit, Point, Session, Source};

fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe")
}

fn remove_note(s: &mut Session) {
    s.edit_plan(PlanEdit::Remove {
        source: Source::Note("n1".into()),
    })
    .unwrap();
}

#[test]
fn a_fresh_session_has_no_step() {
    let s = Session::open(&example()).unwrap();

    assert_eq!(s.undo_step(), None);
    assert_eq!(s.redo_step(), None);
}

#[test]
fn undo_names_the_last_edit_and_redo_keeps_the_name() {
    let mut s = Session::open(&example()).unwrap();
    remove_note(&mut s);

    assert_eq!(s.undo_step(), Some("Remove"));
    assert_eq!(s.redo_step(), None);

    s.undo();

    assert_eq!(s.undo_step(), None);
    assert_eq!(s.redo_step(), Some("Remove"));

    s.redo();

    assert_eq!(s.undo_step(), Some("Remove"));
}

#[test]
fn the_next_step_is_the_one_before_the_last() {
    let mut s = Session::open(&example()).unwrap();
    remove_note(&mut s);
    s.edit_plan(PlanEdit::AddObject {
        kind: "table".into(),
        at: Point::new(2.0, 2.0),
    })
    .unwrap();

    assert_eq!(s.undo_step(), Some("AddObject"));
    s.undo();
    assert_eq!(s.undo_step(), Some("Remove"));
    assert_eq!(s.redo_step(), Some("AddObject"));
}

#[test]
fn shot_and_plan_steps_are_named_too() {
    let mut s = Session::open(&example()).unwrap();

    s.add_shot(None).unwrap();

    assert_eq!(s.undo_step(), Some("AddShot"));
}
