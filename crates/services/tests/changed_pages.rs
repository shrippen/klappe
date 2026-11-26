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
    // The pages with a mark, and 2B, which the text ran over onto and has no mark.
    assert!(
        marked.iter().all(|label| changed.contains(label)),
        "{changed:?}"
    );
    let want: Vec<Vec<String>> = ["2.", "2A.", "2B.", "4."]
        .iter()
        .map(|l| vec![(*l).to_owned()])
        .collect();
    assert_eq!(changed, want);
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

/// Every text of every page the print has.
fn texts(s: &mut Session, print: &ScriptPrint) -> Vec<Vec<String>> {
    s.print_pages(print)
        .iter()
        .map(|page| page.texts.iter().map(|t| t.text.clone()).collect())
        .collect()
}

fn has(page: &[String], text: &str) -> bool {
    page.iter().any(|t| t == text)
}

/// Text that runs over onto a new page 1A has no mark there, but the page is new: whoever
/// swaps page 1 needs it.
#[test]
fn a_new_page_without_a_mark_is_printed() {
    let mut s = session("changed-new-page", 4);
    lock(&mut s);
    let text = s.script_text().replace(
        "Scene 1 line 0.\n",
        &format!("Scene 1 line 0.\n{}", "More text.\n".repeat(6)),
    );
    s.set_script_text(&text);

    let all = texts(&mut s, &PRINT);
    let changed = texts(&mut s, &CHANGED);

    let carried = all.iter().find(|p| has(p, "Scene 2 line 8.")).unwrap();
    assert!(has(carried, "1A."), "{all:?}");
    assert!(!has(carried, "*"), "{all:?}");
    assert!(changed.iter().any(|p| has(p, "1A.")), "{changed:?}");
    assert!(changed.len() < all.len(), "{changed:?}");
}

/// A struck scene leaves a sheet `2-3 OMITTED` and a shorter page 1: both go to the ones who
/// hold the script.
#[test]
fn struck_pages_and_the_page_that_lost_text_are_printed() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("changed-struck");
    let _ = std::fs::remove_dir_all(&dir);
    let mut s = Session::create(&dir, "Nachtwache", Language::English).unwrap();
    s.set_script_text(&format!(
        "{}{}{}",
        scene(1, 40),
        scene(2, 150),
        scene(3, 40)
    ));
    lock(&mut s);
    let text = s.script_text().to_owned();
    let (from, to) = (
        text.find("INT. ROOM 2").unwrap(),
        text.find("INT. ROOM 3").unwrap(),
    );
    s.set_script_text(&format!("{}{}", &text[..from], &text[to..]));

    let all = texts(&mut s, &PRINT);
    let changed = texts(&mut s, &CHANGED);

    assert!(
        all.iter().any(|p| p.iter().any(|t| t.contains("OMITTED"))),
        "{all:?}"
    );
    assert!(
        changed
            .iter()
            .any(|p| p.iter().any(|t| t.contains("OMITTED"))),
        "{changed:?}"
    );
    // Page 1 lost the start of scene 2 and has no label of its own.
    assert!(
        changed.iter().any(|p| has(p, "Scene 1 line 3.")),
        "{changed:?}"
    );
    assert!(changed.len() < all.len(), "{changed:?}");
}

/// Only the changed-pages print refuses an empty result: a book of a title page prints.
#[test]
fn a_book_of_a_title_page_still_prints_without_revisions() {
    let mut s = session("changed-title-only", 1);
    s.set_script_text("Title: Nachtwache\nAuthor: X\n");
    let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join("changed-title-only.pdf");
    let off = ScriptPrint {
        revisions: Revisions::Off,
        ..PRINT
    };

    s.export_script_pdf(&path, &off).unwrap();

    assert!(path.exists());
}

/// Paragraphs of `name`'s scene, one row apart: a scene is about one and a half pages.
fn place(name: &str) -> String {
    let paragraphs: Vec<String> = (0..40).map(|i| format!("{name} line {i}.")).collect();

    format!("INT. {name} - DAY\n\n{}\n\n", paragraphs.join("\n\n"))
}

/// A moved scene changes no line, but the pages it left and came to have other text.
#[test]
fn a_moved_scene_prints_the_pages_it_left_and_came_to() {
    let names = ["OFFICE", "HARBOUR", "STREET", "TOWER", "ROOF"];
    let mut s = session("changed-moved", 1);
    s.set_script_text(&names.map(place).concat());
    lock(&mut s);
    let text = s.script_text().to_owned();
    let (harbour, street, tower, roof) = (
        text.find("INT. HARBOUR").unwrap(),
        text.find("INT. STREET").unwrap(),
        text.find("INT. TOWER").unwrap(),
        text.find("INT. ROOF").unwrap(),
    );
    s.set_script_text(&format!(
        "{}{}{}{}{}",
        &text[..harbour],
        &text[tower..roof],
        &text[harbour..tower],
        &text[street..street],
        &text[roof..]
    ));

    let changed = labels(&mut s, &CHANGED);

    let want: Vec<Vec<String>> = ["2.", "2A.", "2B.", "5.", "6."]
        .iter()
        .map(|l| vec![(*l).to_owned()])
        .collect();
    assert_eq!(changed, want);
}

/// A version locked before the pages were kept has none to compare with.
#[test]
fn a_version_without_pages_cannot_print_changed_pages() {
    let mut s = session("changed-no-pages", 3);
    lock(&mut s);
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("changed-no-pages");
    let file = dir.join("drehbuch/fassungen.toml");
    let list = std::fs::read_to_string(&file).unwrap();
    let cut = list.find("[[fassung.seiten]]").unwrap();
    std::fs::write(&file, &list[..cut]).unwrap();
    let mut s = Session::open(&dir).unwrap();
    s.set_script_text(
        &s.script_text()
            .replace("Scene 3 line 4.", "Scene 3 changed."),
    );
    let path = dir.join("old.pdf");

    let result = s.export_script_pdf(&path, &CHANGED);

    assert!(matches!(result, Err(ServiceError::NoLockedPages)));
    assert!(!path.exists());
}
