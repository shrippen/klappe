//! How the screenplay is paged and printed, kept in `projekt.toml` (DN-05, F-01, F-04).
#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::{Path, PathBuf};

use klappe_model::{
    Project, SceneNumbering, ScriptLanguage, ScriptPaper, ScriptSetup, ScriptWords,
};
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
        words: ScriptWords::International,
        paper: ScriptPaper::Letter,
        contd_after_action: true,
        scene_numbering: SceneNumbering::Locked,
        first_scene: 10,
    };

    save(&dir, &project, None).unwrap();
    let text = fs::read_to_string(dir.join(PROJECT_FILE)).unwrap();

    assert!(text.contains("[script]"), "{text}");
    assert!(text.contains("language = \"de\""), "{text}");
    assert!(text.contains("words = \"international\""), "{text}");
    assert!(text.contains("paper = \"letter\""), "{text}");
    assert!(text.contains("scene_numbering = \"locked\""), "{text}");
    assert!(text.contains("first_scene = 10"), "{text}");
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

/// An unknown value falls back to the default, as the plan's `paper` does.
#[test]
fn unknown_values_are_the_default() {
    let dir = test_dir("script-unknown");
    save(&dir, &Project::new("Film"), None).unwrap();
    let path = dir.join(PROJECT_FILE);
    let mut text = fs::read_to_string(&path).unwrap();
    text.push_str("\n[script]\nlanguage = \"fr\"\nwords = \"klingon\"\npaper = \"tabloid\"\ncontd_after_action = true\n");
    fs::write(&path, text).unwrap();

    let setup = load(&dir).unwrap().project.script;

    assert_eq!(
        setup,
        ScriptSetup {
            contd_after_action: true,
            ..ScriptSetup::default()
        }
    );
}
