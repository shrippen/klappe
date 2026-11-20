//! A note's width (LH-16) in the files.
#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::Path;

use klappe_model::NOTE_WIDTH_M;
use klappe_store::{load, save};

const EXAMPLE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../testdata/beispiel.klappe"
);

/// The example's note has no `width_m` in its file: it loads with the usual 2.4 m.
#[test]
fn a_file_without_a_width_loads_with_the_usual_one() {
    let loaded = load(Path::new(EXAMPLE)).unwrap();

    assert_eq!(
        loaded.project.locations[0].set.notes[0].width_m,
        NOTE_WIDTH_M
    );
}

#[test]
fn a_note_width_is_written_and_reads_back() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("note-width");
    let _ = fs::remove_dir_all(&dir);
    let mut project = load(Path::new(EXAMPLE)).unwrap().project;
    project.locations[0].set.notes[0].width_m = 3.5;

    save(&dir, &project, None).unwrap();
    let text = fs::read_to_string(dir.join("drehorte/saengersaal.toml")).unwrap();

    assert_eq!(text.matches("width_m = 3.5").count(), 1, "{text}");
    assert_eq!(load(&dir).unwrap().project, project);
}

/// A card narrower than the smallest one a drag allows does not load: the plan
/// would draw it at the minimum and the grip would not follow the pointer.
#[test]
fn a_note_narrower_than_the_minimum_does_not_load() {
    for (i, width_m) in [0.2, -1.0].into_iter().enumerate() {
        let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("note-width-bad-{i}"));
        let _ = fs::remove_dir_all(&dir);
        let mut project = load(Path::new(EXAMPLE)).unwrap().project;
        project.locations[0].set.notes[0].width_m = width_m;
        // `save` does not check the notes, `load` does.
        save(&dir, &project, None).unwrap();

        let err = load(&dir).unwrap_err();

        assert!(
            err.to_string().contains("m wide, not a finite width"),
            "{i}: {err}"
        );
    }
}

/// A width that is no finite number does not load: the project could never be saved again.
/// `save` refuses it, so the file is written by hand.
#[test]
fn an_endless_note_does_not_load() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("note-width-endless");
    let _ = fs::remove_dir_all(&dir);
    let mut project = load(Path::new(EXAMPLE)).unwrap().project;
    project.locations[0].set.notes[0].width_m = 3.5;
    save(&dir, &project, None).unwrap();
    let file = dir.join("drehorte/saengersaal.toml");
    let text = fs::read_to_string(&file).unwrap();
    fs::write(&file, text.replace("width_m = 3.5", "width_m = inf")).unwrap();

    let err = load(&dir).unwrap_err();

    assert!(err.to_string().contains("width_m"), "{err}");
}
