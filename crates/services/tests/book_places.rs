//! The play locations meet the scene headings of the book (DB-51, DATENMODELL §4).
#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use klappe_model::{FigureColor, Project};
use klappe_services::{PlaceChoice, ProjectEdit, Session};

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
    assert!(s.project().places.is_empty());
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
    let id = s.project().places[0].id.clone();

    s.resolve_place("HOF", PlaceChoice::Variant(id.clone()))
        .unwrap();

    assert!(s.pending_places().is_empty());
    assert_eq!(s.project().places[0].variants, ["HOF"]);
    let again = Session::open(&dir).unwrap();
    assert!(again.pending_places().is_empty());
    assert_eq!(again.project().places[0].id, id);
}

#[test]
fn a_rename_in_the_book_keeps_the_id() {
    let dir = make("places-rename", BOOK);
    let mut s = Session::open(&dir).unwrap();
    s.resolve_place("SÄNGERSAAL", PlaceChoice::Create).unwrap();
    s.resolve_place("HOF", PlaceChoice::Create).unwrap();
    let id = s.project().places[0].id.clone();
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
    assert_eq!(s.project().places[0].id, id);
    assert_eq!(s.project().places[0].name, "FESTSAAL");
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
    let theirs = other.project().places[0].id.clone();

    early.resolve_place("HOF", PlaceChoice::Create).unwrap();

    let places = klappe_store::load_places(&dir).unwrap();
    assert_eq!(places.len(), 2, "{}", orte(&dir));
    assert_eq!(places[0].id, theirs);
    assert_eq!(early.project().places, places);
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
    let hof = other.project().places[0].id.clone();
    let before = orte(&dir);

    // The question is stale: the file already knows the name.
    let answered = early.resolve_place("SÄNGERSAAL", PlaceChoice::Variant(hof));

    assert!(answered.is_err());
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

    assert_eq!(s.project().places.len(), 1);
    s.save().unwrap();
    assert!(orte(&dir).contains("SÄNGERSAAL"));
}

#[test]
fn a_save_keeps_what_another_program_answered() {
    let dir = make("places-save", BOOK);
    let mut early = Session::open(&dir).unwrap();
    let mut other = Session::open(&dir).unwrap();
    other.resolve_place("HOF", PlaceChoice::Create).unwrap();

    early.save().unwrap();

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
        Session::open(&dir).unwrap().save().unwrap();
        let before = snapshot(&dir);

        let mut s = Session::open(&dir).unwrap();
        s.save().unwrap();
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
    Session::open(&dir).unwrap().save().unwrap();
    let before = snapshot(&dir);

    let mut s = Session::open(&dir).unwrap();
    s.save().unwrap();

    assert_eq!(changed(&snapshot(&dir), &before), Vec::<PathBuf>::new());
}
