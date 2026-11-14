//! Pose, pause and pace of marks (LH-22, LH-63), on the example project: Elisabeth's marks lie
//! at 0, 6 and 14 s with holds of 2, 4 and 0 s, at (3, 6.5), (10, 6.5) and (16, 3) (the last
//! way curved); Burgvogt's at 0 s (hold 5 s) and 9 s; camera A has marks at 0 s (hold 6 s) and
//! 6 s, camera B follows Burgvogt.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_services::{
    Layer, Lock, Pace, PlanEdit, Point, Pose, ServiceError, Session, Snapping, Source,
};

const WALK_M_S: f64 = 1.4;
const RUN_M_S: f64 = 4.0;

fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe")
}

fn open() -> Session {
    let mut s = Session::open(&example()).unwrap();
    s.set_snapping(Snapping::Off);
    s
}

fn mark(character: &str, index: usize) -> Source {
    Source::Mark {
        character: character.into(),
        index,
    }
}

fn arrivals(s: &Session, character: &str) -> Vec<f64> {
    let track = s.plan().unwrap().figure(character).unwrap();

    track.marks.iter().map(|m| m.at_s).collect()
}

fn holds(s: &Session, character: &str) -> Vec<f64> {
    let track = s.plan().unwrap().figure(character).unwrap();

    track.marks.iter().map(|m| m.hold_s).collect()
}

fn camera_arrivals(s: &Session, id: &str) -> Vec<f64> {
    let camera = s.plan().unwrap().camera(id).unwrap();

    camera.marks.iter().map(|m| m.at_s).collect()
}

fn pace_at(s: &Session, index: usize) -> f64 {
    s.mark_info(&mark("elisabeth", index))
        .unwrap()
        .pace_m_s
        .unwrap()
}

fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

fn is_locked(result: Result<(), ServiceError>, layer: Layer) -> bool {
    matches!(result, Err(ServiceError::Locked(l)) if l == layer)
}

fn is_bad(result: Result<(), ServiceError>) -> bool {
    matches!(result, Err(ServiceError::BadValue { .. }))
}

#[test]
fn a_mark_can_sit_and_lie() {
    let mut s = open();

    for pose in [Pose::Sitting, Pose::Lying] {
        s.edit_plan(PlanEdit::SetPose {
            source: mark("elisabeth", 1),
            pose,
        })
        .unwrap();

        let poses: Vec<Pose> = s.plan().unwrap().figures[0]
            .marks
            .iter()
            .map(|m| m.pose)
            .collect();
        assert_eq!(poses, [Pose::Standing, pose, Pose::Standing]);
        assert_eq!(s.mark_info(&mark("elisabeth", 1)).unwrap().pose, Some(pose));
    }

    // One undo step each.
    s.undo();
    assert_eq!(
        s.mark_info(&mark("elisabeth", 1)).unwrap().pose,
        Some(Pose::Sitting)
    );
    s.undo();
    assert_eq!(
        s.mark_info(&mark("elisabeth", 1)).unwrap().pose,
        Some(Pose::Standing)
    );
}

/// A figure at the playhead stands for its mark there: at 7 s that is the second.
#[test]
fn the_figure_stands_for_its_mark_at_the_playhead() {
    let mut s = open();
    s.set_time(7.0);

    s.edit_plan(PlanEdit::SetPose {
        source: Source::Figure("elisabeth".into()),
        pose: Pose::Lying,
    })
    .unwrap();

    assert_eq!(s.plan().unwrap().figures[0].marks[1].pose, Pose::Lying);
}

#[test]
fn only_a_figures_mark_has_a_pose() {
    let mut s = open();

    let camera = s.edit_plan(PlanEdit::SetPose {
        source: Source::Camera("a".into()),
        pose: Pose::Lying,
    });

    assert!(is_bad(camera));
    assert_eq!(s.mark_info(&Source::Camera("a".into())).unwrap().pose, None);
}

#[test]
fn a_longer_pause_moves_the_later_marks() {
    let mut s = open();

    s.edit_plan(PlanEdit::SetHold {
        source: mark("elisabeth", 0),
        hold_s: 5.0,
    })
    .unwrap();

    assert_eq!(holds(&s, "elisabeth"), [5.0, 4.0, 0.0]);
    assert_eq!(arrivals(&s, "elisabeth"), [0.0, 9.0, 17.0]);
    // The other figure keeps its times.
    assert_eq!(arrivals(&s, "burgvogt"), [0.0, 9.0]);

    s.undo();
    assert_eq!(holds(&s, "elisabeth"), [2.0, 4.0, 0.0]);
    assert_eq!(arrivals(&s, "elisabeth"), [0.0, 6.0, 14.0]);
}

