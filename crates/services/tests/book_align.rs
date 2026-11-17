//! The register meets the book: ids and plans follow their scene through changes of the
//! screenplay made outside Klappe (DATENMODELL §4, DB-65).
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_model::{BookState, FloorPlan, Kind, Project, Scene};
use klappe_services::{Choice, ProjectEdit, Session};

const SAAL: &str = "INT. SAAL – TAG";
const HOF: &str = "INT. HOF – NACHT";
const KELLER: &str = "INT. KELLER – NACHT";

fn saal(number: &str) -> String {
    format!(
        "{SAAL} #{number}#\n\nAnna betritt den Saal und sieht sich um.\n\nANNA\nGuten Tag, ist hier jemand?\n\n"
    )
}

fn hof(number: &str) -> String {
    format!("{HOF} #{number}#\n\nEin Hund bellt im Hof.\n\nBERT\nSei still, du Köter.\n\n")
}

fn keller(number: &str) -> String {
    format!(
        "{KELLER} #{number}#\n\nWasser tropft von der Decke.\n\nCARL\nHier unten ist es kalt.\n\n"
    )
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

/// A project folder with these entries (no prints yet) and this book.
fn make(name: &str, entries: Vec<Scene>, book: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    let mut project = Project::new("Test");
    project.scenes = entries;
    klappe_store::save(&dir, &project, Some(book)).unwrap();

    dir
}

/// Opens and saves, so the register holds the prints of the book as it is now.
fn settled(name: &str, entries: Vec<Scene>, book: &str) -> PathBuf {
    let dir = make(name, entries, book);
    let mut s = Session::open(&dir).unwrap();
    s.save().unwrap();

    dir
}

fn rewrite_book(dir: &Path, book: &str) {
    std::fs::write(dir.join("drehbuch.fountain"), book).unwrap();
}

fn two() -> Vec<Scene> {
    vec![entry("aaaa", "1", SAAL), entry("bbbb", "2", HOF)]
}

fn scene<'a>(s: &'a Session, id: &str) -> &'a Scene {
    s.project().scenes.iter().find(|x| x.id == id).unwrap()
}

#[test]
fn a_scene_inserted_before_keeps_the_plans_of_the_others() {
    let dir = settled("book-insert", two(), &(saal("1") + &hof("2")));
    rewrite_book(&dir, &(keller("1") + &saal("2") + &hof("3")));

    let s = Session::open(&dir).unwrap();

    assert_eq!(s.project().scenes.len(), 3);
    let ids: Vec<&str> = s.project().scenes.iter().map(|x| x.id.as_str()).collect();
    assert_eq!(ids[1..], ["aaaa", "bbbb"], "book order");
    assert_eq!(scene(&s, "aaaa").number, "2");
    assert_eq!(scene(&s, "bbbb").number, "3");
    assert!(!scene(&s, "aaaa").plans.is_empty() && !scene(&s, "bbbb").plans.is_empty());

    let new = &s.project().scenes[0];
    assert_eq!((new.number.as_str(), new.heading.as_str()), ("1", KELLER));
    assert_eq!(new.kind, Kind::Scene);
    assert!(new.plans.is_empty());
    assert!(s.pending_scenes().is_empty());
}

#[test]
fn renumbering_every_scene_outside_keeps_ids_and_plans() {
    let dir = settled("book-renumber", two(), &(saal("1") + &hof("2")));
    rewrite_book(&dir, &(saal("10") + &hof("20")));

    let s = Session::open(&dir).unwrap();

    assert_eq!(s.project().scenes.len(), 2);
    assert_eq!(scene(&s, "aaaa").number, "10");
    assert_eq!(scene(&s, "bbbb").number, "20");
    assert_eq!(scene(&s, "aaaa").plans.len(), 1);
    assert_eq!(scene(&s, "bbbb").plans.len(), 1);
    assert!(s.pending_scenes().is_empty());
}

/// Scene 2 is a different scene now but still carries the number 2.
fn rewritten() -> String {
    let other =
        format!("{KELLER} #2#\n\nDas Licht flackert und geht aus.\n\nCARL\nWer ist da?\n\n");

    saal("1") + &other
}

#[test]
fn a_rewritten_scene_is_asked_about_not_reassigned() {
    let dir = settled("book-rewrite", two(), &(saal("1") + &hof("2")));
    rewrite_book(&dir, &rewritten());

    let s = Session::open(&dir).unwrap();

    let pending = s.pending_scenes();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].id, "bbbb");
    assert_eq!(pending[0].candidates.len(), 1);
    assert_eq!(pending[0].candidates[0].heading, KELLER);

    // Untouched until the user decides, and the candidate gets no entry of its own.
    let held = scene(&s, "bbbb");
    assert_eq!((held.number.as_str(), held.heading.as_str()), ("2", HOF));
    assert_eq!(held.state, BookState::InBook);
    assert_eq!(held.plans.len(), 1);
    assert_eq!(s.project().scenes.len(), 2);
    assert_eq!(s.book_report().pending, 1);
}

