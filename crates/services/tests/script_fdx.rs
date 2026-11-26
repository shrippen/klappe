//! The screenplay as a Final Draft file: written and read back (DB-72).
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_services::{fdx_loss_text, Language, ServiceError, Session};

const BOOK: &str = "Title: Nachtwache\n\nINT. SÄNGERSAAL - TAG #1#\n\nLicht fällt *durch* die Fenster. [[Regie]]\n\n# Akt\n\nELISABETH\nDer Weg war vereist.\n";

fn session(name: &str, text: &str) -> Session {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    let mut session = Session::create(&dir, "Nachtwache", Language::English).unwrap();
    session.set_script_text(text);

    session
}

fn out(name: &str) -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join(name)
}

#[test]
fn the_book_goes_out_and_comes_back() {
    let path = out("fdx-roundtrip.fdx");
    let loss = session("fdx-out", BOOK).export_script_fdx(&path).unwrap();
    assert_eq!((loss.sections, loss.notes), (1, 1));

    let mut other = session("fdx-in", "");
    let report = other.import_script(&path).unwrap();

    assert_eq!(report.scenes, 1);
    let text = other.script_text();
    assert!(text.contains("Title: Nachtwache"), "{text}");
    assert!(text.contains("INT. SÄNGERSAAL - TAG #1#"), "{text}");
    assert!(text.contains("Licht fällt *durch* die Fenster."), "{text}");
    assert!(text.contains("ELISABETH\nDer Weg war vereist."), "{text}");
    assert!(!text.contains("Regie"), "{text}");
}

#[test]
fn what_is_left_out_is_worded() {
    let path = out("fdx-loss.fdx");
    let loss = session("fdx-loss", BOOK).export_script_fdx(&path).unwrap();

    assert_eq!(
        fdx_loss_text(Language::English, &loss),
        "Not in the FDX file: sections 1, notes 1"
    );
    assert_eq!(
        fdx_loss_text(Language::German, &loss),
        "Nicht in der FDX-Datei: Abschnitte 1, Notizen 1"
    );
    assert_eq!(fdx_loss_text(Language::German, &Default::default()), "");
}

#[test]
fn no_screenplay_writes_no_file() {
    let path = out("fdx-none.fdx");
    let _ = std::fs::remove_file(&path);

    let result = session("fdx-none", "").export_script_fdx(&path);

    assert!(matches!(result, Err(ServiceError::NoScript)));
    assert!(!path.exists());
}

#[test]
fn a_file_that_is_no_fdx_changes_nothing() {
    let path = out("fdx-bad.fdx");
    std::fs::write(&path, "INT. HOF - TAG\n\nText.\n").unwrap();
    let mut s = session("fdx-bad", BOOK);

    let result = s.import_script(&path);

    assert!(matches!(result, Err(ServiceError::BadFdx(_))));
    assert_eq!(s.script_text(), BOOK);
}

#[test]
fn the_extension_is_read_whatever_its_case() {
    let path = out("fdx-upper.FDX");
    session("fdx-up", BOOK).export_script_fdx(&path).unwrap();

    let mut s = session("fdx-upper", "");

    assert_eq!(s.import_script(&path).unwrap().scenes, 1);
}
