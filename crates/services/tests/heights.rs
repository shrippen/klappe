//! Camera height and tilt per mark (LH-34) and the height of objects (LH-17), on the example
//! project: camera A has marks at 0 and 6 s, both at 1.6 m and level; the set has the table
//! `tisch`, which has no height.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_model::LibraryItem;
use klappe_services::ObjectShape;
use klappe_services::{
    Layer, Lock, PlanEdit, Point, ProjectEdit, ServiceError, Session, Snapping, Source,
};

fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe")
}

fn open() -> Session {
    let mut s = Session::open(&example()).unwrap();
    s.set_snapping(Snapping::Off);
    s
}

fn camera() -> Source {
    Source::Camera("a".into())
}

fn aim(height_m: f64, tilt_deg: f64) -> PlanEdit {
    PlanEdit::SetCameraAim {
        source: camera(),
        height_m,
        tilt_deg,
    }
}

/// Height and tilt of both marks of camera A.
fn aims(s: &Session) -> Vec<(f64, f64)> {
    let camera = s.plan().unwrap().camera("a").unwrap();

    camera
        .marks
        .iter()
        .map(|m| (m.height_m, m.tilt_deg))
        .collect()
}

fn height_edit(id: &str, height_m: Option<f64>) -> PlanEdit {
    PlanEdit::SetObjectHeight {
        source: Source::Object(id.into()),
        height_m,
    }
}

fn height_of(s: &Session, id: &str) -> Option<f64> {
    let set = &s.project().locations[0].set;

    set.objects.iter().find(|o| o.id == id).unwrap().height_m
}

fn is_bad(result: Result<(), ServiceError>) -> bool {
    matches!(result, Err(ServiceError::BadValue { .. }))
}

fn place(s: &mut Session, kind: &str) -> Option<f64> {
    s.edit_plan(PlanEdit::AddObject {
        kind: kind.into(),
        at: Point::new(2.0, 2.0),
    })
    .unwrap();

    s.project().locations[0]
        .set
        .objects
        .last()
        .unwrap()
        .height_m
}

#[test]
fn the_mark_at_the_playhead_gets_height_and_tilt() {
    let mut s = open();

    s.edit_plan(aim(2.4, -15.0)).unwrap();

    assert_eq!(aims(&s), [(2.4, -15.0), (1.6, 0.0)]);
    let info = s.mark_info(&camera()).unwrap();
    assert_eq!((info.height_m, info.tilt_deg), (Some(2.4), Some(-15.0)));

    s.undo();
    assert_eq!(aims(&s), [(1.6, 0.0), (1.6, 0.0)]);
}

/// At 7 s the playhead is past the second mark (6 s).
#[test]
fn a_later_playhead_takes_the_later_mark() {
    let mut s = open();
    s.set_time(7.0);

    s.edit_plan(aim(1.0, 10.0)).unwrap();

    assert_eq!(aims(&s), [(1.6, 0.0), (1.0, 10.0)]);
}

#[test]
fn height_and_tilt_keep_their_limits() {
    let mut s = open();

    for (height_m, tilt_deg) in [
        (-0.1, 0.0),
        (50.1, 0.0),
        (f64::NAN, 0.0),
        (f64::INFINITY, 0.0),
        (1.6, 90.1),
        (1.6, -90.1),
        (1.6, f64::NAN),
        (1.6, f64::NEG_INFINITY),
    ] {
        assert!(
            is_bad(s.edit_plan(aim(height_m, tilt_deg))),
            "{height_m} {tilt_deg}"
        );
    }
    assert_eq!(aims(&s), [(1.6, 0.0), (1.6, 0.0)]);

    // The limits themselves are allowed.
    for (height_m, tilt_deg) in [(0.0, -90.0), (50.0, 90.0)] {
        s.edit_plan(aim(height_m, tilt_deg)).unwrap();
        assert_eq!(aims(&s)[0], (height_m, tilt_deg));
    }
}

#[test]
fn a_figure_has_no_height_or_tilt_to_set() {
    let mut s = open();
    let figure = Source::Figure("elisabeth".into());
    let mark = Source::Mark {
        character: "elisabeth".into(),
        index: 1,
    };

    for source in [figure.clone(), mark.clone()] {
        let result = s.edit_plan(PlanEdit::SetCameraAim {
            source: source.clone(),
            height_m: 2.0,
            tilt_deg: 0.0,
        });

        assert!(is_bad(result));
        let info = s.mark_info(&source).unwrap();
        assert_eq!((info.height_m, info.tilt_deg), (None, None));
    }
}

