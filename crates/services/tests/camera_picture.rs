//! The camera's picture (LH-36), on the example project: scene 12, shots `s12-1` to `s12-5`,
//! cameras `a` and `b`.
#![allow(clippy::unwrap_used)]

use std::path::Path;

use klappe_services::{Paper, SceneRole, ServiceError, Session, ShotOrder};

const WIDTH_PX: usize = 120;

fn open() -> Session {
    Session::open(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe"))
        .unwrap()
}

#[test]
fn the_selected_shots_camera_gives_the_picture() {
    let mut s = open();
    s.select_shot("s12-1").unwrap();

    let picture = s.camera_picture(None, WIDTH_PX).unwrap();

    assert_eq!(picture.frame.width, WIDTH_PX);
    assert!(picture.frame.role.contains(&SceneRole::Floor.code()));
}

#[test]
fn without_a_shot_the_picked_camera_gives_it() {
    let s = open();

    assert!(s.camera_picture(None, WIDTH_PX).is_none());
    assert!(s.camera_picture(Some("a"), WIDTH_PX).is_some());
    assert!(s.camera_picture(Some("none"), WIDTH_PX).is_none());
}

#[test]
fn the_picture_follows_the_playhead() {
    let mut s = open();
    s.set_time(0.0);
    let before = s.camera_picture(Some("a"), WIDTH_PX).unwrap();

    s.set_time(14.0);

    assert_ne!(s.camera_picture(Some("a"), WIDTH_PX).unwrap(), before);
}

/// LH-36: a tile shows the shot's camera when the shot starts, wherever the playhead stands.
#[test]
fn a_shot_shows_its_camera_at_its_start() {
    let mut s = open();
    s.set_time(0.0);
    let at_start = s.shot_picture("s12-2", WIDTH_PX).unwrap();
    s.set_time(14.0);

    assert_eq!(s.shot_picture("s12-2", WIDTH_PX).unwrap(), at_start);
    assert_ne!(s.shot_picture("s12-4", WIDTH_PX).unwrap(), at_start);
    assert!(s.shot_picture("nope", WIDTH_PX).is_none());
}

const PNG_SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
/// Where the width sits in a PNG file; the height follows.
const PNG_WIDTH_AT: usize = 16;
const EXPORT_WIDTH_PX: u32 = 1920;

fn png_size(path: &Path) -> (u32, u32) {
    let bytes = std::fs::read(path).unwrap();
    assert!(bytes.starts_with(&PNG_SIGNATURE));
    let number = |at: usize| u32::from_be_bytes(bytes[at..at + 4].try_into().unwrap());

    (number(PNG_WIDTH_AT), number(PNG_WIDTH_AT + 4))
}

fn out(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join(name)
}

/// LH-86: the camera view as PNG, in the camera's aspect, of the selected shot's camera.
#[test]
fn the_camera_view_is_written_as_png() {
    let mut s = open();
    s.select_shot("s12-1").unwrap();
    let path = out("camera-view.png");

    s.export_camera_png(&path, None).unwrap();

    let (width, height) = png_size(&path);
    let picture = s.camera_picture(None, EXPORT_WIDTH_PX as usize).unwrap();
    assert_eq!(width, EXPORT_WIDTH_PX);
    assert_eq!(height as usize, picture.frame.height);
}

/// Without a shot the picked camera counts; with neither there is nothing to write.
#[test]
fn the_camera_view_needs_a_camera() {
    let s = open();
    let path = out("camera-none.png");
    let _ = std::fs::remove_file(&path);

    assert!(matches!(
        s.export_camera_png(&path, None),
        Err(ServiceError::NoPicture)
    ));
    assert!(!path.exists());
    s.export_camera_png(&path, Some("a")).unwrap();
    assert!(path.exists());
}

/// LH-86: the 3D view as PNG keeps the aspect it has on screen, whatever the screen's size.
#[test]
fn the_3d_view_is_written_as_png_in_the_screens_aspect() {
    let mut s = open();
    let path = out("view-3d.png");

    // Not opened yet: no orbit, nothing to show.
    assert!(matches!(
        s.export_view_png(&path, (800.0, 600.0)),
        Err(ServiceError::NoPicture)
    ));

    s.toggle_view();
    s.export_view_png(&path, (800.0, 600.0)).unwrap();

    assert_eq!(png_size(&path), (EXPORT_WIDTH_PX, 1440));
    assert!(matches!(
        s.export_view_png(&path, (0.0, 600.0)),
        Err(ServiceError::NoPicture)
    ));
}

/// The 3D view through a camera, cut, is what the PNG shows: it differs from the orbit's.
#[test]
fn the_3d_png_shows_the_view_through_the_camera() {
    let mut s = open();
    s.toggle_view();
    s.select_shot("s12-1").unwrap();
    let (orbit, through) = (out("view-orbit.png"), out("view-through.png"));
    s.export_view_png(&orbit, (800.0, 450.0)).unwrap();

    assert!(s.look_through(None));
    s.export_view_png(&through, (800.0, 450.0)).unwrap();

    assert_ne!(
        std::fs::read(&orbit).unwrap(),
        std::fs::read(&through).unwrap()
    );
}

/// LH-86: the PDF ends with the camera views of the shots, five shots on one page, each with
/// its number.
#[test]
fn the_pdf_ends_with_the_camera_views() {
    let s = open();
    let path = out("camera-views.pdf");
    let texts = klappe_services::Language::German.print_texts();

    s.export_pdf(&path, Paper::A4, texts, ShotOrder::Cut)
        .unwrap();

    let text = String::from_utf8_lossy(&std::fs::read(&path).unwrap()).into_owned();
    assert_eq!(text.matches("/Type /Page /").count(), 3);
    assert!(text.contains("Kamerablicke)"));
    for shot in 1..=5 {
        assert!(text.contains(&format!("(12.{shot} ")), "{shot}");
    }
}

#[test]
fn edits_name_the_state_the_tiles_are_drawn_from() {
    let mut s = open();
    let opened = s.edits();

    s.select_shot("s12-1").unwrap();
    s.set_time(3.0);
    assert_eq!(s.edits(), opened, "a selection or the playhead is no edit");

    s.copy_plan("Stand 1").unwrap();
    let copied = s.edits();
    assert_ne!(copied, opened);

    s.undo();
    let undone = s.edits();
    assert_ne!(undone, copied);

    s.redo();
    assert_ne!(s.edits(), undone);

    assert_ne!(open().edits(), opened, "another session never repeats one");
}
