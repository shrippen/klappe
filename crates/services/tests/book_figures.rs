//! The characters meet the book (DB-50): a name of the book no character answers to is asked
//! about, never made a character on its own; answers are remembered.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_model::{Character, FigureColor, Project, Scene};
use klappe_services::{FigureChoice, ProjectEdit, ServiceError, Session};

const BOOK: &str =
    "INT. SAAL – TAG #1#\n\nAnna tritt ein.\n\nANNA\nHallo.\n\nALLE\nHallo, Anna!\n\n\
ANNA (V.O.)\nLange her.\n\nBERT ^\nJa.\n\nANNA (CONT'D)\nNoch einmal.\n\n";

fn character(id: &str, name: &str, short: &str, color: FigureColor) -> Character {
    Character {
        id: id.into(),
        name: name.into(),
        short: short.into(),
        color,
        group: None,
        size_m: None,
        aliases: Vec::new(),
    }
}

/// A saved project with these characters and this book.
fn make(name: &str, characters: Vec<Character>, book: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    let mut project = Project::new("Test");
    project.characters = characters;
    project.scenes = vec![Scene {
        id: "aaaa".into(),
        number: "1".into(),
        heading: "INT. SAAL – TAG".into(),
        ..Scene::default()
    }];
    klappe_store::save(&dir, &project, Some(book)).unwrap();

    dir
}

fn asked(s: &Session) -> Vec<(String, usize)> {
    s.pending_figures()
        .iter()
        .map(|p| (p.name.clone(), p.cues))
        .collect()
}

#[test]
fn unknown_names_are_asked_once_without_extension_or_dual_mark() {
    let dir = make("fig-asked", Vec::new(), BOOK);

    let s = Session::open(&dir).unwrap();

    assert_eq!(
        asked(&s),
        [
            ("ANNA".to_owned(), 3),
            ("ALLE".to_owned(), 1),
            ("BERT".to_owned(), 1)
        ]
    );
    assert!(s.project().characters.is_empty(), "no figure on its own");
    assert_eq!(s.book_report().figures, 3);
}

#[test]
fn a_name_a_character_has_is_not_asked_in_any_case() {
    let anna = character("anna", "Anna", "A", FigureColor::Red);
    let dir = make("fig-known", vec![anna], BOOK);

    let s = Session::open(&dir).unwrap();

    assert_eq!(asked(&s), [("ALLE".to_owned(), 1), ("BERT".to_owned(), 1)]);
}

#[test]
fn creating_makes_a_figure_with_short_and_colour_and_is_one_undo_step() {
    let anna = character("anna", "Anna", "A", FigureColor::Red);
    let dir = make("fig-create", vec![anna], BOOK);
    let mut s = Session::open(&dir).unwrap();

    s.resolve_figure("BERT", FigureChoice::Create).unwrap();

    let bert = s.project().characters.last().unwrap();
    assert_eq!((bert.name.as_str(), bert.short.as_str()), ("Bert", "B"));
    assert_ne!(bert.color, FigureColor::Red, "the first colour taken");
    assert_eq!(asked(&s), [("ALLE".to_owned(), 1)]);

    s.undo();

    assert_eq!(s.project().characters.len(), 1);
    assert_eq!(s.pending_figures().len(), 2, "asked again after undo");
}

#[test]
fn an_assignment_is_an_alias_that_the_next_session_still_knows() {
    let anna = character("anna", "Anna", "A", FigureColor::Red);
    let dir = make("fig-assign", vec![anna], BOOK);
    let mut s = Session::open(&dir).unwrap();

    s.resolve_figure("BERT", FigureChoice::Assign("anna".into()))
        .unwrap();
    s.save_register().unwrap();

    assert_eq!(s.project().characters[0].aliases, ["BERT"]);
    let again = Session::open(&dir).unwrap();
    assert_eq!(asked(&again), [("ALLE".to_owned(), 1)]);
    assert_eq!(again.project().characters[0].aliases, ["BERT"]);
}

#[test]
fn not_a_figure_is_remembered_and_makes_no_character() {
    let dir = make("fig-none", Vec::new(), BOOK);
    let mut s = Session::open(&dir).unwrap();

    s.resolve_figure("ALLE", FigureChoice::NotFigure).unwrap();
    s.save_register().unwrap();

    let again = Session::open(&dir).unwrap();
    assert_eq!(asked(&again).len(), 2);
    assert!(again.project().characters.is_empty());
    assert_eq!(again.project().not_figures, ["ALLE"]);
}

#[test]
fn a_rename_in_the_book_offers_the_figure_without_a_cue_first() {
    let anna = character("anna", "Anna", "A", FigureColor::Red);
    let bert = character("bert", "Bert", "B", FigureColor::Blue);
    let dir = make(
        "fig-renamed",
        vec![anna, bert],
        "INT. SAAL – TAG #1#\n\nBERT\nHallo.\n\nHANNAH\nHallo.\n\n",
    );
    let s = Session::open(&dir).unwrap();

    assert_eq!(asked(&s), [("HANNAH".to_owned(), 1)]);
    let lines = s.figure_answers("HANNAH");

    let choices: Vec<_> = lines.iter().map(|l| l.choice.clone()).collect();
    assert_eq!(
        choices,
        [
            FigureChoice::Create,
            FigureChoice::Assign("anna".into()),
            FigureChoice::Assign("bert".into()),
            FigureChoice::NotFigure
        ],
        "Anna has no cue left, so she comes before Bert"
    );
    assert_eq!(lines[1].name, "Anna");
}

#[test]
fn renaming_in_klappe_keeps_the_id_and_the_aliases() {
    let anna = character("anna", "Anna", "A", FigureColor::Red);
    let dir = make("fig-rename", vec![anna], BOOK);
    let mut s = Session::open(&dir).unwrap();
    s.resolve_figure("BERT", FigureChoice::Assign("anna".into()))
        .unwrap();

    s.edit_project(ProjectEdit::EditCharacter {
        id: "anna".into(),
        name: "Annabel".into(),
        short: "A".into(),
        color: FigureColor::Red,
        group: None,
        size_m: None,
    })
    .unwrap();

    let c = &s.project().characters[0];
    assert_eq!((c.id.as_str(), c.name.as_str()), ("anna", "Annabel"));
    assert_eq!(c.aliases, ["BERT"]);
    assert_eq!(
        asked(&s),
        [("ALLE".to_owned(), 1)],
        "the book's ANNA was rewritten"
    );
}

#[test]
fn an_answer_to_a_question_nobody_asked_is_refused() {
    let dir = make("fig-refused", Vec::new(), BOOK);
    let mut s = Session::open(&dir).unwrap();

    let result = s.resolve_figure("NIEMAND", FigureChoice::Create);

    assert!(matches!(result, Err(ServiceError::NoDecision(_))));
    assert!(s.figure_answers("NIEMAND").is_empty());
    assert!(s.answer_figure("ANNA", 99).is_err());
}
