//! Pages of a locked screenplay keep their numbers (DB-34, DB-40): added text runs over on
//! pages 12A, 12B; struck pages stay as one sheet; less text leaves a page short.
#![allow(clippy::unwrap_used)]

use std::path::Path;

use klappe_services::{
    HeaderLine, Language, OnOutside, Revisions, SceneNumbers, ScriptPrint, Session,
};

const PRINT: ScriptPrint = ScriptPrint {
    scene_numbers: SceneNumbers::Shown,
    header: HeaderLine::Off,
    language: Language::English,
    revisions: Revisions::Off,
};

/// Rows of action in a scene; one A4 page holds 59 rows.
const ROWS: usize = 40;

fn scene(number: usize, rows: usize) -> String {
    let lines: Vec<String> = (0..rows)
        .map(|i| format!("Scene {number} line {i}."))
        .collect();

    format!("INT. ROOM {number} - DAY\n\n{}\n\n", lines.join("\n"))
}

/// Scenes 1 to `count`, `ROWS` rows each: about two thirds of a page per scene.
fn book(count: usize) -> String {
    (1..=count).map(|n| scene(n, ROWS)).collect()
}

fn session(name: &str, text: &str) -> Session {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    let mut session = Session::create(&dir, "Nachtwache", Language::English).unwrap();
    session.set_script_text(text);

    session
}

fn lock(s: &mut Session) {
    s.lock_version("Drehfassung", Language::English, OnOutside::Ask)
        .unwrap();
}

/// Scene 9 of 70 rows put before scene 3: more than a page more on page 2.
fn grow(s: &mut Session) {
    let text = s.script_text().to_owned();
    let at = text.find("INT. ROOM 3").unwrap();

    s.set_script_text(&format!("{}{}{}", &text[..at], scene(9, 70), &text[at..]));
}

/// The text without the lines `from..to`.
fn without(s: &mut Session, from: usize, to: usize) {
    let text = s.script_text().to_owned();
    let lines: Vec<&str> = text.split('\n').collect();

    s.set_script_text(&[&lines[..from], &lines[to..]].concat().join("\n"));
}

/// The page labels of the layout just made, the first page included.
fn labels(s: &mut Session) -> Vec<String> {
    s.paginate(Language::English);
    let pages = s.script_pages().unwrap();
    let mut labels = vec!["1".to_owned()];
    labels.extend(pages.breaks.iter().map(|b| b.label.clone()));

    labels
}

/// The line of the text at which the page `label` starts.
fn line_of(s: &mut Session, label: &str) -> usize {
    s.paginate(Language::English);

    s.script_pages()
        .unwrap()
        .breaks
        .iter()
        .find(|b| b.label == label)
        .unwrap()
        .line
}

/// What the PDF prints on page `index` (from 0, no title page without title fields): the
/// texts of its rows.
fn printed(s: &mut Session, index: usize) -> Vec<String> {
    let pages = s.print_pages(&PRINT);

    pages[index].texts.iter().map(|t| t.text.clone()).collect()
}

#[test]
fn without_a_version_the_pages_are_numbered_in_a_row() {
    let mut s = session("locked-none", &book(5));

    assert_eq!(labels(&mut s), ["1", "2", "3", "4"]);
}

#[test]
fn locking_keeps_the_page_starts_in_the_list() {
    let mut s = session("locked-list", &book(5));

    lock(&mut s);

    let pages = s.versions()[0].pages.clone();
    let names: Vec<&str> = pages.iter().map(|p| p.label.as_str()).collect();
    assert_eq!(names, ["1", "2", "3", "4"]);
    assert_eq!(pages[0].line, 0);
    assert_eq!(pages[1].line, line_of(&mut s, "2"));
}

#[test]
fn added_text_runs_over_on_a_page_with_a_letter() {
    let mut s = session("locked-grow", &book(5));
    lock(&mut s);
    let page3 = line_of(&mut s, "3");

    grow(&mut s);

    let now = labels(&mut s);

    assert_eq!(now[..3], ["1", "2", "2A"]);
    assert!(now.contains(&"3".to_owned()), "{now:?}");
    assert!(now.len() > 4);
    // Page 3 still starts where it did, 73 lines on.
    assert_eq!(line_of(&mut s, "3"), page3 + 73);
}

