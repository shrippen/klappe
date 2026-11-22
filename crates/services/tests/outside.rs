//! Changes made outside the session, in the project folder (two sessions on one folder).
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_services::{OnOutside, ProjectEdit, Saved, Session};
use klappe_store::FileKind;

const SCRIPT_FILE: &str = "drehbuch.fountain";
const PROJECT_FILE: &str = "projekt.toml";
const SCENE_FILE: &str = "szenen/m4djbtj6.json";
const ADDED_LINE: &str = "Ein Satz, den das Fenster nicht kennt.\n";
const OUTSIDE_PLAN: &str = "Plan von draussen";

/// A copy of the example to write to.
fn copy(name: &str) -> PathBuf {
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

    let example = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe");
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    copy_dir(&example, &dir);
    dir
}

fn read(dir: &Path, name: &str) -> String {
    std::fs::read_to_string(dir.join(name)).unwrap()
}

fn write(dir: &Path, name: &str, text: &str) {
    std::fs::write(dir.join(name), text).unwrap();
}

/// What another program does to the screenplay.
fn append_line(dir: &Path) -> String {
    let text = format!("{}{ADDED_LINE}", read(dir, SCRIPT_FILE));
    write(dir, SCRIPT_FILE, &text);

    text
}

/// What another program does to the plans of the example's scene.
fn rename_plan_outside(dir: &Path) {
    let scene = read(dir, SCENE_FILE).replace("\"Plan A\"", &format!("\"{OUTSIDE_PLAN}\""));
    write(dir, SCENE_FILE, &scene);
}

#[test]
fn a_save_keeps_the_script_another_session_saved() {
    let dir = copy("outside-script-kept");
    let mut terminal = Session::open(&dir).unwrap();
    let mut window = Session::open(&dir).unwrap();

    let text = format!("{}{ADDED_LINE}", terminal.script_text());
    terminal.set_script_text(&text);
    terminal
        .save_script(klappe_services::OnOutside::Overwrite)
        .unwrap();

    window.add_plan("Plan B").unwrap();
    window.save(OnOutside::Ask).unwrap();

    let on_disk = read(&dir, SCRIPT_FILE);
    assert!(
        on_disk.contains(ADDED_LINE),
        "the window's save lost the text"
    );
}

#[test]
fn an_outside_change_without_own_work_is_taken_silently() {
    let dir = copy("outside-silent");
    let mut s = Session::open(&dir).unwrap();
    let scene = s.scene().map(|c| c.id.clone());
    let plan = s.plan().map(|p| p.id.clone());

    let text = append_line(&dir);
    let found = s.check().unwrap();

    assert_eq!(found.taken.len(), 1);
    assert_eq!(found.taken[0].kind, FileKind::Script);
    assert!(found.both.is_empty());
    assert_eq!(s.script_text(), text);
    assert!(!s.can_undo() && !s.can_redo());
    assert!(!s.is_dirty());
    assert_eq!(s.scene().map(|c| c.id.clone()), scene);
    assert_eq!(s.plan().map(|p| p.id.clone()), plan);
    // Nothing is left to find.
    assert_eq!(s.check().unwrap(), Default::default());
}

#[test]
fn an_outside_script_and_an_own_plan_are_both_kept_without_a_question() {
    let dir = copy("outside-both-files");
    let mut s = Session::open(&dir).unwrap();
    s.add_plan("Plan B").unwrap();

    let text = append_line(&dir);
    let saved = s.save(OnOutside::Ask).unwrap();

    assert_eq!(saved, Saved::Done);
    assert_eq!(read(&dir, SCRIPT_FILE), text);
    assert!(read(&dir, SCENE_FILE).contains("Plan B"));
    assert_eq!(s.script_text(), text);
    assert!(!s.is_dirty());
}

/// An own plan edit and an outside one of the same scene file.
fn conflict(name: &str) -> (PathBuf, Session) {
    let dir = copy(name);
    let mut s = Session::open(&dir).unwrap();
    s.add_plan("Plan B").unwrap();
    rename_plan_outside(&dir);

    (dir, s)
}

#[test]
fn a_file_both_changed_is_a_conflict_and_nothing_is_written() {
    let (dir, mut s) = conflict("outside-conflict");
    let outside = read(&dir, SCENE_FILE);

    assert_eq!(s.save(OnOutside::Ask).unwrap(), Saved::Conflict);

    assert_eq!(read(&dir, SCENE_FILE), outside);
    assert_eq!(s.outside().len(), 1);
    assert_eq!(s.outside()[0].kind, FileKind::Scene);
    assert_eq!(s.outside()[0].name, SCENE_FILE);
    assert!(s.is_dirty());
}

#[test]
fn overwrite_writes_the_session_s_version() {
    let (dir, mut s) = conflict("outside-overwrite");

    assert_eq!(s.save(OnOutside::Overwrite).unwrap(), Saved::Done);

    let scene = read(&dir, SCENE_FILE);
    assert!(scene.contains("Plan B"));
    assert!(!scene.contains(OUTSIDE_PLAN));
    assert!(s.outside().is_empty());
    assert_eq!(s.check().unwrap(), Default::default());
}

