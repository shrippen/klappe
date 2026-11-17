//! Copying a figure or a mark, and taking the grips of a selected way with a box or lasso
//! (LH-08), on the example project (`testdata/beispiel.klappe`). Elisabeth: marks (3, 6.5) at
//! 0 s with a 2 s pause, (10, 6.5) at 6 s, (16, 3) at 14 s. Burgvogt walks from (17, 9.5) to
//! (12, 6.5).
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_services::{
    Bend, Layer, Lock, PlanEdit, Point, ServiceError, Session, Snapping, Source,
};

/// Walking speed that times a way, as new marks do (m/s).
const WALK_M_S: f64 = 1.4;

const BY: (f64, f64) = (0.5, 0.5);

fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe")
}

fn open() -> Session {
    let mut s = Session::open(&example()).unwrap();
    s.set_snapping(Snapping::Off);
    s
}

fn p(x_m: f64, y_m: f64) -> Point {
    Point::new(x_m, y_m)
}

fn near(a: Point, b: Point) -> bool {
    a.distance_m(b) < 1e-9
}

fn elisabeth() -> Source {
    Source::Figure("elisabeth".into())
}

fn mark(index: usize) -> Source {
    Source::Mark {
        character: "elisabeth".into(),
        index,
    }
}

fn copy(s: &mut Session, sources: Vec<Source>) -> Result<(), ServiceError> {
    s.edit_plan(PlanEdit::Duplicate {
        sources,
        dx_m: BY.0,
        dy_m: BY.1,
    })
}

/// Elisabeth's marks as place and time.
fn places(s: &Session) -> Vec<(Point, f64)> {
    let track = s.plan().unwrap().figure("elisabeth").unwrap();

    track.marks.iter().map(|m| (m.at, m.at_s)).collect()
}

#[test]
fn a_copied_figure_is_a_character_of_its_own() {
    let mut s = open();
    let before = s.project().clone();

    copy(&mut s, vec![elisabeth()]).unwrap();

    let copied = s.project().characters.last().unwrap().clone();
    let original = s.project().character("elisabeth").unwrap();
    assert_eq!(s.project().characters.len(), before.characters.len() + 1);
    assert_eq!(copied.name, "Elisabeth 2");
    assert_eq!(copied.short, "E2");
    assert_eq!(copied.color, original.color);
    assert_ne!(copied.id, "elisabeth");

    let plan = s.plan().unwrap();
    let track = plan.figure(&copied.id).unwrap();
    let first = &plan.figure("elisabeth").unwrap().marks;
    assert_eq!(track.marks.len(), first.len());
    for (new, old) in track.marks.iter().zip(first) {
        assert!(near(new.at, p(old.at.x_m + BY.0, old.at.y_m + BY.1)));
        assert_eq!(new.at_s, old.at_s);
    }
    assert!(s
        .display_list()
        .items
        .iter()
        .any(|i| { i.source == Some(Source::Figure(copied.id.clone())) }));
}

/// One step takes character and track back.
#[test]
fn undo_takes_the_whole_copy_back() {
    let mut s = open();
    let before = s.project().clone();

    copy(&mut s, vec![elisabeth()]).unwrap();
    s.undo();

    assert_eq!(*s.project(), before);
    s.redo();
    assert_eq!(s.project().characters.len(), before.characters.len() + 1);
}

#[test]
fn the_next_copy_takes_the_next_free_number() {
    let mut s = open();

    copy(&mut s, vec![elisabeth()]).unwrap();
    copy(&mut s, vec![elisabeth()]).unwrap();
    let second = s.project().characters.last().unwrap().id.clone();
    copy(&mut s, vec![Source::Figure(second)]).unwrap();

    let names: Vec<&str> = s
        .project()
        .characters
        .iter()
        .map(|c| c.name.as_str())
        .collect();
    assert_eq!(
        names,
        [
            "Elisabeth",
            "Burgvogt",
            "Elisabeth 2",
            "Elisabeth 3",
            "Elisabeth 4"
        ]
    );
    let shorts: Vec<&str> = s
        .project()
        .characters
        .iter()
        .map(|c| c.short.as_str())
        .collect();
    assert_eq!(shorts, ["E", "B", "E2", "E3", "E4"]);
    let ids: std::collections::HashSet<_> = s.project().characters.iter().map(|c| &c.id).collect();
    assert_eq!(ids.len(), 5);
}

#[test]
fn a_locked_figure_layer_refuses_the_copy() {
    let mut s = open();
    s.set_lock(Layer::Figures, Lock::Locked);

    let result = copy(&mut s, vec![elisabeth()]);

    assert!(matches!(result, Err(ServiceError::Locked(_))));
    assert_eq!(s.project().characters.len(), 2);
}

#[test]
fn a_copied_mark_follows_the_original_on_the_way() {
    let mut s = open();
    let before = places(&s);

    copy(&mut s, vec![mark(1)]).unwrap();

    let after = places(&s);
    let hold_s = s.plan().unwrap().figure("elisabeth").unwrap().marks[1].hold_s;
    let walk_s = before[1].0.distance_m(p(10.5, 7.0)) / WALK_M_S;
    assert_eq!(after.len(), 4);
    assert_eq!(after[0], before[0]);
    assert_eq!(after[1], before[1]);
    assert!(near(after[2].0, p(10.5, 7.0)));
    assert!((after[2].1 - (before[1].1 + hold_s + walk_s)).abs() < 1e-9);
    // The way on keeps its duration.
    assert!(near(after[3].0, before[2].0));
    assert!((after[3].1 - (before[2].1 + walk_s)).abs() < 1e-9);

    s.undo();

    assert_eq!(places(&s), before);
}

