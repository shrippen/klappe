//! Heights of walls, openings and objects (LH-90), on the example project: the south wall
//! (index 2) has a door 0.9 m wide; the table `tisch` has no height. The set is 2.6 m high,
//! a door 0 to 2.10 m, a window 0.90 to 2.10 m.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_model::{Opening, OpeningKind};
use klappe_services::{
    Field, Hint, Layer, Lock, PlanEdit, Point, ProjectEdit, ServiceError, Session, Source,
};

fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe")
}

fn open() -> Session {
    Session::open(&example()).unwrap()
}

fn door() -> Source {
    Source::Opening { wall: 2, index: 0 }
}

fn bad(result: Result<(), ServiceError>, wanted: Field) -> bool {
    matches!(result, Err(ServiceError::BadValue { field, .. }) if field == wanted)
}

fn opening(s: &Session) -> Opening {
    s.project().locations[0].set.walls[2].openings[0].clone()
}

fn wall_height(s: &Session, wall: usize) -> Option<f64> {
    s.project().locations[0].set.walls[wall].height_m
}

fn base(s: &Session) -> Option<f64> {
    let set = &s.project().locations[0].set;

    set.objects.iter().find(|o| o.id == "tisch").unwrap().base_m
}

fn wall_edit(wall: usize, height_m: Option<f64>) -> PlanEdit {
    PlanEdit::SetWallHeight {
        source: Source::Wall(wall),
        height_m,
    }
}

fn sill(sill_m: Option<f64>) -> PlanEdit {
    PlanEdit::SetOpeningSill {
        source: door(),
        sill_m,
    }
}

fn head(head_m: Option<f64>) -> PlanEdit {
    PlanEdit::SetOpeningHead {
        source: door(),
        head_m,
    }
}

fn base_edit(id: &str, base_m: Option<f64>) -> PlanEdit {
    PlanEdit::SetObjectBase {
        source: Source::Object(id.into()),
        base_m,
    }
}

const BAD: [f64; 5] = [f64::NAN, f64::INFINITY, -1.0, 100.1, f64::NEG_INFINITY];

#[test]
fn a_wall_gets_a_height_and_loses_it() {
    let mut s = open();
    assert_eq!(wall_height(&s, 0), None);

    s.edit_plan(wall_edit(0, Some(1.1))).unwrap();
    assert_eq!(wall_height(&s, 0), Some(1.1));
    assert_eq!(wall_height(&s, 1), None);

    s.edit_plan(wall_edit(0, None)).unwrap();
    assert_eq!(wall_height(&s, 0), None);

    s.undo();
    assert_eq!(wall_height(&s, 0), Some(1.1));
    s.undo();
    assert_eq!(wall_height(&s, 0), None);
}

#[test]
fn a_wall_height_is_more_than_nothing_and_at_most_100_m() {
    let mut s = open();

    for height_m in BAD.into_iter().chain([0.0]) {
        let result = s.edit_plan(wall_edit(0, Some(height_m)));
        assert!(bad(result, Field::Height), "{height_m}");
    }
    assert_eq!(wall_height(&s, 0), None);

    s.edit_plan(wall_edit(0, Some(100.0))).unwrap();
    assert_eq!(wall_height(&s, 0), Some(100.0));
}

#[test]
fn only_walls_that_exist_have_a_height() {
    let mut s = open();

    assert!(matches!(
        s.edit_plan(wall_edit(99, Some(1.0))),
        Err(ServiceError::Gone)
    ));
    let object = PlanEdit::SetWallHeight {
        source: Source::Object("tisch".into()),
        height_m: Some(1.0),
    };
    assert!(bad(s.edit_plan(object), Field::Source));
}

#[test]
fn a_sill_and_a_head_are_set_and_taken_off() {
    let mut s = open();

    s.edit_plan(head(Some(2.4))).unwrap();
    s.edit_plan(sill(Some(0.3))).unwrap();
    let o = opening(&s);
    assert_eq!((o.sill_m, o.head_m), (Some(0.3), Some(2.4)));

    s.edit_plan(sill(None)).unwrap();
    s.edit_plan(head(None)).unwrap();
    let o = opening(&s);
    assert_eq!((o.sill_m, o.head_m), (None, None));

    for _ in 0..2 {
        s.undo();
    }
    let o = opening(&s);
    assert_eq!((o.sill_m, o.head_m), (Some(0.3), Some(2.4)));
    for _ in 0..2 {
        s.undo();
    }
    let o = opening(&s);
    assert_eq!((o.sill_m, o.head_m), (None, None));
}

