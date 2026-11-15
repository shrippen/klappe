//! Doors and windows in walls on disk (LH-10).
#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::{Path, PathBuf};

use klappe_model::{Opening, OpeningKind, Project};
use klappe_store::{load, save, StoreError};

const EXAMPLE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../testdata/beispiel.klappe"
);

fn example() -> Project {
    load(Path::new(EXAMPLE)).unwrap().project
}

/// A fresh, empty folder for one test.
fn test_dir(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&dir);

    dir
}

/// A wall without openings writes no `openings` key, and a file without it loads as before
/// (LH-10); one with openings keeps them in order.
#[test]
fn openings_are_written_only_when_there_are() {
    let dir = test_dir("openings");
    let mut project = example();
    project.locations[0].set.walls[2].openings.clear();

    save(&dir, &project, None).unwrap();
    let text = fs::read_to_string(dir.join("drehorte/saengersaal.toml")).unwrap();
    assert!(!text.contains("openings"), "{text}");
    assert_eq!(load(&dir).unwrap().project, project);

    let wall = &mut project.locations[0].set.walls[0];
    wall.openings = vec![
        Opening {
            kind: OpeningKind::Window,
            at_m: 12.5,
            width_m: 1.2,
        },
        Opening {
            kind: OpeningKind::Door,
            at_m: 3.0,
            width_m: 0.9,
        },
    ];
    save(&dir, &project, None).unwrap();
    let text = fs::read_to_string(dir.join("drehorte/saengersaal.toml")).unwrap();

    assert!(text.contains("kind = \"window\""), "{text}");
    assert_eq!(load(&dir).unwrap().project, project);
}

/// An opening with no width, or none that is a number, is no opening: the file is refused.
#[test]
fn an_unusable_opening_is_refused() {
    for (width_m, usable) in [(0.9, true), (0.0, false), (-1.0, false), (f64::NAN, false)] {
        let dir = test_dir("bad-opening");
        let mut project = example();
        project.locations[0].set.walls[0].openings = vec![Opening {
            kind: OpeningKind::Door,
            at_m: 5.0,
            width_m,
        }];
        let saved = save(&dir, &project, None);
        if saved.is_err() {
            assert!(!usable, "width {width_m}");
            continue;
        }

        let loaded = load(&dir);

        assert_eq!(loaded.is_ok(), usable, "width {width_m}");
        if !usable {
            assert!(
                matches!(loaded, Err(StoreError::Parse { .. })),
                "width {width_m}"
            );
        }
    }
}
