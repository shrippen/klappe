//! Which scene of the book a register entry is and where its heading stands, without scene
//! numbers: the scene list and the script column go by the alignment, not by the number.
#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::Path;

use klappe_services::{outline, Language, OutlineKind, Session};
use serde_json::json;

/// Two scenes without `#n#`: an imported book before it is numbered.
const UNNUMBERED: &str = "INT. HALLE - TAG\n\nSie kommt an.\n\nEXT. HOF - NACHT\n\nEs regnet.\n";

fn imported(name: &str, text: &str) -> Session {
    let base = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).unwrap();
    let mut s = Session::create(&base.join("projekt"), "Film", Language::German).unwrap();
    let file = base.join("buch.fountain");
    fs::write(&file, text).unwrap();
    s.import_script(&file).unwrap();

    s
}

fn line_of(s: &Session, index: usize) -> serde_json::Value {
    serde_json::to_value(s.overview()).unwrap()["scenes"][index]["line"].clone()
}

/// Every scene has the empty number: the row still names its own heading.
#[test]
fn unnumbered_scenes_have_their_own_heading_line() {
    let s = imported("scene-line-unnumbered", UNNUMBERED);

    assert_eq!(s.project().scenes[1].number, "");
    assert_eq!(line_of(&s, 0), json!(0));
    assert_eq!(line_of(&s, 1), json!(4));
}

/// The script column shows the text of the scene on view, not of the first without a number.
#[test]
fn the_script_column_follows_unnumbered_scenes() {
    let mut s = imported("scene-line-column", UNNUMBERED);
    let second = s.project().scenes[1].id.clone();

    s.select_scene(&second).unwrap();

    let lines = s.script_lines();
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].text, "Es regnet.");
}

/// A scene the book lost has no heading to go to.
#[test]
fn a_scene_not_in_the_book_has_no_line() {
    let mut s = imported("scene-line-gone", UNNUMBERED);
    s.set_script_text("INT. HALLE - TAG\n\nSie kommt an.\n");
    let base = Path::new(env!("CARGO_TARGET_TMPDIR")).join("scene-line-gone");
    let file = base.join("neu.fountain");
    fs::write(&file, "EXT. HOF - NACHT\n\nEs regnet.\n").unwrap();
    s.import_script(&file).unwrap();

    let value = serde_json::to_value(s.overview()).unwrap();
    let lines: Vec<_> = value["scenes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| (r["inBook"].clone(), r["line"].clone()))
        .collect();

    assert!(lines.contains(&(json!(true), json!(0))), "{lines:?}");
    assert!(lines.contains(&(json!(false), json!(null))), "{lines:?}");
}

/// The line is the one the outline gives, also under a title page.
#[test]
fn the_line_is_the_outlines_line_under_a_title_page() {
    let text = format!("Title: Film\n\n{UNNUMBERED}");
    let s = imported("scene-line-title", &text);

    let heads: Vec<usize> = outline(&text)
        .iter()
        .filter(|e| e.kind == OutlineKind::Scene)
        .map(|e| e.line)
        .collect();

    assert_eq!(heads.len(), 2);
    assert_eq!(line_of(&s, 0), json!(heads[0]));
    assert_eq!(line_of(&s, 1), json!(heads[1]));
}

fn lines(s: &Session) -> Vec<serde_json::Value> {
    let value = serde_json::to_value(s.overview()).unwrap();

    value["scenes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["line"].clone())
        .collect()
}

/// The kept mapping follows a changed text, an import, an undo and a redo.
#[test]
fn the_mapping_follows_text_import_undo_and_redo() {
    let mut s = imported("scene-line-follow", UNNUMBERED);
    assert_eq!(lines(&s), [json!(0), json!(4)]);

    // A blank line on top moves both headings down.
    s.set_script_text(&format!("\n{UNNUMBERED}"));
    assert_eq!(lines(&s), [json!(1), json!(5)]);

    // An import with a new first scene: three entries, in the order of the book.
    let base = Path::new(env!("CARGO_TARGET_TMPDIR")).join("scene-line-follow");
    let file = base.join("neu.fountain");
    fs::write(&file, format!("INT. NEU - TAG\n\nHallo.\n\n{UNNUMBERED}")).unwrap();
    s.import_script(&file).unwrap();
    assert_eq!(lines(&s), [json!(0), json!(4), json!(8)]);

    s.undo();
    assert_eq!(lines(&s), [json!(1), json!(5)]);
    let column = s.script_lines();
    assert!(column.iter().all(|l| l.text != "Hallo."));

    s.redo();
    assert_eq!(lines(&s), [json!(0), json!(4), json!(8)]);
}
