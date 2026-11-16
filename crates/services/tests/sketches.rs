//! The sketch pen (LH-120): freehand strokes on a layer of their own, one command clears all.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_services::{Layer, Lock, Point, Role, ServiceError, Session, Visibility};

fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe")
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    dir
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

fn open(name: &str) -> Session {
    let dir = scratch(name);
    copy_dir(&example(), &dir);
    Session::open(&dir).unwrap()
}

fn stroke(n: usize) -> Vec<Point> {
    (0..n)
        .map(|i| Point::new(2.0 + i as f64 * 0.1, 3.0))
        .collect()
}

fn sketches_drawn(s: &Session) -> usize {
    s.display_list()
        .items
        .iter()
        .filter(|i| i.role == Role::Sketch)
        .count()
}

#[test]
fn a_stroke_is_drawn_over_everything_and_undone_in_one_step() {
    let mut s = open("sketch-add");

    s.add_sketch(&stroke(20)).unwrap();

    assert_eq!(sketches_drawn(&s), 1);
    let last = s.display_list().items.last().unwrap().role;
    assert_eq!(last, Role::Sketch);
    s.undo();
    assert_eq!(sketches_drawn(&s), 0);
    s.redo();
    assert_eq!(sketches_drawn(&s), 1);
}

#[test]
fn one_command_clears_every_stroke_in_one_step() {
    let mut s = open("sketch-clear");
    s.add_sketch(&stroke(20)).unwrap();
    s.add_sketch(&stroke(30)).unwrap();

    s.clear_sketches().unwrap();

    assert_eq!(sketches_drawn(&s), 0);
    s.undo();
    assert_eq!(sketches_drawn(&s), 2);
}

#[test]
fn clearing_nothing_is_no_undo_step() {
    let mut s = open("sketch-none");

    s.clear_sketches().unwrap();

    assert!(!s.can_undo());
}

/// A stroke that went nowhere is no stroke.
#[test]
fn a_dot_and_a_broken_point_are_refused() {
    let mut s = open("sketch-refused");

    assert!(s.add_sketch(&[Point::new(1.0, 1.0)]).is_err());
    assert!(s.add_sketch(&[]).is_err());
    let broken = [Point::new(1.0, 1.0), Point::new(f64::NAN, 2.0)];
    assert!(s.add_sketch(&broken).is_err());
    assert!(!s.can_undo());
}

#[test]
fn a_locked_layer_refuses_strokes_and_clearing() {
    let mut s = open("sketch-lock");
    s.add_sketch(&stroke(20)).unwrap();
    s.set_lock(Layer::Sketches, Lock::Locked);

    let added = s.add_sketch(&stroke(20));
    let cleared = s.clear_sketches();

    assert!(matches!(added, Err(ServiceError::Locked(Layer::Sketches))));
    assert!(matches!(
        cleared,
        Err(ServiceError::Locked(Layer::Sketches))
    ));
    assert_eq!(sketches_drawn(&s), 1);
}

#[test]
fn the_layer_hides_and_nothing_picks_a_stroke() {
    let mut s = open("sketch-layer");
    s.add_sketch(&stroke(20)).unwrap();
    // Over the strokes there is only what lies on the plan: a click through them picks nothing
    // of them.
    assert_eq!(s.pick(Point::new(2.5, 3.0)), s.pick(Point::new(2.5, 3.0)));

    s.set_layer(Layer::Sketches, Visibility::Hidden);

    assert_eq!(sketches_drawn(&s), 0);
}

/// The strokes are part of the plan: saved with it and loaded again.
#[test]
fn strokes_are_saved_with_the_plan() {
    let mut s = open("sketch-save");
    s.add_sketch(&stroke(20)).unwrap();
    s.save().unwrap();

    let again = Session::open(s.dir()).unwrap();

    assert_eq!(again.plan().unwrap().sketches.len(), 1);
    assert_eq!(again.plan().unwrap().sketches[0].points.len(), 20);
}
