//! Scene numbers (DB-33): set in the text when saving in Klappe, never when opening; the
//! register and the plans follow the scenes, not the numbers.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_model::{BookState, FloorPlan, Kind, Project, SceneNumbering, ScriptLanguage};
use klappe_services::Session;

const SAAL: &str = "INT. SAAL – TAG";
const HOF: &str = "INT. HOF – NACHT";
const KELLER: &str = "INT. KELLER – NACHT";

fn saal(number: &str) -> String {
    format!("{SAAL}{number}\n\nAnna betritt den Saal.\n\nANNA\nGuten Tag, ist hier jemand?\n\n")
}

fn hof(number: &str) -> String {
    format!("{HOF}{number}\n\nEin Hund bellt im Hof.\n\nBERT\nSei still, du Köter.\n\n")
}

fn keller(number: &str) -> String {
    format!("{KELLER}{number}\n\nWasser tropft.\n\nCARL\nHier unten ist es kalt.\n\n")
}

fn entry(id: &str, number: &str, heading: &str) -> klappe_model::Scene {
    klappe_model::Scene {
        id: id.into(),
        kind: Kind::Scene,
        number: number.into(),
        heading: heading.into(),
        plans: vec![FloorPlan::new("plan-a", "Plan A")],
        ..klappe_model::Scene::default()
    }
}