#[test]
fn a_scene_struck_from_the_book_stays_not_in_book() {
    let dir = settled("book-strike", two(), &(saal("1") + &hof("2")));
    rewrite_book(&dir, &saal("1"));

    let s = Session::open(&dir).unwrap();

    let gone = scene(&s, "bbbb");
    assert_eq!(gone.state, BookState::NotInBook);
    assert_eq!(gone.plans.len(), 1);
    assert_eq!(scene(&s, "aaaa").state, BookState::InBook);
    assert!(s.pending_scenes().is_empty());
    assert_eq!(s.book_report().not_in_book, 1);

    // Saved so, and back with the text.
    let mut s = s;
    s.save().unwrap();
    rewrite_book(&dir, &(saal("1") + &hof("2")));
    let s = Session::open(&dir).unwrap();
    assert_eq!(scene(&s, "bbbb").state, BookState::InBook);
}

#[test]
fn a_project_without_prints_is_matched_when_number_and_heading_agree() {
    let entries = vec![
        entry("aaaa", "1", SAAL),
        entry("bbbb", "77", HOF),
        entry("cccc", "99", "INT. NIRGENDWO – TAG"),
    ];
    let dir = make("book-first", entries, &(saal("1") + &hof("2")));

    let s = Session::open(&dir).unwrap();

    let a = scene(&s, "aaaa");
    assert_eq!((a.number.as_str(), a.heading.as_str()), ("1", SAAL));
    assert!(!a.prints.is_empty());
    assert_eq!(a.state, BookState::InBook);
    // Only the heading agrees: asked, not matched.
    let b = scene(&s, "bbbb");
    assert_eq!((b.number.as_str(), b.heading.as_str()), ("77", HOF));
    assert!(b.prints.is_empty());
    assert_eq!(s.pending_scenes().len(), 1);
    assert_eq!(s.pending_scenes()[0].id, "bbbb");
    assert_eq!(scene(&s, "cccc").state, BookState::NotInBook);
    assert_eq!(s.project().scenes.len(), 3);
}

#[test]
fn a_first_run_with_swapped_headings_asks_about_both() {
    let entries = vec![entry("aaaa", "1", KELLER), entry("bbbb", "2", HOF)];
    let book = hof("1") + &keller("2");
    let dir = make("book-first-swap", entries, &book);

    let s = Session::open(&dir).unwrap();

    assert_eq!(s.pending_scenes().len(), 2);
    assert_eq!(scene(&s, "aaaa").heading, KELLER);
    assert_eq!(scene(&s, "bbbb").heading, HOF);
    assert_eq!(s.project().scenes.len(), 2, "no entry for the candidates");
}

#[test]
fn a_first_run_with_a_doubled_scene_asks() {
    let flur = |n: &str, line: &str| format!("INT. FLUR – TAG #{n}#\n\n{line}\n\n");

    // Two scenes of the book carry the same number and heading.
    let book = flur("5", "Schritte hallen.") + &flur("5", "Eine Tür quietscht.");
    let dir = make(
        "book-first-flur-book",
        vec![entry("aaaa", "5", "INT. FLUR – TAG")],
        &book,
    );
    let s = Session::open(&dir).unwrap();
    assert_eq!(s.pending_scenes().len(), 1);
    assert_eq!(s.pending_scenes()[0].candidates.len(), 2);
    assert!(scene(&s, "aaaa").prints.is_empty());

    // Two entries carry the same number and heading, the book one scene.
    let entries = vec![
        entry("aaaa", "5", "INT. FLUR – TAG"),
        entry("bbbb", "5", "INT. FLUR – TAG"),
    ];
    let dir = make(
        "book-first-flur-entries",
        entries,
        &flur("5", "Schritte hallen."),
    );
    let s = Session::open(&dir).unwrap();
    assert_eq!(s.pending_scenes().len(), 2);
    assert!(scene(&s, "aaaa").prints.is_empty() && scene(&s, "bbbb").prints.is_empty());
}

#[test]
fn a_heading_only_scene_whose_heading_is_rewritten_is_asked_about() {
    let haus = "INT. HAUS – TAG #1#\n\n";
    let wald = "INT. WALD – TAG #1#\n\n";
    let dir = settled(
        "book-heading-only",
        vec![entry("aaaa", "1", "INT. HAUS – TAG")],
        haus,
    );
    rewrite_book(&dir, wald);

    let s = Session::open(&dir).unwrap();

    assert_eq!(s.pending_scenes().len(), 1);
    assert_eq!(scene(&s, "aaaa").heading, "INT. HAUS – TAG");
    assert_eq!(s.project().scenes.len(), 1);
}

