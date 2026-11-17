//! The free 3D view (LH-94): plan and 3D are two views of one set.
#![allow(clippy::unwrap_used)]

use std::path::Path;

use klappe_model::Point;
use klappe_services::{Cut, PlanEdit, Session, Source, View, ViewStep};

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

type Leave = fn(&mut Session);

/// The look through a camera belongs to the plan it began on: elsewhere, or without the
/// camera, the view is the orbit again.
#[test]
fn the_look_ends_when_the_plan_or_the_camera_goes() {
    let leaves: [(&str, Leave); 4] = [
        ("add_plan", |s| {
            s.add_plan("Leer").unwrap();
        }),
        ("copy_plan", |s| {
            s.copy_plan("").unwrap();
        }),
        ("remove camera", |s| {
            s.edit_plan(PlanEdit::Remove {
                source: Source::Camera("c".into()),
            })
            .unwrap();
        }),
        ("undo creation", |s| {
            s.undo();
        }),
    ];

    for (name, leave) in leaves {
        let mut s = example();
        s.toggle_view();
        s.edit_plan(PlanEdit::AddCamera {
            at: Point::new(2.0, 2.0),
            facing_deg: 0.0,
        })
        .unwrap();
        assert!(s.look_through(Some("c")), "{name}");

        leave(&mut s);

        assert!(!s.looks_through(), "{name}");
        assert!(s.free_picture(SIZE).is_some(), "{name}");
    }
}

/// Another plan of the scene is not looked through, though it has a camera of that id.
#[test]
fn choosing_another_plan_ends_the_look() {
    let mut s = example();
    let first = s.plan().unwrap().id.clone();
    s.copy_plan("").unwrap();
    s.toggle_view();
    assert!(s.look_through(Some("a")));

    s.select_plan(&first).unwrap();

    assert!(!s.looks_through());
    assert!(s.free_picture(SIZE).is_some());
}

fn looking_at_new_camera() -> Session {
    let mut s = example();
    s.toggle_view();
    s.edit_plan(PlanEdit::AddCamera {
        at: Point::new(2.0, 2.0),
        facing_deg: 0.0,
    })
    .unwrap();
    assert!(s.look_through(Some("c")));
    s
}

fn remove_camera_c(s: &mut Session) {
    s.edit_plan(PlanEdit::Remove {
        source: Source::Camera("c".into()),
    })
    .unwrap();
}

/// An ended look stays ended: coming back to the plan does not revive it.
#[test]
fn the_look_does_not_come_back_with_the_plan() {
    let mut s = example();
    let first = s.plan().unwrap().id.clone();
    s.copy_plan("").unwrap();
    s.toggle_view();
    assert!(s.look_through(Some("a")));

    s.select_plan(&first).unwrap();
    let second = s.scene().unwrap().plans[1].id.clone();
    s.select_plan(&second).unwrap();

    assert!(!s.looks_through());
}

/// Undoing the removal brings the camera back, not the look.
#[test]
fn undoing_the_removal_of_the_camera_does_not_revive_the_look() {
    let mut s = looking_at_new_camera();

    remove_camera_c(&mut s);
    s.undo();

    assert!(!s.looks_through());
}

/// A new camera gets the freed id; the look is not for it.
#[test]
fn a_new_camera_with_the_same_id_is_not_looked_through() {
    let mut s = looking_at_new_camera();

    remove_camera_c(&mut s);
    s.edit_plan(PlanEdit::AddCamera {
        at: Point::new(4.0, 1.0),
        facing_deg: 90.0,
    })
    .unwrap();

    assert!(!s.looks_through());
}

/// Edits that leave the camera and plan alone do not end the look.
#[test]
fn unrelated_edits_keep_the_look() {
    let mut s = looking_at_new_camera();

    s.edit_plan(PlanEdit::AddCamera {
        at: Point::new(5.0, 5.0),
        facing_deg: 0.0,
    })
    .unwrap();

    assert!(s.looks_through());
}

/// The mouse's way to do what a key step does.
type Drag = fn(&mut Session);

