//! Notes on the plan (LH-16), on the example project's set: one note `n1`, "Vorhang
//! schließen", at (3, 11.5).
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_services::{Layer, PlanEdit, Point, Role, ServiceError, Session, Source, Visibility};

fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe")
}

fn open() -> Session {
    Session::open(&example()).unwrap()
}

fn p(x_m: f64, y_m: f64) -> Point {
    Point::new(x_m, y_m)
}

fn note(id: &str) -> Source {
    Source::Note(id.into())
}

fn notes(s: &Session) -> Vec<(String, Point, String)> {
    s.project().locations[0]
        .set
        .notes
        .iter()
        .map(|n| (n.id.clone(), n.at, n.text.clone()))
        .collect()
}

#[test]
fn the_example_shows_its_note_on_top() {
    let list = open().display_list();
    let last = list.items.last().unwrap();

    assert_eq!(last.role, Role::Note);
    assert_eq!(last.label.as_deref(), Some("Vorhang schließen"));
    assert_eq!(last.source, Some(note("n1")));
}

#[test]
fn adds_a_note_and_undoes() {
    let mut s = open();

    s.edit_plan(PlanEdit::AddNote {
        at: p(5.0, 9.0),
        text: "  Tür quietscht ".into(),
    })
    .unwrap();

    assert_eq!(
        notes(&s)[1],
        ("n2".into(), p(5.0, 9.0), "Tür quietscht".into())
    );
    assert!(s.is_dirty());

    s.undo();
    assert_eq!(notes(&s).len(), 1);
}

#[test]
fn a_freed_id_is_taken_again() {
    let mut s = open();
    let add = |s: &mut Session| {
        s.edit_plan(PlanEdit::AddNote {
            at: p(1.0, 1.0),
            text: "x".into(),
        })
        .unwrap();
    };

    add(&mut s);
    s.edit_plan(PlanEdit::Remove { source: note("n1") })
        .unwrap();
    add(&mut s);

    let ids: Vec<_> = notes(&s).into_iter().map(|n| n.0).collect();
    assert_eq!(ids, ["n2", "n1"]);
}

#[test]
fn an_empty_text_is_refused() {
    let mut s = open();
    let add = PlanEdit::AddNote {
        at: p(1.0, 1.0),
        text: "   ".into(),
    };
    let edit = PlanEdit::EditNote {
        id: "n1".into(),
        text: String::new(),
    };

    assert!(matches!(
        s.edit_plan(add),
        Err(ServiceError::BadValue { field: "note", .. })
    ));
    assert!(matches!(
        s.edit_plan(edit),
        Err(ServiceError::BadValue { field: "note", .. })
    ));
    assert!(!s.is_dirty());
}

#[test]
fn a_note_at_nan_is_refused() {
    let mut s = open();

    let edit = PlanEdit::AddNote {
        at: p(f64::NAN, 1.0),
        text: "x".into(),
    };

    assert!(matches!(
        s.edit_plan(edit),
        Err(ServiceError::BadValue { field: "point", .. })
    ));
}

#[test]
fn edits_the_text() {
    let mut s = open();

    s.edit_plan(PlanEdit::EditNote {
        id: "n1".into(),
        text: "Vorhang öffnen".into(),
    })
    .unwrap();

    assert_eq!(notes(&s)[0].2, "Vorhang öffnen");
    s.undo();
    assert_eq!(notes(&s)[0].2, "Vorhang schließen");
}

#[test]
fn editing_a_gone_note_is_an_error() {
    let mut s = open();

    let edit = PlanEdit::EditNote {
        id: "n9".into(),
        text: "x".into(),
    };

    assert!(matches!(s.edit_plan(edit), Err(ServiceError::Gone)));
}

#[test]
fn moves_shifts_and_removes_a_note() {
    let mut s = open();

    s.edit_plan(PlanEdit::Move {
        source: note("n1"),
        to: p(4.0, 10.0),
    })
    .unwrap();
    assert_eq!(notes(&s)[0].1, p(4.0, 10.0));

    // Snapping is on: 4.0 + 0.3 lands on 4.25.
    s.edit_plan(PlanEdit::Shift {
        source: note("n1"),
        dx_m: 0.3,
        dy_m: 0.0,
    })
    .unwrap();
    assert_eq!(notes(&s)[0].1, p(4.25, 10.0));

    s.edit_plan(PlanEdit::Remove { source: note("n1") })
        .unwrap();
    assert!(notes(&s).is_empty());
    assert!(matches!(
        s.edit_plan(PlanEdit::Remove { source: note("n1") }),
        Err(ServiceError::Gone)
    ));
}

/// A note has no facing: turning it changes nothing and takes no undo step.
#[test]
fn turning_a_note_does_nothing() {
    let mut s = open();

    s.edit_plan(PlanEdit::Spin {
        source: note("n1"),
        by_deg: 15.0,
    })
    .unwrap();

    assert!(!s.can_undo());
}

/// The marker has radius 0.15 m, the pick radius adds 0.15: 0.25 m away hits, 0.5 m does not.
#[test]
fn pick_finds_a_note() {
    let s = open();

    assert_eq!(s.pick(p(3.0, 11.75)), Some(note("n1")));
    assert_eq!(s.pick(p(3.0, 12.0)), None);
}

#[test]
fn the_notes_layer_hides_notes_only() {
    let mut s = open();

    s.set_layer(Layer::Notes, Visibility::Hidden);

    let list = s.display_list();
    assert!(list.items.iter().all(|i| i.role != Role::Note));
    assert!(list.items.iter().any(|i| i.role == Role::Wall));
    assert_eq!(s.pick(p(3.0, 11.5)), None);
}

#[test]
fn notes_survive_saving_and_loading() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("notes-save");
    let _ = std::fs::remove_dir_all(&dir);
    copy_dir(&example(), &dir);
    let mut s = Session::open(&dir).unwrap();

    s.edit_plan(PlanEdit::AddNote {
        at: p(5.0, 9.0),
        text: "Tür quietscht".into(),
    })
    .unwrap();
    s.save().unwrap();

    let again = Session::open(&dir).unwrap();
    assert_eq!(again.project(), s.project());
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
            continue;
        }
        std::fs::copy(entry.path(), target).unwrap();
    }
}