#[test]
fn an_unclear_first_match_is_asked_about() {
    // Two scenes with the same heading: the heading does not say which one.
    let book = hof("5") + &format!("{HOF} #6#\n\nEin Pferd wiehert.\n\nDORA\nNa los.\n\n");
    let dir = make("book-first-twin", vec![entry("aaaa", "9", HOF)], &book);

    let s = Session::open(&dir).unwrap();

    assert_eq!(s.pending_scenes().len(), 1);
    assert_eq!(s.pending_scenes()[0].candidates.len(), 2);
    assert_eq!(s.project().scenes.len(), 1);
}

#[test]
fn resolving_assigns_the_scene_in_one_undo_step() {
    let dir = settled("book-resolve", two(), &(saal("1") + &hof("2")));
    rewrite_book(&dir, &rewritten());
    let mut s = Session::open(&dir).unwrap();
    let candidate = s.pending_scenes()[0].candidates[0].clone();

    s.resolve_scene("bbbb", Choice::Scene(candidate)).unwrap();

    let b = scene(&s, "bbbb");
    assert_eq!(b.heading, KELLER);
    assert_eq!(b.plans.len(), 1);
    assert_eq!(b.state, BookState::InBook);
    assert!(s.pending_scenes().is_empty());
    assert_eq!(s.project().scenes.len(), 2);
    assert_eq!(s.undo_step(), Some("ResolveScene"));

    s.undo();

    assert_eq!(scene(&s, "bbbb").heading, HOF);
    assert_eq!(s.pending_scenes().len(), 1);

    s.redo();

    assert_eq!(scene(&s, "bbbb").heading, KELLER);
    assert!(s.pending_scenes().is_empty());
}

#[test]
fn resolving_as_not_in_book_makes_the_candidate_a_new_scene() {
    let dir = settled("book-resolve-out", two(), &(saal("1") + &hof("2")));
    rewrite_book(&dir, &rewritten());
    let mut s = Session::open(&dir).unwrap();

    s.resolve_scene("bbbb", Choice::NotInBook).unwrap();

    assert_eq!(scene(&s, "bbbb").state, BookState::NotInBook);
    assert_eq!(scene(&s, "bbbb").heading, HOF);
    assert_eq!(s.project().scenes.len(), 3);
    assert!(s.project().scenes.iter().any(|x| x.heading == KELLER));
    assert!(s.pending_scenes().is_empty());
}

#[test]
fn resolving_refuses_what_is_no_open_decision() {
    let dir = settled("book-resolve-bad", two(), &(saal("1") + &hof("2")));
    rewrite_book(&dir, &rewritten());
    let mut s = Session::open(&dir).unwrap();

    assert!(s.resolve_scene("aaaa", Choice::NotInBook).is_err());
    let mut wrong = s.pending_scenes()[0].candidates[0].clone();
    wrong.index = 0;
    assert!(s.resolve_scene("bbbb", Choice::Scene(wrong)).is_err());
    assert!(s.pending_scenes().len() == 1 && !s.can_undo());
}

#[test]
fn saving_refreshes_the_register_from_the_book() {
    let dir = settled("book-save", two(), &(saal("1") + &hof("2")));
    let mut s = Session::open(&dir).unwrap();

    s.set_script_text(&(saal("5") + &hof("6")));
    s.save().unwrap();
    let s = Session::open(&dir).unwrap();

    assert_eq!(scene(&s, "aaaa").number, "5");
    assert_eq!(scene(&s, "bbbb").number, "6");
}

#[test]
fn a_choice_made_before_the_book_moved_is_refused() {
    let dir = settled("book-stale", two(), &(saal("1") + &hof("2")));
    rewrite_book(&dir, &rewritten());
    let mut s = Session::open(&dir).unwrap();
    let candidate = s.pending_scenes()[0].candidates[0].clone();

    // Another scene with number 2 goes in before it: index 1 is still a candidate of the
    // entry, but no longer the scene the user saw.
    let garten = "INT. GARTEN – TAG #2#\n\nVögel singen.\n\nEMMA\nWas für ein Morgen.\n\n";
    s.set_script_text(&(saal("1") + garten + &rewritten_second()));
    assert!(s.pending_scenes()[0]
        .candidates
        .iter()
        .any(|c| c.index == candidate.index));
    let refused = s.resolve_scene("bbbb", Choice::Scene(candidate));

    assert!(refused.is_err());
    assert_eq!(scene(&s, "bbbb").heading, HOF);
    assert!(!s.can_undo());
}

