//! Loading a Fountain file into the screenplay: unsaved like any edit, refused whole when the
//! file is no UTF-8 text.
#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::{Path, PathBuf};

use klappe_services::{outline, Language, ServiceError, Session};

const FILE_TEXT: &str = "INT. HALLE - TAG\n\nSie kommt an.\n\nEXT. HOF - NACHT\n\nEs regnet.\n";

/// An empty project and a folder for the files to import, both fresh.
fn session(name: &str) -> (Session, PathBuf) {
    let base = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).unwrap();
    let session = Session::create(&base.join("projekt"), "Film", Language::German).unwrap();

    (session, base)
}

#[test]
fn a_file_replaces_the_script_and_stays_unsaved() {
    let (mut s, base) = session("script-import-ok");
    let file = base.join("buch.fountain");
    // A byte order mark, as some editors write it, is not part of the text.
    fs::write(&file, format!("\u{feff}{FILE_TEXT}")).unwrap();

    s.import_script(&file).unwrap();

    assert_eq!(s.script_text(), FILE_TEXT);
    assert!(s.is_dirty());
    assert_eq!(outline(s.script_text()).len(), 2);
    assert!(!base.join("projekt/drehbuch.fountain").exists());
}

#[test]
fn a_file_that_is_no_utf8_changes_nothing() {
    let (mut s, base) = session("script-import-bad");
    let file = base.join("alt.txt");
    fs::write(&file, [0x49, 0x4e, 0x54, 0x2e, 0x20, 0xe4, 0xff]).unwrap();

    let result = s.import_script(&file);

    assert!(matches!(result, Err(ServiceError::Store(_))), "{result:?}");
    assert_eq!(s.script_text(), "");
    assert!(!s.is_dirty());
}

#[test]
fn a_missing_file_is_an_error() {
    let (mut s, base) = session("script-import-missing");

    assert!(s.import_script(&base.join("fehlt.fountain")).is_err());
}

#[test]
fn line_ends_become_newlines() {
    let (mut s, base) = session("script-import-crlf");
    let file = base.join("windows.fountain");
    fs::write(&file, "INT. HALLE - TAG\r\n\r\nSie kommt an.\rEnde.\r\n").unwrap();

    s.import_script(&file).unwrap();

    assert_eq!(
        s.script_text(),
        "INT. HALLE - TAG\n\nSie kommt an.\nEnde.\n"
    );
}

const SAAL: &str = "INT. SAAL - TAG";
const THREE_SCENES: &str = "INT. SAAL - TAG\n\nAnna.\n\nEXT. HOF - NACHT\n\nEs regnet.\n\nINT. KELLER - NACHT\n\nEs tropft.\n";

/// A project that holds the scene `SAAL` and its book.
fn with_saal(name: &str) -> (Session, PathBuf) {
    let base = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).unwrap();
    let dir = base.join("projekt");
    Session::create(&dir, "Film", Language::German).unwrap();
    fs::write(dir.join("drehbuch.fountain"), format!("{SAAL}\n\nAnna.\n")).unwrap();
    let session = Session::open(&dir).unwrap();

    (session, base)
}

#[test]
fn the_register_follows_the_import_at_once() {
    let (mut s, base) = with_saal("script-import-register");
    let id = s.project().scenes[0].id.clone();
    let file = base.join("buch.fountain");
    fs::write(&file, THREE_SCENES).unwrap();

    let report = s.import_script(&file).unwrap();

    // Before any save: the matched scene keeps its entry, two are new.
    assert_eq!(s.project().scenes.len(), 3);
    assert_eq!(s.project().scenes[0].id, id);
    assert_eq!((report.scenes, report.created), (3, 2));
    assert_eq!((report.pending, report.not_in_book), (0, 0));
}

#[test]
fn the_import_is_one_undo_step() {
    let (mut s, base) = with_saal("script-import-undo");
    let before = s.script_text().to_owned();
    let file = base.join("buch.fountain");
    fs::write(&file, THREE_SCENES).unwrap();
    s.import_script(&file).unwrap();

    s.undo();

    assert_eq!(s.project().scenes.len(), 1);
    assert_eq!(s.script_text(), before);
}

#[test]
fn entries_the_book_lost_are_counted() {
    let (mut s, base) = with_saal("script-import-gone");
    let file = base.join("buch.fountain");
    fs::write(&file, "EXT. HOF - NACHT\n\nEs regnet.\n").unwrap();

    let report = s.import_script(&file).unwrap();

    assert_eq!((report.scenes, report.not_in_book), (1, 1));
}

#[test]
fn a_scene_the_import_cannot_place_is_a_question() {
    let base = Path::new(env!("CARGO_TARGET_TMPDIR")).join("script-import-ask");
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).unwrap();
    let dir = base.join("projekt");
    Session::create(&dir, "Film", Language::German).unwrap();
    let book = "INT. SAAL - TAG #1#\n\nAnna.\n\nEXT. HOF - NACHT #2#\n\nEs regnet.\n";
    fs::write(dir.join("drehbuch.fountain"), book).unwrap();
    let mut s = Session::open(&dir).unwrap();
    // Scene 2 is a different scene now but keeps its number.
    let file = base.join("buch.fountain");
    let rewritten = "INT. SAAL - TAG #1#\n\nAnna.\n\nINT. KELLER - NACHT #2#\n\nEs tropft.\n";
    fs::write(&file, rewritten).unwrap();

    let report = s.import_script(&file).unwrap();

    assert_eq!((report.scenes, report.created, report.pending), (2, 0, 1));
    assert_eq!(s.pending_scenes().len(), 1);
}
