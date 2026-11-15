//! The free 3D view (LH-94): plan and 3D are two views of one set.
#![allow(clippy::unwrap_used)]

use std::path::Path;

use klappe_services::{Cut, Session, Source, View};

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

/// A drag up lifts what it grabbed, a drag down lowers it, the same point lifts nothing; a start
/// in the sky has no level to slide on (LH-95).
#[test]
fn a_pointer_up_in_3d_lifts_and_down_lowers() {
    let mut s = example();
    s.toggle_view();
    let picture = s.free_picture(SIZE).unwrap();
    let i = picture
        .frame
        .source
        .iter()
        .position(|n| matches!(picture.sources.get(*n as usize), Some(Source::Object(_))))
        .unwrap();
    let from = ((i % 320) as f64 + 0.5, (i / 320) as f64 + 0.5);
    let (_, held_m) = s.free_grab(from, SIZE).unwrap();

    let up = s
        .free_lift(from, (from.0, from.1 - 20.0), SIZE, held_m)
        .unwrap();
    let down = s
        .free_lift(from, (from.0, from.1 + 20.0), SIZE, held_m)
        .unwrap();
    let same = s.free_lift(from, from, SIZE, held_m).unwrap();

    assert!(up > 0.0, "up {up}");
    assert!(down < 0.0, "down {down}");
    assert!(same.abs() < 1e-9, "same {same}");

    s.turn_view(0.0, -1000.0);
    assert!(s.free_lift((160.0, 0.0), (160.0, 5.0), SIZE, 0.0).is_none());
}

/// Key Y (LH-98): the cut changes the picture and goes back to the same one; it is a setting
/// of the view, no edit.
#[test]
fn the_cut_changes_the_picture_and_is_no_edit() {
    let mut s = example();
    s.toggle_view();
    let high = s.free_picture(SIZE).unwrap();
    assert_eq!(s.cut(), Cut::Off);

    s.toggle_cut();
    assert_eq!(s.cut(), Cut::Chest);
    assert_ne!(s.free_picture(SIZE).unwrap(), high);
    assert!(!s.can_undo());

    s.toggle_cut();
    assert_eq!(s.free_picture(SIZE).unwrap(), high);
}

/// The camera picture of the shot card is never cut.
#[test]
fn the_cut_leaves_the_shot_cards_picture_alone() {
    let mut s = example();
    s.select_shot("s12-1").unwrap();
    let before = s.camera_picture(None, 120).unwrap();

    s.toggle_cut();

    assert_eq!(s.camera_picture(None, 120).unwrap(), before);
}

/// LH-99: looking through the selected shot's camera shows what `camera_picture` shows.
#[test]
fn the_view_can_look_through_the_selected_shots_camera() {
    let mut s = example();
    s.select_shot("s12-1").unwrap();
    let shot = s.camera_picture(None, 320).unwrap();
    // The view has the camera's shape, so the picture is the same pixel for pixel.
    let plan = s.plan().unwrap();
    let shot_camera = &plan.shots.iter().find(|x| x.id == "s12-1").unwrap().camera;
    let size = (320.0, 320.0 / plan.camera(shot_camera).unwrap().aspect);
    assert!(!s.look_through(None), "only in 3D");

    s.toggle_view();
    let orbit = s.free_picture(size).unwrap();
    assert!(s.look_through(None));

    assert!(s.looks_through());
    assert_eq!(s.free_picture(size).unwrap(), shot);
    assert_ne!(shot.frame.role, orbit.frame.role);

    // The same command goes back.
    assert!(!s.look_through(None));
    assert_eq!(s.free_picture(size).unwrap(), orbit);
}

/// Without a shot the camera picked on the plan serves; no camera, no look.
#[test]
fn the_picked_camera_serves_without_a_shot() {
    let mut s = example();
    s.toggle_view();

    assert!(!s.look_through(None));
    assert!(!s.look_through(Some("none")));
    assert!(s.look_through(Some("a")));
}

/// Time changes move the eye; the picture is the one of the playhead.
#[test]
fn the_view_through_a_camera_follows_the_playhead() {
    let mut s = example();
    s.toggle_view();
    s.set_time(0.0);
    assert!(s.look_through(Some("a")));
    let before = s.free_picture(SIZE).unwrap();

    s.set_time(14.0);

    assert_ne!(s.free_picture(SIZE).unwrap(), before);
}

/// Turning, shifting and zooming leave the camera and go on from the last orbit: the picture
/// is the one of a session that never looked through it.
#[test]
fn navigating_leaves_the_camera() {
    let moves: [fn(&mut Session); 3] = [
        |s| s.turn_view(30.0, 5.0),
        |s| s.pan_view(20.0, 10.0, SIZE),
        |s| s.zoom_view(1.0, (100.0, 90.0), SIZE),
    ];

    for navigate in moves {
        let mut s = example();
        s.toggle_view();
        let mut control = example();
        control.toggle_view();

        assert!(s.look_through(Some("a")));
        navigate(&mut s);
        navigate(&mut control);

        assert!(!s.looks_through());
        assert_eq!(
            s.free_picture(SIZE).unwrap(),
            control.free_picture(SIZE).unwrap()
        );
    }
}

/// A click through the camera's picture picks what is seen there (the same functions).
#[test]
fn clicks_work_through_a_camera() {
    let mut s = example();
    s.toggle_view();
    assert!(s.look_through(Some("a")));
    let picture = s.free_picture(SIZE).unwrap();
    let i = picture
        .frame
        .source
        .iter()
        .position(|n| picture.sources.get(*n as usize).is_some())
        .unwrap();
    let pointer = ((i % 320) as f64 + 0.5, (i / 320) as f64 + 0.5);

    assert!(s.free_pick(pointer, SIZE).is_some());
    assert!(s.free_floor_at((160.0, 230.0), SIZE).is_some());
}

/// The camera looked through is not drawn; the others are.
#[test]
fn the_camera_looked_through_is_not_drawn() {
    let mut s = example();
    s.toggle_view();
    let drawn = |s: &Session| {
        let p = s.free_picture(SIZE).unwrap();
        p.sources
            .iter()
            .any(|source| matches!(source, Source::Camera(id) if id == "a"))
    };

    assert!(s.look_through(Some("a")));

    assert!(!drawn(&s));
}
