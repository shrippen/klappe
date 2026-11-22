//! The opacity of a placed image and swapping its file (LH-18), on a copy of the example
//! project. The images are bare PNG headers: only the pixel size is ever read from them.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_model::{PlanImage, MIN_IMAGE_OPACITY};
use klappe_services::{
    Field, Hint, Layer, Lock, PlanEdit, Point, ServiceError, Session, Shape, Snapping, Source,
};

const EXAMPLE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../testdata/beispiel.klappe"
);
const FILE: &str = "bilder/skizze.png";

fn p(x_m: f64, y_m: f64) -> Point {
    Point::new(x_m, y_m)
}

/// A PNG header of `w` x `h` pixels, 8-bit RGB.
fn png(w: u32, h: u32) -> Vec<u8> {
    let mut b = b"\x89PNG\r\n\x1a\n".to_vec();
    b.extend_from_slice(&13_u32.to_be_bytes());
    b.extend_from_slice(b"IHDR");
    b.extend_from_slice(&w.to_be_bytes());
    b.extend_from_slice(&h.to_be_bytes());
    b.extend_from_slice(&[8, 2, 0, 0, 0]);
    b
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

/// A copy of the example in a fresh folder, and `skizze.png` (1000 x 500 px) beside it.
fn project_dir(name: &str) -> (PathBuf, PathBuf) {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&root);
    let dir = root.join("projekt.klappe");
    copy_dir(Path::new(EXAMPLE), &dir);
    let source = root.join("skizze.png");
    std::fs::write(&source, png(1000, 500)).unwrap();

    (dir, source)
}

/// A session whose set has the image `i1`, 2 m wide, centred on (10, 3), not turned; its file
/// is in `bilder/`. Snapping is off, so a drag lands where it says.
fn with_image(name: &str) -> (Session, PathBuf) {
    let (dir, source) = project_dir(name);
    let mut s = Session::open(&dir).unwrap();
    s.set_snapping(Snapping::Off);
    s.edit_plan(PlanEdit::AddImage {
        source,
        at: p(10.0, 3.0),
    })
    .unwrap();

    (s, dir)
}

fn images(s: &Session) -> Vec<PlanImage> {
    s.project().locations[0].set.images.clone()
}

fn image() -> Source {
    Source::Image("i1".into())
}

fn opacity(opacity: f64) -> PlanEdit {
    PlanEdit::SetImageOpacity {
        source: image(),
        opacity,
    }
}

/// A second file beside the project, 300 x 600 px.
fn other_file(dir: &Path) -> PathBuf {
    let other = dir.join("../zweite.png");
    std::fs::write(&other, png(300, 600)).unwrap();

    other
}

#[test]
fn a_new_image_is_opaque() {
    let (s, _) = with_image("card-new");

    assert_eq!(images(&s)[0].opacity, 1.0);
}

#[test]
fn sets_the_opacity_and_undoes_in_one_step() {
    let (mut s, _) = with_image("card-opacity");

    s.edit_plan(opacity(0.4)).unwrap();
    assert_eq!(images(&s)[0].opacity, 0.4);
    assert_eq!(
        s.image_hint("i1"),
        Some(Hint::Image {
            file: FILE.into(),
            opacity: 0.4
        })
    );

    s.undo();
    assert_eq!(images(&s)[0].opacity, 1.0);
    // Only the opacity edit was undone: the image is still there.
    assert_eq!(images(&s).len(), 1);
}

#[test]
fn the_picture_is_drawn_with_the_opacity() {
    let (mut s, _) = with_image("card-drawn");
    s.edit_plan(opacity(0.25)).unwrap();

    let list = s.display_list();
    let drawn = list.items.iter().find_map(|i| match i.shape {
        Shape::Image { opacity, .. } => Some(opacity),
        _ => None,
    });

    assert_eq!(drawn, Some(0.25));
}

#[test]
fn an_opacity_out_of_range_is_refused_and_changes_nothing() {
    let (mut s, _) = with_image("card-range");

    for value in [
        0.0,
        MIN_IMAGE_OPACITY / 2.0,
        1.1,
        -1.0,
        f64::NAN,
        f64::INFINITY,
    ] {
        assert!(
            matches!(
                s.edit_plan(opacity(value)),
                Err(ServiceError::BadValue {
                    field: Field::Opacity,
                    ..
                })
            ),
            "{value}"
        );
    }
    assert_eq!(images(&s)[0].opacity, 1.0);

    // The limits themselves are allowed.
    s.edit_plan(opacity(MIN_IMAGE_OPACITY)).unwrap();
    s.edit_plan(opacity(1.0)).unwrap();
}

