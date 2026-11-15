//! The free 3D view (LH-94): plan and 3D are two views of one set.
#![allow(clippy::unwrap_used)]

use std::path::Path;

use klappe_services::{Session, Source, View};

const EXAMPLE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../testdata/beispiel.klappe"
);
const SIZE: (f64, f64) = (320.0, 240.0);

fn example() -> Session {
    Session::open(Path::new(EXAMPLE)).unwrap()
}

#[test]
fn j_switches_between_plan_and_3d_and_3d_shows_the_set() {
    let mut s = example();
    assert_eq!(s.view(), View::Plan);
    assert!(s.free_picture(SIZE).is_none());

    s.toggle_view();
    assert_eq!(s.view(), View::Free);
    let picture = s.free_picture(SIZE).unwrap();
    assert_eq!((picture.frame.width, picture.frame.height), (320, 240));

    s.toggle_view();
    assert_eq!(s.view(), View::Plan);
}

/// The first 3D view looks at the middle of the set; the floor there is a plan point the tools
/// can use.
#[test]
fn the_middle_of_the_3d_view_is_the_middle_of_the_set() {
    let mut s = example();
    s.toggle_view();

    let middle = s.free_floor_at((160.0, 120.0), SIZE).unwrap();

    let set = &s.project().locations[0].set;
    assert!((middle.x_m - set.width_m / 2.0).abs() < 1e-6, "{middle:?}");
    assert!((middle.y_m - set.depth_m / 2.0).abs() < 1e-6, "{middle:?}");
}

/// Turning, shifting and zooming change the picture; the view is not an edit (no undo step).
#[test]
fn navigating_changes_the_picture_but_is_no_edit() {
    let mut s = example();
    s.toggle_view();
    let before = s.free_picture(SIZE).unwrap();

    s.turn_view(40.0, 10.0);
    s.pan_view(15.0, -5.0, SIZE);
    s.zoom_view(1.0, (100.0, 150.0), SIZE);

    assert_ne!(s.free_picture(SIZE).unwrap(), before);
    assert!(!s.can_undo());
}

/// A click in 3D takes what one sees there: a wall pixel picks that wall.
#[test]
fn a_click_in_3d_takes_what_is_seen() {
    let mut s = example();
    s.toggle_view();
    let picture = s.free_picture(SIZE).unwrap();
    let i = picture
        .frame
        .source
        .iter()
        .position(|n| matches!(picture.sources.get(*n as usize), Some(Source::Wall(_))))
        .unwrap();
    let pointer = ((i % 320) as f64 + 0.5, (i / 320) as f64 + 0.5);

    assert!(matches!(s.free_pick(pointer, SIZE), Some(Source::Wall(_))));
}