/// A project with scenes `aaaa` (the hall) and `bbbb` (the yard) in this book, numbers as set.
fn make(name: &str, numbering: SceneNumbering, first: u32, book: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    let mut project = Project::new("Test");
    project.script.scene_numbering = numbering;
    project.script.first_scene = first;
    project.script.language = Some(ScriptLanguage::De);
    project.scenes = vec![entry("aaaa", "1", SAAL), entry("bbbb", "2", HOF)];
    klappe_store::save(&dir, &project, Some(book)).unwrap();
    // The first open and save puts the prints of the book in the register.
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

fn number_of(s: &Session, id: &str) -> String {
    s.project()
        .scenes
        .iter()
        .find(|x| x.id == id)
        .unwrap()
        .number
        .clone()
}

#[test]
fn opening_changes_neither_the_text_nor_the_file() {
    let dir = make(
        "sn-open",
        SceneNumbering::Automatic,
        1,
        &(saal("") + &hof("")),
    );
    let text = saal(" #7#") + &hof(" #3#");
    write(&dir, &text);

    let s = Session::open(&dir).unwrap();

    assert_eq!(s.script_text(), text);
    assert_eq!(book(&dir), text);
    assert!(!s.is_dirty());
}

#[test]
fn saving_numbers_automatically_from_the_first_number() {
    let dir = make(
        "sn-auto",
        SceneNumbering::Automatic,
        10,
        &(saal("") + &hof("")),
    );
    let mut s = Session::open(&dir).unwrap();

    s.set_script_text(&(keller(" #99#") + &saal(" #7#") + &hof("")));
    s.save(klappe_services::OnOutside::Overwrite).unwrap();

    assert_eq!(book(&dir), keller(" #10#") + &saal(" #11#") + &hof(" #12#"));
    assert_eq!(s.script_text(), book(&dir));
}

/// The plans hang on the scene, so they come along when a new first scene shifts every number.
#[test]
fn renumbering_keeps_the_plans_with_their_scenes() {
    let dir = make(
        "sn-plans",
        SceneNumbering::Automatic,
        1,
        &(saal("") + &hof("")),
    );
    assert_eq!(book(&dir), saal(" #1#") + &hof(" #2#"));
    let mut s = Session::open(&dir).unwrap();

    s.set_script_text(&(keller(" #1#") + &saal(" #2#") + &hof(" #3#")));
    s.save(klappe_services::OnOutside::Overwrite).unwrap();

    let s = Session::open(&dir).unwrap();
    assert_eq!(number_of(&s, "aaaa"), "2");
    assert_eq!(number_of(&s, "bbbb"), "3");
    let plans = |id: &str| {
        s.project()
            .scenes
            .iter()
            .find(|x| x.id == id)
            .unwrap()
            .plans
            .len()
    };
    assert_eq!((plans("aaaa"), plans("bbbb")), (1, 1));
    assert_eq!(s.project().scenes.len(), 3);
}

#[test]
fn an_outside_change_is_numbered_at_the_next_save() {
    let dir = make(
        "sn-outside",
        SceneNumbering::Automatic,
        1,
        &(saal("") + &hof("")),
    );
    write(&dir, &(keller(" #1#") + &saal(" #2#") + &hof(" #3#")));
    let s = Session::open(&dir).unwrap();
    assert_eq!(number_of(&s, "aaaa"), "2");

    write(&dir, &(keller(" #1#") + &hof(" #3#")));
    let mut s = Session::open(&dir).unwrap();
    s.save(klappe_services::OnOutside::Overwrite).unwrap();

    assert_eq!(book(&dir), keller(" #1#") + &hof(" #2#"));
    assert_eq!(number_of(&s, "bbbb"), "2");
}

#[test]
fn locked_numbers_stay_and_a_new_scene_gets_a_letter() {
    let dir = make(
        "sn-locked",
        SceneNumbering::Locked,
        1,
        &(saal(" #1#") + &hof(" #2#")),
    );
    let mut s = Session::open(&dir).unwrap();
    let text = saal(" #1#") + &keller("") + &hof(" #2#");

    s.set_script_text(&text);
    s.save_script(klappe_services::OnOutside::Overwrite)
        .unwrap();

    assert_eq!(book(&dir), saal(" #1#") + &keller(" #1A#") + &hof(" #2#"));
    assert_eq!(number_of(&Session::open(&dir).unwrap(), "bbbb"), "2");
}

#[test]
fn a_struck_scene_stays_as_a_heading_and_keeps_its_plans() {
    let dir = make(
        "sn-struck",
        SceneNumbering::Locked,
        1,
        &(saal(" #1#") + &hof(" #2#")),
    );
    let mut s = Session::open(&dir).unwrap();

    s.set_script_text(&(saal(" #1#") + &keller("") + "\n"));
    s.save(klappe_services::OnOutside::Overwrite).unwrap();

    let text = book(&dir);
    assert!(text.contains("INT. KELLER – NACHT #1A#"), "{text}");
    assert!(text.contains(".ENTFÄLLT #2#"), "{text}");

    // Opened again, the heading is no scene of its own, and the struck entry waits.
    let s = Session::open(&dir).unwrap();
    assert_eq!(s.project().scenes.len(), 3);
    let struck = s.project().scenes.iter().find(|x| x.id == "bbbb").unwrap();
    assert_eq!(struck.state, BookState::NotInBook);
    assert_eq!(struck.plans.len(), 1);
    assert!(s.pending_scenes().is_empty());

    // Saving again adds nothing.
    let mut s = s;
    s.save(klappe_services::OnOutside::Overwrite).unwrap();
    assert_eq!(book(&dir), text);
}

#[test]
fn an_english_project_says_omitted() {
    let dir = make(
        "sn-omitted",
        SceneNumbering::Locked,
        1,
        &(saal(" #1#") + &hof(" #2#")),
    );
    let s = Session::open(&dir).unwrap();
    let mut project = s.project().clone();
    project.script.language = Some(ScriptLanguage::En);
    klappe_store::save(&dir, &project, None).unwrap();
    let mut s = Session::open(&dir).unwrap();

    s.set_script_text(&saal(" #1#"));
    s.save(klappe_services::OnOutside::Overwrite).unwrap();

    assert!(book(&dir).contains(".OMITTED #2#"), "{}", book(&dir));
}

#[test]
fn the_setting_is_written_to_the_project_file_alone() {
    let dir = make(
        "sn-setting",
        SceneNumbering::Automatic,
        1,
        &(saal("") + &hof("")),
    );
    let mut s = Session::open(&dir).unwrap();
    let text = book(&dir);
    s.set_script_text(&(keller("") + &text));

    s.set_scene_numbering(SceneNumbering::Locked, 5).unwrap();

    let loaded = klappe_store::load(&dir).unwrap().project.script;
    assert_eq!(
        (loaded.scene_numbering, loaded.first_scene),
        (SceneNumbering::Locked, 5)
    );
    assert_eq!(book(&dir), text);
}

/// A project from before the setting has no key and no Sperrstand: its text stays as it is,
/// the unnumbered scene included.
#[test]
fn a_project_without_the_key_keeps_its_text() {
    let dir = make(
        "sn-nokey",
        SceneNumbering::Automatic,
        1,
        &(saal("") + &hof("")),
    );
    let text = saal(" #12#") + &keller("") + &hof(" #13#");
    write(&dir, &text);
    let path = dir.join("projekt.toml");
    let header = std::fs::read_to_string(&path).unwrap();
    let without: String = header
        .lines()
        .filter(|l| !l.starts_with("scene_numbering"))
        .map(|l| format!("{l}\n"))
        .collect();
    std::fs::write(&path, without).unwrap();
    std::fs::remove_file(dir.join("nummern.toml")).unwrap();

    let mut s = Session::open(&dir).unwrap();
    s.save(klappe_services::OnOutside::Overwrite).unwrap();

    assert_eq!(book(&dir), text);
}

#[test]
fn a_new_project_is_automatic_and_says_so_in_its_file() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("sn-create");
    let _ = std::fs::remove_dir_all(&dir);

    let s = Session::create(&dir, "Neu", klappe_services::Language::English).unwrap();

    assert_eq!(
        s.project().script.scene_numbering,
        SceneNumbering::Automatic
    );
    let header = std::fs::read_to_string(dir.join("projekt.toml")).unwrap();
    assert!(
        header.contains("scene_numbering = \"automatic\""),
        "{header}"
    );
}

