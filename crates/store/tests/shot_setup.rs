//! A shot's own setup in the scene file (LH-77).
#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::{Path, PathBuf};

use klappe_store::{load, save};

const SCENE_FILE: &str = "szenen/szene-012.json";

/// A fresh, empty folder for one test.
fn test_dir(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&dir);
    dir
}

/// Written only when set; a file without it loads as before.
#[test]
fn a_shot_setup_is_written_only_when_given() {
    let example = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe");
    let mut project = load(&example).unwrap().project;
    let dir = test_dir("shot-setup");

    save(&dir, &project, None).unwrap();
    let text = fs::read_to_string(dir.join(SCENE_FILE)).unwrap();

    assert!(!text.contains("setup_of"), "{text}");

    project.scenes[0].plans[0].shots[0].setup_of = Some("s12-3".into());
    save(&dir, &project, None).unwrap();
    let text = fs::read_to_string(dir.join(SCENE_FILE)).unwrap();

    assert_eq!(text.matches("\"setup_of\": \"s12-3\"").count(), 1, "{text}");
    assert_eq!(load(&dir).unwrap().project, project);
}