#[test]
fn reload_drops_the_own_changes_and_takes_the_folder() {
    let (dir, mut s) = conflict("outside-reload");
    s.save(OnOutside::Ask).unwrap();

    s.reload().unwrap();

    let names: Vec<_> = s
        .scene()
        .unwrap()
        .plans
        .iter()
        .map(|p| p.name.clone())
        .collect();
    assert_eq!(names, [OUTSIDE_PLAN]);
    assert!(!s.is_dirty());
    assert!(!s.can_undo());
    assert!(s.outside().is_empty());
    assert!(read(&dir, SCENE_FILE).contains(OUTSIDE_PLAN));
}

#[test]
fn a_scene_file_made_outside_survives_a_save() {
    let dir = copy("outside-new-scene");
    let mut s = Session::open(&dir).unwrap();
    s.add_plan("Plan B").unwrap();

    let scene = read(&dir, SCENE_FILE).replace("m4djbtj6", "neu");
    write(&dir, "szenen/neu.json", &scene);
    s.save(OnOutside::Ask).unwrap();

    assert!(dir.join("szenen/neu.json").exists());
    assert!(s.project().scenes.iter().any(|c| c.id == "neu"));
}

#[test]
fn a_scene_the_session_deleted_is_gone_after_a_save() {
    let dir = copy("outside-deleted-scene");
    let mut s = Session::open(&dir).unwrap();
    s.edit_project(ProjectEdit::RemoveScene {
        id: "m4djbtj6".into(),
    })
    .unwrap();

    s.save(OnOutside::Ask).unwrap();

    assert!(!dir.join(SCENE_FILE).exists());
}

#[test]
fn normalising_a_hand_formatted_file_is_no_own_change() {
    let dir = copy("outside-normalised");
    let original = read(&dir, PROJECT_FILE);
    write(&dir, PROJECT_FILE, &format!("# von Hand\n{original}"));
    let mut s = Session::open(&dir).unwrap();
    assert!(!s.is_dirty());

    write(
        &dir,
        PROJECT_FILE,
        &format!(
            "# von Hand, anders\n{}",
            original.replace("Die Wartburg-Szene", "Von draussen")
        ),
    );
    let found = s.check().unwrap();

    assert_eq!(found.taken.len(), 1);
    assert_eq!(found.taken[0].kind, FileKind::Project);
    assert!(found.both.is_empty());
}

#[test]
fn a_broken_outside_file_is_an_error_and_changes_nothing() {
    let dir = copy("outside-broken");
    let good = read(&dir, PROJECT_FILE);
    let mut s = Session::open(&dir).unwrap();
    s.add_plan("Plan B").unwrap();
    let before = s.project().clone();

    write(&dir, PROJECT_FILE, "title = [");
    append_line(&dir);

    assert!(s.check().is_err());
    assert_eq!(*s.project(), before);
    assert!(!s.script_text().contains(ADDED_LINE));
    assert!(s.save(OnOutside::Ask).is_err());
    assert_eq!(read(&dir, PROJECT_FILE), "title = [");

    assert_eq!(s.save(OnOutside::Overwrite).unwrap(), Saved::Done);
    assert_eq!(read(&dir, PROJECT_FILE), good);
}

#[test]
fn keep_contd_after_an_outside_header_change_keeps_both() {
    let dir = copy("outside-contd");
    let mut s = Session::open(&dir).unwrap();

    let header = read(&dir, PROJECT_FILE).replace("Die Wartburg-Szene", "Von draussen");
    write(&dir, PROJECT_FILE, &header);
    s.keep_contd(true).unwrap();

    assert_eq!(s.save(OnOutside::Ask).unwrap(), Saved::Done);
    let on_disk = read(&dir, PROJECT_FILE);
    assert!(on_disk.contains("Von draussen"), "{on_disk}");
    assert!(on_disk.contains("contd_after_action = true"), "{on_disk}");
}

#[test]
fn the_partial_writers_leave_nothing_for_a_check_to_find() {
    let dir = copy("outside-writers-quiet");
    let mut s = Session::open(&dir).unwrap();

    s.keep_contd(true).unwrap();
    assert_eq!(s.check().unwrap(), Default::default());

    s.set_scene_numbering(klappe_services::SceneNumbering::Locked, 5)
        .unwrap();
    assert_eq!(s.check().unwrap(), Default::default());

    s.save_register().unwrap();
    assert_eq!(s.check().unwrap(), Default::default());

    let text = format!("{}{ADDED_LINE}", s.script_text());
    s.set_script_text(&text);
    assert_eq!(s.save_script(OnOutside::Ask).unwrap(), Saved::Done);
    assert_eq!(s.check().unwrap(), Default::default());
}

#[test]
fn a_script_both_changed_asks_in_save_script_too() {
    let dir = copy("outside-script-conflict");
    let mut s = Session::open(&dir).unwrap();
    let text = format!("{}Eigene Zeile.\n", s.script_text());
    s.set_script_text(&text);

    let outside = append_line(&dir);

    assert_eq!(s.save_script(OnOutside::Ask).unwrap(), Saved::Conflict);
    assert_eq!(read(&dir, SCRIPT_FILE), outside);
    assert_eq!(s.save_script(OnOutside::Overwrite).unwrap(), Saved::Done);
    assert_eq!(read(&dir, SCRIPT_FILE), text);
}
