//! Making and taking out the points of an area (LH-19), on the example project's plan, which
//! has one red arrow `d1`. The area `d2` is a triangle (0, 0), (4, 0), (1, 3).
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_services::{
    Dash, DrawingShape, Field, FigureColor, Fill, Layer, Lock, PlanEdit, Point, Role, ServiceError,
    Session, Shape, Snapping, Source,
};

fn open() -> Session {
    let dir: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe");
    let mut s = Session::open(&dir).unwrap();
    s.edit_plan(PlanEdit::AddDrawing {
        shape: area(&[p(0.0, 0.0), p(4.0, 0.0), p(1.0, 3.0)]),
        color: FigureColor::Cyan,
        dash: Dash::Solid,
        fill: Fill::Filled,
    })
    .unwrap();

    s
}

fn p(x_m: f64, y_m: f64) -> Point {
    Point::new(x_m, y_m)
}

fn area(points: &[Point]) -> DrawingShape {
    DrawingShape::Area {
        points: points.to_vec(),
    }
}

fn edge(index: usize) -> Source {
    Source::DrawingEdge {
        drawing: "d2".into(),
        index,
    }
}

fn point(index: usize) -> Source {
    Source::DrawingPoint {
        drawing: "d2".into(),
        index,
    }
}

fn points(s: &Session) -> Vec<Point> {
    match &s.plan().unwrap().drawings[1].shape {
        DrawingShape::Area { points } => points.clone(),
        other => unreachable!("{other:?}"),
    }
}

fn pull(source: Source, dx_m: f64, dy_m: f64) -> PlanEdit {
    PlanEdit::Shift { source, dx_m, dy_m }
}

fn grips(s: &Session, id: &str) -> Vec<(Source, Point)> {
    let selected = Source::Drawing(id.into());

    s.display_list_for(Some(&selected))
        .items
        .iter()
        .filter(|i| i.role == Role::Handle)
        .filter_map(|i| match (&i.source, &i.shape) {
            (Some(source), Shape::Circle { center, .. }) => Some((source.clone(), *center)),
            _ => None,
        })
        .collect()
}

/// An edge grip lies halfway along its edge, the last one on the closing edge.
#[test]
fn an_area_shows_a_grip_halfway_along_each_edge() {
    let s = open();

    let edges: Vec<_> = grips(&s, "d2")
        .into_iter()
        .filter(|(source, _)| matches!(source, Source::DrawingEdge { .. }))
        .collect();

    assert_eq!(
        edges,
        [
            (edge(0), p(2.0, 0.0)),
            (edge(1), p(2.5, 1.5)),
            (edge(2), p(0.5, 1.5)),
        ]
    );
}

/// Only an area has them; a line has its two ends.
#[test]
fn other_shapes_have_no_edge_grips() {
    let s = open();

    assert!(grips(&s, "d1")
        .iter()
        .all(|(source, _)| matches!(source, Source::DrawingPoint { .. })));
}

#[test]
fn an_edge_grip_is_picked_while_its_drawing_is_selected() {
    let s = open();
    let selected = Source::Drawing("d2".into());

    assert_eq!(s.pick_for(p(2.0, 0.0), Some(&selected)), Some(edge(0)));
    assert_eq!(s.pick_for(p(4.0, 0.0), Some(&selected)), Some(point(1)));
    assert_eq!(
        s.pick_for(p(2.0, 0.0), None),
        Some(Source::Drawing("d2".into()))
    );
}

/// Pulling the middle of the first edge down makes a point there, after point 0. One edit,
/// one undo step.
#[test]
fn pulling_an_edge_makes_a_point_in_one_step() {
    let mut s = open();
    s.set_snapping(Snapping::Off);

    s.edit_plan(pull(edge(0), 0.0, -1.0)).unwrap();

    assert_eq!(
        points(&s),
        [p(0.0, 0.0), p(2.0, -1.0), p(4.0, 0.0), p(1.0, 3.0)]
    );
    s.undo();
    assert_eq!(points(&s), [p(0.0, 0.0), p(4.0, 0.0), p(1.0, 3.0)]);
}

