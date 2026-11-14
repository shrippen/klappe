//! The embedded example project (`demo/beispiel-<lang>.klappe`). The copies go to cargo's
//! per-crate test directory, not `/tmp`.
#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use klappe_services::{copy_example, ensure_example, Language, ServiceError, Session};

const DEMO: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../demo");

fn target(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

/// Every file under `dir` by its path relative to `dir`.
fn tree(dir: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut files = BTreeMap::new();
    let mut open = vec![dir.to_owned()];

    while let Some(folder) = open.pop() {
        for entry in std::fs::read_dir(&folder).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                open.push(path);
                continue;
            }
            let name = path
                .strip_prefix(dir)
                .unwrap()
                .to_string_lossy()
                .into_owned();
            files.insert(name, std::fs::read(&path).unwrap());
        }
    }

    files
}

#[test]
fn the_copy_is_the_example_of_its_language() {
    for (language, code) in [(Language::German, "de"), (Language::English, "en")] {
        let dir = target(&format!("example-{code}"));

        copy_example(language, &dir).unwrap();

        let shipped = Path::new(DEMO).join(format!("beispiel-{code}.klappe"));
        assert_eq!(tree(&dir), tree(&shipped), "{code}");
    }
}

#[test]
fn the_copy_opens_with_its_scene() {
    let dir = target("example-open");
    copy_example(Language::English, &dir).unwrap();

    let session = Session::open(&dir).unwrap();

    assert_eq!(session.project().title, "Nightwatch");
    assert_eq!(session.project().scenes.len(), 1);
}

#[test]
fn a_folder_with_files_is_left_alone() {
    let dir = target("example-full");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("mine.txt"), "mine").unwrap();

    let err = copy_example(Language::German, &dir).unwrap_err();

    assert!(matches!(err, ServiceError::NotEmpty(_)), "{err}");
    assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1);
}

/// A second click opens the user's copy as it is: what they changed in it stays.
#[test]
fn the_example_is_copied_once_and_then_left_as_it_is() {
    let dir = target("example-once");

    ensure_example(Language::German, &dir).unwrap();
    assert!(Session::open(&dir).is_ok());
    let marker = dir.join("meins.txt");
    std::fs::write(&marker, "x").unwrap();
    ensure_example(Language::German, &dir).unwrap();

    assert!(marker.exists());
}

#[test]
fn an_empty_example_folder_is_filled_again() {
    let dir = target("example-empty");
    std::fs::create_dir_all(&dir).unwrap();

    ensure_example(Language::English, &dir).unwrap();

    assert!(Session::open(&dir).is_ok());
}

/// Review of PR 52: a copy cut off midway (disk full) left a half folder that later counted as
/// the user's copy and never opened. The copy is written beside the folder and moved in whole;
/// what an earlier cut-off copy left beside it is no obstacle.
#[test]
fn a_cut_off_copy_leaves_no_half_folder() {
    let dir = target("example-cut");
    let beside = dir.with_file_name("example-cut.part");
    let _ = std::fs::remove_dir_all(&beside);
    std::fs::create_dir_all(beside.join("szenen")).unwrap();
    std::fs::write(beside.join("projekt.toml"), "halb").unwrap();

    ensure_example(Language::German, &dir).unwrap();

    let shipped = Path::new(DEMO).join("beispiel-de.klappe");
    assert_eq!(tree(&dir), tree(&shipped));
    assert!(!beside.exists());
}
