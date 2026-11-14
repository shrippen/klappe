//! Plan variants of a scene (LH-04), on a copy of the example project: scene `szene-012` has
//! one plan, `plan-a` ("Plan A") with five shots and two figures.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_services::{Field, PlanEdit, Point, ServiceError, Session, ShotsGo, Source};

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

/// Ids and names of the plans of the scene on view.
fn plans(s: &Session) -> Vec<(String, String)> {
    s.scene()
        .unwrap()
        .plans
        .iter()
        .map(|p| (p.id.clone(), p.name.clone()))
        .collect()
}

fn pair(id: &str, name: &str) -> (String, String) {
    (id.to_owned(), name.to_owned())
}

fn shown(s: &Session) -> String {
    s.plan().unwrap().id.clone()
}

#[test]
fn a_new_plan_is_empty_named_and_shown() {
    let mut s = open("variants-add");

    let id = s.add_plan("").unwrap();

    assert_eq!(id, "plan-b");
    assert_eq!(
        plans(&s),
        [pair("plan-a", "Plan A"), pair("plan-b", "Plan B")]
    );
    assert_eq!(shown(&s), "plan-b");
    let plan = s.plan().unwrap();
    assert!(plan.figures.is_empty() && plan.shots.is_empty() && plan.cameras.is_empty());
}

#[test]
fn a_named_plan_gets_an_id_from_its_name() {
    let mut s = open("variants-named");

    let id = s.add_plan("  Mit Pferd ").unwrap();

    assert_eq!(id, "mit-pferd");
    assert_eq!(s.plan().unwrap().name, "Mit Pferd");
}

#[test]
fn adding_is_one_undo_step_and_the_view_goes_back() {
    let mut s = open("variants-add-undo");
    s.set_time(2.0);
    s.add_plan("").unwrap();

    s.undo();

    assert_eq!(plans(&s), [pair("plan-a", "Plan A")]);
    assert_eq!(shown(&s), "plan-a");
    assert!(!s.can_undo());

    s.redo();

    assert_eq!(plans(&s).len(), 2);
}

#[test]
fn a_name_is_taken_regardless_of_case() {
    let mut s = open("variants-taken");

    let added = s.add_plan("plan a");
    let renamed = s.rename_plan("plan-a", "Plan A");

    assert!(matches!(added, Err(ServiceError::PlanNameTaken(n)) if n == "plan a"));
    assert!(renamed.is_ok());
    assert_eq!(plans(&s), [pair("plan-a", "Plan A")]);
    assert!(!s.can_undo());
}

#[test]
fn an_id_is_never_taken_twice() {
    let mut s = open("variants-ids");
    s.add_plan("Plan B").unwrap();
    // The id `plan-b` stays with the plan while its name changes.
    s.rename_plan("plan-b", "Fassung 2").unwrap();

    let again = s.add_plan("Plan B").unwrap();
    let upper = s.add_plan("PLAN-B").unwrap();

    assert_eq!(again, "plan-b-2");
    assert_eq!(upper, "plan-b-3");
}

#[test]
fn the_free_letter_is_taken_for_an_unnamed_plan() {
    let mut s = open("variants-letters");
    s.add_plan("").unwrap();
    s.add_plan("").unwrap();
    s.remove_plan("plan-b", ShotsGo::Refuse).unwrap();

    let id = s.add_plan("").unwrap();

    assert_eq!(id, "plan-b");
    assert_eq!(plans(&s).len(), 3);
}

#[test]
fn a_snapshot_copies_the_plan_and_is_shown() {
    let mut s = open("variants-copy");
    let original = s.plan().unwrap().clone();

    let id = s.copy_plan("").unwrap();

    assert_eq!(id, "plan-a-2");
    assert_eq!(
        plans(&s),
        [pair("plan-a", "Plan A"), pair("plan-a-2", "Plan A (2)")]
    );
    assert_eq!(shown(&s), "plan-a-2");
    let copy = s.plan().unwrap();
    assert_eq!(copy.figures, original.figures);
    assert_eq!(copy.shots, original.shots);
    assert_eq!(copy.cameras, original.cameras);
}

#[test]
fn a_snapshot_is_independent_of_its_source() {
    let mut s = open("variants-copy-edit");
    s.copy_plan("").unwrap();
    let source = Source::Figure("elisabeth".into());

    s.edit_plan(PlanEdit::Move {
        source,
        to: Point::new(1.0, 1.0),
    })
    .unwrap();

    s.select_plan("plan-a").unwrap();
    let before = s.plan().unwrap().clone();
    s.select_plan("plan-a-2").unwrap();
    assert_ne!(s.plan().unwrap().figures, before.figures);
}