#[test]
fn marks_of_one_way_are_copied_each_behind_its_own() {
    let mut s = open();
    let before = places(&s);

    copy(&mut s, vec![mark(0), mark(1)]).unwrap();

    let after = places(&s);
    assert_eq!(after.len(), 5);
    assert!(near(after[0].0, before[0].0));
    assert!(near(after[1].0, p(3.5, 7.0)));
    assert!(near(after[2].0, before[1].0));
    assert!(near(after[3].0, p(10.5, 7.0)));
    assert!(near(after[4].0, before[2].0));
}

#[test]
fn a_grip_has_still_no_copy() {
    let mut s = open();
    let grip = Source::FigureVia {
        character: "burgvogt".into(),
        index: 1,
        bend: Bend::New(0),
    };

    let result = copy(&mut s, vec![grip]);

    assert!(matches!(result, Err(ServiceError::NotCopyable)));
    assert!(!s.is_dirty());
}

/// Burgvogt's way with two bends, at (14.5, 4) and (12.5, 4.5) in the order of the way.
fn two_bends(s: &mut Session) {
    let pull = |piece: usize, dx_m: f64, dy_m: f64| PlanEdit::Shift {
        source: Source::FigureVia {
            character: "burgvogt".into(),
            index: 1,
            bend: Bend::New(piece),
        },
        dx_m,
        dy_m,
    };

    s.edit_plan(pull(0, 0.0, -4.0)).unwrap();
    let grips = s.display_list_for(Some(&burgvogt()));
    let last = grips
        .items
        .iter()
        .filter_map(|i| match &i.source {
            Some(Source::FigureVia {
                bend: Bend::New(n), ..
            }) => Some(*n),
            _ => None,
        })
        .max()
        .unwrap();
    s.edit_plan(pull(last, -1.0, -3.0)).unwrap();
}

fn burgvogt() -> Source {
    Source::Figure("burgvogt".into())
}

fn bends(s: &Session) -> Vec<Point> {
    s.plan().unwrap().figure("burgvogt").unwrap().marks[1]
        .via
        .clone()
}

fn bend(n: usize) -> Source {
    Source::FigureVia {
        character: "burgvogt".into(),
        index: 1,
        bend: Bend::At(n),
    }
}

const WHOLE: (Point, Point) = (
    Point { x_m: 0.0, y_m: 0.0 },
    Point {
        x_m: 20.0,
        y_m: 13.0,
    },
);

#[test]
fn a_box_takes_the_grips_of_the_selected_way() {
    let mut s = open();
    two_bends(&mut s);
    let via = bends(&s);
    assert_eq!(via.len(), 2);

    let all = s.pick_grips_in(&burgvogt(), WHOLE.0, WHOLE.1);
    let one = s.pick_grips_in(
        &burgvogt(),
        p(via[0].x_m - 0.2, via[0].y_m - 0.2),
        p(via[0].x_m + 0.2, via[0].y_m + 0.2),
    );

    // The bends only: the grips for a bend not made yet are none to move.
    assert_eq!(all, vec![bend(0), bend(1)]);
    assert_eq!(one, vec![bend(0)]);
}

#[test]
fn a_box_never_takes_the_grips_of_another_element() {
    let mut s = open();
    two_bends(&mut s);

    let found = s.pick_grips_in(&elisabeth(), WHOLE.0, WHOLE.1);

    assert!(found.iter().all(|grip| matches!(
        grip,
        Source::FigureVia { character, .. } if character == "elisabeth"
    )));
}

#[test]
fn a_lasso_takes_the_grips_of_the_selected_way() {
    let mut s = open();
    two_bends(&mut s);
    let via = bends(&s);
    let around = |at: Point| {
        vec![
            p(at.x_m - 0.5, at.y_m - 0.5),
            p(at.x_m + 0.5, at.y_m - 0.5),
            p(at.x_m + 0.5, at.y_m + 0.5),
            p(at.x_m - 0.5, at.y_m + 0.5),
        ]
    };

    let found = s.pick_grips_lasso(&burgvogt(), &around(via[1]));

    assert_eq!(found, vec![bend(1)]);
    assert!(s.pick_grips_lasso(&elisabeth(), &around(via[1])).is_empty());
}

#[test]
fn the_grips_of_a_box_move_together() {
    let mut s = open();
    two_bends(&mut s);
    let before = bends(&s);
    let grips = s.pick_grips_in(&burgvogt(), WHOLE.0, WHOLE.1);

    s.edit_plan(PlanEdit::ShiftGroup {
        sources: grips,
        dx_m: 1.0,
        dy_m: 0.5,
    })
    .unwrap();

    let after = bends(&s);
    assert_eq!(after.len(), 2);
    for (new, old) in after.iter().zip(&before) {
        assert!(near(*new, p(old.x_m + 1.0, old.y_m + 0.5)));
    }
}

#[test]
fn a_locked_layer_keeps_its_grips_out_of_the_box() {
    let mut s = open();
    two_bends(&mut s);
    s.set_lock(Layer::Figures, Lock::Locked);

    assert!(s.pick_grips_in(&burgvogt(), WHOLE.0, WHOLE.1).is_empty());
}
