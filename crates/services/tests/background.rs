//! The set's background image (LH-13): setting, calibrating, moving, removing. Numbers worked
//! out by hand. The images are bare PNG headers: only the pixel size is ever read from them.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_model::Background;
use klappe_services::{Layer, PlanEdit, Point, Role, ServiceError, Session, Shape, Visibility};

const EXAMPLE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../testdata/beispiel.klappe"
);
const IMAGE: &str = "bilder/saal.png";

fn p(x_m: f64, y_m: f64) -> Point {
    Point::new(x_m, y_m)
}

/// A PNG header of `w` x `h` pixels.
fn png(w: u32, h: u32) -> Vec<u8> {
    let mut b = b"\x89PNG\r\n\x1a\n".to_vec();
    b.extend_from_slice(&13_u32.to_be_bytes());
    b.extend_from_slice(b"IHDR");
    b.extend_from_slice(&w.to_be_bytes());
    b.extend_from_slice(&h.to_be_bytes());
    b
}

/// A copy of the example project in a fresh folder, and a place to take an image from.
fn project_dir(name: &str) -> (PathBuf, PathBuf) {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&root);
    let dir = root.join("projekt.klappe");
    copy_dir(Path::new(EXAMPLE), &dir);
    let source = root.join("saal.png");
    std::fs::write(&source, png(1000, 500)).unwrap();

    (dir, source)
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
            continue;
        }
        std::fs::copy(entry.path(), target).unwrap();
    }
}

/// The folder with `bilder/saal.png` (1000 x 500 px) set 10 m wide at (2, 1), turned 30°.
fn with_background(name: &str) -> (Session, PathBuf) {
    let (dir, source) = project_dir(name);
    std::fs::create_dir_all(dir.join("bilder")).unwrap();
    std::fs::copy(&source, dir.join(IMAGE)).unwrap();

    let mut loaded = klappe_store::load(&dir).unwrap();
    loaded.project.locations[0].set.background = Some(Background {
        file: IMAGE.into(),
        at: p(2.0, 1.0),
        width_m: 10.0,
        rotation_deg: 30.0,
        opacity: 0.8,
    });
    klappe_store::save(&dir, &loaded.project, loaded.script.as_deref()).unwrap();

    (Session::open(&dir).unwrap(), dir)
}

fn background(s: &Session) -> Option<Background> {
    s.project().locations[0].set.background.clone()
}

#[test]
fn sets_a_background_as_wide_as_the_set_and_it_survives_save_and_undo() {
    let (dir, source) = project_dir("bg-set");
    let mut s = Session::open(&dir).unwrap();

    s.edit_plan(PlanEdit::SetBackground { source }).unwrap();

    // The example's set is 20 m wide.
    assert_eq!(
        background(&s),
        Some(Background {
            file: IMAGE.into(),
            at: p(0.0, 0.0),
            width_m: 20.0,
            rotation_deg: 0.0,
            opacity: 0.5,
        })
    );
    assert!(dir.join(IMAGE).is_file());

    s.save().unwrap();
    let reopened = Session::open(&dir).unwrap();
    assert_eq!(background(&reopened), background(&s));

    // Undo takes the model back; the copied file stays.
    s.undo();
    assert_eq!(background(&s), None);
    assert!(dir.join(IMAGE).is_file());
}

#[test]
fn a_file_that_is_no_image_changes_nothing() {
    let (dir, source) = project_dir("bg-bad-file");
    std::fs::write(&source, b"no image").unwrap();
    let mut s = Session::open(&dir).unwrap();

    let result = s.edit_plan(PlanEdit::SetBackground { source });

    assert!(matches!(result, Err(ServiceError::Store(_))));
    assert_eq!(background(&s), None);
    assert!(!s.is_dirty());
}

#[test]
fn a_preview_copies_no_file() {
    let (dir, source) = project_dir("bg-preview");
    let s = Session::open(&dir).unwrap();

    s.preview(&PlanEdit::SetBackground { source }).unwrap();

    assert!(!dir.join("bilder").exists());
}

/// 1000 px set 10 m wide at (2, 1). Two points 4 m apart on the plan are 5 m in truth: the
/// image grows by 5 / 4 = 1.25 to 12.5 m. Its corner is 2 m left of and 2 m above (4, 3), so
/// it moves to 2.5 m left and above: (1.5, 0.5). The turn stays.
#[test]
fn calibrating_scales_the_image_about_the_first_point() {
    let (mut s, _) = with_background("bg-calibrate");

    s.edit_plan(PlanEdit::Calibrate {
        from: p(4.0, 3.0),
        to: p(8.0, 3.0),
        distance_m: 5.0,
    })
    .unwrap();

    let bg = background(&s).unwrap();
    assert_eq!(bg.width_m, 12.5);
    assert_eq!(bg.at, p(1.5, 0.5));
    assert_eq!(bg.rotation_deg, 30.0);
    assert_eq!(bg.opacity, 0.8);

    // 12.5 m wide, 1000 x 500 px: 6.25 m deep.
    let depth_m = s.display_list().items.iter().find_map(|i| match i.shape {
        Shape::Image { depth_m, .. } => Some(depth_m),
        _ => None,
    });
    assert_eq!(depth_m, Some(6.25));
}

