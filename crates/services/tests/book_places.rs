//! The scripted locations meet the scene headings of the book (DB-51, DATENMODELL §4).
#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use klappe_model::{FigureColor, Project};
use klappe_services::{Language, PlaceChoice, ProjectEdit, QuestionKey, Session};

const BOOK: &str = "INT. SÄNGERSAAL – TAG #1#\n\nAnna wartet.\n\n\
EXT. HOF - NACHT #2#\n\nEin Hund.\n\nINNEN SÄNGERSAAL - NACHT #3#\n\nStille.\n\n";

fn make(name: &str, book: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    klappe_store::save(&dir, &Project::new("Test"), Some(book)).unwrap();

    dir
}

fn names(s: &Session) -> Vec<(String, usize)> {
    s.pending_places()
        .iter()
        .map(|p| (p.name.clone(), p.headings))
        .collect()
}

fn orte(dir: &Path) -> String {
    std::fs::read_to_string(dir.join("orte.toml")).unwrap_or_default()
}

/// Every file of the folder with its bytes.
fn snapshot(dir: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut found = BTreeMap::new();
    let mut open = vec![dir.to_path_buf()];
    while let Some(folder) = open.pop() {
        for entry in std::fs::read_dir(&folder).unwrap() {
            let path = entry.unwrap().path();
            match path.is_dir() {
                true => open.push(path),
                false => {
                    found.insert(
                        path.strip_prefix(dir).unwrap().to_path_buf(),
                        std::fs::read(&path).unwrap(),
                    );
                }
            }
        }
    }

    found
}

/// The files that differ between two snapshots, or are in one only.
fn changed(a: &BTreeMap<PathBuf, Vec<u8>>, b: &BTreeMap<PathBuf, Vec<u8>>) -> Vec<PathBuf> {
    let keys = a.keys().chain(b.keys());

    keys.filter(|k| a.get(*k) != b.get(*k)).cloned().collect()
}

fn copy_tree(from: &Path, to: &Path) {
    let _ = std::fs::remove_dir_all(to);
    for (rel, bytes) in snapshot(from) {
        let target = to.join(rel);
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::write(target, bytes).unwrap();
    }
}

#[test]
fn headings_are_asked_about_never_made_places() {
    let dir = make("places-ask", BOOK);

    let s = Session::open(&dir).unwrap();

    assert_eq!(
        names(&s),
        [("SÄNGERSAAL".to_owned(), 2), ("HOF".to_owned(), 1)]
    );
    assert!(s.places().is_empty());
    assert!(!dir.join("orte.toml").exists());
    assert_eq!(s.book_report().places, 2);
}

#[test]
fn an_answer_is_written_at_once_and_matches_the_headings() {
    let dir = make("places-answer", BOOK);
    let mut s = Session::open(&dir).unwrap();

    s.resolve_place("SÄNGERSAAL", PlaceChoice::Create).unwrap();

    assert_eq!(names(&s), [("HOF".to_owned(), 1)]);
    assert!(
        orte(&dir).contains("name = \"SÄNGERSAAL\""),
        "{}",
        orte(&dir)
    );
    let id = s.places()[0].id.clone();

    s.resolve_place("HOF", PlaceChoice::Variant(id.clone()))
        .unwrap();

    assert!(s.pending_places().is_empty());
    assert_eq!(s.places()[0].variants, ["HOF"]);
    let again = Session::open(&dir).unwrap();
    assert!(again.pending_places().is_empty());
    assert_eq!(again.places()[0].id, id);
}

#[test]
fn a_rename_in_the_book_keeps_the_id() {
    let dir = make("places-rename", BOOK);
    let mut s = Session::open(&dir).unwrap();
    s.resolve_place("SÄNGERSAAL", PlaceChoice::Create).unwrap();
    s.resolve_place("HOF", PlaceChoice::Create).unwrap();
    let id = s.places()[0].id.clone();
    drop(s);

    std::fs::write(
        dir.join("drehbuch.fountain"),
        BOOK.replace("SÄNGERSAAL", "FESTSAAL"),
    )
    .unwrap();
    let mut s = Session::open(&dir).unwrap();

    assert_eq!(names(&s), [("FESTSAAL".to_owned(), 2)]);
    // The place the headings do not name any more is offered first, as a rename.
    let answers = s.place_answers("FESTSAAL");
    assert_eq!(answers[0].choice, PlaceChoice::Create);
    assert_eq!(answers[1].choice, PlaceChoice::Rename(id.clone()));
    assert_eq!(answers[1].name, "SÄNGERSAAL");
    assert_eq!(answers[2].choice, PlaceChoice::Variant(id.clone()));

    s.answer_place("FESTSAAL", 1).unwrap();

    assert!(s.pending_places().is_empty());
    assert_eq!(s.places()[0].id, id);
    assert_eq!(s.places()[0].name, "FESTSAAL");
}

