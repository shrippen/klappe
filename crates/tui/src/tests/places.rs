//! The questions about the headings of the book (`Ctrl+K`, DB-51): a location no play
//! location is called by.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::support::*;
use super::*;

const BOOK: &str = "INT. SÄNGERSAAL – TAG #12#\n\nElisabeth tritt an den Tisch.\n\n";

/// The example project without its play locations, so the headings of `book` ask.
fn bare(book: &str) -> Scratch {
    let scratch = Scratch::new(book);
    let _ = fs::remove_file(scratch.dir.join("orte.toml"));

    scratch
}

/// Saves once: the register settles what follows the book (prints), not the places' business.
fn settle(scratch: &Scratch) {
    let mut app = scratch.app();
    let _ = screen(&mut app, 100, 24);
    ctrl(&mut app, 's');
}

fn orte(scratch: &Scratch) -> String {
    fs::read_to_string(scratch.dir.join("orte.toml")).unwrap_or_default()
}

/// Every file of the folder with its bytes.
fn snapshot(dir: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut found = BTreeMap::new();
    let mut open = vec![dir.to_path_buf()];
    while let Some(folder) = open.pop() {
        for entry in fs::read_dir(&folder).unwrap_or_else(|e| unreachable!("{e}")) {
            let path = entry.unwrap_or_else(|e| unreachable!("{e}")).path();
            if path.is_dir() {
                open.push(path);
                continue;
            }
            let bytes = fs::read(&path).unwrap_or_default();
            found.insert(path.strip_prefix(dir).unwrap_or(&path).to_path_buf(), bytes);
        }
    }

    found
}

fn changed(a: &BTreeMap<PathBuf, Vec<u8>>, b: &BTreeMap<PathBuf, Vec<u8>>) -> Vec<String> {
    let mut keys: Vec<_> = a.keys().chain(b.keys()).collect();
    keys.sort();
    keys.dedup();

    keys.into_iter()
        .filter(|k| a.get(*k) != b.get(*k))
        .map(|k| k.display().to_string())
        .collect()
}

#[test]
fn ctrl_k_asks_about_a_location_no_place_has() {
    let scratch = bare(BOOK);
    let mut app = scratch.app();
    let _ = screen(&mut app, 100, 24);
    assert!(app.book_note.contains("locations"), "{}", app.book_note);

    ctrl(&mut app, 'k');

    assert_eq!(app.mode, Mode::Book);
    let frame = screen(&mut app, 100, 24);
    assert!(
        frame.contains("SÄNGERSAAL, in 1 scene heading(s)"),
        "{frame}"
    );
    assert!(
        frame.contains("New scripted location SÄNGERSAAL"),
        "{frame}"
    );
    assert!(
        frame.contains("Scripted location of the scene headings"),
        "{frame}"
    );
}

#[test]
fn opening_and_looking_without_an_answer_changes_no_file() {
    let scratch = bare(BOOK);
    settle(&scratch);
    let before = snapshot(&scratch.dir);

    let mut app = scratch.app();
    let _ = screen(&mut app, 100, 24);
    ctrl(&mut app, 'k');
    press(&mut app, KeyCode::Esc);
    ctrl(&mut app, 's');

    assert_eq!(
        changed(&before, &snapshot(&scratch.dir)),
        Vec::<String>::new()
    );
}

#[test]
fn an_answer_writes_only_the_places() {
    let scratch = bare(BOOK);
    settle(&scratch);
    let mut app = scratch.app();
    let _ = screen(&mut app, 100, 24);
    let before = snapshot(&scratch.dir);
    ctrl(&mut app, 'k');

    press(&mut app, KeyCode::Enter);

    assert_eq!(app.mode, Mode::Normal, "no question left");
    assert_eq!(app.book_note, "");
    assert!(orte(&scratch).contains("name = \"SÄNGERSAAL\""));
    assert_eq!(
        changed(&before, &snapshot(&scratch.dir)),
        ["orte.toml".to_owned()]
    );
    let again = scratch.app();
    assert!(again.place_question().is_none());
}

#[test]
fn an_answer_keeps_what_another_program_wrote_meanwhile() {
    let scratch = bare("INT. SÄNGERSAAL – TAG #12#\n\nA.\n\nINT. HOF – NACHT #13#\n\nB.\n\n");
    let mut app = scratch.app();
    let _ = screen(&mut app, 100, 24);
    fs::write(
        scratch.dir.join("orte.toml"),
        "[[ort]]\nid = \"xxxxxxxx\"\nname = \"GARTEN\"\n",
    )
    .unwrap_or_else(|e| unreachable!("{e}"));

    ctrl(&mut app, 'k');
    press(&mut app, KeyCode::Enter);

    let file = orte(&scratch);
    assert!(
        file.contains("GARTEN") && file.contains("SÄNGERSAAL"),
        "{file}"
    );
    assert!(app.place_question().is_some(), "HOF is still open");
}

#[test]
fn a_known_location_asks_nothing_and_its_file_stays() {
    let scratch = Scratch::new(BOOK);
    settle(&scratch);
    let before = snapshot(&scratch.dir);
    assert!(before.contains_key(Path::new("orte.toml")));

    let mut app = scratch.app();
    let _ = screen(&mut app, 100, 24);
    ctrl(&mut app, 'k');
    ctrl(&mut app, 's');

    assert_eq!(app.mode, Mode::Normal);
    assert_eq!(
        changed(&before, &snapshot(&scratch.dir)),
        Vec::<String>::new()
    );
}
