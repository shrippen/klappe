//! The shape of a way in the scene file (LH-108): a smooth one writes no key.
#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::Path;

use klappe_model::Curve;
use klappe_store::{load, save};

const EXAMPLE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../testdata/beispiel.klappe"
);

/// A smooth way writes no `curve` key, and a file without it loads as before; a straight one
/// keeps it, on figures and cameras.
#[test]
fn a_straight_way_is_written_only_when_set() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("curve");
    let _ = fs::remove_dir_all(&dir);
    let mut project = load(Path::new(EXAMPLE)).unwrap().project;

    save(&dir, &project, None).unwrap();
    let text = fs::read_to_string(dir.join("szenen/szene-012.json")).unwrap();
    assert!(!text.contains("curve"), "{text}");

    let plan = &mut project.scenes[0].plans[0];
    plan.figures[0].marks[1].curve = Curve::Straight;
    plan.cameras[0].marks[1].curve = Curve::Straight;

    save(&dir, &project, None).unwrap();
    let text = fs::read_to_string(dir.join("szenen/szene-012.json")).unwrap();
    assert_eq!(text.matches("\"curve\": \"straight\"").count(), 2, "{text}");
    assert_eq!(load(&dir).unwrap().project, project);
}
