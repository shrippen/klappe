//! The grips that set the width and depth of a set object (LH-25), on the example project's
//! set: the table `tisch` at (10, 6.5), 6 m wide and 1.2 m deep, unturned, so its left edge is
//! at x = 7 and its top edge at y = 5.9.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_services::{
    Layer, Lock, ObjectGrip, PlanEdit, Point, Role, ServiceError, Session, Shape, Snapping, Source,
};

const MAX_M: f64 = 1000.0;

fn open() -> Session {
    let dir: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe");
    let mut s = Session::open(&dir).unwrap();
    s.set_snapping(Snapping::Off);

    s
}

fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

fn table() -> Source {
    Source::Object("tisch".into())
}

fn grip(grip: ObjectGrip) -> Source {
    Source::ObjectGrip {
        object: "tisch".into(),
        grip,
    }
}

/// (centre x, centre y, width, depth) of the table.
fn state(s: &Session) -> (f64, f64, f64, f64) {
    let o = &s.project().locations[0].set.objects[0];

    (o.at.x_m, o.at.y_m, o.width_m, o.depth_m)
}

fn drag(which: ObjectGrip, dx_m: f64, dy_m: f64) -> PlanEdit {
    PlanEdit::Shift {
        source: grip(which),
        dx_m,
        dy_m,
    }
}

fn turn_table(s: &mut Session, facing_deg: f64) {
    s.edit_plan(PlanEdit::Turn {
        source: table(),
        facing_deg,
    })
    .unwrap();
}

#[test]
fn a_selected_object_shows_its_grips_and_picks_them() {
    let s = open();
    let list = s.display_list_for(Some(&table()));
    let places: Vec<_> = list
        .items
        .iter()
        .filter(|i| i.role == Role::ObjectHandle)
        .filter_map(|i| match (&i.source, &i.shape) {
            (Some(source), Shape::Circle { center, .. }) => Some((source.clone(), *center)),
            _ => None,
        })
        .collect();

    assert_eq!(places.len(), 2);
    assert_eq!(
        s.pick_for(Point::new(13.45, 6.5), Some(&table())),
        Some(grip(ObjectGrip::Width))
    );
    assert!(s
        .display_list()
        .items
        .iter()
        .all(|i| i.role != Role::ObjectHandle));
}

/// One edit, one undo step; the left edge (x = 7) stays, so the centre moves half the change.
#[test]
fn dragging_the_width_grip_keeps_the_left_edge() {
    let mut s = open();

    s.edit_plan(drag(ObjectGrip::Width, 2.0, 5.0)).unwrap();

    let (x, y, width, depth) = state(&s);
    assert!(near(width, 8.0) && near(x, 11.0), "{x} {width}");
    assert!(near(y, 6.5) && near(depth, 1.2));
    s.undo();
    assert_eq!(state(&s), (10.0, 6.5, 6.0, 1.2));
    assert!(!s.can_undo());
}

/// The top edge (y = 5.9) stays.
#[test]
fn dragging_the_depth_grip_keeps_the_top_edge() {
    let mut s = open();

    s.edit_plan(drag(ObjectGrip::Depth, 5.0, 1.0)).unwrap();

    let (x, y, width, depth) = state(&s);
    assert!(near(depth, 2.2) && near(y, 7.0), "{y} {depth}");
    assert!(near(x, 10.0) && near(width, 6.0));
}

/// Turned by 90 degrees the width runs down the plan: the grip at (10, 9.5) pulled 1 m down
/// widens the table by 1 m and the centre moves 0.5 m down; a sideways pull changes nothing.
#[test]
fn a_turned_object_is_sized_along_its_own_axis() {
    let mut s = open();
    turn_table(&mut s, 90.0);

    s.edit_plan(drag(ObjectGrip::Width, 3.0, 1.0)).unwrap();

    let (x, y, width, depth) = state(&s);
    assert!(near(width, 7.0) && near(y, 7.0), "{y} {width}");
    assert!(near(x, 10.0) && near(depth, 1.2));

    // The bottom edge is now on the left: pulling the grip 1 m further left deepens it.
    s.edit_plan(drag(ObjectGrip::Depth, -1.0, 0.0)).unwrap();

    let (x, _, _, depth) = state(&s);
    assert!(near(depth, 2.2) && near(x, 9.5), "{x} {depth}");
}

#[test]
fn the_grips_stay_within_the_bounds() {
    let mut s = open();

    s.edit_plan(drag(ObjectGrip::Width, -100.0, 0.0)).unwrap();
    let (x, _, width, _) = state(&s);
    assert!(near(width, 0.05) && near(x, 7.025), "{x} {width}");

    s.edit_plan(drag(ObjectGrip::Depth, 0.0, 5000.0)).unwrap();
    let (_, _, _, depth) = state(&s);
    assert!(near(depth, MAX_M));
}

#[test]
fn a_locked_set_layer_refuses_the_grip() {
    let mut s = open();
    s.set_lock(Layer::Set, Lock::Locked);

    let result = s.edit_plan(drag(ObjectGrip::Width, 1.0, 0.0));

    assert!(matches!(result, Err(ServiceError::Locked(Layer::Set))));
    assert_eq!(state(&s), (10.0, 6.5, 6.0, 1.2));
}

#[test]
fn removing_a_grip_removes_nothing() {
    let mut s = open();

    let result = s.edit_plan(PlanEdit::Remove {
        source: grip(ObjectGrip::Depth),
    });

    assert!(result.is_err());
    assert_eq!(s.project().locations[0].set.objects.len(), 3);
}

/// Where the grips of the selected table are shown.
fn grip_places(s: &Session) -> Vec<(Source, Point)> {
    s.display_list_for(Some(&table()))
        .items
        .iter()
        .filter(|i| i.role == Role::ObjectHandle)
        .filter_map(|i| match (&i.source, &i.shape) {
            (Some(source), Shape::Circle { center, .. }) => Some((source.clone(), *center)),
            _ => None,
        })
        .collect()
}

/// A small object (a chair, a door from the library, a 10 cm box): a click on its middle still
/// takes the object, and each grip takes its own grip, so it can be moved and sized.
#[test]
fn the_grips_of_a_small_object_hide_neither_it_nor_each_other() {
    for (width_m, depth_m) in [(0.45, 0.45), (0.9, 0.2), (0.1, 0.1)] {
        let mut s = open();
        s.edit_plan(drag(ObjectGrip::Width, width_m - 6.0, 0.0))
            .unwrap();
        s.edit_plan(drag(ObjectGrip::Depth, 0.0, depth_m - 1.2))
            .unwrap();
        let (x_m, y_m, w, d) = state(&s);
        assert!(near(w, width_m) && near(d, depth_m), "{w} {d}");

        assert_eq!(
            s.pick_for(Point::new(x_m, y_m), Some(&table())),
            Some(table()),
            "{width_m} × {depth_m}: middle"
        );
        for (source, at) in grip_places(&s) {
            assert_eq!(
                s.pick_for(at, Some(&table())),
                Some(source.clone()),
                "{width_m} × {depth_m}: {source:?}"
            );
        }
    }
}

/// With snapping on, the edge lands on the grid, not the grip that sits outside it: the table
/// (left edge x = 7, right edge 13) pulled by half a metre is 6.5 m wide.
#[test]
fn a_snapped_drag_puts_the_edge_on_the_grid() {
    let mut s = open();
    s.set_snapping(Snapping::On);

    s.edit_plan(drag(ObjectGrip::Width, 0.5, 0.0)).unwrap();

    let (_, _, w, _) = state(&s);
    assert!(near(w, 6.5), "{w}");
}
