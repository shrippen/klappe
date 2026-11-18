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

fn add(name: &str) -> ProjectEdit {
    ProjectEdit::AddCharacter {
        name: name.into(),
        short: "X".into(),
        color: FigureColor::Cyan,
        group: None,
        size_m: None,
    }
}

#[test]
fn a_name_is_free_against_names_and_aliases_of_other_figures() {
    let mut anna = character("anna", "Anna", "A", FigureColor::Red);
    anna.aliases = vec!["Anni".into()];
    let bert = character("bert", "Bert", "B", FigureColor::Blue);
    let dir = make("fig-free-name", vec![anna, bert], BOOK);
    let mut s = Session::open(&dir).unwrap();

    assert!(s.edit_project(add("ANNI")).is_err());
    assert!(s.edit_project(add("anna")).is_err());
    let rename = ProjectEdit::EditCharacter {
        id: "bert".into(),
        name: "anni".into(),
        short: "B".into(),
        color: FigureColor::Blue,
        group: None,
        size_m: None,
    };
    assert!(s.edit_project(rename).is_err());
    assert!(s.edit_project(add("Carl")).is_ok());
}

#[test]
fn extensions_after_the_name_are_no_part_of_it() {
    let book = "INT. SAAL – TAG #1#\n\nANNA (V.O.) (FORTS.)\nEins.\n\n\
BERT (O.S.) (V.O.) (FORTS.)\nZwei.\n\n";
    let dir = make("fig-extensions", Vec::new(), book);

    let s = Session::open(&dir).unwrap();

    assert_eq!(asked(&s), [("ANNA".to_owned(), 1), ("BERT".to_owned(), 1)]);
}

fn names(dir: &Path) -> Vec<String> {
    klappe_store::load(dir)
        .unwrap()
        .project
        .characters
        .iter()
        .map(|c| c.name.clone())
        .collect()
}

#[test]
fn an_answer_keeps_a_figure_the_window_saved_meanwhile() {
    let anna = character("anna", "Anna", "A", FigureColor::Red);
    let dir = make("fig-beside", vec![anna], BOOK);
    let mut gui = Session::open(&dir).unwrap();
    let mut tui = Session::open(&dir).unwrap();
    gui.edit_project(add("Carl")).unwrap();
    gui.save().unwrap();

    tui.resolve_figure("ALLE", FigureChoice::NotFigure).unwrap();
    tui.save_register().unwrap();

    assert_eq!(names(&dir), ["Anna", "Carl"]);
    assert_eq!(
        klappe_store::load(&dir).unwrap().project.not_figures,
        ["ALLE"]
    );
}

#[test]
fn an_alias_answer_keeps_a_figure_the_window_saved_meanwhile() {
    let anna = character("anna", "Anna", "A", FigureColor::Red);
    let dir = make("fig-beside-alias", vec![anna], BOOK);
    let mut gui = Session::open(&dir).unwrap();
    let mut tui = Session::open(&dir).unwrap();
    gui.edit_project(add("Carl")).unwrap();
    gui.save().unwrap();

    tui.resolve_figure("BERT", FigureChoice::Assign("anna".into()))
        .unwrap();

    let saved = klappe_store::load(&dir).unwrap().project;
    assert_eq!(names(&dir), ["Anna", "Carl"]);
    assert_eq!(saved.characters[0].aliases, ["BERT"]);
    assert_eq!(
        tui.project().characters.len(),
        2,
        "the session takes the disk"
    );
}

#[test]
fn a_question_answered_elsewhere_disappears_without_a_second_entry() {
    let dir = make("fig-stale", Vec::new(), BOOK);
    let mut first = Session::open(&dir).unwrap();
    let mut second = Session::open(&dir).unwrap();
    first
        .resolve_figure("ALLE", FigureChoice::NotFigure)
        .unwrap();

    second
        .resolve_figure("ALLE", FigureChoice::NotFigure)
        .unwrap();

    assert_eq!(
        klappe_store::load(&dir).unwrap().project.not_figures,
        ["ALLE"]
    );
    assert_eq!(second.project().not_figures, ["ALLE"]);
    assert!(second.pending_figures().iter().all(|p| p.name != "ALLE"));
}

#[test]
fn creating_a_figure_answered_elsewhere_makes_no_second_one() {
    let dir = make("fig-stale-create", Vec::new(), BOOK);
    let mut first = Session::open(&dir).unwrap();
    let mut second = Session::open(&dir).unwrap();
    first.resolve_figure("BERT", FigureChoice::Create).unwrap();

    second.resolve_figure("BERT", FigureChoice::Create).unwrap();

    assert_eq!(names(&dir), ["Bert"]);
}

