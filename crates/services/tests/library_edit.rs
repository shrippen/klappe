//! Renaming, grouping and deleting own library entries (LH-12), on the example project.
#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::{Path, PathBuf};

use klappe_model::{LibraryItem, SetObject};
use klappe_services::{
    Field, Language, ObjectShape, PlanEdit, Point, ProjectEdit, ServiceError, Session,
};

fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe")
}

fn open() -> Session {
    Session::open(&example()).unwrap()
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    dir
}

fn p(x_m: f64, y_m: f64) -> Point {
    Point::new(x_m, y_m)
}

fn objects(s: &Session) -> &[SetObject] {
    &s.project().locations[0].set.objects
}

fn place(s: &mut Session, kind: &str) -> SetObject {
    s.edit_plan(PlanEdit::AddObject {
        kind: kind.into(),
        at: p(2.0, 2.0),
    })
    .unwrap();

    objects(s).last().unwrap().clone()
}

/// The corners of a polygon; none for any other shape, which the tests then miss.
fn entry(s: &Session, kind: &str) -> klappe_services::LibraryEntry {
    s.library().into_iter().find(|e| e.kind == kind).unwrap()
}

fn throne() -> LibraryItem {
    LibraryItem {
        kind: "thron".into(),
        group: String::new(),
        label: "Thron".into(),
        width_m: 0.9,
        depth_m: 0.8,
        shape: ObjectShape::Rect,
        height_m: None,
        form: None,
    }
}

fn pond() -> LibraryItem {
    LibraryItem {
        kind: "teich".into(),
        label: "Teich".into(),
        width_m: 6.0,
        depth_m: 4.0,
        shape: ObjectShape::Ellipse,
        ..throne()
    }
}

fn import(s: &mut Session, items: Vec<LibraryItem>) -> Result<(), ServiceError> {
    s.edit_project(ProjectEdit::ImportLibrary { items })
}

fn own_kinds(s: &Session) -> Vec<&str> {
    s.project()
        .library
        .iter()
        .map(|i| i.kind.as_str())
        .collect()
}

fn file_throne(s: &mut Session) {
    import(s, vec![throne()]).unwrap();
}

fn rename(kind: &str, name: &str, group: &str) -> ProjectEdit {
    ProjectEdit::EditLibraryItem {
        kind: kind.into(),
        name: name.into(),
        group: group.into(),
    }
}

/// LH-12: built-in entries have a group, own ones start without.
#[test]
fn the_list_carries_the_groups() {
    let mut s = open();
    file_throne(&mut s);

    assert_eq!(entry(&s, "table").group, "furniture");
    assert_eq!(entry(&s, "thron").group, "");
}

#[test]
fn an_own_object_is_renamed_and_grouped_in_one_step() {
    let mut s = open();
    file_throne(&mut s);

    s.edit_project(rename("thron", "  Königsthron ", " Requisiten "))
        .unwrap();

    let item = &s.project().library[0];
    assert_eq!(
        (item.label.as_str(), item.group.as_str()),
        ("Königsthron", "Requisiten")
    );
    assert_eq!(item.kind, "thron");
    assert_eq!(entry(&s, "thron").group, "Requisiten");

    s.undo();
    assert_eq!(s.project().library, vec![throne()]);
}

#[test]
fn an_empty_group_takes_an_object_out_of_its_group() {
    let mut s = open();
    file_throne(&mut s);
    s.edit_project(rename("thron", "Thron", "Requisiten"))
        .unwrap();

    s.edit_project(rename("thron", "Thron", "  ")).unwrap();

    assert_eq!(entry(&s, "thron").group, "");
}

#[test]
fn a_rename_needs_a_name_and_an_own_object() {
    let mut s = open();
    file_throne(&mut s);

    assert!(matches!(
        s.edit_project(rename("thron", "  ", "")),
        Err(ServiceError::BadValue {
            field: Field::Name,
            ..
        })
    ));
    // A built-in entry is not the project's to change, nor is one that is not there.
    assert!(matches!(
        s.edit_project(rename("table", "Tafel", "")),
        Err(ServiceError::Gone)
    ));
    assert!(matches!(
        s.edit_project(rename("nix", "Nix", "")),
        Err(ServiceError::Gone)
    ));
    assert_eq!(s.project().library, vec![throne()]);
}

#[test]
fn an_own_object_is_deleted_and_the_deletion_undone() {
    let mut s = open();
    import(&mut s, vec![throne(), pond()]).unwrap();

    s.edit_project(ProjectEdit::RemoveLibraryItem {
        kind: "thron".into(),
    })
    .unwrap();

    assert_eq!(own_kinds(&s), vec!["teich"]);
    assert!(s.library().iter().all(|e| e.kind != "thron"));

    s.undo();
    assert_eq!(own_kinds(&s), vec!["thron", "teich"]);
}

#[test]
fn a_built_in_object_cannot_be_deleted() {
    let mut s = open();
    file_throne(&mut s);

    let result = s.edit_project(ProjectEdit::RemoveLibraryItem {
        kind: "table".into(),
    });

    assert!(matches!(result, Err(ServiceError::Gone)));
    assert_eq!(entry(&s, "table").uses, 1);
}

/// What the plan uses is counted for the question before a delete.
#[test]
fn an_entry_counts_the_objects_placed_from_it() {
    let mut s = open();
    file_throne(&mut s);
    assert_eq!(entry(&s, "thron").uses, 0);

    place(&mut s, "thron");
    place(&mut s, "thron");

    assert_eq!(entry(&s, "thron").uses, 2);
    assert_eq!(entry(&s, "table").uses, 1);
}

/// Placed objects keep size, shape and name when their entry goes or is renamed; a deleted
/// kind is then like an unknown one: a new object of it is a one metre square.
#[test]
fn placed_objects_survive_their_entry() {
    let mut s = open();
    file_throne(&mut s);
    let placed = place(&mut s, "thron");

    s.edit_project(rename("thron", "Sessel", "")).unwrap();
    assert_eq!(objects(&s).last().unwrap().label, "Thron");

    s.edit_project(ProjectEdit::RemoveLibraryItem {
        kind: "thron".into(),
    })
    .unwrap();

    let kept = objects(&s).last().unwrap();
    assert_eq!(kept, &placed);
    let again = place(&mut s, "thron");
    assert_eq!((again.width_m, again.depth_m), (1.0, 1.0));
}

#[test]
fn the_group_goes_into_a_library_file_and_back() {
    let mut s = open();
    let dir = scratch("library-group-file");
    let path = dir.join("bibliothek.toml");
    file_throne(&mut s);
    s.edit_project(rename("thron", "Thron", "Requisiten"))
        .unwrap();
    s.export_library(&path).unwrap();

    let mut other = open();
    other.import_library(&path).unwrap();

    assert_eq!(entry(&other, "thron").group, "Requisiten");
}

#[test]
fn every_built_in_group_has_a_name_in_both_languages() {
    for entry in open().library() {
        let key = format!("objectGroup.{}", entry.group);
        for language in [Language::German, Language::English] {
            assert_ne!(language.text(&key), key, "{key}");
        }
    }
}
