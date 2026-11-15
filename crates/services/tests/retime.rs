//! Dragging marks and shot edges in time (LH-61), on the example project: Elisabeth's marks lie
//! at 0, 6 and 14 s (the last holds 0 s); camera A has marks at 0 and 6 s; scene 12 has the
//! shots `s12-1` to `s12-5`, the scene ends with its last shot.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_services::{Grab, Layer, Lock, ServiceError, Session, TimeSnap};

fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe")
}

fn open() -> Session {
    Session::open(&example()).unwrap()
}

fn elisabeth(index: usize) -> Grab {
    Grab::FigureMark {
        character: "elisabeth".into(),
        index,
    }
}

fn camera_a(index: usize) -> Grab {
    Grab::CameraMark {
        camera: "a".into(),
        index,
    }
}

fn arrivals(s: &Session) -> Vec<f64> {
    let track = s.plan().unwrap().figure("elisabeth").unwrap();

    track.marks.iter().map(|m| m.at_s).collect()
}

fn span(s: &Session, id: &str) -> (f64, f64) {
    let shot = s.plan().unwrap().shots.iter().find(|x| x.id == id).unwrap();

    (shot.from_s, shot.to_s)
}

#[test]
fn a_dropped_mark_arrives_on_the_tenth() {
    let mut s = open();

    s.drop_time(&elisabeth(1), 7.26, TimeSnap::Tenths).unwrap();
    assert_eq!(arrivals(&s), vec![0.0, 7.3, 14.0]);

    s.drop_time(&elisabeth(1), 7.26, TimeSnap::Free).unwrap();
    assert_eq!(arrivals(&s), vec![0.0, 7.26, 14.0]);
}

#[test]
fn a_mark_stays_between_its_neighbours() {
    let mut s = open();

    s.drop_time(&elisabeth(1), 20.0, TimeSnap::Tenths).unwrap();
    assert_eq!(arrivals(&s), vec![0.0, 14.0, 14.0]);

    s.drop_time(&elisabeth(1), -3.0, TimeSnap::Tenths).unwrap();
    assert_eq!(arrivals(&s), vec![0.0, 0.0, 14.0]);

    // The first has no mark before it, the last none after it: the scene's end is its limit.
    s.drop_time(&elisabeth(0), -1.0, TimeSnap::Tenths).unwrap();
    s.drop_time(&elisabeth(2), 99.0, TimeSnap::Tenths).unwrap();
    assert_eq!(arrivals(&s), vec![0.0, 0.0, s.duration_s()]);
}

#[test]
fn the_drag_shows_what_the_drop_does() {
    let s = open();

    assert_eq!(
        s.drag_time(&elisabeth(1), 9.04, TimeSnap::Tenths).unwrap(),
        9.0
    );
    assert_eq!(
        s.drag_time(&elisabeth(1), 30.0, TimeSnap::Tenths).unwrap(),
        14.0
    );
    assert_eq!(
        s.drag_time(&elisabeth(2), 99.0, TimeSnap::Tenths).unwrap(),
        s.duration_s()
    );
    assert_eq!(
        s.drag_time(&camera_a(1), 2.0, TimeSnap::Tenths).unwrap(),
        2.0
    );
}

#[test]
fn a_camera_mark_moves_alone() {
    let mut s = open();

    s.drop_time(&camera_a(1), 4.5, TimeSnap::Tenths).unwrap();

    let marks = &s.plan().unwrap().camera("a").unwrap().marks;
    assert_eq!(
        marks.iter().map(|m| m.at_s).collect::<Vec<_>>(),
        vec![0.0, 4.5]
    );
    assert_eq!(arrivals(&s), vec![0.0, 6.0, 14.0]);
}

#[test]
fn a_drop_is_one_undo_step() {
    let mut s = open();

    s.drop_time(&elisabeth(1), 8.0, TimeSnap::Tenths).unwrap();
    s.undo();
    assert_eq!(arrivals(&s), vec![0.0, 6.0, 14.0]);
    assert!(!s.can_undo());
}

#[test]
fn a_drop_that_changes_nothing_leaves_no_undo_step() {
    let mut s = open();

    s.drop_time(&elisabeth(1), 6.02, TimeSnap::Tenths).unwrap();
    assert!(!s.can_undo());
}

#[test]
fn a_locked_layer_refuses_the_drop() {
    let mut s = open();

    s.set_lock(Layer::Figures, Lock::Locked);
    let result = s.drop_time(&elisabeth(1), 8.0, TimeSnap::Tenths);
    assert!(matches!(result, Err(ServiceError::Locked(Layer::Figures))));

    s.set_lock(Layer::Cameras, Lock::Locked);
    let result = s.drop_time(&camera_a(1), 4.0, TimeSnap::Tenths);
    assert!(matches!(result, Err(ServiceError::Locked(Layer::Cameras))));
    let result = s.drop_time(&Grab::ShotTo("s12-1".into()), 4.0, TimeSnap::Tenths);
    assert!(matches!(result, Err(ServiceError::Locked(Layer::Cameras))));

    assert_eq!(arrivals(&s), vec![0.0, 6.0, 14.0]);
    assert!(!s.can_undo());
}

#[test]
fn a_mark_that_is_gone_is_refused() {
    let mut s = open();

    assert!(matches!(
        s.drop_time(&elisabeth(9), 1.0, TimeSnap::Tenths),
        Err(ServiceError::Gone)
    ));
    assert!(matches!(
        s.drop_time(&Grab::ShotTo("nope".into()), 1.0, TimeSnap::Tenths),
        Err(ServiceError::NoShot(_))
    ));
}

#[test]
fn the_edges_of_a_shot_move_and_keep_their_order() {
    let mut s = open();
    let (from, to) = span(&s, "s12-2");

    s.drop_time(
        &Grab::ShotFrom("s12-2".into()),
        from + 1.0,
        TimeSnap::Tenths,
    )
    .unwrap();
    assert_eq!(span(&s, "s12-2"), (from + 1.0, to));

    // The start stops a tenth short of the end, the end a tenth behind the start.
    s.drop_time(&Grab::ShotFrom("s12-2".into()), to + 5.0, TimeSnap::Tenths)
        .unwrap();
    assert_eq!(span(&s, "s12-2"), (to - 0.1, to));

    s.drop_time(&Grab::ShotTo("s12-2".into()), 0.0, TimeSnap::Tenths)
        .unwrap();
    let (new_from, new_to) = span(&s, "s12-2");
    assert!((new_to - new_from - 0.1).abs() < 1e-9);
}
