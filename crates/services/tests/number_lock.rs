//! The Sperrstand (DB-33): the numbers of the saved book, kept in `nummern.toml`. Saving
//! changes no number without it, and a number only changes where the book differs from it.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_model::{FloorPlan, Kind, Project, Scene, SceneNumbering, ScriptLanguage};
use klappe_services::Session;

const SAAL: &str = "INT. SAAL – TAG";
const HOF: &str = "INT. HOF – NACHT";
const KELLER: &str = "INT. KELLER – NACHT";
const LOCK_FILE: &str = "nummern.toml";

fn saal(number: &str) -> String {
    format!("{SAAL}{number}\n\nAnna betritt den Saal.\n\nANNA\nGuten Tag, ist hier jemand?\n\n")
}

fn hof(number: &str) -> String {
    format!("{HOF}{number}\n\nEin Hund bellt im Hof.\n\nBERT\nSei still, du Köter.\n\n")
}

fn keller(number: &str) -> String {
    format!("{KELLER}{number}\n\nWasser tropft.\n\nCARL\nHier unten ist es kalt.\n\n")
}

fn entry(id: &str, number: &str, heading: &str) -> Scene {
    Scene {
        id: id.into(),
        kind: Kind::Scene,
        number: number.into(),
        heading: heading.into(),
        plans: vec![FloorPlan::new("plan-a", "Plan A")],
        ..Scene::default()
    }
}

/// Entries `aaaa` (hall, 1) and `bbbb` (yard, 2) and this book; opened and saved once, so
/// the register holds the book and the Sperrstand exists.
fn make(name: &str, numbering: SceneNumbering, book: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    let mut project = Project::new("Test");
    project.script.scene_numbering = numbering;
    project.script.language = Some(ScriptLanguage::De);
    project.scenes = vec![entry("aaaa", "1", SAAL), entry("bbbb", "2", HOF)];
    klappe_store::save(&dir, &project, Some(book)).unwrap();
    Session::open(&dir)
        .unwrap()
        .save(klappe_services::OnOutside::Overwrite)
        .unwrap();

    dir
}

fn book(dir: &Path) -> String {
    std::fs::read_to_string(dir.join("drehbuch.fountain")).unwrap()
}

fn write(dir: &Path, text: &str) {
    std::fs::write(dir.join("drehbuch.fountain"), text).unwrap();
}

/// The register gets an entry that never met the book, with this number.
fn add_open_entry(dir: &Path, id: &str, number: &str, heading: &str) {
    let mut project = klappe_store::load(dir).unwrap().project;
    let mut asked = entry(id, number, heading);
    asked.plans.clear();
    project.scenes.push(asked);
    klappe_store::save(dir, &project, None).unwrap();
}

fn copy_of_yard() -> String {
    hof(" #2#").replace("Hund bellt", "Pferd wiehert")
}

/// Review round 3, 1: a foreign open question holds the second scene with number 2. Nothing
/// was edited, so saving leaves the book as it is.
#[test]
fn a_foreign_open_question_leaves_the_book_alone() {
    let dir = make(
        "nl-foreign",
        SceneNumbering::Locked,
        &(saal(" #1#") + &hof(" #2#")),
    );
    add_open_entry(&dir, "cccc", "2", "INT. WALD – TAG");
    let text = saal(" #1#") + &hof(" #2#") + &keller(" #2#");
    write(&dir, &text);

    let mut s = Session::open(&dir).unwrap();
    assert_eq!(s.pending_scenes().len(), 1);
    s.save(klappe_services::OnOutside::Overwrite).unwrap();

    assert_eq!(book(&dir), text);
}

/// Review round 3, 2: a copy behind the original has the number twice and an open question;
/// once answered, the scene of the register keeps 2 and the other one is inserted.
#[test]
fn a_copy_behind_the_original_gets_a_letter_after_the_answer() {
    let dir = make(
        "nl-copy",
        SceneNumbering::Locked,
        &(saal(" #1#") + &hof(" #2#")),
    );
    let mut s = Session::open(&dir).unwrap();
    s.set_script_text(&(saal(" #1#") + &hof(" #2#") + &copy_of_yard()));
    assert_eq!(s.pending_scenes().len(), 1);

    s.answer_book("bbbb", 0).unwrap();
    s.save_script(klappe_services::OnOutside::Overwrite)
        .unwrap();

    let text = book(&dir);
    assert_eq!(text.matches("#2#").count(), 1, "{text}");
    assert_eq!(text.matches("#2A#").count(), 1, "{text}");
}

