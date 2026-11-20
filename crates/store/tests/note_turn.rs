//! A note's turn (LH-08) survives saving and loading; an unturned note writes none.
#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::Path;

use klappe_store::{load, save};

const EXAMPLE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../testdata/beispiel.klappe"
);

/// The note part of the location file: other elements write `rotation_deg` too.
fn note_part(dir: &Path) -> String {
    let written = fs::read_to_string(dir.join("drehorte/saengersaal.toml")).unwrap();

    written.split("[[set.notes]]").nth(1).unwrap().to_owned()
}

#[test]
fn a_turned_note_round_trips() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("note-turn");
    let _ = fs::remove_dir_all(&dir);
    let mut project = load(Path::new(EXAMPLE)).unwrap().project;
    project.locations[0].set.notes[0].rotation_deg = 30.0;

    save(&dir, &project, None).unwrap();

    assert_eq!(load(&dir).unwrap().project, project);
    assert!(note_part(&dir).contains("rotation_deg = 30"));

    project.locations[0].set.notes[0].rotation_deg = 0.0;
    save(&dir, &project, None).unwrap();

    assert!(!note_part(&dir).contains("rotation_deg"));
}

/// A turn that is no number does not load: the note could be drawn nowhere and the project
/// never saved again (`save` refuses it, so the file is written by hand).
#[test]
fn a_note_turned_by_no_number_does_not_load() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("note-turn-nan");
    let _ = fs::remove_dir_all(&dir);
    let mut project = load(Path::new(EXAMPLE)).unwrap().project;
    project.locations[0].set.notes[0].rotation_deg = 30.0;
    save(&dir, &project, None).unwrap();
    let file = dir.join("drehorte/saengersaal.toml");
    let text = fs::read_to_string(&file).unwrap();
    let (before, notes) = text.split_once("[[set.notes]]").unwrap();
    let turned = notes.replacen("rotation_deg = 30.0", "rotation_deg = nan", 1);
    fs::write(&file, format!("{before}[[set.notes]]{turned}")).unwrap();

    let err = load(&dir).unwrap_err();

    assert!(err.to_string().contains("note n1"), "{err}");
}
