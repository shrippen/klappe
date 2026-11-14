//! The embedded example project (`demo/beispiel-<lang>.klappe`). The copies go to cargo's
//! per-crate test directory, not `/tmp`.
#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use klappe_services::{copy_example, Language, ServiceError, Session};

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