/// Review round 3, 3: a scene struck from the book stays as a heading, whatever the register
/// went through between striking and saving.
#[test]
fn a_struck_scene_stays_after_opening() {
    let dir = make(
        "nl-struck-open",
        SceneNumbering::Locked,
        &(saal(" #1#") + &hof(" #2#")),
    );
    write(&dir, &saal(" #1#"));

    let mut s = Session::open(&dir).unwrap();
    s.save(klappe_services::OnOutside::Overwrite).unwrap();

    assert_eq!(book(&dir), saal(" #1#") + ".ENTFÄLLT #2#\n");
}

#[test]
fn a_struck_scene_stays_after_an_answer() {
    let dir = make(
        "nl-struck-answer",
        SceneNumbering::Locked,
        &(saal(" #1#") + &hof(" #2#")),
    );
    add_open_entry(&dir, "cccc", "9", "INT. WALD – TAG");
    let twin = keller(" #9#").replace("Wasser tropft", "Es riecht nach Rauch");
    let mut s = Session::open(&dir).unwrap();
    s.set_script_text(&(saal(" #1#") + &keller(" #9#") + &twin));
    assert_eq!(s.pending_scenes().len(), 1);

    s.answer_book("cccc", 0).unwrap();
    s.save(klappe_services::OnOutside::Overwrite).unwrap();

    assert!(book(&dir).contains(".ENTFÄLLT #2#"), "{}", book(&dir));
}

#[test]
fn a_struck_scene_stays_after_restoring_a_backup() {
    let dir = make(
        "nl-struck-backup",
        SceneNumbering::Locked,
        &(saal(" #1#") + &hof(" #2#")),
    );
    let mut s = Session::open(&dir).unwrap();
    s.set_script_text(&saal(" #1#"));
    s.autosave().unwrap();
    drop(s);

    let mut s = Session::open(&dir).unwrap();
    s.restore_backup().unwrap();
    s.save(klappe_services::OnOutside::Overwrite).unwrap();

    assert!(book(&dir).contains(".ENTFÄLLT #2#"), "{}", book(&dir));
}

/// A book from before the Sperrstand, numbers mixed: opening and saving changes nothing in
/// the text, and the Sperrstand is made from it.
#[test]
fn a_book_without_a_lock_is_saved_byte_for_byte() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("nl-none");
    let _ = std::fs::remove_dir_all(&dir);
    let mut project = Project::new("Test");
    project.script.scene_numbering = SceneNumbering::Locked;
    let text = saal(" #12#")
        + &keller("")
        + &hof(" #3#")
        + &saal(" #12#").replace("Anna", "Berta")
        + ".ENTFÄLLT #7#\n";
    klappe_store::save(&dir, &project, Some(&text)).unwrap();
    assert!(!dir.join(LOCK_FILE).exists());

    let mut s = Session::open(&dir).unwrap();
    s.save(klappe_services::OnOutside::Overwrite).unwrap();

    assert_eq!(book(&dir), text);
    assert!(dir.join(LOCK_FILE).exists());
}

/// The text is what the Sperrstand says; the register differs (edited by hand, merged):
/// no number changes.
#[test]
fn a_text_equal_to_the_lock_ignores_the_register() {
    let dir = make(
        "nl-register",
        SceneNumbering::Locked,
        &(saal(" #1#") + &hof(" #2#")),
    );
    let text = book(&dir);
    let mut project = klappe_store::load(&dir).unwrap().project;
    project.scenes[0].number = "7".into();
    project.scenes[1].number = "1".into();
    project.scenes.swap(0, 1);
    klappe_store::save(&dir, &project, None).unwrap();

    let mut s = Session::open(&dir).unwrap();
    s.save(klappe_services::OnOutside::Overwrite).unwrap();

    assert_eq!(book(&dir), text);
}

#[test]
fn automatic_removes_the_struck_headings() {
    let dir = make(
        "nl-auto",
        SceneNumbering::Automatic,
        &(saal(" #1#") + &hof(" #2#")),
    );
    let mut s = Session::open(&dir).unwrap();

    s.set_script_text(&(saal(" #1#") + ".ENTFÄLLT #2#\n\n" + &keller(" #3#")));
    s.save(klappe_services::OnOutside::Overwrite).unwrap();

    assert_eq!(book(&dir), saal(" #1#") + &keller(" #2#"));
}

#[test]
fn a_number_rewritten_by_hand_strikes_the_old_one() {
    let dir = make(
        "nl-rewrite",
        SceneNumbering::Locked,
        &(saal(" #1#") + &hof(" #2#")),
    );
    let mut s = Session::open(&dir).unwrap();

    s.set_script_text(&(saal(" #1#") + &hof(" #13#")));
    s.save(klappe_services::OnOutside::Overwrite).unwrap();

    assert_eq!(book(&dir), saal(" #1#") + &hof(" #13#") + ".ENTFÄLLT #2#\n");
}

