//! A free form on disk (LH-92): an object with the shape `polygon`.
#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::{Path, PathBuf};

use klappe_model::{LibraryItem, ObjectShape, Point, Project};
use klappe_store::{load, load_library, save, save_library, StoreError};

const EXAMPLE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../testdata/beispiel.klappe"
);
const LOCATION: &str = "drehorte/saengersaal.toml";

fn test_dir(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    dir
}

/// An L round (0, 0), 2 m wide and 3 m deep, as the points of a form centred on its object.
fn l_shape() -> ObjectShape {
    let points = [
        (-1.0, -1.5),
        (1.0, -1.5),
        (1.0, -0.5),
        (0.0, -0.5),
        (0.0, 1.5),
        (-1.0, 1.5),
    ];

    ObjectShape::Polygon {
        points: points.iter().map(|&(x, y)| Point::new(x, y)).collect(),
    }
}

fn with_form(shape: ObjectShape) -> Project {
    let mut project = load(Path::new(EXAMPLE)).unwrap().project;
    let set = &mut project.locations[0].set;
    set.objects[0].kind = "form".into();
    set.objects[0].width_m = 2.0;
    set.objects[0].depth_m = 3.0;
    set.objects[0].shape = shape;

    project
}

/// The outline is written as a list of points and read back as it was.
#[test]
fn a_free_form_round_trips() {
    let dir = test_dir("free-form-round");
    let project = with_form(l_shape());

    save(&dir, &project, None).unwrap();
    let text = fs::read_to_string(dir.join(LOCATION)).unwrap();

    assert!(text.contains("polygon"), "{text}");
    assert_eq!(load(&dir).unwrap().project, project);
}

/// A rectangle still writes no `shape`; an older file loads unchanged.
#[test]
fn a_rect_writes_no_shape() {
    let dir = test_dir("free-form-rect");

    save(&dir, &with_form(ObjectShape::Rect), None).unwrap();

    let text = fs::read_to_string(dir.join(LOCATION)).unwrap();
    assert!(
        !text.contains("polygon") && !text.contains("shape ="),
        "{text}"
    );
}

/// A file whose outline has too few points or no area does not load.
#[test]
fn a_free_form_without_an_area_does_not_load() {
    let points = |list: &[(f64, f64)]| ObjectShape::Polygon {
        points: list.iter().map(|&(x, y)| Point::new(x, y)).collect(),
    };
    let bad = [
        points(&[]),
        points(&[(0.0, 0.0), (1.0, 1.0)]),
        points(&[(0.0, 0.0), (1.0, 1.0), (2.0, 2.0)]),
    ];

    for (i, shape) in bad.into_iter().enumerate() {
        let dir = test_dir(&format!("free-form-bad-{i}"));
        // `save` does not check the objects, `load` does (a number that is none cannot be saved).
        save(&dir, &with_form(shape), None).unwrap();

        let err = load(&dir).unwrap_err();

        assert!(matches!(err, StoreError::Parse { .. }), "{i}: {err}");
        assert!(err.to_string().contains("free form"), "{i}: {err}");
    }
}

/// A free form kept in a library file keeps its outline; a bad one is refused.
#[test]
fn the_library_keeps_a_free_form() {
    let dir = test_dir("free-form-library");
    let path = dir.join("formen.toml");
    let item = |shape| LibraryItem {
        kind: "l".into(),
        group: String::new(),
        label: "L".into(),
        width_m: 2.0,
        depth_m: 3.0,
        shape,
        height_m: Some(1.0),
    };

    save_library(&path, &[item(l_shape())]).unwrap();
    assert_eq!(load_library(&path).unwrap(), vec![item(l_shape())]);

    let flat = ObjectShape::Polygon {
        points: vec![Point::new(0.0, 0.0), Point::new(1.0, 0.0)],
    };
    let err = save_library(&path, &[item(flat)]).unwrap_err();
    assert!(matches!(err, StoreError::Encode { .. }));
}
