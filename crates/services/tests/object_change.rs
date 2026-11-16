//! Changing an object (LH-114 kind, LH-115 mirror, LH-116 size of its entry) and labels bound
//! to elements (LH-118), on the example project: the table `tisch` (6 x 1.2 m, named "Tafel", at
//! (10, 6.5)), Elisabeth's marks at (3, 6.5), (10, 6.5) and (16, 3), the note `n1` at
//! (3, 11.5).
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_model::SetObject;
use klappe_services::{
    Field, Layer, Lock, ObjectChange, ObjectShape, PlanEdit, Point, ServiceError, Session, Source,
};

fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe")
}

fn open() -> Session {
    Session::open(&example()).unwrap()
}

fn p(x_m: f64, y_m: f64) -> Point {
    Point::new(x_m, y_m)
}

fn object(s: &Session, id: &str) -> SetObject {
    let objects = &s.project().locations[0].set.objects;

    objects.iter().find(|o| o.id == id).unwrap().clone()
}

fn note_at(s: &Session, id: &str) -> Point {
    let notes = &s.project().locations[0].set.notes;

    notes.iter().find(|n| n.id == id).unwrap().at
}

fn bound(s: &Session, id: &str) -> Option<String> {
    let notes = &s.project().locations[0].set.notes;

    notes.iter().find(|n| n.id == id).unwrap().bound.clone()
}

fn obj(id: &str) -> Source {
    Source::Object(id.into())
}

fn change(source: Source, change: ObjectChange) -> PlanEdit {
    PlanEdit::SetObject { source, change }
}

/// A new table, turned 90 degrees, as `o1`.
fn placed_table(s: &mut Session) -> String {
    s.edit_plan(PlanEdit::AddObject {
        kind: "table".into(),
        at: p(2.0, 3.0),
    })
    .unwrap();
    let id = s.project().locations[0]
        .set
        .objects
        .last()
        .unwrap()
        .id
        .clone();
    s.edit_plan(PlanEdit::Turn {
        source: obj(&id),
        facing_deg: 90.0,
    })
    .unwrap();

    id
}

fn bad_kind(result: Result<(), ServiceError>) -> bool {
    matches!(
        result,
        Err(ServiceError::BadValue {
            field: Field::Kind,
            ..
        })
    )
}

/// A table as the library made it takes the new kind's size, height and name; place, turn and
/// layer stay; undo gives the table back.
#[test]
fn a_new_kind_takes_the_entry_and_keeps_place_and_turn() {
    let mut s = open();
    let id = placed_table(&mut s);
    let before = object(&s, &id);

    s.edit_plan(change(obj(&id), ObjectChange::Kind("pillar".into())))
        .unwrap();
    let pillar = object(&s, &id);

    assert_eq!(pillar.kind, "pillar");
    assert_eq!((pillar.width_m, pillar.depth_m), (0.4, 0.4));
    assert_eq!(pillar.height_m, Some(3.0));
    assert_eq!(pillar.shape, ObjectShape::Ellipse);
    assert_eq!((pillar.at, pillar.rotation_deg), (p(2.0, 3.0), 90.0));

    s.undo();

    assert_eq!(object(&s, &id), before);
}

/// A size or name the user set stays: a table made longer is a long bench.
#[test]
fn a_size_and_name_of_the_user_stay() {
    let mut s = open();

    s.edit_plan(change(obj("tisch"), ObjectChange::Kind("sofa".into())))
        .unwrap();
    let sofa = object(&s, "tisch");

    assert_eq!(sofa.kind, "sofa");
    assert_eq!((sofa.width_m, sofa.depth_m), (6.0, 1.2));
    assert_eq!(sofa.label, "Tafel");
}

/// A kind no library knows, and the free form's, are refused; the same kind changes nothing.
#[test]
fn an_unknown_kind_is_refused() {
    let mut s = open();

    assert!(bad_kind(s.edit_plan(change(
        obj("tisch"),
        ObjectChange::Kind("ufo".into())
    ))));
    assert!(bad_kind(s.edit_plan(change(
        obj("tisch"),
        ObjectChange::Kind("form".into())
    ))));
    assert!(!s.can_undo());

    let result = s.edit_plan(change(Source::Wall(0), ObjectChange::Kind("chair".into())));
    assert!(matches!(
        result,
        Err(ServiceError::BadValue {
            field: Field::Source,
            ..
        })
    ));
}

/// A locked set refuses all three.
#[test]
fn a_locked_set_refuses_the_object_changes() {
    let mut s = open();
    s.set_lock(Layer::Set, Lock::Locked);

    for what in [
        ObjectChange::Kind("chair".into()),
        ObjectChange::Mirror,
        ObjectChange::ResetSize,
    ] {
        let result = s.edit_plan(change(obj("tisch"), what));

        assert!(matches!(result, Err(ServiceError::Locked(Layer::Set))));
    }
}

