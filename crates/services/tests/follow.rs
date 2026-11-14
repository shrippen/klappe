//! A camera coupled to a figure (LH-37), on the example project: Burgvogt's marks begin at 0 s
//! (holds 5 s, so until 5 s) and 9 s; camera B follows him and has one mark at 0 s, (18, 4)
//! facing 200°. Elisabeth's marks lie at 0, 6 and 14 s with holds of 2, 4 and 0 s; camera A
//! has marks at 0 s and 6 s.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_services::{Layer, Lock, PlanEdit, Point, ServiceError, Session, Snapping, Source};

fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe")
}

fn open() -> Session {
    Session::open(&example()).unwrap()
}

fn p(x_m: f64, y_m: f64) -> Point {
    Point::new(x_m, y_m)
}

fn mark(character: &str, index: usize) -> Source {
    Source::Mark {
        character: character.into(),
        index,
    }
}

fn near(a: Point, b: Point) -> bool {
    a.distance_m(b) < 1e-9
}

/// Position and facing of every mark of camera `id`.
fn camera_marks(s: &Session, id: &str) -> Vec<(Point, f64)> {
    let camera = s.plan().unwrap().camera(id).unwrap();

    camera.marks.iter().map(|m| (m.at, m.facing_deg)).collect()
}

fn couple(s: &mut Session, camera: &str, figure: &str) {
    s.edit_plan(PlanEdit::Follow {
        camera: camera.into(),
        figure: Some(figure.into()),
    })
    .unwrap();
}

#[test]
fn moving_the_figure_carries_the_camera() {
    let mut s = open();

    s.edit_plan(PlanEdit::Move {
        source: mark("burgvogt", 0),
        to: p(15.0, 8.5),
    })
    .unwrap();

    let b = camera_marks(&s, "b");
    assert!(near(b[0].0, p(16.0, 3.0)), "{:?}", b[0].0);
    assert_eq!(b[0].1, 200.0);
}

#[test]
fn shifting_the_figure_carries_the_camera() {
    let mut s = open();
    s.set_snapping(Snapping::Off);

    s.edit_plan(PlanEdit::Shift {
        source: mark("burgvogt", 0),
        dx_m: 1.5,
        dy_m: -2.0,
    })
    .unwrap();

    assert!(near(camera_marks(&s, "b")[0].0, p(19.5, 2.0)));
}

#[test]
fn a_figure_at_the_playhead_carries_it_too() {
    let mut s = open();
    s.set_time(1.0);

    s.edit_plan(PlanEdit::Move {
        source: Source::Figure("burgvogt".into()),
        to: p(16.0, 9.5),
    })
    .unwrap();

    assert!(near(camera_marks(&s, "b")[0].0, p(17.0, 4.0)));
}

/// Burgvogt's second mark (9 s) has no mark of B in its window.
#[test]
fn a_mark_without_a_camera_mark_leaves_the_camera() {
    let mut s = open();
    let before = camera_marks(&s, "b");

    s.edit_plan(PlanEdit::Move {
        source: mark("burgvogt", 1),
        to: p(1.0, 1.0),
    })
    .unwrap();

    assert_eq!(camera_marks(&s, "b"), before);
}

/// Elisabeth's mark 1 spans 6 to 10 s: A's mark at 6 s moves, the one at 0 s does not.
#[test]
fn only_camera_marks_in_the_window_move() {
    let mut s = open();
    couple(&mut s, "a", "elisabeth");
    let before = camera_marks(&s, "a");

    s.edit_plan(PlanEdit::Move {
        source: mark("elisabeth", 1),
        to: p(12.0, 6.5),
    })
    .unwrap();

    let after = camera_marks(&s, "a");
    assert_eq!(after[0], before[0]);
    assert!(near(after[1].0, p(11.0, 8.0)));
}

/// Mark 0 spans 0 to 2 s and mark 2 (14 s) has no camera mark.
#[test]
fn the_window_starts_at_the_arrival() {
    let mut s = open();
    couple(&mut s, "a", "elisabeth");
    let before = camera_marks(&s, "a");

    s.edit_plan(PlanEdit::Move {
        source: mark("elisabeth", 2),
        to: p(1.0, 1.0),
    })
    .unwrap();
    assert_eq!(camera_marks(&s, "a"), before);

    s.edit_plan(PlanEdit::Move {
        source: mark("elisabeth", 0),
        to: p(4.0, 6.5),
    })
    .unwrap();
    let after = camera_marks(&s, "a");
    assert!(near(after[0].0, p(9.0, 11.0)));
    assert_eq!(after[1], before[1]);
}

/// Burgvogt turns from 180° to 270°; B at (18, 4) lies (1, -5.5) from him and swings to
/// (5.5, 1) from him, clockwise on the plan, facing 90° further.
#[test]
fn turning_the_figure_swings_the_camera_about_it() {
    let mut s = open();

    s.edit_plan(PlanEdit::Turn {
        source: mark("burgvogt", 0),
        facing_deg: 270.0,
    })
    .unwrap();

    let b = camera_marks(&s, "b");
    assert!(near(b[0].0, p(22.5, 10.5)), "{:?}", b[0].0);
    assert!((b[0].1 - 290.0).abs() < 1e-9, "{}", b[0].1);
}

