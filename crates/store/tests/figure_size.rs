//! A figure's size in `figuren.toml` (LH-25).
#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::{Path, PathBuf};

use klappe_model::Project;
use klappe_store::{load, save, StoreError};

const EXAMPLE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../testdata/beispiel.klappe"
);

fn test_dir(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&dir);

    dir
}

fn example() -> Project {
    load(Path::new(EXAMPLE)).unwrap().project
}

/// A size is written only when given, and reads back; a file without it loads as standard.
#[test]
fn a_figure_size_is_written_only_when_given() {
    let dir = test_dir("figure-size");
    let mut project = example();
    save(&dir, &project, None).unwrap();
    let text = fs::read_to_string(dir.join("figuren.toml")).unwrap();
    assert!(!text.contains("size_m"), "{text}");
    assert_eq!(load(&dir).unwrap().project.characters[0].size_m, None);

    project.characters[0].size_m = Some(2.4);
    save(&dir, &project, None).unwrap();
    let text = fs::read_to_string(dir.join("figuren.toml")).unwrap();

    assert_eq!(text.matches("size_m = 2.4").count(), 1, "{text}");
    assert_eq!(load(&dir).unwrap().project, project);
}

/// The file of a project with a size outside 0.1 to 20 m is refused.
#[test]
fn a_bad_figure_size_is_refused() {
    for (size_m, usable) in [(0.0, false), (0.1, true), (20.0, true), (20.5, false)] {
        let dir = test_dir("bad-figure-size");
        let mut project = example();
        project.characters[0].size_m = Some(size_m);
        save(&dir, &project, None).unwrap();

        let loaded = load(&dir);

        assert_eq!(loaded.is_ok(), usable, "size {size_m}");
        if !usable {
            assert!(
                matches!(loaded, Err(StoreError::Parse { .. })),
                "size {size_m}"
            );
        }
    }
}
