//! The start page's project cards (D-07), on a copy of the example project.
#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use klappe_services::{project_card, CardState, Language, Session};

const PNG_MAGIC: &[u8] = b"\x89PNG";

fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe")
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&dir);
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

fn project(name: &str) -> (PathBuf, PathBuf) {
    let dir = scratch(name);
    copy_dir(&example(), &dir.join("projekt"));

    (dir.join("projekt"), dir.join("cache"))
}

#[test]
fn a_card_names_counts_and_previews_the_first_plan() {
    let (dir, cache) = project("start-card");
    let title = Session::open(&dir).unwrap().project().title.clone();

    let card = project_card(&dir, &cache, Language::German);

    assert_eq!(card.state, CardState::Ready);
    assert_eq!(card.name, title);
    assert_eq!(card.scenes, 1);
    assert!(card.modified_s > 0);
    let preview = card.preview.unwrap();
    assert!(preview.starts_with(&cache));
    assert!(fs::read(preview).unwrap().starts_with(PNG_MAGIC));
}

#[test]
fn the_preview_is_reused_until_the_project_is_newer() {
    let (dir, cache) = project("start-reuse");
    let preview = project_card(&dir, &cache, Language::German)
        .preview
        .unwrap();

    // A marker in place of the picture shows whether it is made again.
    fs::write(&preview, b"marker").unwrap();
    project_card(&dir, &cache, Language::German);
    assert_eq!(fs::read(&preview).unwrap(), b"marker");

    let file = fs::File::options()
        .write(true)
        .open(dir.join("projekt.toml"))
        .unwrap();
    file.set_modified(SystemTime::now() + Duration::from_secs(60))
        .unwrap();
    let fresh = project_card(&dir, &cache, Language::German)
        .preview
        .unwrap();

    assert_ne!(fresh, preview);
    assert!(fs::read(&fresh).unwrap().starts_with(PNG_MAGIC));
    assert!(!preview.exists());
}

#[test]
fn a_project_without_a_plan_gets_the_placeholder() {
    let dir = scratch("start-empty").join("leer");
    Session::create(&dir, "Leer").unwrap();

    let card = project_card(&dir, &scratch("start-empty-cache"), Language::English);

    assert_eq!(card.state, CardState::Ready);
    assert_eq!(card.name, "Leer");
    assert_eq!(card.scenes, 0);
    assert_eq!(card.preview, None);
}

#[test]
fn a_gone_folder_is_missing_and_a_broken_one_unreadable() {
    let base = scratch("start-gone");
    let gone = base.join("weg");
    let broken = base.join("kaputt");
    fs::create_dir_all(&broken).unwrap();

    let missing = project_card(&gone, &base, Language::German);
    let unreadable = project_card(&broken, &base, Language::German);

    assert_eq!(missing.state, CardState::Missing);
    assert_eq!(missing.name, "weg");
    assert_eq!(unreadable.state, CardState::Unreadable);
    assert_eq!(unreadable.preview, None);
}