#[test]
fn spinning_the_figure_keeps_the_distance() {
    let mut s = open();
    let from = p(17.0, 9.5);
    let distance_m = from.distance_m(camera_marks(&s, "b")[0].0);

    s.edit_plan(PlanEdit::Spin {
        source: mark("burgvogt", 0),
        by_deg: -30.0,
    })
    .unwrap();

    let b = camera_marks(&s, "b");
    assert!((from.distance_m(b[0].0) - distance_m).abs() < 1e-9);
    assert!((b[0].1 - 170.0).abs() < 1e-9, "{}", b[0].1);
}

#[test]
fn figure_and_camera_undo_together() {
    let mut s = open();
    let before = s.project().clone();

    s.edit_plan(PlanEdit::Move {
        source: mark("burgvogt", 0),
        to: p(15.0, 8.5),
    })
    .unwrap();
    let after = s.project().clone();
    assert_ne!(after, before);

    s.undo();
    assert_eq!(*s.project(), before);

    s.redo();
    assert_eq!(*s.project(), after);
}

#[test]
fn moving_the_camera_alone_changes_only_it() {
    let mut s = open();
    let figures = s.plan().unwrap().figures.clone();
    s.set_time(0.0);

    s.edit_plan(PlanEdit::Move {
        source: Source::Camera("b".into()),
        to: p(1.0, 1.0),
    })
    .unwrap();

    assert_eq!(s.plan().unwrap().figures, figures);
    assert_eq!(camera_marks(&s, "b")[0].0, p(1.0, 1.0));
}

#[test]
fn a_camera_follows_and_lets_go() {
    let mut s = open();
    let follows = |s: &Session| s.plan().unwrap().camera("a").unwrap().follows.clone();

    couple(&mut s, "a", "elisabeth");
    assert_eq!(follows(&s), Some("elisabeth".to_owned()));

    s.edit_plan(PlanEdit::Follow {
        camera: "a".into(),
        figure: None,
    })
    .unwrap();
    assert_eq!(follows(&s), None);

    s.undo();
    assert_eq!(follows(&s), Some("elisabeth".to_owned()));
}

#[test]
fn following_needs_a_camera_and_a_figure_of_the_plan() {
    let mut s = open();

    let no_camera = s.edit_plan(PlanEdit::Follow {
        camera: "z".into(),
        figure: Some("elisabeth".into()),
    });
    assert!(matches!(no_camera, Err(ServiceError::Gone)));

    let no_figure = s.edit_plan(PlanEdit::Follow {
        camera: "a".into(),
        figure: Some("nobody".into()),
    });
    assert!(matches!(no_figure, Err(ServiceError::BadValue { .. })));
}

#[test]
fn coupling_is_refused_on_locked_cameras() {
    let mut s = open();
    s.set_lock(Layer::Cameras, Lock::Locked);

    let result = s.edit_plan(PlanEdit::Follow {
        camera: "a".into(),
        figure: Some("elisabeth".into()),
    });

    assert!(matches!(result, Err(ServiceError::Locked(Layer::Cameras))));
}

/// A figure edit that would move a locked camera is refused whole; one that leaves every
/// camera mark alone goes through.
#[test]
fn locked_cameras_refuse_the_figure_that_would_carry_them() {
    let mut s = open();
    s.set_lock(Layer::Cameras, Lock::Locked);
    let before = s.project().clone();

    let carried = s.edit_plan(PlanEdit::Move {
        source: mark("burgvogt", 0),
        to: p(15.0, 8.5),
    });
    assert!(matches!(carried, Err(ServiceError::Locked(Layer::Cameras))));
    assert_eq!(*s.project(), before);

    let free = s.edit_plan(PlanEdit::Move {
        source: mark("burgvogt", 1),
        to: p(1.0, 1.0),
    });
    assert!(free.is_ok());
    assert!(s
        .preview(&PlanEdit::Move {
            source: mark("burgvogt", 0),
            to: p(15.0, 8.5),
        })
        .is_err());
}

#[test]
fn removing_the_figure_works_under_a_camera_lock() {
    let mut s = open();
    s.set_lock(Layer::Cameras, Lock::Locked);

    let result = s.edit_plan(PlanEdit::Remove {
        source: Source::Figure("burgvogt".into()),
    });

    assert!(result.is_ok());
    assert_eq!(s.plan().unwrap().camera("b").unwrap().follows, None);
}

/// Review of PR 27: the bend of a following camera's move stayed where it was while its marks
/// moved with the figure, so the move could turn into a loop.
#[test]
fn the_bend_of_a_following_camera_moves_too() {
    let mut s = open();
    couple(&mut s, "a", "elisabeth");
    s.edit_plan(PlanEdit::Move {
        source: Source::CameraVia {
            camera: "a".into(),
            index: 1,
        },
        to: p(10.0, 6.0),
    })
    .unwrap();
    let via = |s: &Session| s.plan().unwrap().camera("a").unwrap().marks[1].via.unwrap();
    let (before, at) = (
        via(&s),
        s.plan().unwrap().figure("elisabeth").unwrap().marks[1].at,
    );

    s.edit_plan(PlanEdit::Move {
        source: mark("elisabeth", 1),
        to: p(at.x_m + 1.0, at.y_m + 2.0),
    })
    .unwrap();

    assert!(
        near(via(&s), p(before.x_m + 1.0, before.y_m + 2.0)),
        "{:?}",
        via(&s)
    );
}
