//! Appearing and leaving at a mark (LH-26), on the example project: Elisabeth's marks lie at
//! 0, 6 and 14 s with holds of 2, 4 and 0 s; camera A has marks.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_services::{
    Arrival, Departure, Layer, Lock, PlanEdit, Role, ServiceError, Session, Snapping, Source,
};

fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe")
}

fn open() -> Session {
    let mut s = Session::open(&example()).unwrap();
    s.set_snapping(Snapping::Off);
    s
}

fn mark(index: usize) -> Source {
    Source::Mark {
        character: "elisabeth".into(),
        index,
    }
}

fn arrival_of(s: &Session, index: usize) -> Arrival {
    s.mark_info(&mark(index)).unwrap().arrival.unwrap()
}

fn departure_of(s: &Session, index: usize) -> Departure {
    s.mark_info(&mark(index)).unwrap().departure.unwrap()
}

#[test]
fn a_mark_can_appear_and_leave() {
    let mut s = open();

    assert_eq!(arrival_of(&s, 1), Arrival::Walk);
    assert_eq!(departure_of(&s, 1), Departure::Stay);

    s.edit_plan(PlanEdit::SetArrival {
        source: mark(1),
        arrival: Arrival::Appear,
    })
    .unwrap();
    s.edit_plan(PlanEdit::SetDeparture {
        source: mark(2),
        departure: Departure::Leave,
    })
    .unwrap();

    assert_eq!(arrival_of(&s, 1), Arrival::Appear);
    assert_eq!(arrival_of(&s, 0), Arrival::Walk);
    assert_eq!(departure_of(&s, 2), Departure::Leave);
    assert_eq!(departure_of(&s, 1), Departure::Stay);

    // One undo step each.
    s.undo();
    assert_eq!(departure_of(&s, 2), Departure::Stay);
    assert_eq!(arrival_of(&s, 1), Arrival::Appear);
    s.undo();
    assert_eq!(arrival_of(&s, 1), Arrival::Walk);
}

/// The figure at the playhead stands for its mark there: at 7 s that is the second.
#[test]
fn the_figure_stands_for_its_mark_at_the_playhead() {
    let mut s = open();
    s.set_time(7.0);

    s.edit_plan(PlanEdit::SetDeparture {
        source: Source::Figure("elisabeth".into()),
        departure: Departure::Leave,
    })
    .unwrap();

    assert_eq!(departure_of(&s, 1), Departure::Leave);
}

/// The edit shows in the display list: after the pause at the leaving mark (6 s + 4 s) the
/// figure is not drawn.
#[test]
fn a_figure_that_left_is_not_drawn() {
    let mut s = open();
    let bodies = |s: &Session| {
        s.display_list()
            .items
            .iter()
            .filter(|i| matches!(i.role, Role::Figure(_)))
            .filter(|i| matches!(i.source, Some(Source::Figure(ref c)) if c == "elisabeth"))
            .count()
    };
    s.edit_plan(PlanEdit::SetDeparture {
        source: mark(1),
        departure: Departure::Leave,
    })
    .unwrap();

    s.set_time(8.0);
    assert_eq!(bodies(&s), 2);
    s.set_time(11.0);
    assert_eq!(bodies(&s), 0);
}

#[test]
fn a_camera_mark_has_neither() {
    let mut s = open();
    let camera = Source::Camera("a".into());

    let info = s.mark_info(&camera).unwrap();
    assert_eq!((info.arrival, info.departure), (None, None));

    let result = s.edit_plan(PlanEdit::SetArrival {
        source: camera,
        arrival: Arrival::Appear,
    });
    assert!(matches!(result, Err(ServiceError::BadValue { .. })));
}

#[test]
fn the_edits_follow_the_figure_lock() {
    let mut s = open();
    s.set_lock(Layer::Figures, Lock::Locked);

    let arrival = s.edit_plan(PlanEdit::SetArrival {
        source: mark(1),
        arrival: Arrival::Appear,
    });
    let departure = s.edit_plan(PlanEdit::SetDeparture {
        source: mark(1),
        departure: Departure::Leave,
    });

    assert!(matches!(arrival, Err(ServiceError::Locked(Layer::Figures))));
    assert!(matches!(
        departure,
        Err(ServiceError::Locked(Layer::Figures))
    ));
    assert_eq!(arrival_of(&s, 1), Arrival::Walk);
    assert_eq!(departure_of(&s, 1), Departure::Stay);
}

#[test]
fn an_unknown_mark_is_gone() {
    let mut s = open();

    let result = s.edit_plan(PlanEdit::SetArrival {
        source: mark(9),
        arrival: Arrival::Appear,
    });

    assert!(matches!(result, Err(ServiceError::Gone)));
}