#[test]
fn a_struck_heading_deleted_by_hand_comes_back() {
    let dir = make(
        "nl-deleted",
        SceneNumbering::Locked,
        &(saal(" #1#") + &hof(" #2#")),
    );
    let mut s = Session::open(&dir).unwrap();
    s.set_script_text(&saal(" #1#"));
    s.save(klappe_services::OnOutside::Overwrite).unwrap();
    let struck = book(&dir);
    assert!(struck.contains(".ENTFÄLLT #2#"), "{struck}");

    s.set_script_text(&saal(" #1#"));
    s.save(klappe_services::OnOutside::Overwrite).unwrap();

    assert_eq!(book(&dir), struck);
}

/// The copy in front of the scene the register knows is the inserted one.
#[test]
fn a_copy_before_the_original_gets_a_letter() {
    let dir = make(
        "nl-before",
        SceneNumbering::Locked,
        &(saal(" #1#") + &hof(" #2#")),
    );
    let mut s = Session::open(&dir).unwrap();

    s.set_script_text(&(saal(" #1#") + &copy_of_yard() + &hof(" #2#")));
    s.save(klappe_services::OnOutside::Overwrite).unwrap();

    let expected = saal(" #1#") + &copy_of_yard().replace("#2#", "#1A#") + &hof(" #2#");
    assert_eq!(book(&dir), expected);
}

#[test]
fn a_new_project_has_a_lock() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("nl-create");
    let _ = std::fs::remove_dir_all(&dir);

    Session::create(&dir, "Neu", klappe_services::Language::German).unwrap();

    assert!(dir.join(LOCK_FILE).exists());
}

#[test]
fn conflict_marks_in_the_lock_refuse_the_project() {
    let dir = make(
        "nl-conflict",
        SceneNumbering::Locked,
        &(saal(" #1#") + &hof(" #2#")),
    );
    std::fs::write(dir.join(LOCK_FILE), "<<<<<<< HEAD\n").unwrap();

    let error = Session::open(&dir).err().unwrap().to_string();

    assert!(error.contains("conflict"), "{error}");
}

const UNNUMBERED: &str =
    "INT. SAAL - TAG\n\nAnna.\n\nINT. HOF - NACHT\n\nBert.\n\nINT. KELLER - NACHT\n\nCarl.\n";

/// Review round 4: a locked book that never had a number (from another program) stays so
/// on the second and later saves, and a new scene in it gets none.
#[test]
fn a_book_without_any_number_stays_without_any() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("nl-none-ever");
    let _ = std::fs::remove_dir_all(&dir);
    let mut project = Project::new("Test");
    project.script.scene_numbering = SceneNumbering::Locked;
    klappe_store::save(&dir, &project, Some(UNNUMBERED)).unwrap();

    for _ in 0..3 {
        Session::open(&dir)
            .unwrap()
            .save(klappe_services::OnOutside::Overwrite)
            .unwrap();
        assert_eq!(book(&dir), UNNUMBERED);
    }

    let more = format!("{UNNUMBERED}\nINT. DACH - TAG\n\nDora.\n");
    let mut s = Session::open(&dir).unwrap();
    s.set_script_text(&more);
    s.save(klappe_services::OnOutside::Overwrite).unwrap();

    assert_eq!(book(&dir), more);
}

/// Review round 4: `.ENTFÄLLT` is no scripted location.
#[test]
fn a_struck_heading_is_no_pending_place() {
    let dir = make(
        "nl-places",
        SceneNumbering::Locked,
        &(saal(" #1#") + &hof(" #2#")),
    );
    write(&dir, &saal(" #1#"));
    let mut s = Session::open(&dir).unwrap();
    s.save(klappe_services::OnOutside::Overwrite).unwrap();

    assert!(book(&dir).contains(".ENTFÄLLT #2#"));
    let names: Vec<String> = s.pending_places().iter().map(|p| p.name.clone()).collect();
    assert!(!names.iter().any(|n| n.contains("ENTFÄLLT")), "{names:?}");
}

/// Review round 4: setting the numbering writes only its keys; what another program saved
/// in `projekt.toml` meanwhile stays.
#[test]
fn setting_the_numbering_keeps_what_another_session_saved() {
    let dir = make(
        "nl-header",
        SceneNumbering::Locked,
        &(saal(" #1#") + &hof(" #2#")),
    );
    let mut tui = Session::open(&dir).unwrap();
    let mut window = Session::open(&dir).unwrap();
    let paper = klappe_model::PaperSetup::from("a3-landscape".to_owned());
    window.set_paper(paper);
    window.save(klappe_services::OnOutside::Overwrite).unwrap();

    tui.set_scene_numbering(SceneNumbering::Automatic, 5)
        .unwrap();

    let saved = klappe_store::load(&dir).unwrap().project;
    assert_eq!(saved.paper, paper);
    assert_eq!(saved.script.scene_numbering, SceneNumbering::Automatic);
    assert_eq!(saved.script.first_scene, 5);
}
