//! The project folder as bytes: `read_tree`, `encode_tree`, `load_tree`, `write_changes`.
#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::{Path, PathBuf};

use klappe_store::{
    encode_tree, load, load_tree, read_tree, save, write_changes, StoreError, Tree,
};

const EXAMPLE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../testdata/beispiel.klappe"
);

/// A copy of the example to write to.
fn copy(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&dir);
    copy_dir(Path::new(EXAMPLE), &dir);

    dir
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
            continue;
        }
        fs::copy(entry.path(), target).unwrap();
    }
}

#[test]
fn encoding_twice_gives_the_same_bytes() {
    let loaded = load(Path::new(EXAMPLE)).unwrap();
    let first = encode_tree(&loaded.project, loaded.script.as_deref()).unwrap();
    let second = encode_tree(&loaded.project, loaded.script.as_deref()).unwrap();

    assert_eq!(first, second);
    assert!(first.contains_key("projekt.toml"));
    assert!(first.contains_key("drehbuch.fountain"));
    assert!(first.keys().any(|n| n.starts_with("szenen/")));
}

#[test]
fn the_tree_read_after_a_save_is_the_tree_encoded() {
    let dir = copy("disk-roundtrip");
    let loaded = load(&dir).unwrap();
    save(&dir, &loaded.project, loaded.script.as_deref()).unwrap();

    let read = read_tree(&dir).unwrap();
    let encoded = encode_tree(&loaded.project, loaded.script.as_deref()).unwrap();

    assert_eq!(read, encoded);
}

#[test]
fn the_overlay_gives_the_mixed_version_and_leaves_no_folder() {
    let dir = copy("disk-overlay");
    let mut tree = read_tree(&dir).unwrap();
    tree.insert("drehbuch.fountain".to_owned(), b"INT. NEU - TAG\n".to_vec());

    let loaded = load_tree(&dir, &tree).unwrap();

    assert_eq!(loaded.script.as_deref(), Some("INT. NEU - TAG\n"));
    assert_eq!(
        loaded.project,
        load(&dir).unwrap().project,
        "the plans stay those of the folder"
    );
    assert!(!dir.join(".abgleich").exists());
    // The folder itself is untouched.
    assert_ne!(
        fs::read_to_string(dir.join("drehbuch.fountain")).unwrap(),
        "INT. NEU - TAG\n"
    );
}

#[test]
fn a_leftover_overlay_is_removed_by_the_next_call() {
    let dir = copy("disk-leftover");
    fs::create_dir_all(dir.join(".abgleich/szenen")).unwrap();
    fs::write(dir.join(".abgleich/szenen/stale.json"), "{}").unwrap();

    let tree = read_tree(&dir).unwrap();
    let loaded = load_tree(&dir, &tree).unwrap();

    assert_eq!(loaded.project.scenes.len(), 1);
    assert!(!dir.join(".abgleich").exists());
}

#[test]
fn a_broken_file_fails_and_the_folder_is_untouched() {
    let dir = copy("disk-broken");
    let before = read_tree(&dir).unwrap();
    let mut tree = before.clone();
    tree.insert("projekt.toml".to_owned(), b"title = [".to_vec());

    let error = load_tree(&dir, &tree).unwrap_err();

    assert!(matches!(error, StoreError::Parse { .. }), "{error:?}");
    assert!(!dir.join(".abgleich").exists());
    assert_eq!(read_tree(&dir).unwrap(), before);
}

#[test]
fn changes_write_and_remove_only_what_they_name() {
    let dir = copy("disk-changes");
    let before = read_tree(&dir).unwrap();
    let scene = before
        .keys()
        .find(|n| n.starts_with("szenen/"))
        .unwrap()
        .clone();

    write_changes(
        &dir,
        &[
            (
                "szenen.toml".to_owned(),
                Some(before["szenen.toml"].clone()),
            ),
            ("szenen/neu.json".to_owned(), Some(before[&scene].clone())),
            (scene.clone(), None),
        ],
    )
    .unwrap();

    let mut expected: Tree = before.clone();
    expected.remove(&scene);
    expected.insert("szenen/neu.json".to_owned(), before[&scene].clone());
    assert_eq!(read_tree(&dir).unwrap(), expected);
    // What is not a unit stays.
    assert!(dir.join("orte.toml").exists());
}