#[test]
fn copies_are_numbered_on_and_a_name_can_be_chosen() {
    let mut s = open("variants-copy-names");
    s.copy_plan("").unwrap();

    let third = s.copy_plan("").unwrap();
    let named = s.copy_plan("Vor dem Umbau").unwrap();

    assert_eq!(third, "plan-a-3");
    assert_eq!(named, "vor-dem-umbau");
    let names: Vec<_> = plans(&s).into_iter().map(|p| p.1).collect();
    assert_eq!(
        names,
        ["Plan A", "Plan A (2)", "Plan A (3)", "Vor dem Umbau"]
    );
    // The copy lands behind the plan it was made of.
    s.select_plan("plan-a").unwrap();
    s.copy_plan("Zwischen").unwrap();
    assert_eq!(plans(&s)[1].1, "Zwischen");
}

#[test]
fn a_snapshot_can_be_undone() {
    let mut s = open("variants-copy-undo");
    s.copy_plan("").unwrap();

    s.undo();

    assert_eq!(plans(&s), [pair("plan-a", "Plan A")]);
    assert_eq!(shown(&s), "plan-a");
}

#[test]
fn going_back_is_choosing_the_snapshot_again() {
    let mut s = open("variants-back");
    s.copy_plan("Stand 1").unwrap();
    s.select_plan("plan-a").unwrap();
    s.edit_plan(PlanEdit::Remove {
        source: Source::Figure("burgvogt".into()),
    })
    .unwrap();
    let edited = s.plan().unwrap().figures.len();

    s.select_plan("stand-1").unwrap();

    assert_eq!(s.plan().unwrap().figures.len(), edited + 1);
    assert!(matches!(
        s.select_plan("nirgends"),
        Err(ServiceError::NoPlan(_))
    ));
}

#[test]
fn renaming_keeps_the_id_and_can_be_undone() {
    let mut s = open("variants-rename");

    s.rename_plan("plan-a", " Hauptfassung ").unwrap();

    assert_eq!(plans(&s), [pair("plan-a", "Hauptfassung")]);
    assert!(s.plan_title().ends_with("Hauptfassung"));
    s.undo();
    assert_eq!(plans(&s), [pair("plan-a", "Plan A")]);
}

#[test]
fn renaming_is_refused_for_bad_names_and_unknown_plans() {
    let mut s = open("variants-rename-bad");
    s.add_plan("").unwrap();

    assert!(matches!(
        s.rename_plan("plan-b", "  "),
        Err(ServiceError::BadValue {
            field: Field::Name,
            ..
        })
    ));
    assert!(matches!(
        s.rename_plan("plan-b", "plan a"),
        Err(ServiceError::PlanNameTaken(_))
    ));
    assert!(matches!(
        s.rename_plan("plan-z", "Z"),
        Err(ServiceError::NoPlan(_))
    ));
    assert_eq!(plans(&s)[1], pair("plan-b", "Plan B"));
}

#[test]
fn the_last_plan_stays() {
    let mut s = open("variants-last");

    let refused = s.remove_plan("plan-a", ShotsGo::Refuse);

    assert!(matches!(refused, Err(ServiceError::LastPlan(id)) if id == "plan-a"));
    assert_eq!(plans(&s).len(), 1);
}

#[test]
fn a_plan_with_shots_stays_and_the_refusal_counts_them() {
    let mut s = open("variants-shots");
    s.add_plan("").unwrap();

    let refused = s.remove_plan("plan-a", ShotsGo::Refuse);

    assert!(matches!(
        refused,
        Err(ServiceError::PlanHasShots { id, shots: 5 }) if id == "plan-a"
    ));
    assert_eq!(plans(&s).len(), 2);
}

#[test]
fn a_plan_with_shots_goes_with_them_when_told_to() {
    let mut s = open("variants-shots-go");
    s.add_plan("").unwrap();
    s.select_plan("plan-a").unwrap();

    s.remove_plan("plan-a", ShotsGo::Delete).unwrap();

    assert_eq!(plans(&s), [pair("plan-b", "Plan B")]);
    assert_eq!(shown(&s), "plan-b");
}

