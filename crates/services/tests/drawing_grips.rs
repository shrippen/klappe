//! Grips on the points of a drawing (LH-19), on the example project's plan: one red arrow
//! `d1` from (6, 10) to (6, 7.5).
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_services::{
    Dash, DrawingShape, Field, FigureColor, Fill, Layer, Lock, PlanEdit, Point, Role, ServiceError,
    Session, Shape, Snapping, Source, Visibility,
};

fn open() -> Session {
    let dir: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe");

    Session::open(&dir).unwrap()
}

fn p(x_m: f64, y_m: f64) -> Point {
    Point::new(x_m, y_m)
}

fn drawing(id: &str) -> Source {
    Source::Drawing(id.into())
}

fn add(shape: DrawingShape, fill: Fill) -> PlanEdit {
    PlanEdit::AddDrawing {
        shape,
        color: FigureColor::Cyan,
        dash: Dash::Dotted,
        fill,
    }
}

fn rect(from: Point, to: Point) -> DrawingShape {
    DrawingShape::Rect { from, to }
}

fn shapes(s: &Session) -> Vec<(String, DrawingShape)> {
    s.plan()
        .unwrap()
        .drawings
        .iter()
        .map(|d| (d.id.clone(), d.shape.clone()))
        .collect()
}

fn near(a: Point, b: Point) -> bool {
    a.distance_m(b) < 1e-9
}

fn point(id: &str, index: usize) -> Source {
    Source::DrawingPoint {
        drawing: id.into(),
        index,
    }
}

/// Where the grips of the selected drawing `id` are shown, by index.
fn grips(s: &Session, id: &str) -> Vec<Point> {
    let list = s.display_list_for(Some(&drawing(id)));
    let mut found: Vec<_> = list
        .items
        .iter()
        .filter(|i| i.role == Role::Handle)
        .filter_map(|i| match (&i.source, &i.shape) {
            (Some(Source::DrawingPoint { index, .. }), Shape::Circle { center, .. }) => {
                Some((*index, *center))
            }
            _ => None,
        })
        .collect();
    found.sort_by_key(|g| g.0);

    found.into_iter().map(|g| g.1).collect()
}

fn drag(index: usize, id: &str, dx_m: f64, dy_m: f64) -> PlanEdit {
    PlanEdit::Shift {
        source: point(id, index),
        dx_m,
        dy_m,
    }
}

#[test]
fn a_selected_drawing_shows_a_grip_on_each_point() {
    let mut s = open();
    s.edit_plan(add(rect(p(0.0, 0.0), p(4.0, 2.0)), Fill::Outline))
        .unwrap();
    s.edit_plan(add(
        DrawingShape::Area {
            points: vec![p(8.0, 0.0), p(10.0, 0.0), p(9.0, 3.0)],
        },
        Fill::Outline,
    ))
    .unwrap();

    assert_eq!(grips(&s, "d1"), [p(6.0, 10.0), p(6.0, 7.5)]);
    assert_eq!(
        grips(&s, "d2"),
        [p(0.0, 0.0), p(4.0, 0.0), p(4.0, 2.0), p(0.0, 2.0)]
    );
    assert_eq!(grips(&s, "d3").len(), 3);
    // Only the selected one, and never in the plain list (or the export).
    let plain = s.display_list();
    assert!(plain.items.iter().all(|i| i.role != Role::Handle));
    let other = s.display_list_for(Some(&Source::Wall(0)));
    assert!(other.items.iter().all(|i| i.role != Role::Handle));
}

/// A grip lies in front of its drawing; without the drawing selected it is not there to take.
#[test]
fn a_grip_is_picked_only_while_its_drawing_is_selected() {
    let s = open();

    assert_eq!(
        s.pick_for(p(6.0, 7.5), Some(&drawing("d1"))),
        Some(point("d1", 1))
    );
    assert_eq!(s.pick_for(p(6.0, 7.5), None), Some(drawing("d1")));
    assert_eq!(
        s.pick_for(p(6.0, 9.0), Some(&drawing("d1"))),
        Some(drawing("d1"))
    );
}

/// Dragging the end of the arrow by (1, 0): the other end stays. One edit, one undo step.
#[test]
fn a_dragged_end_moves_alone_and_undoes_in_one_step() {
    let mut s = open();

    s.edit_plan(drag(1, "d1", 1.0, 0.0)).unwrap();

    assert_eq!(
        shapes(&s)[0].1,
        DrawingShape::Arrow {
            from: p(6.0, 10.0),
            to: p(7.0, 7.5)
        }
    );
    s.undo();
    assert_eq!(
        shapes(&s)[0].1,
        DrawingShape::Arrow {
            from: p(6.0, 10.0),
            to: p(6.0, 7.5)
        }
    );
    assert!(!s.can_undo());
}

/// Snapping is on, as when drawing: the end (6, 7.5) + 0.3 lands on 6.25.
#[test]
fn a_dragged_point_snaps() {
    let mut s = open();

    s.edit_plan(drag(1, "d1", 0.3, 0.0)).unwrap();

    assert_eq!(
        shapes(&s)[0].1,
        DrawingShape::Arrow {
            from: p(6.0, 10.0),
            to: p(6.25, 7.5)
        }
    );
}

