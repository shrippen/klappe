//! The sheet an export prints on, kept in `projekt.toml`.
#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::{Path, PathBuf};

use klappe_model::{Orientation, Paper, PaperSetup, Project};
use klappe_store::{load, save};

const PROJECT_FILE: &str = "projekt.toml";

/// A fresh, empty folder for one test.
fn test_dir(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&dir);

    dir
}

#[test]
fn the_paper_is_written_and_read_back() {
    let dir = test_dir("paper-round-trip");
    let mut project = Project::new("Film");
    project.paper = PaperSetup {
        size: Paper::Letter,
        orientation: Orientation::Portrait,
    };

    save(&dir, &project, None).unwrap();
    let text = fs::read_to_string(dir.join(PROJECT_FILE)).unwrap();

    assert!(text.contains("paper = \"letter-portrait\""), "{text}");
    assert_eq!(load(&dir).unwrap().project, project);
}

/// The default writes no key, and a file without it is A4 landscape.
#[test]
fn a_file_without_paper_is_a4_landscape() {
    let dir = test_dir("paper-default");
    save(&dir, &Project::new("Film"), None).unwrap();
    let text = fs::read_to_string(dir.join(PROJECT_FILE)).unwrap();

    assert!(!text.contains("paper"), "{text}");
    assert_eq!(load(&dir).unwrap().project.paper, PaperSetup::default());
    assert_eq!(PaperSetup::default().name(), "a4-landscape");
}
