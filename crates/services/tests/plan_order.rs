//! Moving the plans of a scene (LH-04), on a copy of the example project: scene `m4djbtj6`
//! has one plan, `plan-a`, to which the tests add `plan-b` and `plan-c`.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_services::{PlanShift, ServiceError, Session};

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

/// Three plans, `plan-a` on view first.
fn three(name: &str) -> Session {
    let mut s = open(name);
    s.add_plan("").unwrap();
    s.add_plan("").unwrap();
    s.select_plan("plan-a").unwrap();
    s
}

fn ids(s: &Session) -> Vec<String> {
    s.scene()
        .unwrap()
        .plans
        .iter()
        .map(|p| p.id.clone())
        .collect()
}

#[test]
fn a_plan_moves_later_and_earlier() {
    let mut s = three("order-moves");

    s.move_plan("plan-a", PlanShift::Later).unwrap();
    assert_eq!(ids(&s), ["plan-b", "plan-a", "plan-c"]);

    s.move_plan("plan-c", PlanShift::Earlier).unwrap();
    assert_eq!(ids(&s), ["plan-b", "plan-c", "plan-a"]);
}

#[test]
fn the_view_stays_on_the_plan_it_showed() {
    let mut s = three("order-view");
    s.select_plan("plan-b").unwrap();

    // Another plan moves across the one on view, then the one on view moves itself.
    s.move_plan("plan-a", PlanShift::Later).unwrap();
    assert_eq!(s.plan().unwrap().id, "plan-b");

    s.move_plan("plan-b", PlanShift::Later).unwrap();
    assert_eq!(ids(&s), ["plan-a", "plan-b", "plan-c"]);
    assert_eq!(s.plan().unwrap().id, "plan-b");
}

#[test]
fn moving_is_one_undo_step() {
    let mut s = three("order-undo");
    let before = ids(&s);

    s.move_plan("plan-a", PlanShift::Later).unwrap();
    s.undo();

    assert_eq!(ids(&s), before);
    assert_eq!(s.plan().unwrap().id, "plan-a");

    s.redo();

    assert_eq!(ids(&s), ["plan-b", "plan-a", "plan-c"]);
}

#[test]
fn the_ends_do_nothing() {
    let mut s = three("order-ends");
    let before = ids(&s);

    s.move_plan("plan-a", PlanShift::Earlier).unwrap();
    s.move_plan("plan-c", PlanShift::Later).unwrap();

    assert_eq!(ids(&s), before);
    // The last undo step is still the third plan's, not a move.
    s.undo();
    assert_eq!(ids(&s), ["plan-a", "plan-b"]);
}

#[test]
fn a_single_plan_has_no_move() {
    let mut s = open("order-single");

    s.move_plan("plan-a", PlanShift::Later).unwrap();
    s.move_plan("plan-a", PlanShift::Earlier).unwrap();

    assert!(!s.can_undo());
}

#[test]
fn an_unknown_plan_is_refused() {
    let mut s = three("order-unknown");

    let result = s.move_plan("nope", PlanShift::Later);

    assert!(matches!(result, Err(ServiceError::NoPlan(id)) if id == "nope"));
}
