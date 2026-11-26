//! Only the pages with a revision mark are printed, under their locked numbers (DB-43).
#![allow(clippy::unwrap_used)]

use std::path::Path;

use klappe_services::{
    HeaderLine, Language, OnOutside, Revisions, SceneNumbers, ScriptPrint, ServiceError, Session,
};

const PRINT: ScriptPrint = ScriptPrint {
    scene_numbers: SceneNumbers::Hidden,
    header: HeaderLine::Off,
    language: Language::English,
    revisions: Revisions::On,
};

const CHANGED: ScriptPrint = ScriptPrint {
    revisions: Revisions::ChangedPages,
    ..PRINT
};

/// Rows of action in a scene; one A4 page holds 59 rows.
const ROWS: usize = 40;

fn scene(number: usize, rows: usize) -> String {
    let lines: Vec<String> = (0..rows)
        .map(|i| format!("Scene {number} line {i}."))
        .collect();

    format!("INT. ROOM {number} - DAY\n\n{}\n\n", lines.join("\n"))
}

fn session(name: &str, count: usize) -> Session {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    let mut session = Session::create(&dir, "Nachtwache", Language::English).unwrap();
    session.set_script_text(&(1..=count).map(|n| scene(n, ROWS)).collect::<String>());

    session
}

fn lock(s: &mut Session) {
    s.lock_version("Drehfassung", Language::English, OnOutside::Ask)
        .unwrap();
}

/// The page numbers printed (`3.`, `3A.`) of the pages the print has, page 1 has none.
fn labels(s: &mut Session, print: &ScriptPrint) -> Vec<Vec<String>> {
    s.print_pages(print)
        .iter()
        .map(|page| {
            page.texts
                .iter()
                .map(|t| t.text.clone())
                .filter(|t| {
                    t.ends_with('.') && t.chars().next().is_some_and(|c| c.is_ascii_digit())
                })
                .filter(|t| t.len() <= 4)
                .collect()
        })
        .collect()
}

/// Scene 9 of 70 rows put before scene 3, so that page 2 runs over (2A) and one line changes.
fn change(s: &mut Session) {
    let text = s.script_text().to_owned();
    let at = text.find("INT. ROOM 3").unwrap();
    let text = format!("{}{}{}", &text[..at], scene(9, 70), &text[at..]);

    s.set_script_text(&text.replace("Scene 5 line 7.", "Scene 5 changed."));
}

#[test]
fn only_pages_with_a_mark_are_printed_under_their_numbers() {
    let mut s = session("changed-pages", 5);
    lock(&mut s);
    change(&mut s);

    let all = labels(&mut s, &PRINT);
    let marked: Vec<Vec<String>> = s
        .print_pages(&PRINT)
        .iter()
        .zip(&all)
        .filter(|(page, _)| page.texts.iter().any(|t| t.text == "*"))
        .map(|(_, label)| label.clone())
        .collect();
    let changed = labels(&mut s, &CHANGED);

    assert!(
        changed.len() >= 2 && changed.len() < all.len(),
        "{changed:?}"
    );
    assert_eq!(changed, marked);
    assert!(
        changed.iter().flatten().any(|l| l.ends_with("A.")),
        "{changed:?}"
    );
}

#[test]
fn the_pdf_has_the_changed_pages_and_the_revision_header() {
    let mut s = session("changed-pdf", 5);
    lock(&mut s);
    change(&mut s);
    let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join("changed-pdf.pdf");
    let count = s.print_pages(&CHANGED).len();

    s.export_script_pdf(&path, &CHANGED).unwrap();

    let pdf = String::from_utf8_lossy(&std::fs::read(&path).unwrap()).into_owned();
    assert!(pdf.contains(&format!("/Count {count}")), "{pdf}");
    assert!(pdf.contains("(Blue Revision"), "{pdf}");
    assert!(pdf.contains("Scene 5 changed."), "{pdf}");
    assert!(!pdf.contains("Scene 1 line 3."), "{pdf}");
}

#[test]
fn the_title_page_stays_out() {
    let mut s = session("changed-title", 5);
    s.set_script_text(&format!("Title: Nachtwache\n\n{}", s.script_text()));
    lock(&mut s);
    s.set_script_text(
        &s.script_text()
            .replace("Scene 5 line 4.", "Scene 5 changed."),
    );

    let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join("changed-title.pdf");
    s.export_script_pdf(&path, &CHANGED).unwrap();
    let pdf = String::from_utf8_lossy(&std::fs::read(&path).unwrap()).into_owned();

    assert!(!pdf.contains("(NACHTWACHE) Tj"), "{pdf}");
    assert!(pdf.contains("/Count 1"), "{pdf}");
}

#[test]
fn without_a_locked_version_nothing_is_written() {
    let mut s = session("changed-none", 2);
    let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join("changed-none.pdf");
    let _ = std::fs::remove_file(&path);

    let result = s.export_script_pdf(&path, &CHANGED);

    assert!(matches!(result, Err(ServiceError::NoVersions)));
    assert!(!path.exists());
}

#[test]
fn without_a_change_nothing_is_written() {
    let mut s = session("changed-same", 2);
    lock(&mut s);
    let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join("changed-same.pdf");
    let _ = std::fs::remove_file(&path);

    let result = s.export_script_pdf(&path, &CHANGED);

    assert!(matches!(result, Err(ServiceError::NoChangedPages)));
    assert!(!path.exists());
}
