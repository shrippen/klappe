//! Set objects that figures move (LH-24), on the example project: the pillar `saeule-1` stands
//! at (6, 4); Elisabeth walks from (3, 6.5) at 0 s, pauses 2 s, and arrives at (10, 6.5) at 6 s.
#![allow(clippy::unwrap_used)]

use std::path::Path;

use klappe_services::{Layer, Lock, PlanEdit, Point, Role, ServiceError, Session, Shape, Source};

const PILLAR: &str = "saeule-1";

fn open() -> Session {
    Session::open(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe"))
        .unwrap()
}

fn p(x_m: f64, y_m: f64) -> Point {
    Point::new(x_m, y_m)
}

fn near(a: Point, b: Point) -> bool {
    a.distance_m(b) < 1e-6
}

fn pillar_mark(index: usize) -> Source {
    Source::ObjectMark {
        object: PILLAR.into(),
        index,
    }
}

fn add(s: &mut Session, at: Point) {
    s.edit_plan(PlanEdit::AddObjectMark {
        object: PILLAR.into(),
        at,
    })
    .unwrap();
}

/// Where the pillar is drawn at the playhead.
fn pillar_at(s: &Session) -> Point {
    let list = s.display_list();
    let body = list
        .items
        .iter()
        .find(|i| i.role == Role::Object && i.source == Some(Source::Object(PILLAR.into())))
        .unwrap();
    let Shape::Polygon { points } = &body.shape else {
        return p(f64::NAN, f64::NAN);
    };
    let n = points.len() as f64;

    p(
        points.iter().map(|q| q.x_m).sum::<f64>() / n,
        points.iter().map(|q| q.y_m).sum::<f64>() / n,
    )
}

fn marks(s: &Session) -> Vec<(f64, Point)> {
    s.plan()
        .unwrap()
        .object_track(PILLAR)
        .map(|t| t.marks.iter().map(|m| (m.at_s, m.at)).collect())
        .unwrap_or_default()
}

/// The first click marks where the pillar stands and where it goes; the way is walked at
/// 1.4 m/s: 6 m take 6 / 1.4 s. One undo step takes both marks back.
#[test]
fn the_first_mark_adds_the_start_too_and_undo_takes_both() {
    let mut s = open();
    s.set_time(0.0);

    add(&mut s, p(6.0, 10.0));

    let found = marks(&s);
    assert_eq!(found.len(), 2);
    assert!(near(found[0].1, p(6.0, 4.0)) && found[0].0 == 0.0);
    assert!(near(found[1].1, p(6.0, 10.0)));
    assert!((found[1].0 - 6.0 / 1.4).abs() < 1e-9);

    s.set_time(found[1].0 / 2.0);
    assert!(near(pillar_at(&s), p(6.0, 7.0)));

    assert_eq!(s.undo_step(), Some("AddObjectMark"));
    s.undo();
    assert!(marks(&s).is_empty());
    s.set_time(3.0);
    assert!(near(pillar_at(&s), p(6.0, 4.0)));
}

/// A later click adds one mark after the last; the playhead does not matter.
#[test]
fn a_later_mark_follows_the_last() {
    let mut s = open();
    add(&mut s, p(6.0, 10.0));
    s.set_time(1.0);
    add(&mut s, p(9.0, 10.0));

    let found = marks(&s);
    assert_eq!(found.len(), 3);
    assert!((found[2].0 - (found[1].0 + 3.0 / 1.4)).abs() < 1e-9);
}

#[test]
fn moving_a_mark_or_the_object_at_the_playhead_edits_the_track() {
    let mut s = open();
    add(&mut s, p(6.0, 10.0));

    s.edit_plan(PlanEdit::Move {
        source: pillar_mark(1),
        to: p(8.0, 10.0),
    })
    .unwrap();
    assert!(near(marks(&s)[1].1, p(8.0, 10.0)));

    // The playhead stands at the first mark: dragging the body moves that one, not the set.
    s.set_time(0.0);
    s.edit_plan(PlanEdit::Move {
        source: Source::Object(PILLAR.into()),
        to: p(5.0, 5.0),
    })
    .unwrap();
    assert!(near(marks(&s)[0].1, p(5.0, 5.0)));
    s.set_time(0.0);
    assert!(near(pillar_at(&s), p(5.0, 5.0)));

    s.edit_plan(PlanEdit::Turn {
        source: pillar_mark(1),
        facing_deg: 90.0,
    })
    .unwrap();
    assert_eq!(
        s.plan().unwrap().object_track(PILLAR).unwrap().marks[1].rotation_deg,
        90.0
    );
}

#[test]
fn a_pause_moves_the_later_marks() {
    let mut s = open();
    add(&mut s, p(6.0, 10.0));
    let before = marks(&s)[1].0;

    s.edit_plan(PlanEdit::SetHold {
        source: pillar_mark(0),
        hold_s: 3.0,
    })
    .unwrap();

    assert!((marks(&s)[1].0 - (before + 3.0)).abs() < 1e-9);
    assert_eq!(s.mark_info(&pillar_mark(0)).map(|i| i.hold_s), Some(3.0));
}

#[test]
fn removing_the_last_marks_gives_the_object_back_to_the_set() {
    let mut s = open();
    add(&mut s, p(6.0, 10.0));

    s.edit_plan(PlanEdit::Remove {
        source: pillar_mark(1),
    })
    .unwrap();
    assert_eq!(marks(&s).len(), 1);
    s.edit_plan(PlanEdit::Remove {
        source: pillar_mark(0),
    })
    .unwrap();

    assert!(s.plan().unwrap().object_track(PILLAR).is_none());
    assert!(matches!(
        s.edit_plan(PlanEdit::Remove {
            source: pillar_mark(0)
        }),
        Err(ServiceError::Gone)
    ));
}

/// The marks, the way and the object are on the set's layer: locked, none changes.
#[test]
fn the_set_layer_locks_the_marks() {
    let mut s = open();
    add(&mut s, p(6.0, 10.0));
    s.set_lock(Layer::Set, Lock::Locked);

    let add_again = s.edit_plan(PlanEdit::AddObjectMark {
        object: PILLAR.into(),
        at: p(1.0, 1.0),
    });
    let moved = s.edit_plan(PlanEdit::Move {
        source: pillar_mark(1),
        to: p(1.0, 1.0),
    });
    let carried = s.edit_plan(PlanEdit::SetCarrier {
        source: pillar_mark(0),
        figure: Some("elisabeth".into()),
    });

    for result in [add_again, moved, carried] {
        assert!(matches!(result, Err(ServiceError::Locked(Layer::Set))));
    }
    assert_eq!(marks(&s).len(), 2);
}

/// From mark 1 on Elisabeth carries the pillar, which keeps its offset (3, -2.5) to her until
/// its next mark; undo takes the carrier away again.
#[test]
fn a_figure_carries_the_object_until_the_next_mark() {
    let mut s = open();
    s.set_time(0.0);
    add(&mut s, p(13.0, 4.0));
    s.edit_plan(PlanEdit::SetCarrier {
        source: pillar_mark(0),
        figure: Some("elisabeth".into()),
    })
    .unwrap();

    // 3 s: Elisabeth left (3, 6.5) at 2 s and has walked a quarter of the way to (10, 6.5).
    s.set_time(3.0);
    assert!(near(pillar_at(&s), p(3.0 + 7.0 / 4.0 + 3.0, 4.0)));
    assert_eq!(
        s.mark_info(&pillar_mark(0)).unwrap().carried_by.as_deref(),
        Some("elisabeth")
    );

    // At its next mark (5 s) the pillar is set down there.
    s.set_time(5.0);
    assert!(near(pillar_at(&s), p(13.0, 4.0)));

    assert_eq!(s.undo_step(), Some("SetCarrier"));
    s.undo();
    s.set_time(3.0);
    assert!(s.mark_info(&pillar_mark(0)).unwrap().carried_by.is_none());
    assert!(pillar_at(&s).x_m > 6.0 && pillar_at(&s).x_m < 13.0);
}

#[test]
fn only_a_figure_of_the_plan_carries() {
    let mut s = open();
    add(&mut s, p(6.0, 10.0));

    let result = s.edit_plan(PlanEdit::SetCarrier {
        source: pillar_mark(0),
        figure: Some("nobody".into()),
    });

    assert!(matches!(result, Err(ServiceError::BadValue { .. })));
    assert!(s.mark_info(&pillar_mark(0)).unwrap().carried_by.is_none());
}

/// Taking the figure out of the plan sets the object down: the mark names nobody any more.
#[test]
fn removing_the_carrier_sets_the_object_down() {
    let mut s = open();
    add(&mut s, p(6.0, 10.0));
    s.edit_plan(PlanEdit::SetCarrier {
        source: pillar_mark(0),
        figure: Some("burgvogt".into()),
    })
    .unwrap();

    s.edit_plan(PlanEdit::Remove {
        source: Source::Figure("burgvogt".into()),
    })
    .unwrap();

    assert!(s.mark_info(&pillar_mark(0)).unwrap().carried_by.is_none());
}

/// Taking the object out of the set takes its way with it; the playhead stops on its marks.
#[test]
fn the_playhead_steps_to_the_marks_and_a_removed_object_takes_its_way() {
    let mut s = open();
    add(&mut s, p(6.0, 10.0));
    assert!(s.marks_s().iter().any(|t| (t - 6.0 / 1.4).abs() < 1e-9));

    s.edit_plan(PlanEdit::Remove {
        source: Source::Object(PILLAR.into()),
    })
    .unwrap();

    assert!(s.plan().unwrap().object_tracks.is_empty());
}

/// Camera B at (18, 4) looks 200° (towards 14.2, 2.6 four metres on): once the pillar's way has
/// led there, the picture shows it close; at the first mark it is where it stood.
#[test]
fn the_camera_picture_uses_the_place_at_the_playhead() {
    let mut s = open();
    // Without a height an object is only a few see-through pixels.
    s.edit_plan(PlanEdit::SetObjectHeight {
        source: Source::Object(PILLAR.into()),
        height_m: Some(2.5),
    })
    .unwrap();
    let pixels_of_pillar = |s: &Session| {
        let picture = s.camera_picture(Some("b"), 96).unwrap();
        let index = picture
            .sources
            .iter()
            .position(|source| *source == Source::Object(PILLAR.into()))
            .unwrap() as u32;

        picture.frame.source.iter().filter(|i| **i == index).count()
    };

    let at_home = pixels_of_pillar(&s);
    add(&mut s, p(14.24, 2.63));
    s.set_time(0.0);
    assert_eq!(pixels_of_pillar(&s), at_home);

    s.set_time(20.0);
    assert!(pixels_of_pillar(&s) > 2 * at_home);
}