#[test]
fn an_answer_changes_only_what_it_touches() {
    let dir = make("places-stale", BOOK);
    let mut early = Session::open(&dir).unwrap();

    // Another program answers the first question after this session opened.
    let mut other = Session::open(&dir).unwrap();
    other
        .resolve_place("SÄNGERSAAL", PlaceChoice::Create)
        .unwrap();
    let theirs = other.places()[0].id.clone();

    early.resolve_place("HOF", PlaceChoice::Create).unwrap();

    let places = klappe_store::load_places(&dir).unwrap();
    assert_eq!(places.len(), 2, "{}", orte(&dir));
    assert_eq!(places[0].id, theirs);
    assert_eq!(early.places(), places);
}

#[test]
fn a_name_belongs_to_one_place() {
    let dir = make("places-one", BOOK);
    let mut early = Session::open(&dir).unwrap();
    let mut other = Session::open(&dir).unwrap();
    other.resolve_place("HOF", PlaceChoice::Create).unwrap();
    other
        .resolve_place("SÄNGERSAAL", PlaceChoice::Create)
        .unwrap();
    let hof = other.places()[0].id.clone();
    let before = orte(&dir);

    // The question is stale: the file already knows the name. It goes, the file stays.
    early
        .resolve_place("SÄNGERSAAL", PlaceChoice::Variant(hof))
        .unwrap();

    assert!(early
        .pending_places()
        .iter()
        .all(|p| p.name != "SÄNGERSAAL"));
    assert_eq!(orte(&dir), before);
}

#[test]
fn a_file_with_one_name_on_two_places_does_not_load() {
    let dir = make("places-dup", BOOK);
    std::fs::write(
        dir.join("orte.toml"),
        "[[ort]]\nid = \"a\"\nname = \"Straße\"\n\n[[ort]]\nid = \"b\"\nname = \"X\"\nvariants = [\"STRASSE\"]\n",
    )
    .unwrap();

    let error = Session::open(&dir).err().unwrap().to_string();

    assert!(error.contains("orte.toml"), "{error}");
}

#[test]
fn undo_and_a_save_keep_the_places() {
    let dir = make("places-undo", BOOK);
    let mut s = Session::open(&dir).unwrap();
    s.resolve_place("SÄNGERSAAL", PlaceChoice::Create).unwrap();
    s.edit_project(ProjectEdit::AddCharacter {
        name: "Anna".into(),
        short: "A".into(),
        color: FigureColor::Red,
        group: None,
        size_m: None,
    })
    .unwrap();
    s.undo();

    assert_eq!(s.places().len(), 1);
    s.save(klappe_services::OnOutside::Overwrite).unwrap();
    assert!(orte(&dir).contains("SÄNGERSAAL"));
}

#[test]
fn a_save_keeps_what_another_program_answered() {
    let dir = make("places-save", BOOK);
    let mut early = Session::open(&dir).unwrap();
    let mut other = Session::open(&dir).unwrap();
    other.resolve_place("HOF", PlaceChoice::Create).unwrap();

    early.save(klappe_services::OnOutside::Overwrite).unwrap();

    assert!(orte(&dir).contains("HOF"));
}

#[test]
fn opening_and_saving_without_an_answer_changes_no_byte() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let sources = [
        root.join("testdata/beispiel.klappe"),
        root.join("demo/beispiel-de.klappe"),
        root.join("demo/beispiel-en.klappe"),
    ];

    for (n, source) in sources.iter().enumerate() {
        let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("places-bytes-{n}"));
        copy_tree(source, &dir);
        // The first save settles what follows the book (ids, prints): not the places' business.
        Session::open(&dir)
            .unwrap()
            .save(klappe_services::OnOutside::Overwrite)
            .unwrap();
        let before = snapshot(&dir);

        let mut s = Session::open(&dir).unwrap();
        s.save(klappe_services::OnOutside::Overwrite).unwrap();
        s.save_script().unwrap();

        assert_eq!(
            changed(&snapshot(&dir), &before),
            Vec::<PathBuf>::new(),
            "{}",
            source.display()
        );
    }
}

#[test]
fn a_file_written_by_an_answer_is_saved_unchanged() {
    let dir = make("places-bytes-own", BOOK);
    let mut s = Session::open(&dir).unwrap();
    s.resolve_place("SÄNGERSAAL", PlaceChoice::Create).unwrap();
    s.resolve_place("HOF", PlaceChoice::Create).unwrap();
    Session::open(&dir)
        .unwrap()
        .save(klappe_services::OnOutside::Overwrite)
        .unwrap();
    let before = snapshot(&dir);

    let mut s = Session::open(&dir).unwrap();
    s.save(klappe_services::OnOutside::Overwrite).unwrap();

    assert_eq!(changed(&snapshot(&dir), &before), Vec::<PathBuf>::new());
}