/// Mirroring is a toggle with its own undo step.
#[test]
fn mirroring_toggles_and_undoes() {
    let mut s = open();

    s.edit_plan(change(obj("tisch"), ObjectChange::Mirror))
        .unwrap();
    assert!(object(&s, "tisch").mirrored);

    s.edit_plan(change(obj("tisch"), ObjectChange::Mirror))
        .unwrap();
    assert!(!object(&s, "tisch").mirrored);

    s.undo();
    assert!(object(&s, "tisch").mirrored);
}

/// A free form is mirrored in its points: the outline flips, the grips stay on it, and a
/// second flip gives the first outline back.
#[test]
fn a_free_form_is_mirrored_in_its_points() {
    let mut s = open();
    s.edit_plan(PlanEdit::AddShape {
        points: vec![p(0.0, 0.0), p(4.0, 0.0), p(4.0, 1.0), p(1.0, 3.0)],
    })
    .unwrap();
    let id = s.project().locations[0]
        .set
        .objects
        .last()
        .unwrap()
        .id
        .clone();
    let first = object(&s, &id);

    s.edit_plan(change(obj(&id), ObjectChange::Mirror)).unwrap();
    let flipped = object(&s, &id);

    assert!(!flipped.mirrored);
    assert_ne!(flipped.shape, first.shape);
    assert_eq!((flipped.at, flipped.width_m), (first.at, first.width_m));

    s.edit_plan(change(obj(&id), ObjectChange::Mirror)).unwrap();

    assert_eq!(object(&s, &id), first);
}

/// The size of the entry again, also for a table that was resized, named and given a height;
/// place, turn and name stay.
#[test]
fn the_size_of_the_entry_comes_back() {
    let mut s = open();
    let before = object(&s, "tisch");

    s.edit_plan(change(obj("tisch"), ObjectChange::ResetSize))
        .unwrap();
    let table = object(&s, "tisch");

    assert_eq!((table.width_m, table.depth_m), (1.6, 0.9));
    assert_eq!(table.height_m, Some(0.75));
    assert_eq!(
        (table.at, table.rotation_deg, table.label.as_str()),
        (before.at, before.rotation_deg, "Tafel")
    );

    s.undo();

    assert_eq!(object(&s, "tisch"), before);
}

/// An object of a kind no library knows has no size to go back to.
#[test]
fn a_free_form_has_no_entry_to_reset_to() {
    let mut s = open();
    s.edit_plan(PlanEdit::AddShape {
        points: vec![p(0.0, 0.0), p(4.0, 0.0), p(4.0, 1.0)],
    })
    .unwrap();
    let id = s.project().locations[0]
        .set
        .objects
        .last()
        .unwrap()
        .id
        .clone();

    assert!(bad_kind(
        s.edit_plan(change(obj(&id), ObjectChange::ResetSize))
    ));
}

fn label(source: Source, text: &str) -> PlanEdit {
    change(source, ObjectChange::Label(text.into()))
}

/// A label sits beside its object, is bound to it and follows when the object is moved; undo
/// takes the move back, the note with it.
#[test]
fn a_label_follows_its_object() {
    let mut s = open();

    s.edit_plan(label(obj("tisch"), "  Gedeck  ")).unwrap();
    let id = s.project().locations[0]
        .set
        .notes
        .last()
        .unwrap()
        .id
        .clone();
    let at = note_at(&s, &id);

    assert_eq!(bound(&s, &id).as_deref(), Some("object:tisch"));
    assert!(at.x_m > 10.0 && at.y_m < 6.5);

    s.edit_plan(PlanEdit::Move {
        source: obj("tisch"),
        to: p(12.0, 8.0),
    })
    .unwrap();
    assert_eq!(note_at(&s, &id), p(at.x_m + 2.0, at.y_m + 1.5));

    s.undo();
    assert_eq!(note_at(&s, &id), at);
}

/// A note moved alone stays bound at its new place; a group of both moves them together once.
#[test]
fn the_note_and_its_object_in_one_group_move_once() {
    let mut s = open();
    s.edit_plan(label(obj("tisch"), "Gedeck")).unwrap();
    let id = s.project().locations[0]
        .set
        .notes
        .last()
        .unwrap()
        .id
        .clone();
    let at = note_at(&s, &id);

    s.edit_plan(PlanEdit::Move {
        source: Source::Note(id.clone()),
        to: p(at.x_m + 1.0, at.y_m),
    })
    .unwrap();
    assert_eq!(bound(&s, &id).as_deref(), Some("object:tisch"));
    assert_eq!(object(&s, "tisch").at, p(10.0, 6.5));

    let at = note_at(&s, &id);
    s.edit_plan(PlanEdit::ShiftGroup {
        sources: vec![obj("tisch"), Source::Note(id.clone())],
        dx_m: 1.0,
        dy_m: 0.0,
    })
    .unwrap();

    assert_eq!(note_at(&s, &id), p(at.x_m + 1.0, at.y_m));
}

