//! The team: `personen.toml` (persons, casting, locked numbers) and `kontakte.toml` (contacts
//! apart, DATENMODELL §10); both optional.
#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::{Path, PathBuf};

use klappe_model::{Casting, Contact, Department, Person, Project};
use klappe_store::{
    encode_tree, file_name, kind_of, load, load_backup, read_tree, save, save_backup, FileKind,
    StoreError,
};

fn dir(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&dir);

    dir
}

fn person(id: &str, name: &str) -> Person {
    Person {
        id: id.into(),
        name: name.into(),
        department: Some(Department::Lighting),
        function: "Oberbeleuchter".into(),
        agency: "Agentur Licht".into(),
    }
}

fn project() -> Project {
    let mut project = Project::new("T");
    project.team.persons = vec![person("p1", "Ines"), person("p2", "Jonas")];
    project.team.castings = vec![
        Casting {
            character: "anna".into(),
            fixed: Some("p1".into()),
            proposed: vec!["p2".into(), "p1".into()],
            cast_number: Some(2),
        },
        // An unknown character and an unknown person load and stay.
        Casting {
            character: "gone".into(),
            fixed: None,
            proposed: vec!["nobody".into()],
            cast_number: None,
        },
    ];
    project.team.numbers_locked = true;
    project.team.contacts.insert(
        "p1".into(),
        Contact {
            phone: "0151 1234".into(),
            mail: "ines@example.org".into(),
            address: "Hauptstr. 1\n10115 Berlin".into(),
        },
    );

    project
}

#[test]
fn the_team_survives_a_save() {
    let dir = dir("team-roundtrip");
    let project = project();

    save(&dir, &project, None).unwrap();
    let loaded = load(&dir).unwrap().project;

    assert_eq!(loaded.team, project.team);
}

#[test]
fn contacts_have_a_file_of_their_own() {
    let dir = dir("team-files");
    save(&dir, &project(), None).unwrap();

    let people = fs::read_to_string(dir.join("personen.toml")).unwrap();
    let contacts = fs::read_to_string(dir.join("kontakte.toml")).unwrap();

    assert!(people.contains("[[person]]"));
    assert!(people.contains("department = \"lighting\""));
    assert!(people.contains("numbers_locked = true"));
    assert!(!people.contains("0151"));
    assert!(!people.contains("Berlin"));
    assert!(contacts.contains("0151 1234"));
    assert!(contacts.contains("Berlin"));
    assert!(!contacts.contains("Oberbeleuchter"));
}

#[test]
fn the_files_exist_only_with_content() {
    let dir = dir("team-optional");
    let mut project = project();
    save(&dir, &project, None).unwrap();

    project.team.contacts.clear();
    save(&dir, &project, None).unwrap();
    assert!(dir.join("personen.toml").is_file());
    assert!(!dir.join("kontakte.toml").exists());

    project.team = Default::default();
    save(&dir, &project, None).unwrap();
    assert!(!dir.join("personen.toml").exists());

    let empty = self::dir("team-none");
    save(&empty, &Project::new("T"), None).unwrap();
    assert!(!empty.join("personen.toml").exists());
    assert!(!empty.join("kontakte.toml").exists());
    assert_eq!(load(&empty).unwrap().project.team, Default::default());
}

#[test]
fn a_person_id_used_twice_does_not_load_or_save() {
    let dir = dir("team-duplicate");
    save(&dir, &project(), None).unwrap();
    fs::write(
        dir.join("personen.toml"),
        "[[person]]\nid = \"p1\"\nname = \"A\"\n\n[[person]]\nid = \"p1\"\nname = \"B\"\n",
    )
    .unwrap();

    assert!(matches!(load(&dir), Err(StoreError::Parse { .. })));

    let mut twice = project();
    twice.team.persons.push(person("p1", "Noch einer"));
    assert!(matches!(
        save(&dir, &twice, None),
        Err(StoreError::Parse { .. })
    ));
}

#[test]
fn a_character_cast_twice_does_not_load() {
    let dir = dir("team-casting-twice");
    save(&dir, &project(), None).unwrap();
    fs::write(
        dir.join("personen.toml"),
        "[[besetzung]]\ncharacter = \"anna\"\n\n[[besetzung]]\ncharacter = \"anna\"\n",
    )
    .unwrap();

    assert!(matches!(load(&dir), Err(StoreError::Parse { .. })));
}

#[test]
fn unreadable_or_invalid_values_do_not_load() {
    let dir = dir("team-invalid");
    save(&dir, &project(), None).unwrap();

    let bad = [
        (
            "personen.toml",
            "[[person]]\nid = \"p1\"\nname = \"A\"\ndepartment = \"nonsense\"\n",
        ),
        ("personen.toml", "[[person]]\nid = \"\"\nname = \"A\"\n"),
        (
            "personen.toml",
            "[[besetzung]]\ncharacter = \"a\"\ncast_number = 0\n",
        ),
        (
            "personen.toml",
            "[[besetzung]]\ncharacter = \"a\"\ncast_number = 3\n\n[[besetzung]]\ncharacter = \"b\"\ncast_number = 3\n",
        ),
        ("personen.toml", "this is not toml"),
        (
            "kontakte.toml",
            "[[kontakt]]\nperson = \"p1\"\n\n[[kontakt]]\nperson = \"p1\"\n",
        ),
        ("kontakte.toml", "[[kontakt]]\nperson = \"\"\n"),
        ("kontakte.toml", "<<<<<<< HEAD\n"),
    ];
    for (file, text) in bad {
        save(&dir, &project(), None).unwrap();
        fs::write(dir.join(file), text).unwrap();

        assert!(load(&dir).is_err(), "{file}: {text}");
    }
}

#[test]
fn the_files_are_part_of_the_tree_and_the_backup() {
    let dir = dir("team-tree");
    let project = project();
    save(&dir, &project, None).unwrap();

    let tree = read_tree(&dir).unwrap();
    assert!(tree.contains_key("personen.toml"));
    assert!(tree.contains_key("kontakte.toml"));
    assert_eq!(tree, encode_tree(&project, None).unwrap());

    assert_eq!(kind_of("personen.toml"), Some(FileKind::Team));
    assert_eq!(kind_of("kontakte.toml"), Some(FileKind::Contacts));
    assert_eq!(file_name(FileKind::Team), Some("personen.toml"));
    assert_eq!(file_name(FileKind::Contacts), Some("kontakte.toml"));

    save_backup(&dir, &project, None).unwrap();
    assert_eq!(
        load_backup(&dir).unwrap().unwrap().project.team,
        project.team
    );
}
