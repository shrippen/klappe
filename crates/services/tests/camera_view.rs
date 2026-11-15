//! The camera's view (LH-36), on the example project: scene 12, shots `s12-1` to `s12-5`,
//! cameras `a` and `b`.
#![allow(clippy::unwrap_used)]

use std::path::Path;

use klappe_services::{Session, ViewRole};

fn open() -> Session {
    Session::open(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe"))
        .unwrap()
}

#[test]
fn the_selected_shots_camera_gives_the_view() {
    let mut s = open();
    s.select_shot("s12-1").unwrap();

    let view = s.camera_view(None).unwrap();

    assert!(view.items.iter().any(|i| i.role == ViewRole::Floor));
}

#[test]
fn without_a_shot_the_picked_camera_gives_it() {
    let s = open();

    assert!(s.camera_view(None).is_none());
    assert!(s.camera_view(Some("a")).is_some());
    assert!(s.camera_view(Some("none")).is_none());
}

#[test]
fn the_view_follows_the_playhead() {
    let mut s = open();
    s.set_time(0.0);
    let before = s.camera_view(Some("a")).unwrap();

    s.set_time(14.0);

    assert_ne!(s.camera_view(Some("a")).unwrap(), before);
}
