//! Fields of objects and notes on disk: mirrored objects (LH-115), bound notes (LH-118), and a
//! group's head count (LH-23).
#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::{Path, PathBuf};

use klappe_model::{Note, Point, Project};
use klappe_store::{load, save};

const EXAMPLE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../testdata/beispiel.klappe"
);
const LOCATION: &str = "drehorte/saengersaal.toml";

fn example() -> Project {
    load(Path::new(EXAMPLE)).unwrap().project
}

/// A fresh, empty folder for one test.
fn test_dir(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&dir);

    dir
}

/// A group's head count is written only for a group, reads back, and a file without it
/// loads as a single person (LH-23).
#[test]
fn a_group_is_written_only_when_given() {
    let dir = test_dir("group");
    let mut project = example();
    save(&dir, &project, None).unwrap();
    let text = fs::read_to_string(dir.join("figuren.toml")).unwrap();
    assert!(!text.contains("group"), "{text}");
    assert_eq!(load(&dir).unwrap().project.characters[0].group, None);

    project.characters[0].group = Some(12);
    save(&dir, &project, None).unwrap();
    let text = fs::read_to_string(dir.join("figuren.toml")).unwrap();

    assert_eq!(text.matches("group = 12").count(), 1, "{text}");
    assert_eq!(load(&dir).unwrap().project, project);
}

/// `mirrored` is written only for a mirrored object and reads back (LH-115); the example has
/// none and saves without the key.
#[test]
fn mirrored_is_written_only_when_set() {
    let dir = test_dir("mirrored");
    let mut project = example();
    save(&dir, &project, None).unwrap();
    let text = fs::read_to_string(dir.join(LOCATION)).unwrap();
    assert!(!text.contains("mirrored"), "{text}");

    project.locations[0].set.objects[0].mirrored = true;
    save(&dir, &project, None).unwrap();
    let text = fs::read_to_string(dir.join(LOCATION)).unwrap();

    assert_eq!(text.matches("mirrored = true").count(), 1, "{text}");
    assert_eq!(load(&dir).unwrap().project, project);
}

/// `bound` is written only for a bound note and reads back (LH-118).
#[test]
fn bound_is_written_only_for_a_bound_note() {
    let dir = test_dir("bound");
    let mut project = example();
    save(&dir, &project, None).unwrap();
    let text = fs::read_to_string(dir.join(LOCATION)).unwrap();
    assert!(!text.contains("bound"), "{text}");

    project.locations[0].set.notes[0].bound = Some("object:tisch".into());
    save(&dir, &project, None).unwrap();
    let text = fs::read_to_string(dir.join(LOCATION)).unwrap();

    assert_eq!(
        text.matches("bound = \"object:tisch\"").count(),
        1,
        "{text}"
    );
    assert_eq!(load(&dir).unwrap().project, project);
}

/// A plan's `labels` are written only when it has some, and read back (LH-118).
#[test]
fn plan_labels_are_written_only_when_given() {
    let dir = test_dir("labels");
    let mut project = example();
    save(&dir, &project, None).unwrap();
    let text = fs::read_to_string(dir.join("szenen/szene-012.json")).unwrap();
    assert!(!text.contains("labels"), "{text}");

    project.scenes[0].plans[0].labels.push(Note {
        id: "t1".into(),
        at: Point::new(4.0, 5.0),
        text: "Auftritt".into(),
        width_m: 2.4,
        bound: Some("figure:elisabeth".into()),
    });
    save(&dir, &project, None).unwrap();
    let text = fs::read_to_string(dir.join("szenen/szene-012.json")).unwrap();

    assert_eq!(text.matches("\"labels\"").count(), 1, "{text}");
    assert_eq!(load(&dir).unwrap().project, project);
}
