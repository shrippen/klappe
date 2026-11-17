//! Characters, their groups and sizes (LH-01 to LH-03, LH-25), on a copy of the example
//! project: characters `elisabeth` and `burgvogt` (both in Plan A).
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_services::{Field, FigureColor, Layer, Lock, ProjectEdit, ServiceError, Session};

/// The limits of a figure's size, as `FORMAT.md` states them.
const MIN_FIGURE_M: f64 = 0.1;
const MAX_FIGURE_M: f64 = 20.0;

fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe")
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    dir
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

fn open(name: &str) -> Session {
    let dir = scratch(name);
    copy_dir(&example(), &dir);
    Session::open(&dir).unwrap()
}

fn add_character(name: &str, short: &str) -> ProjectEdit {
    ProjectEdit::AddCharacter {
        name: name.into(),
        short: short.into(),
        color: FigureColor::Green,
        group: None,
        size_m: None,
    }
}

#[test]
fn adds_and_changes_a_character() {
    let mut s = open("manage-character");

    s.edit_project(add_character("Graf Dietrich", "GD"))
        .unwrap();
    s.edit_project(add_character("Graf Dietrich!", "G2"))
        .unwrap();

    let ids: Vec<_> = s
        .project()
        .characters
        .iter()
        .map(|c| c.id.as_str())
        .collect();
    assert_eq!(
        ids,
        ["elisabeth", "burgvogt", "graf-dietrich", "graf-dietrich-2"]
    );

    s.edit_project(ProjectEdit::EditCharacter {
        id: "graf-dietrich".into(),
        name: "Dietrich".into(),
        short: "D".into(),
        color: FigureColor::Red,
        group: None,
        size_m: None,
    })
    .unwrap();

    let c = s.project().character("graf-dietrich").unwrap();
    assert_eq!((c.name.as_str(), c.short.as_str()), ("Dietrich", "D"));
    assert_eq!(c.color, FigureColor::Red);
}

#[test]
fn a_character_needs_a_name_and_a_short_label() {
    let mut s = open("manage-character-bad");

    let err = s.edit_project(add_character("", "X")).unwrap_err();
    assert!(matches!(
        err,
        ServiceError::BadValue {
            field: Field::Name,
            ..
        }
    ));

    for short in ["", "  ", "ABCD"] {
        let err = s.edit_project(add_character("Graf", short)).unwrap_err();
        assert!(matches!(
            err,
            ServiceError::BadValue {
                field: Field::Short,
                ..
            }
        ));
    }

    let err = s
        .edit_project(ProjectEdit::EditCharacter {
            id: "nobody".into(),
            name: "x".into(),
            short: "x".into(),
            color: FigureColor::Blue,
            group: None,
            size_m: None,
        })
        .unwrap_err();
    assert!(matches!(err, ServiceError::NoCharacter(_)));
}

fn group_edit(id: &str, group: Option<u32>) -> ProjectEdit {
    ProjectEdit::EditCharacter {
        id: id.into(),
        name: "Gäste".into(),
        short: "G".into(),
        color: FigureColor::Cyan,
        group,
        size_m: None,
    }
}

fn size_edit(id: &str, size_m: Option<f64>) -> ProjectEdit {
    ProjectEdit::EditCharacter {
        id: id.into(),
        name: "Schimmel".into(),
        short: "S".into(),
        color: FigureColor::Cyan,
        group: None,
        size_m,
    }
}

fn size_of(s: &Session, id: &str) -> Option<f64> {
    s.project().character(id).unwrap().size_m
}

/// LH-25: a size is set, changed and cleared, and undo takes it back.
#[test]
fn a_figure_size_is_set_changed_and_cleared_with_undo() {
    let mut s = open("manage-size");

    s.edit_project(ProjectEdit::AddCharacter {
        name: "Schimmel".into(),
        short: "S".into(),
        color: FigureColor::Cyan,
        group: None,
        size_m: Some(2.4),
    })
    .unwrap();
    assert_eq!(size_of(&s, "schimmel"), Some(2.4));

    s.edit_project(size_edit("schimmel", Some(MAX_FIGURE_M)))
        .unwrap();
    assert_eq!(size_of(&s, "schimmel"), Some(MAX_FIGURE_M));

    s.edit_project(size_edit("schimmel", None)).unwrap();
    assert_eq!(size_of(&s, "schimmel"), None);

    s.undo();
    assert_eq!(size_of(&s, "schimmel"), Some(MAX_FIGURE_M));
}

