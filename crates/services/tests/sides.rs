//! A character's sides: the scenes they speak in, as a PDF of their own (DB-62).
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_model::{Character, FigureColor, Project};
use klappe_services::{
    HeaderLine, Language, Revisions, SceneNumbers, ScriptPrint, ServiceError, Session,
};

const PRINT: ScriptPrint = ScriptPrint {
    scene_numbers: SceneNumbers::Shown,
    header: HeaderLine::Off,
    language: Language::English,
    revisions: Revisions::Off,
};

/// Scene 3 is spoken by ANNI, Anna's other name; in 4 Bert speaks as the second of a dual
/// dialogue.
const BOOK: &str = "INT. SAAL – TAG #1#\n\nANNA\nHallo.\n\nBERT\nJa.\n\n\
INT. KÜCHE – TAG #2#\n\nNur Handlung in der Küche.\n\n\
INT. HOF – TAG #3#\n\nANNI\nLange her.\n\n\
INT. KELLER – NACHT #3A#\n\nANNA\nZugleich.\n\nBERT ^\nGleichzeitig.\n\n";

fn character(id: &str, name: &str, aliases: &[&str]) -> Character {
    Character {
        id: id.into(),
        name: name.into(),
        short: name[..1].into(),
        color: FigureColor::Red,
        group: None,
        size_m: None,
        aliases: aliases.iter().map(|a| (*a).into()).collect(),
    }
}

/// A saved project with Anna (also ANNI), Bert and Carl, who never speaks.
fn open(name: &str, book: &str) -> Session {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    let mut project = Project::new("Test");
    project.characters = vec![
        character("anna", "Anna", &["ANNI"]),
        character("bert", "Bert", &[]),
        character("carl", "Carl", &[]),
    ];
    klappe_store::save(&dir, &project, Some(book)).unwrap();

    Session::open(&dir).unwrap()
}

fn sides(s: &mut Session, name: &str, figure: &str) -> Result<String, ServiceError> {
    let path: PathBuf = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("{name}.pdf"));
    let _ = std::fs::remove_file(&path);
    s.export_sides_pdf(&path, figure, &PRINT)?;

    Ok(String::from_utf8_lossy(&std::fs::read(&path).unwrap()).into_owned())
}

#[test]
fn only_the_scenes_the_figure_speaks_in_are_printed() {
    let mut s = open("sides-anna", BOOK);

    let pdf = sides(&mut s, "sides-anna", "Anna").unwrap();

    // Scene 1, scene 3 (by the alias) and 3A, each whole.
    assert!(pdf.contains("(Hallo.) Tj"), "{pdf}");
    assert!(pdf.contains("(Ja.) Tj"), "{pdf}");
    assert!(pdf.contains("(Lange her.) Tj"), "{pdf}");
    assert!(pdf.contains("(Zugleich.) Tj"), "{pdf}");
    assert!(!pdf.contains("Küche"), "{pdf}");
    // The numbers are the script's, not the place among the sides.
    for number in ["1", "3", "3A"] {
        assert_eq!(
            pdf.matches(&format!("({number}) Tj")).count(),
            2,
            "{number}"
        );
    }
    assert!(!pdf.contains("(2) Tj"), "{pdf}");
    // No title page; the page is the first of the sides.
    assert!(pdf.contains("/Count 1"), "{pdf}");
}

#[test]
fn the_name_of_the_alias_finds_the_character_and_heads_the_pages() {
    let mut s = open("sides-alias", BOOK);

    let pdf = sides(&mut s, "sides-alias", "anni").unwrap();

    assert!(pdf.contains("(Anna) Tj"), "{pdf}");
    assert!(pdf.contains("(Lange her.) Tj"), "{pdf}");
}

#[test]
fn the_second_of_a_dual_dialogue_speaks_too() {
    let mut s = open("sides-dual", BOOK);

    let pdf = sides(&mut s, "sides-dual", "Bert").unwrap();

    assert!(pdf.contains("(Gleichzeitig.) Tj"), "{pdf}");
    assert!(pdf.contains("(Hallo.) Tj"), "{pdf}");
    assert!(!pdf.contains("Lange her."), "{pdf}");
}

/// A heading without a number counts like the script's: its place in the book.
#[test]
fn an_unnumbered_scene_keeps_its_place_in_the_book() {
    let book = "INT. SAAL – TAG\n\nANNA\nHallo.\n\nINT. KÜCHE – TAG\n\nBERT\nJa.\n\n";
    let mut s = open("sides-ordinal", book);

    let pdf = sides(&mut s, "sides-ordinal", "Bert").unwrap();

    assert_eq!(pdf.matches("(2) Tj").count(), 2, "{pdf}");
    assert!(!pdf.contains("(1) Tj"), "{pdf}");
}

/// The pages are numbered from the sides' first, and break fresh.
#[test]
fn the_sides_have_their_own_page_numbers() {
    let lines: Vec<String> = (0..70).map(|i| format!("Line {i}.")).collect();
    let book = format!(
        "INT. SAAL – TAG #1#\n\nANNA\nHallo.\n\n{}\n\nINT. HOF – TAG #7#\n\nANNA\nNoch einmal.\n\n",
        lines.join("\n")
    );
    let mut s = open("sides-pages", &book);

    let pdf = sides(&mut s, "sides-pages", "Anna").unwrap();

    assert!(pdf.contains("/Count 2"), "{pdf}");
    assert!(pdf.contains("(2.) Tj"), "{pdf}");
}

#[test]
fn a_cue_without_a_character_has_sides_too() {
    let book = "INT. SAAL – TAG #1#\n\nALLE\nHallo.\n\nINT. HOF – TAG #2#\n\nBERT\nJa.\n\n";
    let mut s = open("sides-cue", book);

    let pdf = sides(&mut s, "sides-cue", "alle").unwrap();

    assert!(pdf.contains("(Hallo.) Tj"), "{pdf}");
    assert!(!pdf.contains("(Ja.) Tj"), "{pdf}");
}

#[test]
fn an_unknown_figure_or_one_without_dialogue_is_refused() {
    let mut s = open("sides-refused", BOOK);

    let unknown = sides(&mut s, "sides-refused", "Dora");
    let silent = sides(&mut s, "sides-refused", "Carl");

    assert!(matches!(unknown, Err(ServiceError::NoCharacter(name)) if name == "Dora"));
    assert!(matches!(silent, Err(ServiceError::NoDialogue(name)) if name == "Carl"));
    assert!(!Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("sides-refused.pdf")
        .exists());
}

#[test]
fn a_project_without_a_script_has_no_sides() {
    let mut s = open("sides-empty", "");

    let result = sides(&mut s, "sides-empty", "Anna");

    assert!(matches!(result, Err(ServiceError::NoScript)));
}

/// What stands before the first heading is a unit of its own, and a book without a heading
/// is one.
#[test]
fn dialogue_before_the_first_heading_counts() {
    let book = "ANNA\nHallo.\n\nINT. HOF – TAG\n\nBERT\nJa.\n\n";
    let mut s = open("sides-preamble", book);

    let pdf = sides(&mut s, "sides-preamble", "Anna").unwrap();

    assert!(pdf.contains("(Hallo.) Tj"), "{pdf}");
    assert!(!pdf.contains("(Ja.) Tj"), "{pdf}");
    assert!(!pdf.contains("(1) Tj"), "{pdf}");

    let mut s = open("sides-no-heading", "ANNA\nHallo.\n");
    let pdf = sides(&mut s, "sides-no-heading", "Anna").unwrap();

    assert!(pdf.contains("(Hallo.) Tj"), "{pdf}");
}
