//! How the screenplay is paged and printed, kept in `projekt.toml` (DN-05, F-01, F-04).
#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::{Path, PathBuf};

use klappe_model::{Project, ScriptLanguage, ScriptPaper, ScriptSetup};
use klappe_store::{load, save};

const PROJECT_FILE: &str = "projekt.toml";

/// A fresh, empty folder for one test.
fn test_dir(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&dir);

    dir
}

#[test]
fn the_setup_is_written_and_read_back() {
    let dir = test_dir("script-round-trip");
    let mut project = Project::new("Film");
    project.script = ScriptSetup {
        language: Some(ScriptLanguage::De),
        paper: ScriptPaper::Letter,
        contd_after_action: true,
    };

    save(&dir, &project, None).unwrap();
    let text = fs::read_to_string(dir.join(PROJECT_FILE)).unwrap();

    assert!(text.contains("[script]"), "{text}");
    assert!(text.contains("language = \"de\""), "{text}");
    assert!(text.contains("paper = \"letter\""), "{text}");
    assert_eq!(load(&dir).unwrap().project, project);
}

/// The default writes no table, and a file without it is A4, no language chosen, no
/// `(CONT'D)` after action.
#[test]
fn a_file_without_the_table_is_the_default() {
    let dir = test_dir("script-default");
    save(&dir, &Project::new("Film"), None).unwrap();
    let text = fs::read_to_string(dir.join(PROJECT_FILE)).unwrap();

    assert!(!text.contains("[script]"), "{text}");
    assert_eq!(load(&dir).unwrap().project.script, ScriptSetup::default());
    assert_eq!(ScriptSetup::default().paper, ScriptPaper::A4);
}
