//! Free drawing on the plan (LH-19), on the example project's plan: one red arrow `d1` from
//! (6, 10) to (6, 7.5).
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_services::{
    Dash, DrawingShape, Field, FigureColor, Fill, Layer, Lock, PlanEdit, Point, Role, ServiceError,
    Session, Shape, Snapping, Source, Visibility,
};

fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe")
}

fn open() -> Session {
    Session::open(&example()).unwrap()
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

#[test]
fn the_example_shows_its_arrow() {
    let list = open().display_list();
    let of_d1 = |role: &dyn Fn(Role) -> bool| {
        list.items
            .iter()
            .filter(|i| i.source == Some(drawing("d1")) && role(i.role))
            .count()
    };

    assert_eq!(of_d1(&|r| matches!(r, Role::Drawing { .. })), 1);
    assert_eq!(of_d1(&|r| r == Role::ArrowHead(FigureColor::Red)), 1);
}

#[test]
fn adds_a_drawing_with_its_style_and_undoes() {
    let mut s = open();

    s.edit_plan(add(rect(p(1.0, 1.0), p(3.0, 2.0)), Fill::Filled))
        .unwrap();

    let plan = s.plan().unwrap();
    assert_eq!(plan.drawings[1].id, "d2");
    assert_eq!(plan.drawings[1].color, FigureColor::Cyan);
    assert_eq!(plan.drawings[1].dash, Dash::Dotted);
    assert_eq!(plan.drawings[1].fill, Fill::Filled);
    assert!(s.is_dirty());

    s.undo();
    assert_eq!(shapes(&s).len(), 1);
}

#[test]
fn a_freed_id_is_taken_again() {
    let mut s = open();
    let line = DrawingShape::Line {
        from: p(1.0, 1.0),
        to: p(2.0, 1.0),
    };

    s.edit_plan(add(line.clone(), Fill::Outline)).unwrap();
    s.edit_plan(PlanEdit::Remove {
        source: drawing("d1"),
    })
    .unwrap();
    s.edit_plan(add(line, Fill::Outline)).unwrap();

    let ids: Vec<_> = shapes(&s).into_iter().map(|d| d.0).collect();
    assert_eq!(ids, ["d2", "d1"]);
}

/// Nothing to draw: no length, no width or height, fewer than three points, no number.
#[test]
fn a_drawing_of_nothing_is_refused() {
    let at = p(2.0, 2.0);
    let cases = [
        (DrawingShape::Line { from: at, to: at }, Field::Drawing),
        (DrawingShape::Arrow { from: at, to: at }, Field::Drawing),
        (rect(at, p(5.0, 2.0)), Field::Drawing),
        (
            DrawingShape::Ellipse {
                from: at,
                to: p(2.0, 5.0),
            },
            Field::Drawing,
        ),
        (
            DrawingShape::Area {
                points: vec![at, p(3.0, 3.0)],
            },
            Field::Drawing,
        ),
        (rect(at, p(f64::NAN, 5.0)), Field::Point),
    ];

    for (shape, field) in cases {
        let mut s = open();

        let result = s.edit_plan(add(shape.clone(), Fill::Outline));

        assert!(
            matches!(result, Err(ServiceError::BadValue { field: f, .. }) if f == field),
            "{shape:?}"
        );
        assert!(!s.is_dirty());
    }
}

/// The middle of the arrow's box, (6, 8.75), lands on (10, 5): both ends move by (4, -3.75).
#[test]
fn a_drawing_moves_as_a_whole() {
    let mut s = open();

    s.edit_plan(PlanEdit::Move {
        source: drawing("d1"),
        to: p(10.0, 5.0),
    })
    .unwrap();

    assert_eq!(
        shapes(&s)[0].1,
        DrawingShape::Arrow {
            from: p(10.0, 6.25),
            to: p(10.0, 3.75),
        }
    );
}

/// Snapping is on: the first point (6, 10) + 0.3 lands on 6.25, so the whole arrow moves
/// 0.25 and keeps its ends on the grid.
#[test]
fn a_shifted_drawing_snaps_its_first_point() {
    let mut s = open();

    s.edit_plan(PlanEdit::Shift {
        source: drawing("d1"),
        dx_m: 0.3,
        dy_m: 0.0,
    })
    .unwrap();

    assert_eq!(
        shapes(&s)[0].1,
        DrawingShape::Arrow {
            from: p(6.25, 10.0),
            to: p(6.25, 7.5),
        }
    );
}

#[test]
fn an_area_moves_by_the_middle_of_its_box() {
    let mut s = open();
    let triangle = DrawingShape::Area {
        points: vec![p(0.0, 0.0), p(4.0, 0.0), p(0.0, 2.0)],
    };
    s.edit_plan(add(triangle, Fill::Filled)).unwrap();

    // Box (0,0)-(4,2), middle (2,1), to (5,5): moves by (3, 4).
    s.edit_plan(PlanEdit::Move {
        source: drawing("d2"),
        to: p(5.0, 5.0),
    })
    .unwrap();

    assert_eq!(
        shapes(&s)[1].1,
        DrawingShape::Area {
            points: vec![p(3.0, 4.0), p(7.0, 4.0), p(3.0, 6.0)],
        }
    );
}

#[test]
fn removes_a_drawing_once() {
    let mut s = open();
    let remove = || PlanEdit::Remove {
        source: drawing("d1"),
    };

    s.edit_plan(remove()).unwrap();

    assert!(shapes(&s).is_empty());
    assert!(matches!(s.edit_plan(remove()), Err(ServiceError::Gone)));
    s.undo();
    assert_eq!(shapes(&s).len(), 1);
}

/// The shaft of drawing `id` as the plan shows it: its first item.
fn shaft(s: &Session, id: &str) -> (Point, Point) {
    let list = s.display_list();
    let item = list
        .items
        .iter()
        .find(|i| i.source == Some(drawing(id)) && matches!(i.role, Role::Drawing { .. }))
        .unwrap();
    let Shape::Line { from, to, .. } = item.shape else {
        panic!("{:?}", item.shape);
    };

    (from, to)
}

fn near(a: Point, b: Point) -> bool {
    a.distance_m(b) < 1e-9
}

/// The arrow (6, 10) to (6, 7.5) turns about its middle (6, 8.75), clockwise on screen; its
/// points stay as they are, the turn is a field of the drawing.
#[test]
fn a_drawing_turns_about_its_middle_and_undoes() {
    let mut s = open();

    s.edit_plan(PlanEdit::Spin {
        source: drawing("d1"),
        by_deg: 90.0,
    })
    .unwrap();

    let (from, to) = shaft(&s, "d1");
    assert!(near(from, p(4.75, 8.75)), "{from:?}");
    assert!(near(to, p(7.25, 8.75)), "{to:?}");
    let d = &s.plan().unwrap().drawings[0];
    assert_eq!(d.rotation_deg, 90.0);
    assert_eq!(
        d.shape,
        DrawingShape::Arrow {
            from: p(6.0, 10.0),
            to: p(6.0, 7.5)
        }
    );

    s.edit_plan(PlanEdit::Turn {
        source: drawing("d1"),
        facing_deg: 450.0,
    })
    .unwrap();
    assert_eq!(s.plan().unwrap().drawings[0].rotation_deg, 90.0);

    s.undo();
    s.undo();
    assert_eq!(s.plan().unwrap().drawings[0].rotation_deg, 0.0);
}

/// A rectangle turned 90 degrees stands on its end: 4 wide and 2 deep become 2 and 4, about
/// the same middle.
#[test]
fn a_rectangle_turns_with_its_corners() {
    let mut s = open();
    s.edit_plan(add(rect(p(0.0, 0.0), p(4.0, 2.0)), Fill::Outline))
        .unwrap();

    s.edit_plan(PlanEdit::Spin {
        source: drawing("d2"),
        by_deg: 90.0,
    })
    .unwrap();

    let list = s.display_list();
    let Shape::Polyline { points } = &list
        .items
        .iter()
        .find(|i| i.source == Some(drawing("d2")))
        .unwrap()
        .shape
    else {
        panic!("not an outline");
    };
    let xs = points.iter().map(|q| q.x_m);
    let ys = points.iter().map(|q| q.y_m);
    let span = |v: Vec<f64>| {
        v.iter().cloned().fold(f64::MIN, f64::max) - v.iter().cloned().fold(f64::MAX, f64::min)
    };
    assert!((span(xs.collect()) - 2.0).abs() < 1e-9);
    assert!((span(ys.collect()) - 4.0).abs() < 1e-9);
    assert_eq!(s.pick(p(2.0, 3.0)), Some(drawing("d2")));
    assert_eq!(s.pick(p(4.0, 1.0)), None);
}

/// A group turns about the middle of its box, and a drawing in it turns as well as swings.
#[test]
fn a_drawing_in_a_group_turns_with_it() {
    let mut s = open();
    s.edit_plan(add(rect(p(0.0, 0.0), p(4.0, 2.0)), Fill::Outline))
        .unwrap();

    s.edit_plan(PlanEdit::SpinGroup {
        sources: vec![drawing("d2")],
        by_deg: 30.0,
    })
    .unwrap();

    assert_eq!(s.plan().unwrap().drawings[1].rotation_deg, 30.0);
}

/// A turned drawing moves and snaps by where its first point lies on the plan.
#[test]
fn a_turned_drawing_shifts_by_its_shown_first_point() {
    let mut s = open();
    s.edit_plan(PlanEdit::Spin {
        source: drawing("d1"),
        by_deg: 90.0,
    })
    .unwrap();

    // The shown first point is (4.75, 8.75); +0.3 snaps to 5.0, so the arrow moves 0.25.
    s.edit_plan(PlanEdit::Shift {
        source: drawing("d1"),
        dx_m: 0.3,
        dy_m: 0.0,
    })
    .unwrap();

    let (from, _) = shaft(&s, "d1");
    assert!(near(from, p(5.0, 8.75)), "{from:?}");
}

#[test]
fn a_locked_drawings_layer_keeps_the_turn() {
    let mut s = open();
    s.set_lock(Layer::Drawings, Lock::Locked);

    let result = s.edit_plan(PlanEdit::Spin {
        source: drawing("d1"),
        by_deg: 15.0,
    });

    assert!(matches!(result, Err(ServiceError::Locked(Layer::Drawings))));
    assert!(!s.can_undo());
}

fn restyle(source: Source) -> PlanEdit {
    PlanEdit::SetDrawingStyle {
        source,
        color: FigureColor::Green,
        dash: Dash::Dashed,
        fill: Fill::Filled,
    }
}

#[test]
fn a_drawing_changes_its_style_and_undoes() {
    let mut s = open();

    s.edit_plan(restyle(drawing("d1"))).unwrap();

    let d = &s.plan().unwrap().drawings[0];
    assert_eq!(
        (d.color, d.dash, d.fill),
        (FigureColor::Green, Dash::Dashed, Fill::Filled)
    );
    assert_eq!(
        d.shape,
        DrawingShape::Arrow {
            from: p(6.0, 10.0),
            to: p(6.0, 7.5)
        }
    );
    assert!(s.is_dirty());
    let role = Role::Drawing {
        color: FigureColor::Green,
        dash: Dash::Dashed,
        fill: Fill::Filled,
    };
    assert!(s.display_list().items.iter().any(|i| i.role == role));

    s.undo();

    let d = &s.plan().unwrap().drawings[0];
    assert_eq!(
        (d.color, d.dash, d.fill),
        (FigureColor::Red, Dash::Solid, Fill::Outline)
    );
}

#[test]
fn only_a_known_drawing_changes_its_style() {
    let mut s = open();

    assert!(matches!(
        s.edit_plan(restyle(drawing("d9"))),
        Err(ServiceError::Gone)
    ));
    assert!(matches!(
        s.edit_plan(restyle(Source::Wall(0))),
        Err(ServiceError::BadValue {
            field: Field::Source,
            ..
        })
    ));
    assert!(!s.is_dirty());
}

#[test]
fn a_locked_drawings_layer_keeps_the_style() {
    let mut s = open();
    s.set_lock(Layer::Drawings, Lock::Locked);

    let result = s.edit_plan(restyle(drawing("d1")));

    assert!(matches!(result, Err(ServiceError::Locked(Layer::Drawings))));
    assert!(!s.is_dirty());
}

#[test]
fn a_drawing_hint_tells_style_and_whether_it_has_an_inside() {
    let mut s = open();
    s.edit_plan(add(rect(p(1.0, 1.0), p(3.0, 2.0)), Fill::Filled))
        .unwrap();

    let line = serde_json::to_value(s.drawing_hint("d1").unwrap()).unwrap();
    let boxed = serde_json::to_value(s.drawing_hint("d2").unwrap()).unwrap();

    assert_eq!(line["kind"], "drawing");
    assert_eq!(line["closed"], false);
    assert_eq!(boxed["closed"], true);
    assert_eq!(
        (boxed["color"].as_str(), boxed["fill"].as_str()),
        (Some("cyan"), Some("filled"))
    );
    assert!(s.drawing_hint("d9").is_none());
}

/// The shaft is hairline: 0.15 m beside it still hits, 0.4 m does not.
#[test]
fn pick_finds_a_line_by_its_stroke() {
    let s = open();

    assert_eq!(s.pick(p(6.1, 9.0)), Some(drawing("d1")));
    assert_eq!(s.pick(p(6.4, 9.0)), None);
}

/// Inside a hollow shape a click reaches what lies below; a filled one is taken.
#[test]
fn only_a_filled_shape_is_picked_from_inside() {
    let mut s = open();
    let at = p(16.0, 11.0);
    s.edit_plan(add(rect(p(15.0, 10.0), p(18.0, 12.0)), Fill::Outline))
        .unwrap();
    assert_eq!(s.pick(at), None);
    assert_eq!(s.pick(p(15.0, 11.0)), Some(drawing("d2")));

    s.edit_plan(add(rect(p(15.0, 10.0), p(18.0, 12.0)), Fill::Filled))
        .unwrap();

    assert_eq!(s.pick(at), Some(drawing("d3")));
}

#[test]
fn the_drawings_layer_hides_drawings_only() {
    let mut s = open();

    s.set_layer(Layer::Drawings, Visibility::Hidden);

    let list = s.display_list();
    let is_drawing = |r: Role| matches!(r, Role::Drawing { .. } | Role::ArrowHead(_));
    assert!(list.items.iter().all(|i| !is_drawing(i.role)));
    assert!(list.items.iter().any(|i| i.role == Role::Wall));
    assert_eq!(s.pick(p(6.0, 9.0)), None);
}

#[test]
fn the_drawings_layer_locks_like_the_others() {
    let mut s = open();
    s.set_lock(Layer::Drawings, Lock::Locked);

    let edits = [
        PlanEdit::Remove {
            source: drawing("d1"),
        },
        PlanEdit::Move {
            source: drawing("d1"),
            to: p(1.0, 1.0),
        },
        add(rect(p(1.0, 1.0), p(2.0, 2.0)), Fill::Outline),
    ];

    for edit in edits {
        assert!(
            matches!(
                s.edit_plan(edit.clone()),
                Err(ServiceError::Locked(Layer::Drawings))
            ),
            "{edit:?}"
        );
    }
    assert_eq!(s.pick(p(6.0, 9.0)), None);
    assert!(!s.is_dirty());
}

/// A drag shows the new drawing without keeping it.
#[test]
fn a_preview_draws_without_committing() {
    let s = open();
    let before = s.display_list();

    let list = s
        .preview(&add(rect(p(1.0, 1.0), p(3.0, 2.0)), Fill::Filled))
        .unwrap();

    assert_eq!(list.items.len(), before.items.len() + 1);
    assert_eq!(shapes(&s).len(), 1);
    assert!(!s.is_dirty());
}

#[test]
fn drawings_survive_saving_and_loading() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("drawings-save");
    let _ = std::fs::remove_dir_all(&dir);
    copy_dir(&example(), &dir);
    let mut s = Session::open(&dir).unwrap();

    s.edit_plan(add(
        DrawingShape::Ellipse {
            from: p(2.0, 2.0),
            to: p(4.0, 3.0),
        },
        Fill::Filled,
    ))
    .unwrap();
    s.save().unwrap();

    let again = Session::open(&dir).unwrap();
    assert_eq!(again.project(), s.project());
    assert_eq!(shapes(&again).len(), 2);
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
            continue;
        }
        std::fs::copy(entry.path(), target).unwrap();
    }
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
        .filter(|i| i.role == Role::DrawingHandle)
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
    assert!(plain.items.iter().all(|i| i.role != Role::DrawingHandle));
    let other = s.display_list_for(Some(&Source::Wall(0)));
    assert!(other.items.iter().all(|i| i.role != Role::DrawingHandle));
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
        .filter(|i| i.role == Role::DrawingHandle)
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