fn rewritten_second() -> String {
    rewritten().split_off(saal("1").len())
}

#[test]
fn content_evidence_ranks_before_a_first_run_agreement() {
    let dir = settled("book-rank", two(), &(saal("1") + &hof("2")));
    // An entry that never met a book, number and heading equal to the rewritten scene's.
    let loaded = klappe_store::load(&dir).unwrap();
    let mut project = loaded.project;
    project.scenes.push(Scene {
        state: BookState::NotInBook,
        ..entry("cccc", "3", HOF)
    });
    klappe_store::save(&dir, &project, None).unwrap();
    let rewritten = format!("{HOF} #3#\n\nEin Hund bellt im Hof.\n\nBERT\nGanz neue Rede.\n\n");
    rewrite_book(&dir, &(saal("1") + &rewritten));

    let s = Session::open(&dir).unwrap();

    let asked: Vec<&str> = s.pending_scenes().iter().map(|p| p.id.as_str()).collect();
    assert_eq!(asked, ["bbbb", "cccc"]);
    assert_eq!(scene(&s, "bbbb").state, BookState::InBook);
    assert!(scene(&s, "cccc").prints.is_empty());
    assert_eq!(s.project().scenes.len(), 3, "no entry for the candidate");
}

/// Two entries ask about the same scene of the book (as in
/// `content_evidence_ranks_before_a_first_run_agreement`).
fn two_ask_one(name: &str) -> PathBuf {
    let dir = settled(name, two(), &(saal("1") + &hof("2")));
    let loaded = klappe_store::load(&dir).unwrap();
    let mut project = loaded.project;
    project.scenes.push(Scene {
        state: BookState::NotInBook,
        ..entry("cccc", "3", HOF)
    });
    klappe_store::save(&dir, &project, None).unwrap();
    let rewritten = format!("{HOF} #3#\n\nEin Hund bellt im Hof.\n\nBERT\nGanz neue Rede.\n\n");
    rewrite_book(&dir, &(saal("1") + &rewritten));

    dir
}

#[test]
fn an_answer_not_in_book_holds_after_reopening() {
    let dir = two_ask_one("book-refused");
    let mut s = Session::open(&dir).unwrap();

    // bbbb is not the scene: cccc, whose number and heading agree, takes it.
    s.resolve_scene("bbbb", Choice::NotInBook).unwrap();

    assert!(s.pending_scenes().is_empty(), "{:?}", s.pending_scenes());
    assert_eq!(scene(&s, "cccc").state, BookState::InBook);
    s.save().unwrap();
    let s = Session::open(&dir).unwrap();
    assert!(s.pending_scenes().is_empty(), "{:?}", s.pending_scenes());
    assert_eq!(scene(&s, "bbbb").state, BookState::NotInBook);
    assert_eq!(scene(&s, "cccc").state, BookState::InBook);
    assert_eq!(s.project().scenes.len(), 3);
}

/// The entry "9 HOF" is open: the book has two scenes with its heading.
fn open_hof(name: &str) -> Session {
    let book = hof("9") + &hof("9");
    let dir = make(name, vec![entry("aaaa", "9", HOF)], &book);
    let s = Session::open(&dir).unwrap();
    assert_eq!(s.pending_scenes().len(), 1);

    s
}

#[test]
fn removing_an_entry_closes_its_question() {
    let mut s = open_hof("book-remove-open");

    s.edit_project(ProjectEdit::RemoveScene { id: "aaaa".into() })
        .unwrap();

    assert!(s.project().scenes.is_empty());
    assert!(s.pending_scenes().is_empty());
    assert_eq!(s.book_report().pending, 0);
}

#[test]
fn adding_or_renaming_an_entry_refreshes_the_questions() {
    let mut s = open_hof("book-edit-open");
    s.edit_project(ProjectEdit::AddLocation {
        name: "Halle".into(),
        width_m: 8.0,
        depth_m: 5.5,
        wall_height_m: 2.6,
    })
    .unwrap();
    let halle = s.project().locations[0].id.clone();

    // The entry no longer carries the doubled heading: no question.
    s.edit_project(ProjectEdit::EditScene {
        id: "aaaa".into(),
        number: "9".into(),
        heading: KELLER.into(),
        location: halle.clone(),
    })
    .unwrap();
    assert!(s.pending_scenes().is_empty());

    // A new entry with the doubled heading is a question of its own.
    s.edit_project(ProjectEdit::AddScene {
        number: "10".into(),
        heading: HOF.into(),
        location: halle.clone(),
    })
    .unwrap();
    assert_eq!(s.pending_scenes().len(), 1);
    assert_ne!(s.pending_scenes()[0].id, "aaaa");
    assert_eq!(s.book_report().pending, 1);
}