#[test]
fn a_locked_camera_layer_refuses_height_and_tilt() {
    let mut s = open();
    s.set_lock(Layer::Cameras, Lock::Locked);

    assert!(matches!(
        s.edit_plan(aim(3.0, 5.0)),
        Err(ServiceError::Locked(Layer::Cameras))
    ));
    assert_eq!(aims(&s), [(1.6, 0.0), (1.6, 0.0)]);

    s.set_lock(Layer::Figures, Lock::Locked);
    s.set_lock(Layer::Cameras, Lock::Unlocked);
    s.edit_plan(aim(3.0, 5.0)).unwrap();
}

#[test]
fn a_new_object_takes_the_height_of_its_library_entry() {
    let mut s = open();

    assert_eq!(place(&mut s, "table"), Some(0.75));
    assert_eq!(place(&mut s, "door"), Some(2.1));
    // A kind no library knows has no height.
    assert_eq!(place(&mut s, "zeppelin"), None);
}

#[test]
fn an_own_entry_without_a_height_gives_none() {
    let mut s = open();
    s.edit_project(ProjectEdit::ImportLibrary {
        items: vec![LibraryItem {
            kind: "teppich".into(),
            label: "Teppich".into(),
            width_m: 3.0,
            depth_m: 2.0,
            shape: ObjectShape::Rect,
            height_m: None,
        }],
    })
    .unwrap();

    assert_eq!(place(&mut s, "teppich"), None);
}

#[test]
fn an_object_gets_a_height_and_loses_it() {
    let mut s = open();
    assert_eq!(height_of(&s, "tisch"), None);

    s.edit_plan(height_edit("tisch", Some(0.8))).unwrap();
    assert_eq!(height_of(&s, "tisch"), Some(0.8));

    s.edit_plan(height_edit("tisch", None)).unwrap();
    assert_eq!(height_of(&s, "tisch"), None);

    s.undo();
    assert_eq!(height_of(&s, "tisch"), Some(0.8));
    s.undo();
    assert_eq!(height_of(&s, "tisch"), None);
}

#[test]
fn an_object_height_is_more_than_nothing_and_at_most_100_m() {
    let mut s = open();

    for height_m in [0.0, -1.0, f64::NAN, f64::INFINITY, 100.1] {
        assert!(
            is_bad(s.edit_plan(height_edit("tisch", Some(height_m)))),
            "{height_m}"
        );
    }
    assert_eq!(height_of(&s, "tisch"), None);

    s.edit_plan(height_edit("tisch", Some(100.0))).unwrap();
    assert_eq!(height_of(&s, "tisch"), Some(100.0));
}

#[test]
fn only_objects_that_exist_have_a_height() {
    let mut s = open();

    assert!(matches!(
        s.edit_plan(height_edit("o99", Some(1.0))),
        Err(ServiceError::Gone)
    ));
    let wall = PlanEdit::SetObjectHeight {
        source: Source::Wall(0),
        height_m: Some(1.0),
    };
    assert!(is_bad(s.edit_plan(wall)));
}

#[test]
fn a_locked_set_refuses_an_object_height() {
    let mut s = open();
    s.set_lock(Layer::Set, Lock::Locked);

    assert!(matches!(
        s.edit_plan(height_edit("tisch", Some(0.8))),
        Err(ServiceError::Locked(Layer::Set))
    ));
    assert_eq!(height_of(&s, "tisch"), None);
}

#[test]
fn filing_an_object_in_the_library_takes_its_height() {
    let mut s = open();
    s.edit_plan(height_edit("tisch", Some(0.8))).unwrap();

    s.edit_project(ProjectEdit::LibraryFromObject {
        location: "saengersaal".into(),
        object: "tisch".into(),
        name: "Lange Tafel".into(),
    })
    .unwrap();

    let entry = s
        .library()
        .into_iter()
        .find(|e| e.kind == "lange-tafel")
        .unwrap();
    assert_eq!(entry.height_m, Some(0.8));
    assert_eq!(place(&mut s, "lange-tafel"), Some(0.8));
}

#[test]
fn an_import_with_an_unusable_height_changes_nothing() {
    let mut s = open();
    let item = |height_m| LibraryItem {
        kind: "thron".into(),
        label: "Thron".into(),
        width_m: 0.9,
        depth_m: 0.8,
        shape: ObjectShape::Rect,
        height_m: Some(height_m),
    };

    for bad in [0.0, -1.0, f64::NAN, 100.5] {
        let result = s.edit_project(ProjectEdit::ImportLibrary {
            items: vec![item(bad)],
        });

        assert!(is_bad(result), "{bad}");
        assert!(s.project().library.is_empty());
    }

    // The same entry with another height is a different entry, not a repeat.
    s.edit_project(ProjectEdit::ImportLibrary {
        items: vec![item(1.0)],
    })
    .unwrap();
    s.edit_project(ProjectEdit::ImportLibrary {
        items: vec![item(1.2)],
    })
    .unwrap();
    assert_eq!(s.project().library.len(), 2);
}
