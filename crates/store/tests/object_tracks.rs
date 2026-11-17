//! The ways of set objects in the scene file (LH-24): no track writes no key; a track and its
//! carrier round-trip.
#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::{Path, PathBuf};

use klappe_model::{ObjectMark, ObjectTrack, Point};
use klappe_store::{load, save};

const EXAMPLE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../testdata/beispiel.klappe"
);

fn test_dir(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&dir);

    dir
}

fn mark(at_s: f64, x_m: f64, carried_by: Option<&str>) -> ObjectMark {
    ObjectMark {
        at_s,
        at: Point::new(x_m, 4.0),
        rotation_deg: 90.0,
        hold_s: 1.0,
        carried_by: carried_by.map(str::to_owned),
    }
}

#[test]
fn a_track_is_written_only_when_there_is_one() {
    let dir = test_dir("object-tracks");
    let mut project = load(Path::new(EXAMPLE)).unwrap().project;

    save(&dir, &project, None).unwrap();
    let text = fs::read_to_string(dir.join("szenen/m4djbtj6.json")).unwrap();
    assert!(
        !text.contains("object_tracks") && !text.contains("carried_by"),
        "{text}"
    );

    project.scenes[0].plans[0].object_tracks = vec![ObjectTrack {
        object: "saeule-1".into(),
        marks: vec![mark(0.0, 6.0, Some("elisabeth")), mark(5.0, 9.0, None)],
    }];

    save(&dir, &project, None).unwrap();
    let text = fs::read_to_string(dir.join("szenen/m4djbtj6.json")).unwrap();
    assert_eq!(text.matches("\"carried_by\"").count(), 1, "{text}");
    assert_eq!(load(&dir).unwrap().project, project);
}
