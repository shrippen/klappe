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
