//! Placed images (LH-18) in the files.
#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::Path;

use klappe_model::{PlanImage, Point};
use klappe_store::{load, save};

const EXAMPLE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../testdata/beispiel.klappe"
);

fn image(width_m: f64, rotation_deg: f64) -> PlanImage {
    PlanImage {
        id: "i1".into(),
        file: "bilder/skizze.png".into(),
        at: Point::new(4.0, 5.5),
        width_m,
        rotation_deg,
    }
}

fn fresh(name: &str) -> std::path::PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&dir);

    dir
}

#[test]
fn the_example_has_no_images_and_writes_no_section() {
    let dir = fresh("images-none");
    let project = load(Path::new(EXAMPLE)).unwrap().project;

    save(&dir, &project, None).unwrap();
    let text = fs::read_to_string(dir.join("drehorte/saengersaal.toml")).unwrap();

    assert!(project.locations[0].set.images.is_empty());
    assert!(!text.contains("images"), "{text}");
}

#[test]
fn images_are_written_and_read_back() {
    let dir = fresh("images-round");
    let mut project = load(Path::new(EXAMPLE)).unwrap().project;
    project.locations[0].set.images = vec![image(2.0, 0.0), image(1.5, 30.0)];
    project.locations[0].set.images[1].id = "i2".into();

    save(&dir, &project, None).unwrap();
    let text = fs::read_to_string(dir.join("drehorte/saengersaal.toml")).unwrap();

    assert_eq!(text.matches("[[set.images]]").count(), 2, "{text}");
    // A turn of 0° is not written.
    let images = text.split_once("[[set.images]]").unwrap().1;
    assert_eq!(images.matches("rotation_deg").count(), 1, "{text}");
    assert_eq!(images.matches("rotation_deg = 30.0").count(), 1, "{text}");
    assert_eq!(load(&dir).unwrap().project, project);
}

/// A width a drag of the grip could not have set does not load.
#[test]
fn an_image_of_no_usable_width_does_not_load() {
    for (i, width_m) in [0.0, -1.0, 1e9].into_iter().enumerate() {
        let dir = fresh(&format!("images-bad-{i}"));
        let mut project = load(Path::new(EXAMPLE)).unwrap().project;
        project.locations[0].set.images = vec![image(width_m, 0.0)];
        save(&dir, &project, None).unwrap();

        let error = load(&dir).unwrap_err().to_string();

        assert!(error.contains("image i1"), "{error}");
    }
}