fn aliases_of(dir: &Path, id: &str) -> Vec<String> {
    let project = klappe_store::load(dir).unwrap().project;

    project.character(id).unwrap().aliases.clone()
}

#[test]
fn undoing_an_answer_keeps_a_figure_the_window_saved_meanwhile() {
    let anna = character("anna", "Anna", "A", FigureColor::Red);
    let dir = make("fig-undo-beside", vec![anna], BOOK);
    let mut gui = Session::open(&dir).unwrap();
    let mut tui = Session::open(&dir).unwrap();
    gui.edit_project(add("Carl")).unwrap();
    gui.save().unwrap();
    tui.resolve_figure("ALLE", FigureChoice::NotFigure).unwrap();

    tui.undo();
    tui.save_answer().unwrap();

    assert_eq!(names(&dir), ["Anna", "Carl"]);
    assert!(klappe_store::load(&dir)
        .unwrap()
        .project
        .not_figures
        .is_empty());

    tui.redo();
    tui.save_answer().unwrap();

    assert_eq!(names(&dir), ["Anna", "Carl"]);
    assert_eq!(
        klappe_store::load(&dir).unwrap().project.not_figures,
        ["ALLE"]
    );
}

#[test]
fn undoing_an_alias_or_a_creation_takes_back_only_that_entry() {
    let anna = character("anna", "Anna", "A", FigureColor::Red);
    let dir = make("fig-undo-entry", vec![anna], BOOK);
    let mut gui = Session::open(&dir).unwrap();
    let mut tui = Session::open(&dir).unwrap();
    tui.resolve_figure("BERT", FigureChoice::Assign("anna".into()))
        .unwrap();
    tui.resolve_figure("ALLE", FigureChoice::Create).unwrap();
    gui.edit_project(add("Carl")).unwrap();
    gui.save().unwrap();

    tui.undo();
    tui.save_answer().unwrap();

    assert_eq!(names(&dir), ["Anna", "Carl"]);
    assert_eq!(aliases_of(&dir, "anna"), ["BERT"]);

    tui.undo();
    tui.save_answer().unwrap();

    assert_eq!(names(&dir), ["Anna", "Carl"]);
    assert!(aliases_of(&dir, "anna").is_empty());
}

#[test]
fn an_undo_that_is_no_figure_answer_leaves_the_figures_file_alone() {
    let anna = character("anna", "Anna", "A", FigureColor::Red);
    let dir = make("fig-undo-scene", vec![anna], BOOK);
    let mut tui = Session::open(&dir).unwrap();
    let mut gui = Session::open(&dir).unwrap();
    gui.edit_project(add("Carl")).unwrap();
    gui.save().unwrap();
    tui.edit_project(add("Dora")).unwrap();

    tui.undo();
    tui.save_answer().unwrap();

    assert_eq!(names(&dir), ["Anna", "Carl"]);
}

#[test]
fn a_save_keeps_a_figure_and_a_not_figure_another_program_wrote() {
    let anna = character("anna", "Anna", "A", FigureColor::Red);
    let dir = make("fig-save-merge", vec![anna], BOOK);
    let mut gui = Session::open(&dir).unwrap();
    let mut tui = Session::open(&dir).unwrap();
    tui.resolve_figure("ALLE", FigureChoice::NotFigure).unwrap();
    tui.resolve_figure("BERT", FigureChoice::Assign("anna".into()))
        .unwrap();

    gui.edit_project(add("Carl")).unwrap();
    gui.save().unwrap();

    assert_eq!(names(&dir), ["Anna", "Carl"]);
    assert_eq!(aliases_of(&dir, "anna"), ["BERT"]);
    assert_eq!(
        klappe_store::load(&dir).unwrap().project.not_figures,
        ["ALLE"]
    );
    assert!(gui.pending_figures().iter().all(|p| p.name != "ALLE"));
}

#[test]
fn a_save_still_deletes_and_renames_what_the_window_changed() {
    let anna = character("anna", "Anna", "A", FigureColor::Red);
    let bert = character("bert", "Bert", "B", FigureColor::Blue);
    let dir = make("fig-save-own", vec![anna, bert], BOOK);
    let mut gui = Session::open(&dir).unwrap();
    let mut tui = Session::open(&dir).unwrap();
    tui.resolve_figure("ALLE", FigureChoice::Create).unwrap();

    gui.edit_project(ProjectEdit::RemoveCharacter { id: "bert".into() })
        .unwrap();
    let rename = ProjectEdit::EditCharacter {
        id: "anna".into(),
        name: "Annabelle".into(),
        short: "A".into(),
        color: FigureColor::Red,
        group: None,
        size_m: None,
    };
    gui.edit_project(rename).unwrap();
    gui.save().unwrap();

    assert_eq!(names(&dir), ["Annabelle", "Alle"]);
}