/// An unbound note stays where it is; so does one whose object was removed, now free.
#[test]
fn a_note_that_let_go_stays() {
    let mut s = open();
    s.edit_plan(label(obj("tisch"), "Gedeck")).unwrap();
    let id = s.project().locations[0]
        .set
        .notes
        .last()
        .unwrap()
        .id
        .clone();
    let at = note_at(&s, &id);

    s.edit_plan(change(Source::Note(id.clone()), ObjectChange::Unbind))
        .unwrap();
    s.edit_plan(PlanEdit::Move {
        source: obj("tisch"),
        to: p(12.0, 8.0),
    })
    .unwrap();
    assert_eq!((bound(&s, &id), note_at(&s, &id)), (None, at));

    s.undo();
    s.undo();
    s.edit_plan(PlanEdit::Remove {
        source: obj("tisch"),
    })
    .unwrap();

    assert_eq!((bound(&s, &id), note_at(&s, &id)), (None, at));
}

/// A figure's label follows the figure's place at the playhead; a mark's follows its mark, and
/// lets go when a mark before it goes (its index would mean another mark).
#[test]
fn labels_of_figures_and_marks_follow_them() {
    let mut s = open();
    s.set_time(6.0);

    s.edit_plan(label(Source::Figure("elisabeth".into()), "Auftritt"))
        .unwrap();
    let figure = s.project().locations[0]
        .set
        .notes
        .last()
        .unwrap()
        .id
        .clone();
    let mark = Source::Mark {
        character: "elisabeth".into(),
        index: 2,
    };
    s.edit_plan(label(mark.clone(), "Abgang")).unwrap();
    let last = s.project().locations[0]
        .set
        .notes
        .last()
        .unwrap()
        .id
        .clone();
    let (at_figure, at_mark) = (note_at(&s, &figure), note_at(&s, &last));

    s.edit_plan(PlanEdit::Move {
        source: mark,
        to: p(17.0, 4.0),
    })
    .unwrap();
    assert_eq!(note_at(&s, &last), p(at_mark.x_m + 1.0, at_mark.y_m + 1.0));
    assert_eq!(note_at(&s, &figure), at_figure);

    s.edit_plan(PlanEdit::Move {
        source: Source::Figure("elisabeth".into()),
        to: p(11.0, 7.0),
    })
    .unwrap();
    assert_eq!(
        note_at(&s, &figure),
        p(at_figure.x_m + 1.0, at_figure.y_m + 0.5)
    );

    s.edit_plan(PlanEdit::Remove {
        source: Source::Mark {
            character: "elisabeth".into(),
            index: 0,
        },
    })
    .unwrap();

    assert_eq!(bound(&s, &last), None);
}

/// A label needs a text and an element it can be bound to, and the notes layer open.
#[test]
fn a_label_needs_text_element_and_open_layer() {
    let mut s = open();

    assert!(matches!(
        s.edit_plan(label(obj("tisch"), "  ")),
        Err(ServiceError::BadValue {
            field: Field::Note,
            ..
        })
    ));
    assert!(matches!(
        s.edit_plan(label(Source::Wall(0), "Wand")),
        Err(ServiceError::BadValue {
            field: Field::Source,
            ..
        })
    ));
    assert!(matches!(
        s.edit_plan(label(obj("nope"), "Nichts")),
        Err(ServiceError::Gone)
    ));

    s.set_lock(Layer::Notes, Lock::Locked);

    assert!(matches!(
        s.edit_plan(label(obj("tisch"), "Gedeck")),
        Err(ServiceError::Locked(Layer::Notes))
    ));
}

/// The built-in library has the crew places, vehicles, the dog and the small props (LH-117),
/// each with a height, in a group the catalogue names, and a body form where it has one.
#[test]
fn the_library_has_crew_vehicles_a_dog_and_props() {
    let list = open().library();
    let want = [
        ("boom", "crew"),
        ("monitor", "crew"),
        ("director", "crew"),
        ("bus", "vehicles"),
        ("truck", "vehicles"),
        ("motorcycle", "vehicles"),
        ("dog", "nature"),
        ("phone", "props"),
        ("laptop", "props"),
    ];

    for (kind, group) in want {
        let entry = list.iter().find(|e| e.kind == kind).expect(kind);

        assert_eq!(entry.group, group, "{kind}");
        assert!(entry.height_m.is_some(), "{kind}");
    }
}