/// An entry that is not in the book already (made in Stellprobe, or struck long ago) gets no
/// heading: only a scene struck since the last save does.
#[test]
fn an_entry_not_in_the_book_already_gets_no_heading() {
    let dir = make(
        "sn-old",
        SceneNumbering::Locked,
        1,
        &(saal(" #1#") + &hof(" #2#")),
    );
    let mut project = klappe_store::load(&dir).unwrap().project;
    let mut old = entry("cccc", "9", KELLER);
    old.state = BookState::NotInBook;
    project.scenes.push(old);
    klappe_store::save(&dir, &project, None).unwrap();
    let before = book(&dir);

    let mut s = Session::open(&dir).unwrap();
    s.save(klappe_services::OnOutside::Overwrite).unwrap();
    s.set_script_text(&(saal(" #1#") + "\n"));
    s.save(klappe_services::OnOutside::Overwrite).unwrap();

    let text = book(&dir);
    assert!(!text.contains("#9#"), "{text}");
    assert!(text.contains(".ENTFÄLLT #2#"), "{text}");
    assert_eq!(before, saal(" #1#") + &hof(" #2#"));
}

#[test]
fn saving_a_locked_book_with_old_entries_leaves_the_text_alone() {
    let dir = make(
        "sn-old-same",
        SceneNumbering::Locked,
        1,
        &(saal(" #1#") + &hof(" #2#")),
    );
    let mut project = klappe_store::load(&dir).unwrap().project;
    let mut old = entry("cccc", "9", KELLER);
    old.state = BookState::NotInBook;
    project.scenes.push(old);
    klappe_store::save(&dir, &project, None).unwrap();
    let before = book(&dir);

    Session::open(&dir)
        .unwrap()
        .save(klappe_services::OnOutside::Overwrite)
        .unwrap();

    assert_eq!(book(&dir), before);
}

/// A copy of a scene (new lines) placed before the scene the register knows gets a number between
/// of its own: the register's scene keeps `2`, the copy is the inserted one.
#[test]
fn a_copy_placed_before_leaves_the_number_with_the_register_scene() {
    let dir = make(
        "sn-copy-before",
        SceneNumbering::Locked,
        1,
        &(saal(" #1#") + &hof(" #2#")),
    );
    let mut s = Session::open(&dir).unwrap();
    let copy = hof(" #2#").replace("Hund bellt", "Pferd wiehert");

    s.set_script_text(&(saal(" #1#") + &copy + &hof(" #2#")));
    s.save_script(klappe_services::OnOutside::Overwrite)
        .unwrap();

    let expected = saal(" #1#") + &copy.replace("#2#", "#1A#") + &hof(" #2#");
    assert_eq!(book(&dir), expected);
    assert_eq!(number_of(&s, "bbbb"), "2");
}

/// Two scenes with `9` from outside, and no Sperrstand names the holder: saving without a
/// change of the text leaves the book as it is.
#[test]
fn a_duplicate_number_from_outside_stays_when_no_scene_is_known_as_holder() {
    let dir = make(
        "sn-dup-outside",
        SceneNumbering::Locked,
        1,
        &(saal(" #1#") + &hof(" #2#")),
    );
    let text = saal(" #9#") + &hof(" #9#");
    write(&dir, &text);
    std::fs::remove_file(dir.join("nummern.toml")).unwrap();

    let mut s = Session::open(&dir).unwrap();
    s.save(klappe_services::OnOutside::Overwrite).unwrap();

    assert_eq!(book(&dir), text);
}

/// An entry that never met the book asks which of two scenes with its number it is: until
/// answered, neither is the holder and neither gets a letter.
#[test]
fn open_twin_questions_write_no_number() {
    let dir = make(
        "sn-twins",
        SceneNumbering::Locked,
        1,
        &(saal(" #1#") + &hof(" #2#")),
    );
    let mut project = klappe_store::load(&dir).unwrap().project;
    let mut asked = entry("cccc", "9", KELLER);
    asked.plans.clear();
    project.scenes.push(asked);
    klappe_store::save(&dir, &project, None).unwrap();
    let twin = keller(" #9#").replace("Wasser tropft", "Es riecht nach Rauch");
    let text = saal(" #1#") + &keller(" #9#") + &twin + &hof(" #2#");
    write(&dir, &text);

    let mut s = Session::open(&dir).unwrap();
    assert_eq!(s.pending_scenes().len(), 1);
    s.save(klappe_services::OnOutside::Overwrite).unwrap();

    assert_eq!(book(&dir), text);
}