/// The closing edge's point goes last.
#[test]
fn the_closing_edge_adds_at_the_end() {
    let mut s = open();
    s.set_snapping(Snapping::Off);

    s.edit_plan(pull(edge(2), -1.0, 0.0)).unwrap();

    assert_eq!(
        points(&s),
        [p(0.0, 0.0), p(4.0, 0.0), p(1.0, 3.0), p(-0.5, 1.5)]
    );
}

/// A grip that is not pulled makes no point, snapping or not.
#[test]
fn an_edge_that_is_not_pulled_makes_no_point() {
    let mut s = open();

    s.edit_plan(pull(edge(0), 0.0, 0.0)).unwrap();

    assert_eq!(points(&s).len(), 3);
}

#[test]
fn a_new_point_snaps() {
    let mut s = open();

    s.edit_plan(pull(edge(0), 0.1, -1.1)).unwrap();

    assert_eq!(points(&s)[1], p(2.0, -1.0));
}

/// On a turned area the other points stay where they are shown.
#[test]
fn a_turned_area_keeps_its_points_in_place() {
    let mut s = open();
    s.edit_plan(PlanEdit::Spin {
        source: Source::Drawing("d2".into()),
        by_deg: 40.0,
    })
    .unwrap();
    s.set_snapping(Snapping::Off);
    let before: Vec<_> = grips(&s, "d2").into_iter().collect();
    let at =
        |list: &[(Source, Point)], source: &Source| list.iter().find(|g| g.0 == *source).unwrap().1;

    s.edit_plan(pull(edge(1), 1.0, 0.5)).unwrap();

    let after = grips(&s, "d2");
    let from = at(&before, &edge(1));
    // The new point is the second after the first (index 2 now).
    assert!(at(&after, &point(2)).distance_m(p(from.x_m + 1.0, from.y_m + 0.5)) < 1e-9);
    assert!(at(&after, &point(0)).distance_m(at(&before, &point(0))) < 1e-9);
    assert!(at(&after, &point(1)).distance_m(at(&before, &point(1))) < 1e-9);
    assert!(at(&after, &point(3)).distance_m(at(&before, &point(2))) < 1e-9);
}

#[test]
fn removing_a_point_takes_it_out_in_one_step() {
    let mut s = open();
    s.set_snapping(Snapping::Off);
    s.edit_plan(pull(edge(0), 0.0, -1.0)).unwrap();

    s.edit_plan(PlanEdit::Remove { source: point(1) }).unwrap();

    assert_eq!(points(&s), [p(0.0, 0.0), p(4.0, 0.0), p(1.0, 3.0)]);
    s.undo();
    assert_eq!(points(&s).len(), 4);
}

/// An area keeps three points: the refusal says why, nothing changes.
#[test]
fn an_area_keeps_three_points() {
    let mut s = open();
    let before = points(&s);

    let result = s.edit_plan(PlanEdit::Remove { source: point(0) });

    assert!(matches!(result, Err(ServiceError::FewPoints)), "{result:?}");
    assert_eq!(points(&s), before);
}

#[test]
fn only_an_area_loses_a_point() {
    let mut s = open();

    let result = s.edit_plan(PlanEdit::Remove {
        source: Source::DrawingPoint {
            drawing: "d1".into(),
            index: 0,
        },
    });

    assert!(matches!(
        result,
        Err(ServiceError::BadValue {
            field: Field::Source,
            ..
        })
    ));
}

#[test]
fn a_gone_point_or_edge_is_gone() {
    let mut s = open();

    assert!(matches!(
        s.edit_plan(PlanEdit::Remove { source: point(7) }),
        Err(ServiceError::Gone)
    ));
    assert!(matches!(
        s.edit_plan(pull(edge(7), 1.0, 0.0)),
        Err(ServiceError::Gone)
    ));
    assert!(matches!(
        s.edit_plan(PlanEdit::Remove { source: edge(0) }),
        Err(ServiceError::BadValue { .. })
    ));
}

#[test]
fn the_drawings_layer_locks_both_edits() {
    let mut s = open();
    s.set_lock(Layer::Drawings, Lock::Locked);

    for edit in [
        pull(edge(0), 0.0, -1.0),
        PlanEdit::Remove { source: point(0) },
    ] {
        let result = s.edit_plan(edit);

        assert!(matches!(result, Err(ServiceError::Locked(Layer::Drawings))));
    }
    assert_eq!(points(&s).len(), 3);
}
