//! Making, copying and deleting shots (LH-70), on the example project: scene 12 with the
//! shots `s12-1` to `s12-5` (12.1 to 12.5), cameras `a` and `b`; marks at 0, 6, 9 and 14 s.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_services::{Layer, LinkKind, Lock, PlanEdit, ServiceError, Session, ShotField, Source};

fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe")
}

fn open() -> Session {
    Session::open(&example()).unwrap()
}

/// A copy of the example to write to.
fn copy(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    copy_dir(&example(), &dir);
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

fn numbers(s: &Session) -> Vec<String> {
    s.shots().into_iter().map(|r| r.number).collect()
}

fn ids(s: &Session) -> Vec<String> {
    s.shots().into_iter().map(|r| r.id).collect()
}

#[test]
fn a_new_shot_starts_at_the_playhead_and_ends_at_the_next_mark() {
    let mut s = open();
    s.set_time(6.0);

    let id = s.add_shot(Some("b")).unwrap();

    let row = s.shots().into_iter().find(|r| r.id == id).unwrap();
    assert_eq!((row.from_s, row.to_s), (6.0, 9.0));
    assert_eq!(row.camera, "b");
    assert_eq!(row.number, "12.6");
    assert_eq!(s.selected_shot(), Some(id.as_str()));
}

#[test]
fn a_new_shot_after_the_last_mark_gets_the_default_length() {
    let mut s = open();
    s.set_time(14.0);

    let id = s.add_shot(None).unwrap();

    let row = s.shots().into_iter().find(|r| r.id == id).unwrap();
    assert_eq!((row.from_s, row.to_s), (14.0, 19.0));
}

#[test]
fn a_mark_right_behind_the_playhead_does_not_end_the_shot() {
    let mut s = open();
    s.set_time(5.8);

    let id = s.add_shot(None).unwrap();

    let row = s.shots().into_iter().find(|r| r.id == id).unwrap();
    assert_eq!((row.from_s, row.to_s), (5.8, 9.0));
}

#[test]
fn the_camera_defaults_to_the_selected_shots_then_the_first() {
    let mut s = open();

    let first = s.add_shot(None).unwrap();
    let row = |s: &Session, id: &str| s.shots().into_iter().find(|r| r.id == id).unwrap();
    assert_eq!(row(&s, &first).camera, "a");

    s.select_shot("s12-3").unwrap();
    let second = s.add_shot(None).unwrap();
    assert_eq!(row(&s, &second).camera, "b");
}

#[test]
fn an_unknown_camera_is_refused() {
    let mut s = open();

    let result = s.add_shot(Some("z"));

    assert!(matches!(result, Err(ServiceError::BadValue { .. })));
    assert_eq!(s.shots().len(), 5);
    assert!(!s.can_undo());
}

#[test]
fn a_new_shot_goes_into_the_cut_by_its_start() {
    let mut s = open();
    s.set_time(7.0);

    let id = s.add_shot(None).unwrap();

    // 12.3 starts at 6 s, 12.4 at 9 s.
    assert_eq!(
        ids(&s),
        ["s12-1", "s12-2", "s12-3", id.as_str(), "s12-4", "s12-5"]
    );
}

#[test]
fn numbers_go_on_after_the_highest_and_keep_gaps() {
    let mut s = open();
    s.delete_shot("s12-2").unwrap();
    s.edit_shot("s12-3", ShotField::Number, "12.7").unwrap();

    let id = s.add_shot(None).unwrap();

    assert_eq!(
        s.shots().into_iter().find(|r| r.id == id).unwrap().number,
        "12.8"
    );
}

#[test]
fn a_lettered_version_counts_as_its_number() {
    let mut s = open();
    s.edit_shot("s12-5", ShotField::Number, "12.5A").unwrap();

    let id = s.add_shot(None).unwrap();

    assert_eq!(
        s.shots().into_iter().find(|r| r.id == id).unwrap().number,
        "12.6"
    );
}

#[test]
fn numbers_of_other_scenes_do_not_count() {
    let mut s = open();
    s.edit_shot("s12-5", ShotField::Number, "120.9").unwrap();
    s.edit_shot("s12-4", ShotField::Number, "12").unwrap();

    let id = s.add_shot(None).unwrap();

    // Only 12.1 to 12.3 are left of this scene.
    assert_eq!(
        s.shots().into_iter().find(|r| r.id == id).unwrap().number,
        "12.4"
    );
}

#[test]
fn ids_stay_unique_when_a_number_was_changed() {
    let mut s = open();
    // `s12-6` exists but is numbered 12.2 now: the next shot must not take its id.
    let moved = s.add_shot(None).unwrap();
    assert_eq!(moved, "s12-6");
    s.edit_shot(&moved, ShotField::Number, "12.2A").unwrap();
    s.edit_shot("s12-5", ShotField::Number, "99").unwrap();
    s.delete_shot("s12-4").unwrap();

    let id = s.add_shot(None).unwrap();

    assert_eq!(ids(&s).iter().filter(|i| **i == id).count(), 1);
    assert_ne!(id, moved);
}

#[test]
fn a_copy_has_the_fields_and_a_new_number_and_stands_behind_the_original() {
    let mut s = open();
    s.edit_shot("s12-3", ShotField::Description, "Blick zur Tür")
        .unwrap();
    s.edit_shot("s12-3", ShotField::Lens, "85").unwrap();

    let id = s.copy_shot("s12-3").unwrap();

    let rows = s.shots();
    let (original, copy) = (&rows[2], &rows[3]);
    assert_eq!(copy.id, id);
    assert_ne!(copy.id, original.id);
    assert_eq!(copy.number, "12.6");
    assert_eq!(
        (
            &copy.camera,
            &copy.description,
            copy.own_lens_mm,
            copy.from_s,
            copy.to_s
        ),
        (
            &original.camera,
            &original.description,
            original.own_lens_mm,
            original.from_s,
            original.to_s
        )
    );
    // Same camera, place and lens: one setup yields both shots.
    assert_eq!(copy.setup, original.setup);
    assert_eq!(s.selected_shot(), Some(id.as_str()));
}

#[test]
fn copying_an_unknown_shot_fails() {
    let mut s = open();

    assert!(matches!(s.copy_shot("nope"), Err(ServiceError::NoShot(_))));
    assert!(!s.can_undo());
}

#[test]
fn deleting_the_selected_shot_selects_the_next_one() {
    let mut s = open();
    s.select_shot("s12-2").unwrap();

    s.delete_shot("s12-2").unwrap();

    assert_eq!(ids(&s), ["s12-1", "s12-3", "s12-4", "s12-5"]);
    assert_eq!(s.selected_shot(), Some("s12-3"));
}

#[test]
fn deleting_the_last_shot_selects_the_one_before() {
    let mut s = open();
    s.select_shot("s12-5").unwrap();

    s.delete_shot("s12-5").unwrap();

    assert_eq!(s.selected_shot(), Some("s12-4"));
}

#[test]
fn deleting_the_only_shot_selects_none() {
    let mut s = open();
    for id in ["s12-1", "s12-2", "s12-3", "s12-4"] {
        s.delete_shot(id).unwrap();
    }
    s.select_shot("s12-5").unwrap();

    s.delete_shot("s12-5").unwrap();

    assert!(s.shots().is_empty());
    assert_eq!(s.selected_shot(), None);
}

#[test]
fn deleting_another_shot_keeps_the_selection() {
    let mut s = open();
    s.select_shot("s12-4").unwrap();

    s.delete_shot("s12-1").unwrap();

    assert_eq!(s.selected_shot(), Some("s12-4"));
}

#[test]
fn deleting_a_shot_removes_the_script_links_to_it() {
    let mut s = open();
    let chips = |s: &Session| {
        s.script_lines()
            .iter()
            .flat_map(|l| l.chips.iter())
            .filter(|c| c.kind == LinkKind::Shot)
            .count()
    };
    assert_eq!(chips(&s), 1);

    s.delete_shot("s12-2").unwrap();
    assert_eq!(chips(&s), 0);

    s.undo();
    assert_eq!(chips(&s), 1);
}

#[test]
fn deleting_an_unknown_shot_fails() {
    let mut s = open();

    assert!(matches!(
        s.delete_shot("nope"),
        Err(ServiceError::NoShot(_))
    ));
    assert!(!s.can_undo());
}

#[test]
fn every_change_is_one_undo_step_and_marks_the_project_changed() {
    let mut s = open();
    assert!(!s.is_dirty());
    let before = s.shots();

    s.add_shot(None).unwrap();
    assert!(s.is_dirty());
    s.undo();
    assert_eq!(s.shots(), before);

    s.copy_shot("s12-1").unwrap();
    s.undo();
    assert_eq!(s.shots(), before);

    s.delete_shot("s12-1").unwrap();
    s.undo();
    assert_eq!(s.shots(), before);
    s.redo();
    assert_eq!(s.shots().len(), 4);
}

#[test]
fn undoing_the_new_shot_drops_the_selection_on_it() {
    let mut s = open();
    s.add_shot(None).unwrap();
    assert!(s.selected_shot().is_some());

    s.undo();

    assert_eq!(s.selected_shot(), None);
}

#[test]
fn a_locked_camera_layer_refuses_all_three() {
    let mut s = open();
    s.set_lock(Layer::Cameras, Lock::Locked);

    for result in [
        s.add_shot(None).map(drop),
        s.copy_shot("s12-1").map(drop),
        s.delete_shot("s12-1"),
    ] {
        assert!(matches!(result, Err(ServiceError::Locked(Layer::Cameras))));
    }
    assert_eq!(s.shots().len(), 5);
    assert!(!s.can_undo());

    s.set_lock(Layer::Cameras, Lock::Unlocked);
    s.add_shot(None).unwrap();
}

#[test]
fn other_locks_do_not_stop_shots() {
    let mut s = open();
    s.set_lock(Layer::Set, Lock::Locked);
    s.set_lock(Layer::Figures, Lock::Locked);

    s.add_shot(None).unwrap();
}

#[test]
fn a_plan_without_cameras_cannot_have_a_shot_made() {
    let mut s = open();
    // Shots keep a camera in use, so clear them first.
    for id in ids(&s) {
        s.delete_shot(&id).unwrap();
    }
    for camera in ["a", "b"] {
        s.edit_plan(PlanEdit::Remove {
            source: Source::Camera(camera.into()),
        })
        .unwrap();
    }

    assert!(matches!(s.add_shot(None), Err(ServiceError::NoCamera)));
}

#[test]
fn new_shots_survive_saving_and_loading() {
    let dir = copy("shots-save");
    let mut s = Session::open(&dir).unwrap();
    s.set_time(7.0);
    let added = s.add_shot(Some("b")).unwrap();
    let copied = s.copy_shot("s12-1").unwrap();
    s.delete_shot("s12-5").unwrap();
    s.save().unwrap();
    let saved = s.shots();

    let back = Session::open(&dir).unwrap();

    assert_eq!(back.shots(), saved);
    assert!(ids(&back).contains(&added));
    assert!(ids(&back).contains(&copied));
    assert!(!ids(&back).contains(&"s12-5".to_owned()));
    assert_eq!(numbers(&back).len(), 6);
}
