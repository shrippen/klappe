//! The free form edit: an outline becomes an object without a height (LH-92).
#![allow(clippy::unwrap_used)]

use std::path::Path;

use klappe_services::{
    Field, Layer, Lock, ObjectShape, PlanEdit, Point, ProjectEdit, ServiceError, Session,
};

fn open() -> Session {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe");

    Session::open(&path).unwrap()
}

fn p(x_m: f64, y_m: f64) -> Point {
    Point::new(x_m, y_m)
}

/// An L, 2 m wide and 3 m deep, with its box from (4, 5) to (6, 8).
fn l_shape() -> Vec<Point> {
    vec![
        p(4.0, 5.0),
        p(6.0, 5.0),
        p(6.0, 6.0),
        p(5.0, 6.0),
        p(5.0, 8.0),
        p(4.0, 8.0),
    ]
}

fn object_count(s: &Session) -> usize {
    s.project().locations[0].set.objects.len()
}

#[test]
fn an_outline_becomes_an_object_without_height() {
    let mut s = open();
    let before = object_count(&s);
    let missing = s.missing_heights();

    s.edit_plan(PlanEdit::AddShape { points: l_shape() })
        .unwrap();

    let objects = &s.project().locations[0].set.objects;
    assert_eq!(objects.len(), before + 1);
    let form = objects.last().unwrap();
    assert_eq!(form.kind, "form");
    assert_eq!(
        (form.at, form.width_m, form.depth_m),
        (p(5.0, 6.5), 2.0, 3.0)
    );
    assert_eq!(form.height_m, None);
    assert_eq!(s.missing_heights(), missing + 1);

    let ObjectShape::Polygon { points } = &form.shape else {
        panic!("not a polygon: {:?}", form.shape);
    };
    assert_eq!(points.len(), 6);
    assert_eq!(points[0], p(-1.0, -1.5));
    assert_eq!(points[4], p(0.0, 1.5));
}

#[test]
fn a_free_form_is_one_undo_step() {
    let mut s = open();
    let before = object_count(&s);

    s.edit_plan(PlanEdit::AddShape { points: l_shape() })
        .unwrap();
    s.undo();

    assert_eq!(object_count(&s), before);
    assert!(!s.can_undo());
}

#[test]
fn an_outline_without_an_area_is_refused() {
    let mut s = open();
    let before = object_count(&s);
    let flat = [
        vec![],
        vec![p(1.0, 1.0), p(2.0, 2.0)],
        vec![p(1.0, 1.0), p(2.0, 2.0), p(3.0, 3.0)],
        vec![p(1.0, 1.0), p(2.0, 1.0), p(f64::NAN, 3.0)],
    ];

    for points in flat {
        let err = s.edit_plan(PlanEdit::AddShape { points }).unwrap_err();

        assert!(matches!(
            err,
            ServiceError::BadValue {
                field: Field::Points,
                ..
            }
        ));
    }
    assert_eq!(object_count(&s), before);
    assert!(!s.can_undo());
}

#[test]
fn a_locked_set_layer_refuses_a_free_form() {
    let mut s = open();
    s.set_lock(Layer::Set, Lock::Locked);

    let err = s
        .edit_plan(PlanEdit::AddShape { points: l_shape() })
        .unwrap_err();

    assert!(matches!(err, ServiceError::Locked(Layer::Set)));
}

/// A free form filed in the library keeps its outline; a new object of that kind has it.
#[test]
fn a_filed_free_form_keeps_its_outline() {
    let mut s = open();
    s.edit_plan(PlanEdit::AddShape { points: l_shape() })
        .unwrap();
    let id = s.project().locations[0]
        .set
        .objects
        .last()
        .unwrap()
        .id
        .clone();

    s.edit_project(ProjectEdit::LibraryFromObject {
        location: "saengersaal".into(),
        object: id,
        name: "Winkel".into(),
    })
    .unwrap();
    s.edit_plan(PlanEdit::AddObject {
        kind: "winkel".into(),
        at: p(1.0, 1.0),
    })
    .unwrap();

    let objects = &s.project().locations[0].set.objects;
    assert_eq!(
        objects.last().unwrap().shape,
        objects[objects.len() - 2].shape
    );
    assert!(matches!(
        objects.last().unwrap().shape,
        ObjectShape::Polygon { .. }
    ));
}