#[test]
fn a_sill_starts_at_the_floor_and_stays_below_the_head() {
    let mut s = open();

    s.edit_plan(sill(Some(0.0))).unwrap();
    for sill_m in [f64::NAN, f64::INFINITY, -0.1, 100.1] {
        assert!(
            bad(s.edit_plan(sill(Some(sill_m))), Field::Sill),
            "{sill_m}"
        );
    }

    // The door's own head is 2.10 m.
    assert!(bad(s.edit_plan(sill(Some(2.1))), Field::Sill));
    assert!(bad(s.edit_plan(sill(Some(3.0))), Field::Sill));
    assert_eq!(opening(&s).sill_m, Some(0.0));
    s.edit_plan(sill(Some(2.0))).unwrap();

    // Taking it off with a head below the kind's sill: the window's default 0.90 m is too high.
    let mut s = open();
    s.edit_plan(PlanEdit::AddOpening {
        kind: OpeningKind::Window,
        at: Point::new(9.0, 13.0),
    })
    .unwrap();
    let window = Source::Opening { wall: 2, index: 1 };
    s.edit_plan(PlanEdit::SetOpeningHead {
        source: window.clone(),
        head_m: Some(0.8),
    })
    .unwrap_err();
    s.edit_plan(PlanEdit::SetOpeningSill {
        source: window.clone(),
        sill_m: Some(0.2),
    })
    .unwrap();
    s.edit_plan(PlanEdit::SetOpeningHead {
        source: window.clone(),
        head_m: Some(0.8),
    })
    .unwrap();
    let result = s.edit_plan(PlanEdit::SetOpeningSill {
        source: window,
        sill_m: None,
    });
    assert!(bad(result, Field::Sill));
}

#[test]
fn a_head_is_more_than_nothing_at_most_100_m_and_above_the_sill() {
    let mut s = open();

    for head_m in BAD.into_iter().chain([0.0]) {
        assert!(
            bad(s.edit_plan(head(Some(head_m))), Field::Head),
            "{head_m}"
        );
    }

    s.edit_plan(sill(Some(1.0))).unwrap();
    assert!(bad(s.edit_plan(head(Some(1.0))), Field::Head));
    assert!(bad(s.edit_plan(head(Some(0.5))), Field::Head));
    s.edit_plan(head(Some(100.0))).unwrap();
    assert_eq!(opening(&s).head_m, Some(100.0));
}

#[test]
fn only_openings_that_exist_have_a_sill_and_a_head() {
    let mut s = open();
    let gone = Source::Opening { wall: 2, index: 5 };

    for edit in [
        PlanEdit::SetOpeningSill {
            source: gone.clone(),
            sill_m: Some(0.5),
        },
        PlanEdit::SetOpeningHead {
            source: gone,
            head_m: Some(2.0),
        },
    ] {
        assert!(matches!(s.edit_plan(edit), Err(ServiceError::Gone)));
    }
    let wall = PlanEdit::SetOpeningSill {
        source: Source::Wall(2),
        sill_m: Some(0.5),
    };
    assert!(bad(s.edit_plan(wall), Field::Source));
}

#[test]
fn an_object_gets_an_underside_and_loses_it() {
    let mut s = open();
    assert_eq!(base(&s), None);

    s.edit_plan(base_edit("tisch", Some(0.4))).unwrap();
    assert_eq!(base(&s), Some(0.4));
    s.edit_plan(base_edit("tisch", Some(0.0))).unwrap();
    assert_eq!(base(&s), Some(0.0));

    s.edit_plan(base_edit("tisch", None)).unwrap();
    assert_eq!(base(&s), None);

    s.undo();
    assert_eq!(base(&s), Some(0.0));
    s.undo();
    assert_eq!(base(&s), Some(0.4));
}