/// NF-06: each key is the drag of the same name, so the picture is the mouse's. A pan key shows
/// more of its side, as on the plan: the content moves the other way.
#[test]
fn a_key_step_is_the_drag_of_the_same_name() {
    // Turning 15 degrees is 50 px of drag at 0.3 degrees per px; a shift is a tenth of the width.
    let pairs: [(ViewStep, Drag); 9] = [
        (ViewStep::TurnLeft, |s| s.turn_view(-50.0, 0.0)),
        (ViewStep::TurnRight, |s| s.turn_view(50.0, 0.0)),
        (ViewStep::TurnUp, |s| s.turn_view(0.0, -50.0)),
        (ViewStep::TurnDown, |s| s.turn_view(0.0, 50.0)),
        (ViewStep::PanLeft, |s| s.pan_view(32.0, 0.0, SIZE)),
        (ViewStep::PanRight, |s| s.pan_view(-32.0, 0.0, SIZE)),
        (ViewStep::PanUp, |s| s.pan_view(0.0, 32.0, SIZE)),
        (ViewStep::PanDown, |s| s.pan_view(0.0, -32.0, SIZE)),
        (ViewStep::ZoomIn, |s| s.zoom_view(1.0, (160.0, 120.0), SIZE)),
    ];

    for (step, drag) in pairs {
        let (mut keyed, mut dragged) = (example(), example());
        keyed.toggle_view();
        dragged.toggle_view();
        let start = keyed.free_picture(SIZE).unwrap();

        keyed.step_view(step, SIZE);
        drag(&mut dragged);

        let picture = keyed.free_picture(SIZE).unwrap();
        assert_ne!(picture, start, "{step:?} changes nothing");
        assert_eq!(picture, dragged.free_picture(SIZE).unwrap(), "{step:?}");
        assert!(!keyed.can_undo(), "{step:?} is an edit");
    }
}

#[test]
fn zooming_out_by_key_undoes_zooming_in() {
    let mut s = example();
    s.toggle_view();
    let middle = s.free_floor_at((160.0, 120.0), SIZE).unwrap();

    s.step_view(ViewStep::ZoomIn, SIZE);
    s.step_view(ViewStep::ZoomOut, SIZE);

    let after = s.free_floor_at((160.0, 120.0), SIZE).unwrap();
    assert!((after.x_m - middle.x_m).abs() < 1e-6 && (after.y_m - middle.y_m).abs() < 1e-6);
}

/// 24 turns of 15 degrees make a full circle: the sideways key never runs out.
#[test]
fn turning_by_key_goes_round_and_tilting_stops_at_the_ends() {
    let mut s = example();
    s.toggle_view();
    let corner = (40.0, 200.0);
    let start = s.free_floor_at(corner, SIZE).unwrap();

    for _ in 0..24 {
        s.step_view(ViewStep::TurnLeft, SIZE);
    }
    let round = s.free_floor_at(corner, SIZE).unwrap();
    assert!((round.x_m - start.x_m).abs() < 1e-6 && (round.y_m - start.y_m).abs() < 1e-6);

    for _ in 0..20 {
        s.step_view(ViewStep::TurnDown, SIZE);
    }
    let down = s.free_picture(SIZE).unwrap();
    s.step_view(ViewStep::TurnDown, SIZE);
    assert_eq!(
        s.free_picture(SIZE).unwrap(),
        down,
        "straight down is the end"
    );
}

#[test]
fn a_key_step_leaves_the_look_through_a_camera() {
    let mut s = example();
    s.toggle_view();
    assert!(s.look_through(Some("a")));

    s.step_view(ViewStep::PanLeft, SIZE);

    assert!(!s.looks_through());
}

#[test]
fn the_plan_ignores_key_steps() {
    let mut s = example();

    s.step_view(ViewStep::TurnLeft, SIZE);
    s.toggle_view();
    let mut fresh = example();
    fresh.toggle_view();

    assert_eq!(s.free_picture(SIZE), fresh.free_picture(SIZE));
}

#[test]
fn step_names_are_read_back() {
    assert_eq!(ViewStep::from_name("turnLeft"), Some(ViewStep::TurnLeft));
    assert_eq!(ViewStep::from_name("zoomOut"), Some(ViewStep::ZoomOut));
    assert_eq!(ViewStep::from_name("spin"), None);
}