/// Review of PR 256: a question another session had answered stayed and said
/// "no open decision for scene HOF".
#[test]
fn a_stale_question_takes_the_disk_and_goes() {
    let dir = make("places-stale-question", BOOK);
    let mut early = Session::open(&dir).unwrap();
    let mut other = Session::open(&dir).unwrap();
    other.resolve_place("HOF", PlaceChoice::Create).unwrap();

    early.resolve_place("HOF", PlaceChoice::Create).unwrap();

    assert_eq!(names(&early), [("SÄNGERSAAL".to_owned(), 2)]);
    assert_eq!(orte(&dir).matches("HOF").count(), 1);
    assert_eq!(early.places().len(), 1);

    let mut third = Session::open(&dir).unwrap();
    other
        .resolve_place("SÄNGERSAAL", PlaceChoice::Create)
        .unwrap();
    third.answer_place("SÄNGERSAAL", 0).unwrap();
    assert!(third.pending_places().is_empty());
}

/// A backup holds no scripted locations: restoring it leaves them to the disk.
#[test]
fn a_restored_backup_leaves_the_places_to_the_disk() {
    let dir = make("places-backup", BOOK);
    klappe_store::save_backup(&dir, &Project::new("Test"), Some(BOOK)).unwrap();
    assert!(!dir.join(".sicherung/orte.toml").exists());

    let mut s = Session::open(&dir).unwrap();
    s.restore_backup().unwrap();
    s.save(klappe_services::OnOutside::Overwrite).unwrap();

    assert!(s.places().is_empty());
    assert_eq!(orte(&dir), "");
}

/// Audit 17, finding 1: the session words the questions, so window and terminal show the same.
#[test]
fn the_questions_come_worded_and_are_answered_by_position() {
    let dir = make("places-worded", BOOK);
    let mut s = Session::open(&dir).unwrap();

    let questions = s.book_questions(Language::German);
    let first = &questions[0];
    assert_eq!(first.key, QuestionKey::Place("SÄNGERSAAL".to_owned()));
    assert_eq!(first.title, Language::German.text("book.placeTitle"));
    assert!(first.ask.contains("SÄNGERSAAL"), "{}", first.ask);
    assert!(
        first.answers[0].text.contains("SÄNGERSAAL"),
        "{:?}",
        first.answers
    );

    s.answer_question(&first.key.clone(), 0).unwrap();

    let next = s.book_questions(Language::German);
    assert_eq!(next.len(), 1);
    assert_eq!(next[0].key, QuestionKey::Place("HOF".to_owned()));
    let lines: Vec<_> = next[0].answers.iter().map(|a| a.text.as_str()).collect();
    assert_eq!(lines, ["Neuer Spielort HOF", "Schreibweise von SÄNGERSAAL"]);
}

/// Review of PR 295: the window words the questions at every refresh (each frame of playback),
/// so wording many is cheap. 80 open place names take about 10 ms per wording in a debug
/// build here and took 400 ms before; the bound leaves room for a slow CI runner.
#[test]
fn many_questions_are_worded_fast() {
    const PLACES: usize = 80;
    const ROUNDS: u32 = 10;
    const BUDGET_PER_ROUND: std::time::Duration = std::time::Duration::from_millis(150);

    let book: String = (0..PLACES)
        .map(|i| format!("INT. ORT {i} - TAG\n\nEtwas.\n\n"))
        .collect();
    let dir = make("places-many", &book);
    let s = Session::open(&dir).unwrap();
    assert_eq!(s.book_questions(Language::German).len(), PLACES);

    let start = std::time::Instant::now();
    for _ in 0..ROUNDS {
        let _ = s.book_questions(Language::German);
    }
    let per_round = start.elapsed() / ROUNDS;

    assert!(per_round < BUDGET_PER_ROUND, "{per_round:?} per wording");
}

/// Review of PR 296: an older backup still holds an `orte.toml`; restoring it ignores the file,
/// the disk wins.
#[test]
fn the_places_of_an_older_backup_are_ignored() {
    let dir = make("places-old-backup", BOOK);
    klappe_store::save_backup(&dir, &Project::new("Test"), Some(BOOK)).unwrap();
    std::fs::write(
        dir.join(".sicherung/orte.toml"),
        "[[ort]]\nid = \"zz\"\nname = \"HOF\"\n",
    )
    .unwrap();

    let mut s = Session::open(&dir).unwrap();
    s.restore_backup().unwrap();

    assert!(s.places().is_empty());
    assert_eq!(names(&s).len(), 2, "HOF is still asked about");
    s.save(klappe_services::OnOutside::Overwrite).unwrap();
    assert!(!dir.join("orte.toml").exists());
}