/// A rectangle's grips are its four corners; the opposite one holds still.
#[test]
fn a_dragged_corner_resizes_the_rectangle() {
    let mut s = open();
    s.edit_plan(add(rect(p(0.0, 0.0), p(4.0, 2.0)), Fill::Outline))
        .unwrap();

    // The bottom right corner, to (6, 3).
    s.edit_plan(drag(2, "d2", 2.0, 1.0)).unwrap();
    assert_eq!(shapes(&s)[1].1, rect(p(6.0, 3.0), p(0.0, 0.0)));
    assert_eq!(
        grips(&s, "d2"),
        [p(0.0, 0.0), p(6.0, 0.0), p(6.0, 3.0), p(0.0, 3.0)]
    );

    // The top right corner, to (5, 1): its opposite, (0, 3), stays.
    s.edit_plan(drag(1, "d2", -1.0, 1.0)).unwrap();
    assert_eq!(
        grips(&s, "d2"),
        [p(0.0, 1.0), p(5.0, 1.0), p(5.0, 3.0), p(0.0, 3.0)]
    );
}

/// A turned rectangle: the dragged corner lands where it was put and the opposite corner,
/// as shown, does not move.
#[test]
fn a_turned_drawing_keeps_its_other_points_in_place() {
    let mut s = open();
    s.edit_plan(add(rect(p(0.0, 0.0), p(4.0, 2.0)), Fill::Outline))
        .unwrap();
    s.edit_plan(PlanEdit::Spin {
        source: drawing("d2"),
        by_deg: 30.0,
    })
    .unwrap();
    let before = grips(&s, "d2");

    // Snapping off: the target is exact.
    s.set_snapping(Snapping::Off);
    s.edit_plan(drag(1, "d2", 0.7, -0.4)).unwrap();

    let after = grips(&s, "d2");
    assert!(
        near(after[1], p(before[1].x_m + 0.7, before[1].y_m - 0.4)),
        "{after:?}"
    );
    assert!(near(after[3], before[3]), "{after:?}");
    assert_eq!(s.plan().unwrap().drawings[1].rotation_deg, 30.0);
}

#[test]
fn a_turned_area_keeps_its_other_points_in_place() {
    let mut s = open();
    s.edit_plan(add(
        DrawingShape::Area {
            points: vec![p(0.0, 0.0), p(4.0, 0.0), p(1.0, 3.0)],
        },
        Fill::Filled,
    ))
    .unwrap();
    s.edit_plan(PlanEdit::Spin {
        source: drawing("d2"),
        by_deg: 40.0,
    })
    .unwrap();
    let before = grips(&s, "d2");

    s.set_snapping(Snapping::Off);
    s.edit_plan(drag(2, "d2", -1.0, 2.0)).unwrap();

    let after = grips(&s, "d2");
    assert!(
        near(after[2], p(before[2].x_m - 1.0, before[2].y_m + 2.0)),
        "{after:?}"
    );
    assert!(near(after[0], before[0]), "{after:?}");
    assert!(near(after[1], before[1]), "{after:?}");
}

/// Nothing left to draw: an end dragged onto the other one, a rectangle with no width.
#[test]
fn a_drag_that_leaves_nothing_is_refused() {
    let mut s = open();
    s.edit_plan(add(rect(p(0.0, 0.0), p(4.0, 2.0)), Fill::Outline))
        .unwrap();

    for edit in [drag(1, "d1", 0.0, 2.5), drag(2, "d2", -4.0, 0.0)] {
        let result = s.edit_plan(edit);

        assert!(
            matches!(
                result,
                Err(ServiceError::BadValue {
                    field: Field::Drawing,
                    ..
                })
            ),
            "{result:?}"
        );
    }
    assert_eq!(
        shapes(&s)[0].1,
        DrawingShape::Arrow {
            from: p(6.0, 10.0),
            to: p(6.0, 7.5)
        }
    );
    assert_eq!(shapes(&s)[1].1, rect(p(0.0, 0.0), p(4.0, 2.0)));
}

#[test]
fn a_grip_is_no_element_to_remove_and_a_gone_one_is_gone() {
    let mut s = open();

    assert!(matches!(
        s.edit_plan(PlanEdit::Remove {
            source: point("d1", 0)
        }),
        Err(ServiceError::BadValue {
            field: Field::Source,
            ..
        })
    ));
    assert!(matches!(
        s.edit_plan(drag(5, "d1", 1.0, 0.0)),
        Err(ServiceError::Gone)
    ));
    assert!(matches!(
        s.edit_plan(drag(0, "d9", 1.0, 0.0)),
        Err(ServiceError::Gone)
    ));
    assert_eq!(shapes(&s).len(), 1);
}

#[test]
fn the_drawings_layer_locks_its_grips() {
    let mut s = open();
    s.set_lock(Layer::Drawings, Lock::Locked);

    let result = s.edit_plan(drag(1, "d1", 1.0, 0.0));

    assert!(matches!(result, Err(ServiceError::Locked(Layer::Drawings))));
    assert!(!s.is_dirty());
    // The grips are not there to take, and hidden with the layer.
    assert_eq!(s.pick_for(p(6.0, 7.5), Some(&drawing("d1"))), None);
    s.set_lock(Layer::Drawings, Lock::Unlocked);
    s.set_layer(Layer::Drawings, Visibility::Hidden);
    assert!(grips(&s, "d1").is_empty());
}

/// A drag shows the moved end without keeping it.
#[test]
fn a_preview_shows_the_dragged_grip() {
    let s = open();

    let list = s
        .preview_for(&drag(1, "d1", 1.0, 0.0), Some(&drawing("d1")))
        .unwrap();

    let ends: Vec<_> = list
        .items
        .iter()
        .filter(|i| i.role == Role::Handle)
        .collect();
    assert_eq!(ends.len(), 2);
    assert_eq!(
        shapes(&s)[0].1,
        DrawingShape::Arrow {
            from: p(6.0, 10.0),
            to: p(6.0, 7.5)
        }
    );
    assert!(!s.is_dirty());
}