/// The measured stretch is not along an axis: 3-4-5 triangle, 5 m shown, 2 m in truth.
#[test]
fn calibrating_works_on_a_slanted_stretch() {
    let (mut s, _) = with_background("bg-slanted");

    s.edit_plan(PlanEdit::Calibrate {
        from: p(2.0, 1.0),
        to: p(5.0, 5.0),
        distance_m: 2.0,
    })
    .unwrap();

    // From the corner itself: the corner stays, the width shrinks by 2 / 5.
    let bg = background(&s).unwrap();
    assert_eq!(bg.at, p(2.0, 1.0));
    assert_eq!(bg.width_m, 4.0);
}

#[test]
fn calibrating_refuses_what_has_no_scale() {
    let (mut s, _) = with_background("bg-refuse");
    let before = background(&s);
    let calibrate = |from, to, distance_m| PlanEdit::Calibrate {
        from,
        to,
        distance_m,
    };

    let same_point = calibrate(p(1.0, 1.0), p(1.0, 1.0), 3.0);
    let no_distance = calibrate(p(0.0, 0.0), p(1.0, 0.0), 0.0);
    let negative = calibrate(p(0.0, 0.0), p(1.0, 0.0), -2.0);
    let not_a_number = calibrate(p(0.0, 0.0), p(1.0, 0.0), f64::NAN);
    let infinite = calibrate(p(0.0, 0.0), p(1.0, 0.0), f64::INFINITY);
    let bad_point = calibrate(p(f64::NAN, 0.0), p(1.0, 0.0), 2.0);

    for edit in [
        same_point,
        no_distance,
        negative,
        not_a_number,
        infinite,
        bad_point,
    ] {
        let result = s.edit_plan(edit);
        assert!(matches!(result, Err(ServiceError::BadValue { .. })));
    }
    assert_eq!(background(&s), before);
    assert!(!s.is_dirty());
}

#[test]
fn without_a_background_there_is_nothing_to_change() {
    let (dir, _) = project_dir("bg-none");
    let mut s = Session::open(&dir).unwrap();

    let edits = [
        PlanEdit::Calibrate {
            from: p(0.0, 0.0),
            to: p(1.0, 0.0),
            distance_m: 1.0,
        },
        PlanEdit::MoveBackground {
            dx_m: 1.0,
            dy_m: 1.0,
        },
        PlanEdit::RemoveBackground,
    ];
    for edit in edits {
        assert!(matches!(s.edit_plan(edit), Err(ServiceError::Gone)));
    }
}

#[test]
fn moves_and_removes_with_undo() {
    let (mut s, dir) = with_background("bg-move");

    s.edit_plan(PlanEdit::MoveBackground {
        dx_m: 1.5,
        dy_m: -0.5,
    })
    .unwrap();
    let moved = background(&s).unwrap();
    assert_eq!(moved.at, p(3.5, 0.5));
    assert_eq!(moved.width_m, 10.0);

    s.edit_plan(PlanEdit::RemoveBackground).unwrap();
    assert_eq!(background(&s), None);
    assert!(dir.join(IMAGE).is_file());

    s.undo();
    assert_eq!(background(&s), Some(moved));
}

#[test]
fn the_image_is_the_hindmost_item_of_the_set_layer_and_cannot_be_picked() {
    let (mut s, _) = with_background("bg-list");

    let list = s.display_list();
    let first = list.items.first().unwrap();
    assert_eq!(first.role, Role::Background);
    assert!(matches!(
        &first.shape,
        Shape::Image { file, width_m, depth_m, rotation_deg, opacity, .. }
            if file == IMAGE && *width_m == 10.0 && *depth_m == 5.0
                && *rotation_deg == 30.0 && *opacity == 0.8
    ));
    // Inside the image, where no wall, object or figure is.
    assert_eq!(s.pick(p(3.0, 2.0)), None);

    s.set_layer(Layer::Set, Visibility::Hidden);
    let hidden = s.display_list();
    assert!(hidden.items.iter().all(|i| i.role != Role::Background));
}

/// A missing file draws nothing (the plan still opens); the size of a read one is remembered.
#[test]
fn a_missing_image_is_left_out_and_a_known_size_is_remembered() {
    let (s, dir) = with_background("bg-missing");
    let has_image = |s: &Session| {
        s.display_list()
            .items
            .iter()
            .any(|i| i.role == Role::Background)
    };

    assert!(has_image(&s));
    std::fs::remove_file(dir.join(IMAGE)).unwrap();
    assert!(has_image(&s));

    let fresh = Session::open(&dir).unwrap();
    assert!(!has_image(&fresh));
}
