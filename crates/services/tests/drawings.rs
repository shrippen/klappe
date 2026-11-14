//! Free drawing on the plan (LH-19), on the example project's plan: one red arrow `d1` from
//! (6, 10) to (6, 7.5).
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_services::{
    Dash, DrawingShape, FigureColor, Fill, Layer, Lock, PlanEdit, Point, Role, ServiceError,
    Session, Source, Visibility,
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
        (DrawingShape::Line { from: at, to: at }, "drawing"),
        (DrawingShape::Arrow { from: at, to: at }, "drawing"),
        (rect(at, p(5.0, 2.0)), "drawing"),
        (
            DrawingShape::Ellipse {
                from: at,
                to: p(2.0, 5.0),
            },
            "drawing",
        ),
        (
            DrawingShape::Area {
                points: vec![at, p(3.0, 3.0)],
            },
            "drawing",
        ),
        (rect(at, p(f64::NAN, 5.0)), "point"),
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

/// Rectangles and ellipses stay axis-parallel: turning a drawing changes nothing.
#[test]
fn turning_a_drawing_does_nothing() {
    let mut s = open();

    for edit in [
        PlanEdit::Spin {
            source: drawing("d1"),
            by_deg: 15.0,
        },
        PlanEdit::Turn {
            source: drawing("d1"),
            facing_deg: 90.0,
        },
    ] {
        s.edit_plan(edit).unwrap();
    }

    assert!(!s.can_undo());
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