#[test]
fn a_figure_size_stays_within_its_limits() {
    let mut s = open("manage-size-bounds");
    s.edit_project(add_character("Schimmel", "S")).unwrap();

    for size in [0.0, MIN_FIGURE_M / 2.0, MAX_FIGURE_M + 1.0, -1.0, f64::NAN] {
        let err = s
            .edit_project(size_edit("schimmel", Some(size)))
            .unwrap_err();
        assert!(matches!(
            err,
            ServiceError::BadValue {
                field: Field::Size,
                ..
            }
        ));
    }
    assert_eq!(size_of(&s, "schimmel"), None);

    s.edit_project(size_edit("schimmel", Some(MIN_FIGURE_M)))
        .unwrap();
    assert_eq!(size_of(&s, "schimmel"), Some(MIN_FIGURE_M));
}

/// LH-25: a locked figure layer keeps the sizes; other fields of the character still change.
#[test]
fn a_locked_figure_layer_keeps_the_size() {
    let mut s = open("manage-size-lock");
    s.edit_project(add_character("Schimmel", "S")).unwrap();
    s.set_lock(Layer::Figures, Lock::Locked);

    let err = s
        .edit_project(size_edit("schimmel", Some(2.0)))
        .unwrap_err();
    assert!(matches!(err, ServiceError::Locked(Layer::Figures)));
    assert_eq!(size_of(&s, "schimmel"), None);

    s.edit_project(size_edit("schimmel", None)).unwrap();
    s.set_lock(Layer::Figures, Lock::Unlocked);
    s.edit_project(size_edit("schimmel", Some(2.0))).unwrap();
    assert_eq!(size_of(&s, "schimmel"), Some(2.0));
}

#[test]
fn a_group_is_set_changed_and_cleared_with_undo() {
    let mut s = open("manage-group");

    s.edit_project(ProjectEdit::AddCharacter {
        name: "Gäste".into(),
        short: "G".into(),
        color: FigureColor::Cyan,
        group: Some(12),
        size_m: None,
    })
    .unwrap();
    assert_eq!(s.project().character("gaeste").unwrap().group, Some(12));

    s.edit_project(group_edit("gaeste", Some(999))).unwrap();
    assert_eq!(s.project().character("gaeste").unwrap().group, Some(999));

    s.edit_project(group_edit("gaeste", None)).unwrap();
    assert_eq!(s.project().character("gaeste").unwrap().group, None);

    s.undo();
    assert_eq!(s.project().character("gaeste").unwrap().group, Some(999));
}

#[test]
fn a_group_has_two_to_999_people() {
    let mut s = open("manage-group-bounds");
    s.edit_project(add_character("Gäste", "G")).unwrap();

    for count in [0, 1, 1000] {
        let err = s
            .edit_project(group_edit("gaeste", Some(count)))
            .unwrap_err();
        assert!(matches!(
            err,
            ServiceError::BadValue {
                field: Field::Group,
                ..
            }
        ));
    }
    assert_eq!(s.project().character("gaeste").unwrap().group, None);

    s.edit_project(group_edit("gaeste", Some(2))).unwrap();
    assert_eq!(s.project().character("gaeste").unwrap().group, Some(2));

    let err = s
        .edit_project(ProjectEdit::AddCharacter {
            name: "Wachen".into(),
            short: "W".into(),
            color: FigureColor::Red,
            group: Some(1),
            size_m: None,
        })
        .unwrap_err();
    assert!(matches!(
        err,
        ServiceError::BadValue {
            field: Field::Group,
            ..
        }
    ));
}

#[test]
fn a_character_in_a_plan_cannot_go() {
    let mut s = open("manage-character-in-use");

    let err = s
        .edit_project(ProjectEdit::RemoveCharacter {
            id: "elisabeth".into(),
        })
        .unwrap_err();

    assert!(matches!(
        err,
        ServiceError::CharacterInUse { id, plans: 1 } if id == "elisabeth"
    ));
    assert!(s.project().character("elisabeth").is_some());
}

#[test]
fn an_unused_character_goes() {
    let mut s = open("manage-character-remove");
    s.edit_project(add_character("Graf", "G")).unwrap();

    s.edit_project(ProjectEdit::RemoveCharacter { id: "graf".into() })
        .unwrap();
    assert!(s.project().character("graf").is_none());

    s.undo();
    assert!(s.project().character("graf").is_some());
}