#[test]
fn a_shorter_pause_pulls_the_later_marks_in() {
    let mut s = open();

    s.edit_plan(PlanEdit::SetHold {
        source: mark("elisabeth", 1),
        hold_s: 0.0,
    })
    .unwrap();

    assert_eq!(arrivals(&s, "elisabeth"), [0.0, 6.0, 10.0]);
    assert_eq!(s.mark_info(&mark("elisabeth", 1)).unwrap().hold_s, 0.0);
}

#[test]
fn the_last_mark_has_no_later_marks_to_move() {
    let mut s = open();

    s.edit_plan(PlanEdit::SetHold {
        source: mark("elisabeth", 2),
        hold_s: 3.0,
    })
    .unwrap();

    assert_eq!(arrivals(&s, "elisabeth"), [0.0, 6.0, 14.0]);
    assert_eq!(holds(&s, "elisabeth"), [2.0, 4.0, 3.0]);
}

/// A camera's pause: camera A at the playhead (0 s) waits 8 s instead of 6.
#[test]
fn a_camera_has_a_pause_too() {
    let mut s = open();

    s.edit_plan(PlanEdit::SetHold {
        source: Source::Camera("a".into()),
        hold_s: 8.0,
    })
    .unwrap();

    assert_eq!(camera_arrivals(&s, "a"), [0.0, 8.0]);
    assert_eq!(
        s.mark_info(&Source::Camera("a".into())).unwrap().hold_s,
        8.0
    );
}

#[test]
fn a_pause_is_a_time_of_zero_or_more() {
    let mut s = open();

    for hold_s in [-1.0, f64::NAN, f64::INFINITY] {
        let result = s.edit_plan(PlanEdit::SetHold {
            source: mark("elisabeth", 0),
            hold_s,
        });

        assert!(is_bad(result), "{hold_s}");
    }
    assert_eq!(holds(&s, "elisabeth"), [2.0, 4.0, 0.0]);
}

/// Mark 2 is 7 m from mark 1, which is left at 2 s. Walking, mark 2 is reached at
/// 2 + 7 / 1.4 = 7 s, 1 s later than before; mark 3 follows by the same second. Running takes
/// 7 / 4 = 1.75 s: arrival 3.75 s, 2.25 s earlier.
#[test]
fn the_pace_sets_the_arrival_and_moves_the_later_marks() {
    let mut s = open();
    assert!(near(pace_at(&s, 1), 7.0 / 4.0));

    s.edit_plan(PlanEdit::SetPace {
        source: mark("elisabeth", 1),
        pace: Pace::Walk,
    })
    .unwrap();
    assert!(near(arrivals(&s, "elisabeth")[1], 2.0 + 7.0 / WALK_M_S));
    assert!(near(arrivals(&s, "elisabeth")[2], 15.0));
    assert!(near(pace_at(&s, 1), WALK_M_S));

    s.edit_plan(PlanEdit::SetPace {
        source: mark("elisabeth", 1),
        pace: Pace::Run,
    })
    .unwrap();
    assert!(near(arrivals(&s, "elisabeth")[1], 2.0 + 7.0 / RUN_M_S));
    assert!(near(arrivals(&s, "elisabeth")[2], 14.0 - 2.25));
    assert!(near(pace_at(&s, 1), RUN_M_S));

    s.undo();
    assert!(near(arrivals(&s, "elisabeth")[1], 2.0 + 7.0 / WALK_M_S));
    s.undo();
    assert_eq!(arrivals(&s, "elisabeth"), [0.0, 6.0, 14.0]);
}