#[test]
fn undo_brings_back_the_plan_and_its_shots_in_one_step() {
    let mut s = open("variants-shots-undo");
    s.add_plan("").unwrap();
    let before = s.scene().unwrap().plans.clone();
    s.remove_plan("plan-a", ShotsGo::Delete).unwrap();

    s.undo();

    assert_eq!(s.scene().unwrap().plans, before);
    assert_eq!(s.scene().unwrap().plans[0].shots.len(), 5);
    s.redo();
    assert_eq!(plans(&s), [pair("plan-b", "Plan B")]);
}

#[test]
fn the_last_plan_stays_even_with_its_shots_to_go() {
    let mut s = open("variants-shots-last");

    let refused = s.remove_plan("plan-a", ShotsGo::Delete);

    assert!(matches!(refused, Err(ServiceError::LastPlan(id)) if id == "plan-a"));
    assert_eq!(s.plan().unwrap().shots.len(), 5);
}

#[test]
fn an_unknown_plan_cannot_go() {
    let mut s = open("variants-unknown");
    s.add_plan("").unwrap();

    assert!(matches!(
        s.remove_plan("plan-z", ShotsGo::Refuse),
        Err(ServiceError::NoPlan(_))
    ));
}

#[test]
fn deleting_the_plan_on_view_shows_the_one_before() {
    let mut s = open("variants-remove-shown");
    s.add_plan("").unwrap();
    s.add_plan("").unwrap();
    s.select_plan("plan-b").unwrap();

    s.remove_plan("plan-b", ShotsGo::Refuse).unwrap();

    assert_eq!(
        plans(&s),
        [pair("plan-a", "Plan A"), pair("plan-c", "Plan C")]
    );
    assert_eq!(shown(&s), "plan-a");
}

#[test]
fn deleting_another_plan_keeps_the_view() {
    let mut s = open("variants-remove-other");
    s.add_plan("").unwrap();
    s.add_plan("").unwrap();
    s.select_plan("plan-a").unwrap();
    s.set_time(1.5);
    assert_eq!(s.time_s(), 1.5);

    s.remove_plan("plan-c", ShotsGo::Refuse).unwrap();

    assert_eq!(shown(&s), "plan-a");
    assert_eq!(s.time_s(), 1.5);
}

#[test]
fn deleting_the_first_plan_shows_the_next_when_it_was_on_view() {
    let mut s = open("variants-remove-first");
    s.add_plan("").unwrap();
    s.select_plan("plan-a").unwrap();
    // Plan A holds shots: clear them first, one step each.
    let ids: Vec<_> = s
        .plan()
        .unwrap()
        .shots
        .iter()
        .map(|x| x.id.clone())
        .collect();
    for id in ids {
        s.delete_shot(&id).unwrap();
    }

    s.remove_plan("plan-a", ShotsGo::Refuse).unwrap();

    assert_eq!(shown(&s), "plan-b");
}

#[test]
fn deleting_is_undone_with_the_plan_back_on_view() {
    let mut s = open("variants-remove-undo");
    s.add_plan("").unwrap();
    s.add_plan("").unwrap();
    s.select_plan("plan-b").unwrap();
    s.remove_plan("plan-b", ShotsGo::Refuse).unwrap();

    s.undo();

    assert_eq!(
        plans(&s),
        [
            pair("plan-a", "Plan A"),
            pair("plan-b", "Plan B"),
            pair("plan-c", "Plan C")
        ]
    );
    s.redo();
    assert_eq!(plans(&s).len(), 2);
}

#[test]
fn undo_stays_with_the_plan_on_view_when_indices_shift() {
    let mut s = open("variants-follow");
    s.add_plan("").unwrap();
    s.add_plan("").unwrap();
    s.select_plan("plan-c").unwrap();
    s.remove_plan("plan-b", ShotsGo::Refuse).unwrap();

    // Plan B comes back before the plan on view, which moves from index 1 to 2.
    s.undo();

    assert_eq!(shown(&s), "plan-c");
    s.redo();
    assert_eq!(shown(&s), "plan-c");
}

#[test]
fn variants_survive_saving_and_opening() {
    let mut s = open("variants-save");
    s.copy_plan("Stand 1").unwrap();
    s.add_plan("").unwrap();
    s.save().unwrap();

    let reopened = Session::open(s.dir()).unwrap();

    assert_eq!(
        plans(&reopened),
        [
            pair("plan-a", "Plan A"),
            pair("stand-1", "Stand 1"),
            pair("plan-b", "Plan B")
        ]
    );
    let copy = &reopened.scene().unwrap().plans[1];
    assert_eq!(copy.shots.len(), 5);
}
