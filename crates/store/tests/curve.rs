//! The ways of marks in the scene file: bends (LH-21) and their shape (LH-108); the default
//! of each writes no key.
#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::{Path, PathBuf};

use klappe_model::{Curve, Point};
use klappe_store::{load, save};

const EXAMPLE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../testdata/beispiel.klappe"
);

/// A fresh, empty folder for one test.
fn test_dir(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&dir);

    dir
}

/// A way without a bend writes no `via` key, and a file without it loads as before (LH-21);
/// one with bends keeps them in order, on figures and cameras.
#[test]
fn a_bend_is_written_only_when_there_is() {
    let dir = test_dir("via");
    let mut project = load(Path::new(EXAMPLE)).unwrap().project;
    let plan = &mut project.scenes[0].plans[0];
    plan.figures[0].marks[2].via.clear();

    save(&dir, &project, None).unwrap();
    let text = fs::read_to_string(dir.join("szenen/m4djbtj6.json")).unwrap();
    assert!(!text.contains("via"), "{text}");
    assert_eq!(load(&dir).unwrap().project, project);

    let plan = &mut project.scenes[0].plans[0];
    plan.figures[0].marks[2].via = vec![Point::new(11.0, 2.5), Point::new(13.0, 4.0)];
    plan.cameras[0].marks[1].via = vec![Point::new(8.0, 9.0)];

    save(&dir, &project, None).unwrap();
    let text = fs::read_to_string(dir.join("szenen/m4djbtj6.json")).unwrap();
    assert_eq!(text.matches("\"via\"").count(), 2, "{text}");
    assert_eq!(load(&dir).unwrap().project, project);
}

/// A smooth way writes no `curve` key, and a file without it loads as before; a straight one
/// keeps it, on figures and cameras.
#[test]
fn a_straight_way_is_written_only_when_set() {
    let dir = test_dir("curve");
    let mut project = load(Path::new(EXAMPLE)).unwrap().project;

    save(&dir, &project, None).unwrap();
    let text = fs::read_to_string(dir.join("szenen/m4djbtj6.json")).unwrap();
    assert!(!text.contains("curve"), "{text}");

    let plan = &mut project.scenes[0].plans[0];
    plan.figures[0].marks[1].curve = Curve::Straight;
    plan.cameras[0].marks[1].curve = Curve::Straight;

    save(&dir, &project, None).unwrap();
    let text = fs::read_to_string(dir.join("szenen/m4djbtj6.json")).unwrap();
    assert_eq!(text.matches("\"curve\": \"straight\"").count(), 2, "{text}");
    assert_eq!(load(&dir).unwrap().project, project);
}