#[test]
fn an_underside_is_from_the_floor_up_to_100_m() {
    let mut s = open();

    for base_m in [f64::NAN, f64::INFINITY, -0.1, 100.1] {
        let result = s.edit_plan(base_edit("tisch", Some(base_m)));
        assert!(bad(result, Field::Base), "{base_m}");
    }
    assert_eq!(base(&s), None);

    s.edit_plan(base_edit("tisch", Some(100.0))).unwrap();
    assert_eq!(base(&s), Some(100.0));
}

#[test]
fn only_objects_that_exist_have_an_underside() {
    let mut s = open();

    assert!(matches!(
        s.edit_plan(base_edit("o99", Some(1.0))),
        Err(ServiceError::Gone)
    ));
    let wall = PlanEdit::SetObjectBase {
        source: Source::Wall(0),
        base_m: Some(1.0),
    };
    assert!(bad(s.edit_plan(wall), Field::Source));
}

#[test]
fn a_locked_set_refuses_every_height() {
    let mut s = open();
    s.set_lock(Layer::Set, Lock::Locked);

    for edit in [
        wall_edit(0, Some(1.0)),
        sill(Some(0.2)),
        head(Some(2.0)),
        base_edit("tisch", Some(0.4)),
    ] {
        assert!(matches!(
            s.edit_plan(edit),
            Err(ServiceError::Locked(Layer::Set))
        ));
    }
}

#[test]
fn the_cards_read_the_effective_values_and_whether_they_are_defaults() {
    let mut s = open();

    let Some(Hint::Wall {
        height_m,
        height_default,
    }) = s.wall_hint(0)
    else {
        panic!("no wall hint");
    };
    assert_eq!((height_m, height_default), (2.6, true));
    s.edit_plan(wall_edit(0, Some(1.1))).unwrap();
    let Some(Hint::Wall {
        height_m,
        height_default,
    }) = s.wall_hint(0)
    else {
        panic!("no wall hint");
    };
    assert_eq!((height_m, height_default), (1.1, false));
    assert!(s.wall_hint(99).is_none());

    s.edit_plan(head(Some(2.4))).unwrap();
    let Some(Hint::Opening {
        sill_m,
        head_m,
        sill_default,
        head_default,
        ..
    }) = s.opening_hint(2, 0)
    else {
        panic!("no opening hint");
    };
    assert_eq!((sill_m, head_m), (0.0, 2.4));
    assert_eq!((sill_default, head_default), (true, false));

    s.edit_plan(base_edit("tisch", Some(0.4))).unwrap();
    let Some(Hint::Object {
        base_m,
        base_default,
        ..
    }) = s.object_hint("tisch")
    else {
        panic!("no object hint");
    };
    assert_eq!((base_m, base_default), (0.4, false));
}

fn room(name: &str, wall_height_m: f64) -> [ProjectEdit; 2] {
    [
        ProjectEdit::AddLocation {
            name: name.into(),
            width_m: 8.0,
            depth_m: 5.0,
            wall_height_m,
        },
        ProjectEdit::EditLocation {
            id: "saengersaal".into(),
            name: name.into(),
            width_m: 8.0,
            depth_m: 5.0,
            wall_height_m,
        },
    ]
}

#[test]
fn a_room_height_is_more_than_nothing_and_at_most_100_m() {
    let mut s = open();

    for wall_height_m in [0.0, -2.6, f64::NAN, f64::INFINITY, 100.1] {
        for edit in room("Halle", wall_height_m) {
            let result = s.edit_project(edit);
            assert!(
                matches!(
                    result,
                    Err(ServiceError::BadValue {
                        field: Field::RoomHeight,
                        ..
                    })
                ),
                "{wall_height_m}"
            );
        }
    }
    assert_eq!(s.project().locations.len(), 1);
    assert_eq!(s.project().locations[0].set.wall_height_m, 2.6);

    let [add, edit] = room("Hof", 100.0);
    s.edit_project(add).unwrap();
    assert_eq!(
        s.project().location("hof").unwrap().set.wall_height_m,
        100.0
    );
    let height = |s: &Session| {
        s.project()
            .location("saengersaal")
            .unwrap()
            .set
            .wall_height_m
    };

    s.edit_project(edit).unwrap();
    assert_eq!(height(&s), 100.0);
    s.undo();
    assert_eq!(height(&s), 2.6);
}
