//! Renames carried into the screenplay text, and where scenes and characters occur (D-08), on a
//! copy of the example project: scene `m4djbtj6` ("12"), characters `elisabeth` and `burgvogt`.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_services::{Field, FigureColor, ProjectEdit, ServiceError, Session};

fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe")
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
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    copy_dir(&example(), &dir);
    Session::open(&dir).unwrap()
}

fn rename_character(name: &str) -> ProjectEdit {
    ProjectEdit::EditCharacter {
        id: "elisabeth".into(),
        name: name.into(),
        short: "E".into(),
        color: FigureColor::Purple,
        group: None,
        size_m: None,
        aliases: None,
    }
}

/// D-08: the cues in the script say what a rename will change, the rename does it, and one
/// undo takes project and text back.
#[test]
fn a_character_rename_carries_into_the_script_in_one_undo_step() {
    let mut s = open("manage-rename-cue");
    let before = s.script_text().to_owned();

    let changes = s.character_script_changes("elisabeth", "Elise", None);
    assert_eq!(changes.len(), 1);
    assert_eq!(
        (changes[0].from.as_str(), changes[0].to.as_str()),
        ("ELISABETH", "ELISE")
    );
    assert_eq!(changes[0].count, 1);
    assert_eq!(s.script_text(), before);

    s.edit_project(rename_character("Elise")).unwrap();
    assert!(s.script_text().contains("ELISE\nIhr kommt spät."));
    assert!(!s.script_text().contains("ELISABETH\n"));
    assert_eq!(s.script_lines()[1].character, "elisabeth");

    s.undo();
    assert_eq!(s.script_text(), before);
    assert_eq!(
        s.project().character("elisabeth").unwrap().name,
        "Elisabeth"
    );

    s.redo();
    assert!(s.script_text().contains("ELISE\n"));
    assert_eq!(s.project().character("elisabeth").unwrap().name, "Elise");
}

/// A change that does not touch the text leaves it, and undo then leaves it too.
#[test]
fn a_colour_change_leaves_the_script_alone() {
    let mut s = open("manage-colour-only");
    let before = s.script_text().to_owned();
    let edit = ProjectEdit::EditCharacter {
        id: "elisabeth".into(),
        name: "Elisabeth".into(),
        short: "E".into(),
        color: FigureColor::Red,
        group: None,
        size_m: None,
        aliases: None,
    };

    s.edit_project(edit).unwrap();
    s.set_script_text("Anders.\n");
    s.undo();

    assert_eq!(s.script_text(), "Anders.\n");
    assert_ne!(before, "Anders.\n");
}

#[test]
fn a_scene_edit_carries_heading_and_number_into_the_script() {
    let mut s = open("manage-rename-scene");
    let before = s.script_text().to_owned();
    let edit = ProjectEdit::EditScene {
        id: "m4djbtj6".into(),
        number: "12A".into(),
        heading: "INT. SÄNGERSAAL – NACHT".into(),
        location: "saengersaal".into(),
        sun: None,
    };

    let changes = s.scene_script_changes("m4djbtj6", "12A", "INT. SÄNGERSAAL – NACHT");
    assert_eq!(changes.len(), 2);

    s.edit_project(edit).unwrap();
    assert!(s
        .script_text()
        .starts_with("INT. SÄNGERSAAL – NACHT #12A#\n"));
    // The script is still found by the scene's new number.
    assert_eq!(s.script_lines()[0].text, "Elisabeth tritt an den Tisch.");

    s.undo();
    assert_eq!(s.script_text(), before);
}

#[test]
fn uses_tell_where_a_character_and_a_scene_occur() {
    let s = open("manage-uses");

    let uses = s.character_uses("burgvogt");
    assert_eq!(uses.len(), 1);
    assert_eq!((uses[0].number.as_str(), uses[0].cues), ("12", 1));
    assert!(s.character_uses("nobody").is_empty());

    let card = s.scene_card("m4djbtj6").unwrap();
    assert_eq!(card.time, "TAG");
    assert_eq!(card.cast.len(), 2);
    assert_eq!(card.cast[0].cues, 1);
    assert!(s.scene_card("nobody").is_none());
}

/// A name another character carries (whatever its case) is refused: the cues of both would
/// merge in the script.
#[test]
fn a_character_cannot_take_the_name_of_another() {
    let mut s = open("manage-name-taken");
    let before = s.script_text().to_owned();

    let err = s.edit_project(rename_character("burgvogt")).unwrap_err();

    assert!(matches!(
        err,
        ServiceError::BadValue {
            field: Field::Name,
            ..
        }
    ));
    assert_eq!(s.script_text(), before);
    assert_eq!(
        s.project().character("elisabeth").unwrap().name,
        "Elisabeth"
    );
}

/// A new character cannot take a name another one carries either (case aside).
#[test]
fn a_new_character_cannot_take_a_taken_name() {
    let mut s = open("manage-new-name-taken");

    let err = s
        .edit_project(ProjectEdit::AddCharacter {
            name: "BURGVOGT".into(),
            short: "B".into(),
            color: FigureColor::Red,
            group: None,
            size_m: None,
        })
        .unwrap_err();

    assert!(matches!(
        err,
        ServiceError::BadValue {
            field: Field::Name,
            ..
        }
    ));
}

/// A cue compares in capitals, where "ß" is "SS": "Strauß" and "Strauss" would merge their cues.
#[test]
fn names_that_share_a_cue_are_taken() {
    let mut s = open("manage-name-sharp-s");
    let add = |name: &str| ProjectEdit::AddCharacter {
        name: name.into(),
        short: "S".into(),
        color: FigureColor::Red,
        group: None,
        size_m: None,
    };
    s.edit_project(add("Strauss")).unwrap();

    let err = s.edit_project(add("Strauß")).unwrap_err();

    assert!(matches!(
        err,
        ServiceError::BadValue {
            field: Field::Name,
            ..
        }
    ));
}

/// A number another heading of the script carries is refused, also when the project does not
/// know that scene.
#[test]
fn a_scene_cannot_take_a_number_the_script_has() {
    let mut s = open("manage-number-in-script");
    s.set_script_text("INT. A #12#\n\nA.\n\nEXT. HOF – NACHT #13#\n\nB.\n");
    let edit = ProjectEdit::EditScene {
        id: "m4djbtj6".into(),
        number: "13".into(),
        heading: "INT. A".into(),
        location: "saengersaal".into(),
        sun: None,
    };

    let err = s.edit_project(edit).unwrap_err();

    assert!(matches!(err, ServiceError::NumberTaken(n) if n == "13"));
    assert!(s.script_text().contains("#12#"));
}

/// A rename of the letters' case only is no change of the text.
#[test]
fn a_case_only_rename_leaves_the_text_and_the_undo_steps() {
    let mut s = open("manage-case-only");
    let before = s.script_text().to_owned();

    assert!(s
        .character_script_changes("elisabeth", "ELISABETH", None)
        .is_empty());
    s.edit_project(rename_character("ELISABETH")).unwrap();
    assert_eq!(s.script_text(), before);

    s.set_script_text("Anders.\n");
    s.undo();
    assert_eq!(s.script_text(), "Anders.\n");
}