#[test]
fn only_an_image_takes_an_opacity() {
    let (mut s, _) = with_image("card-source");

    for source in [
        Source::Wall(0),
        Source::Note("n1".into()),
        Source::Image("i9".into()),
    ] {
        assert!(s
            .edit_plan(PlanEdit::SetImageOpacity {
                source,
                opacity: 0.5
            })
            .is_err());
    }
}

#[test]
fn a_copy_keeps_the_opacity() {
    let (mut s, _) = with_image("card-copy");
    s.edit_plan(opacity(0.6)).unwrap();

    s.edit_plan(PlanEdit::Duplicate {
        sources: vec![image()],
        dx_m: 1.0,
        dy_m: 0.0,
    })
    .unwrap();

    assert_eq!(images(&s)[1].opacity, 0.6);
}

#[test]
fn replaces_the_file_and_keeps_the_rest() {
    let (mut s, dir) = with_image("card-replace");
    s.edit_plan(opacity(0.5)).unwrap();
    s.edit_plan(PlanEdit::Spin {
        source: image(),
        by_deg: 30.0,
    })
    .unwrap();
    s.edit_plan(PlanEdit::Shift {
        source: Source::ImageSize("i1".into()),
        dx_m: 1.0,
        dy_m: 0.0,
    })
    .unwrap();
    let before = images(&s)[0].clone();

    s.edit_plan(PlanEdit::ReplaceImage {
        source: image(),
        with: other_file(&dir),
    })
    .unwrap();

    let after = images(&s);
    assert_eq!(after.len(), 1);
    assert_eq!(after[0].file, "bilder/zweite.png");
    assert_eq!(
        PlanImage {
            file: FILE.into(),
            ..after[0].clone()
        },
        before
    );
    assert!(dir.join("bilder/zweite.png").is_file());

    // The height follows the new file: 300 x 600 px at the kept width.
    let list = s.display_list();
    let depth = list.items.iter().find_map(|i| match i.shape {
        Shape::Image { depth_m, .. } => Some(depth_m),
        _ => None,
    });
    assert_eq!(depth, Some(after[0].width_m * 2.0));
}

#[test]
fn undoing_a_replacement_gives_the_old_file_back_in_one_step() {
    let (mut s, dir) = with_image("card-replace-undo");
    let before = images(&s);

    s.edit_plan(PlanEdit::ReplaceImage {
        source: image(),
        with: other_file(&dir),
    })
    .unwrap();
    s.undo();

    assert_eq!(images(&s), before);
    // Both copies are the user's files.
    assert!(dir.join(FILE).is_file());
    assert!(dir.join("bilder/zweite.png").is_file());
}

#[test]
fn a_replacement_that_is_no_image_is_refused_and_leaves_the_image() {
    let (mut s, dir) = with_image("card-replace-bad");
    let text = dir.join("notiz.txt");
    std::fs::write(&text, "x").unwrap();
    let before = images(&s);

    for with in [text, dir.join("fehlt.png")] {
        let source = image();
        assert!(s
            .edit_plan(PlanEdit::ReplaceImage { source, with })
            .is_err());
    }

    assert_eq!(images(&s), before);
    assert!(s.can_undo());
}

#[test]
fn only_an_image_takes_a_file() {
    let (mut s, dir) = with_image("card-replace-source");

    for source in [Source::Wall(0), Source::Image("i9".into())] {
        let with = other_file(&dir);
        assert!(s
            .edit_plan(PlanEdit::ReplaceImage { source, with })
            .is_err());
    }
}

#[test]
fn the_notes_layer_locks_the_opacity_and_the_replacement() {
    let (mut s, dir) = with_image("card-lock");
    s.set_lock(Layer::Notes, Lock::Locked);

    for edit in [
        opacity(0.5),
        PlanEdit::ReplaceImage {
            source: image(),
            with: other_file(&dir),
        },
    ] {
        assert!(matches!(
            s.edit_plan(edit),
            Err(ServiceError::Locked(Layer::Notes))
        ));
    }
    assert_eq!(images(&s)[0].opacity, 1.0);
    assert_eq!(images(&s)[0].file, FILE);
}

#[test]
fn the_opacity_survives_saving() {
    let (mut s, dir) = with_image("card-save");
    s.edit_plan(opacity(0.3)).unwrap();
    s.save(klappe_services::OnOutside::Overwrite).unwrap();

    assert_eq!(images(&Session::open(&dir).unwrap())[0].opacity, 0.3);
}
