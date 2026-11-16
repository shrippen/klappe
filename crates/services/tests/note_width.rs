//! The grip that sets the width of a note's card (LH-16), on the example project's set: one
//! note `n1`, "Vorhang schließen", at (3, 11.5), 2.4 m wide, one line of text.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_services::{
    Layer, Lock, PlanEdit, Point, Role, ServiceError, Session, Shape, Snapping, Source,
};

fn open() -> Session {
    let dir: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe");
    let mut s = Session::open(&dir).unwrap();
    s.set_snapping(Snapping::Off);

    s
}

fn p(x_m: f64, y_m: f64) -> Point {
    Point::new(x_m, y_m)
}

fn note() -> Source {
    Source::Note("n1".into())
}

fn grip() -> Source {
    Source::NoteWidth("n1".into())
}

fn width_m(s: &Session) -> f64 {
    s.project().locations[0].set.notes[0].width_m
}

fn drag(dx_m: f64) -> PlanEdit {
    PlanEdit::Shift {
        source: grip(),
        dx_m,
        dy_m: 0.0,
    }
}

/// Where the width grip is shown while the note is selected.
fn grip_at(s: &Session) -> Option<Point> {
    let list = s.display_list_for(Some(&note()));

    list.items
        .iter()
        .filter(|i| i.role == Role::Handle)
        .find_map(|i| match (&i.source, &i.shape) {
            (Some(source), Shape::Circle { center, .. }) if *source == grip() => Some(*center),
            _ => None,
        })
}

#[test]
fn a_selected_note_has_a_grip_in_the_middle_of_its_right_edge() {
    let s = open();

    assert_eq!(grip_at(&s), Some(p(5.4, 11.7)));
    assert!(s
        .display_list()
        .items
        .iter()
        .all(|i| i.role != Role::Handle));
}

#[test]
fn the_grip_is_picked_only_while_the_note_is_selected() {
    let s = open();

    assert_eq!(s.pick_for(p(5.4, 11.7), Some(&note())), Some(grip()));
    assert_eq!(s.pick_for(p(5.4, 11.7), None), Some(note()));
}

/// One edit, one undo step; the card's top left stays.
#[test]
fn dragging_the_grip_widens_the_note_and_undoes_in_one_step() {
    let mut s = open();

    s.edit_plan(drag(1.0)).unwrap();

    assert!((width_m(&s) - 3.4).abs() < 1e-9);
    assert_eq!(s.project().locations[0].set.notes[0].at, p(3.0, 11.5));
    s.undo();
    assert!((width_m(&s) - 2.4).abs() < 1e-9);
    assert!(!s.can_undo());
}

#[test]
fn the_grip_does_not_make_the_card_narrower_than_the_minimum() {
    let mut s = open();

    s.edit_plan(drag(-10.0)).unwrap();

    assert!((width_m(&s) - 0.6).abs() < 1e-9);
}

#[test]
fn a_locked_notes_layer_refuses_the_grip() {
    let mut s = open();
    s.set_lock(Layer::Notes, Lock::Locked);

    let result = s.edit_plan(drag(1.0));

    assert!(matches!(result, Err(ServiceError::Locked(Layer::Notes))));
    assert!((width_m(&s) - 2.4).abs() < 1e-9);
}

#[test]
fn removing_the_grip_removes_nothing() {
    let mut s = open();

    let result = s.edit_plan(PlanEdit::Remove { source: grip() });

    assert!(result.is_err());
    assert_eq!(s.project().locations[0].set.notes.len(), 1);
}
