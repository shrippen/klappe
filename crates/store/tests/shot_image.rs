//! A shot's storyboard picture in the scene file (LH-74): a file of `bilder/`, nothing outside
//! the project.
#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::{Path, PathBuf};

use klappe_store::{load, save};

fn test_dir(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&dir);
    dir
}

fn load_with(name: &str, image: &str) -> bool {
    let example = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe");
    let mut project = load(&example).unwrap().project;
    project.scenes[0].plans[0].shots[0].image = Some(image.to_owned());
    let dir = test_dir(name);
    save(&dir, &project, None).unwrap();

    load(&dir).is_ok()
}

#[test]
fn a_picture_inside_bilder_loads() {
    assert!(load_with("shot-image-ok", "bilder/skizze.png"));
}

#[test]
fn a_picture_outside_the_project_does_not_load() {
    assert!(!load_with("shot-image-up", "../x.png"));
    assert!(!load_with("shot-image-abs", "/home/u/x.png"));
    assert!(!load_with("shot-image-other", "szenen/m4djbtj6.json"));
}
