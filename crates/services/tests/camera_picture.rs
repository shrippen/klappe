//! The camera's picture (LH-36), on the example project: scene 12, shots `s12-1` to `s12-5`,
//! cameras `a` and `b`.
#![allow(clippy::unwrap_used)]

use std::path::Path;

use klappe_services::{SceneRole, Session};

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