#[test]
fn a_page_with_less_text_stays_short() {
    let mut s = session("locked-shrink", &book(5));
    lock(&mut s);
    let before = labels(&mut s);
    let page3 = line_of(&mut s, "3");

    // Three rows go from scene 1.
    let text = s.script_text().to_owned();
    let text = [
        "Scene 1 line 0.\n",
        "Scene 1 line 1.\n",
        "Scene 1 line 2.\n",
    ]
    .iter()
    .fold(text, |text, line| text.replacen(line, "", 1));
    s.set_script_text(&text);

    assert_eq!(labels(&mut s), before);
    assert_eq!(line_of(&mut s, "3"), page3 - 3);
}

#[test]
fn a_struck_page_stays_as_a_sheet() {
    let mut s = session("locked-strike", &book(8));
    lock(&mut s);
    let before = labels(&mut s);
    assert!(before.len() >= 5, "{before:?}");

    // Everything from the start of page 3 to the start of page 4 goes.
    let (from, to) = (line_of(&mut s, "3"), line_of(&mut s, "4"));
    without(&mut s, from, to);

    s.paginate(Language::English);
    let sheet = printed(&mut s, 2);
    assert!(sheet.iter().any(|t| t == "3 OMITTED"), "{sheet:?}");
    assert_eq!(labels(&mut s).len(), before.len() - 1);
    // The page after keeps its number.
    assert!(labels(&mut s).contains(&"4".to_owned()));
}

#[test]
fn the_struck_sheet_follows_the_project_language() {
    let mut s = session("locked-german", &book(8));
    s.set_script_setup(klappe_services::ScriptSetup {
        language: Some(klappe_services::ScriptLanguage::De),
        ..s.project().script
    });
    lock(&mut s);
    let (from, to) = (line_of(&mut s, "3"), line_of(&mut s, "4"));
    without(&mut s, from, to);

    s.paginate(Language::English);

    assert!(printed(&mut s, 2).iter().any(|t| t == "3 ENTFÄLLT"));
}

#[test]
fn the_pdf_pages_are_the_pages_of_the_editor() {
    let mut s = session("locked-pdf", &book(5));
    lock(&mut s);
    let page2 = line_of(&mut s, "2");
    grow(&mut s);

    let labels = labels(&mut s);
    let pages = s.print_pages(&PRINT);

    assert_eq!(pages.len(), labels.len());
    // From page 2 the label stands at the top right: `2A.`.
    for (page, label) in pages.iter().zip(&labels).skip(1) {
        let top = page.texts.iter().any(|t| t.text == format!("{label}."));
        assert!(top, "{label}");
    }
    assert_eq!(line_of(&mut s, "2"), page2);
}

#[test]
fn a_second_lock_and_a_reopened_project_keep_the_pages() {
    let mut s = session("locked-twice", &book(5));
    lock(&mut s);
    grow(&mut s);
    let grown = labels(&mut s);
    assert!(grown.contains(&"2A".to_owned()));

    s.lock_version("Nachdreh", Language::English, OnOutside::Ask)
        .unwrap();
    s.save(OnOutside::Ask).unwrap();

    // Later text on the second lock: the 2A page is kept.
    assert_eq!(labels(&mut s), grown);
    let mut again = Session::open(s.dir()).unwrap();
    assert_eq!(self::labels(&mut again), grown);
}

#[test]
fn a_version_without_pages_locks_none() {
    let mut s = session("locked-old", &book(5));
    lock(&mut s);
    let dir = s.dir().to_path_buf();
    let list = std::fs::read_to_string(dir.join("drehbuch/fassungen.toml")).unwrap();
    let head: Vec<&str> = list
        .lines()
        .take_while(|l| !l.starts_with("[[fassung.seiten]]"))
        .collect();
    std::fs::write(dir.join("drehbuch/fassungen.toml"), head.join("\n")).unwrap();

    let mut old = Session::open(&dir).unwrap();
    grow(&mut old);

    // Free pages: numbered in a row, no 2A.
    let now = labels(&mut old);
    let rows: Vec<String> = (1..=now.len()).map(|n| n.to_string()).collect();
    assert_eq!(now, rows);
}