/// The way to mark 3 is curved: the pace counts its length, longer than the 6.95 m between
/// the marks, so walking it takes longer than that distance would.
#[test]
fn a_curved_way_is_timed_by_its_length() {
    let mut s = open();
    let chord_m = Point::new(10.0, 6.5).distance_m(Point::new(16.0, 3.0));
    let leave_s = 6.0 + 4.0;

    s.edit_plan(PlanEdit::SetPace {
        source: mark("elisabeth", 2),
        pace: Pace::Walk,
    })
    .unwrap();

    let walk_s = arrivals(&s, "elisabeth")[2] - leave_s;
    assert!(walk_s > chord_m / WALK_M_S + 0.05, "{walk_s}");
    assert!(near(pace_at(&s, 2), WALK_M_S));

    s.edit_plan(PlanEdit::SetPace {
        source: mark("elisabeth", 2),
        pace: Pace::Run,
    })
    .unwrap();
    assert!(near(pace_at(&s, 2), RUN_M_S));
    // Same way, 4 / 1.4 times the speed: that much less time.
    let run_s = arrivals(&s, "elisabeth")[2] - leave_s;
    assert!(near(run_s * RUN_M_S / WALK_M_S, walk_s));
}

#[test]
fn the_first_mark_has_no_way_to_set_a_pace_on() {
    let mut s = open();

    let result = s.edit_plan(PlanEdit::SetPace {
        source: mark("elisabeth", 0),
        pace: Pace::Run,
    });

    assert!(is_bad(result));
    assert_eq!(arrivals(&s, "elisabeth"), [0.0, 6.0, 14.0]);
    let first = s.mark_info(&mark("elisabeth", 0)).unwrap();
    assert!(!first.has_way);
    assert_eq!(first.pace_m_s, None);
    assert!(s.mark_info(&mark("elisabeth", 1)).unwrap().has_way);
}

#[test]
fn a_camera_has_a_pace_to_read_but_not_to_set() {
    let mut s = open();
    s.set_time(6.0);

    // Camera A: from (8, 11) to (9, 8), left at 6 s, arrives at 6 s: no time to walk.
    assert_eq!(
        s.mark_info(&Source::Camera("a".into())).unwrap().pace_m_s,
        None
    );

    let result = s.edit_plan(PlanEdit::SetPace {
        source: Source::Camera("a".into()),
        pace: Pace::Run,
    });
    assert!(is_bad(result));
}

#[test]
fn the_edits_follow_the_locks() {
    let mut s = open();
    s.set_lock(Layer::Figures, Lock::Locked);

    let edits = [
        PlanEdit::SetPose {
            source: mark("elisabeth", 1),
            pose: Pose::Lying,
        },
        PlanEdit::SetHold {
            source: mark("elisabeth", 1),
            hold_s: 1.0,
        },
        PlanEdit::SetPace {
            source: mark("elisabeth", 1),
            pace: Pace::Run,
        },
    ];
    for edit in edits {
        assert!(is_locked(s.edit_plan(edit), Layer::Figures));
    }
    assert_eq!(arrivals(&s, "elisabeth"), [0.0, 6.0, 14.0]);

    // A camera's pause is on the cameras' layer.
    let camera = PlanEdit::SetHold {
        source: Source::Camera("a".into()),
        hold_s: 1.0,
    };
    s.set_lock(Layer::Figures, Lock::Unlocked);
    s.set_lock(Layer::Cameras, Lock::Locked);
    assert!(is_locked(s.edit_plan(camera), Layer::Cameras));
    assert!(s
        .edit_plan(PlanEdit::SetHold {
            source: mark("elisabeth", 1),
            hold_s: 1.0,
        })
        .is_ok());
}

/// Camera B follows Burgvogt and has a mark at 0 s, in the window of his first mark. Timing
/// his marks changes the figure's track only: no camera mark moves, and the coupling for
/// the marks in the window stays.
#[test]
fn timing_a_figure_leaves_the_cameras_times_alone() {
    let mut s = open();
    s.edit_plan(PlanEdit::Follow {
        camera: "b".into(),
        figure: Some("burgvogt".into()),
    })
    .unwrap();

    s.edit_plan(PlanEdit::SetPace {
        source: mark("burgvogt", 1),
        pace: Pace::Run,
    })
    .unwrap();
    s.edit_plan(PlanEdit::SetHold {
        source: mark("burgvogt", 0),
        hold_s: 2.0,
    })
    .unwrap();

    assert_eq!(camera_arrivals(&s, "b"), [0.0]);
    assert_eq!(camera_arrivals(&s, "a"), [0.0, 6.0]);
}

#[test]
fn an_unknown_mark_is_gone() {
    let mut s = open();

    let result = s.edit_plan(PlanEdit::SetHold {
        source: mark("elisabeth", 9),
        hold_s: 1.0,
    });

    assert!(matches!(result, Err(ServiceError::Gone)));
    assert_eq!(s.mark_info(&mark("elisabeth", 9)), None);
}