const TWO: &str = "INT. SÄNGERSAAL - TAG #1#\n\nA.\n\nINT. HOF - NACHT #2#\n\nB.\n\n";

fn place_names(dir: &Path) -> Vec<String> {
    let places = klappe_store::load_places(dir).unwrap();

    places.iter().map(|p| p.name.clone()).collect()
}

#[test]
fn an_answer_counts_as_an_answer_for_undo_and_redo() {
    let dir = make("places-is-answer", BOOK);
    let mut s = Session::open(&dir).unwrap();
    assert!(!s.undo_is_answer());

    s.resolve_place("SÄNGERSAAL", PlaceChoice::Create).unwrap();

    assert!(s.undo_is_answer());
    assert!(!s.redo_is_answer());
    assert_eq!(s.undo_step(), Some("ResolvePlace"));

    s.undo();

    assert!(!s.undo_is_answer());
    assert!(s.redo_is_answer());
}

#[test]
fn undo_and_redo_of_a_new_place_change_the_file_and_the_question() {
    let dir = make("places-undo-create", BOOK);
    let mut s = Session::open(&dir).unwrap();
    s.resolve_place("SÄNGERSAAL", PlaceChoice::Create).unwrap();
    let id = s.places()[0].id.clone();

    s.undo();
    s.save_answer_step().unwrap();

    assert!(!orte(&dir).contains("SÄNGERSAAL"), "{}", orte(&dir));
    assert!(s.places().is_empty());
    assert_eq!(names(&s).len(), 2, "the question is back");

    s.redo();
    s.save_answer_step().unwrap();

    assert_eq!(klappe_store::load_places(&dir).unwrap()[0].id, id);
    assert_eq!(names(&s), [("HOF".to_owned(), 1)]);
}

#[test]
fn undo_and_redo_of_a_variant_and_a_rename() {
    let dir = make("places-undo-variant", BOOK);
    let mut s = Session::open(&dir).unwrap();
    s.resolve_place("SÄNGERSAAL", PlaceChoice::Create).unwrap();
    let id = s.places()[0].id.clone();
    s.resolve_place("HOF", PlaceChoice::Variant(id.clone()))
        .unwrap();

    s.undo();
    s.save_answer_step().unwrap();

    assert_eq!(s.places()[0].variants, Vec::<String>::new());
    assert!(!orte(&dir).contains("HOF"));
    assert_eq!(names(&s), [("HOF".to_owned(), 1)]);

    s.redo();
    s.save_answer_step().unwrap();

    assert!(orte(&dir).contains("HOF"));
    drop(s);

    std::fs::write(
        dir.join("drehbuch.fountain"),
        BOOK.replace("SÄNGERSAAL", "FESTSAAL"),
    )
    .unwrap();
    let mut s = Session::open(&dir).unwrap();
    s.resolve_place("FESTSAAL", PlaceChoice::Rename(id))
        .unwrap();
    assert_eq!(place_names(&dir), ["FESTSAAL"]);

    s.undo();
    s.save_answer_step().unwrap();

    assert_eq!(place_names(&dir), ["SÄNGERSAAL"]);

    s.redo();
    s.save_answer_step().unwrap();

    assert_eq!(place_names(&dir), ["FESTSAAL"]);
}

#[test]
fn undo_takes_back_only_our_entry() {
    let dir = make("places-undo-beside", TWO);
    let mut ours = Session::open(&dir).unwrap();
    let mut other = Session::open(&dir).unwrap();
    ours.resolve_place("SÄNGERSAAL", PlaceChoice::Create)
        .unwrap();
    other.resolve_place("HOF", PlaceChoice::Create).unwrap();

    ours.undo();
    ours.save_answer_step().unwrap();

    assert_eq!(place_names(&dir), ["HOF"]);
    assert_eq!(ours.places().len(), 1);

    ours.redo();
    ours.save_answer_step().unwrap();

    assert_eq!(place_names(&dir), ["HOF", "SÄNGERSAAL"]);
}

#[test]
fn a_failed_write_keeps_the_entry_for_the_next_try() {
    let dir = make("places-undo-retry", BOOK);
    let mut s = Session::open(&dir).unwrap();
    s.resolve_place("SÄNGERSAAL", PlaceChoice::Create).unwrap();
    s.undo();
    let good = orte(&dir);
    std::fs::write(dir.join("orte.toml"), "not [ toml").unwrap();

    assert!(s.save_answer_step().is_err());

    std::fs::write(dir.join("orte.toml"), good).unwrap();
    s.save_answer_step().unwrap();

    assert!(!orte(&dir).contains("SÄNGERSAAL"));
}
