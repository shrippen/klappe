//! The character manager changes the other names of a character and the cues that are no
//! character (DB-50): a name belongs to one character or to none, and a name taken away from
//! the book's cues is asked about again.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_model::{Character, FigureColor, Project, Scene};
use klappe_services::{
    ChangeKind, Field, OnOutside, ProjectEdit, ScriptChange, ServiceError, Session,
};

const BOOK: &str = "INT. SAAL – TAG #1#\n\nANNA\nHallo.\n\nALLE\nHallo, Anna!\n\n\
ANNI\nLange her.\n\nBERT\nJa.\n\nANNI (CONT'D)\nNoch einmal.\n\n";

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

/// A saved project: Anna (also called ANNI) and Bert, `ALLE` no character.
fn make(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    let mut project = Project::new("Test");
    project.characters = vec![
        character("anna", "Anna", &["ANNI"]),
        character("bert", "Bert", &[]),
    ];
    project.not_figures = vec!["ALLE".into()];
    project.scenes = vec![Scene {
        id: "aaaa".into(),
        number: "1".into(),
        heading: "INT. SAAL – TAG".into(),
        ..Scene::default()
    }];
    klappe_store::save(&dir, &project, Some(BOOK)).unwrap();

    dir
}

fn asked(s: &Session) -> Vec<String> {
    s.pending_figures().iter().map(|p| p.name.clone()).collect()
}

fn edit(name: &str, aliases: &[&str]) -> ProjectEdit {
    ProjectEdit::EditCharacter {
        id: "anna".into(),
        name: name.into(),
        short: "A".into(),
        color: FigureColor::Red,
        group: None,
        size_m: None,
        aliases: Some(aliases.iter().map(|a| (*a).into()).collect()),
    }
}

fn aliases(s: &Session) -> Vec<String> {
    s.project().character("anna").unwrap().aliases.clone()
}

fn unknown(from: &str, count: usize) -> ScriptChange {
    ScriptChange {
        kind: ChangeKind::Unknown,
        from: from.into(),
        to: String::new(),
        count,
    }
}

#[test]
fn the_form_sets_the_other_names_trimmed_and_the_next_session_knows_them() {
    let dir = make("names-set");
    let mut s = Session::open(&dir).unwrap();

    s.edit_project(edit("Anna", &[" ANNI ", "Frau A"])).unwrap();
    s.save(OnOutside::Ask).unwrap();

    assert_eq!(aliases(&s), ["ANNI", "Frau A"]);
    let again = Session::open(&dir).unwrap();
    assert_eq!(aliases(&again), ["ANNI", "Frau A"]);
    assert!(asked(&again).is_empty());
}

#[test]
fn an_edit_without_names_keeps_them() {
    let mut s = Session::open(&make("names-kept")).unwrap();

    let mut keep = edit("Anna", &[]);
    if let ProjectEdit::EditCharacter { aliases, .. } = &mut keep {
        *aliases = None;
    }
    s.edit_project(keep).unwrap();

    assert_eq!(aliases(&s), ["ANNI"]);
}

#[test]
fn a_removed_name_is_asked_again_and_undo_brings_it_back() {
    let mut s = Session::open(&make("names-removed")).unwrap();

    s.edit_project(edit("Anna", &[])).unwrap();
    assert_eq!(asked(&s), ["ANNI"]);

    s.undo();
    assert_eq!(aliases(&s), ["ANNI"]);
    assert!(asked(&s).is_empty());
}

#[test]
fn a_name_taken_elsewhere_is_refused_as_another_name() {
    let mut s = Session::open(&make("names-taken")).unwrap();

    // Bert's name, the character's own name, a mark "no character", twice, empty.
    for list in [
        &["BERT"][..],
        &["anna"],
        &["Alle"],
        &["Frau A", "FRAU A"],
        &[" "],
    ] {
        let err = s.edit_project(edit("Anna", list)).unwrap_err();
        assert!(
            matches!(
                err,
                ServiceError::BadValue {
                    field: Field::Alias,
                    ..
                }
            ),
            "{list:?}: {err:?}"
        );
    }
    assert_eq!(aliases(&s), ["ANNI"], "nothing changed");
}

#[test]
fn the_preview_names_the_cues_a_removed_name_leaves_without_a_character() {
    let s = Session::open(&make("names-preview")).unwrap();

    let none: Vec<String> = Vec::new();
    let changes = s.character_script_changes("anna", "Anna", Some(&none));

    assert_eq!(changes, [unknown("ANNI", 2)]);
    assert!(s.character_script_changes("anna", "Anna", None).is_empty());
}

#[test]
fn the_preview_of_a_rename_does_not_count_the_renamed_cues_as_lost() {
    let s = Session::open(&make("names-rename")).unwrap();

    let changes = s.character_script_changes("anna", "Hanna", None);

    assert_eq!(changes.len(), 1, "{changes:?}");
    assert_eq!(changes[0].kind, ChangeKind::Cue);
    assert_eq!(
        (changes[0].from.as_str(), changes[0].to.as_str()),
        ("ANNA", "HANNA")
    );
}

#[test]
fn a_mark_no_character_is_added_and_taken_away() {
    let mut s = Session::open(&make("names-none")).unwrap();
    s.edit_project(edit("Anna", &[])).unwrap();

    s.edit_project(ProjectEdit::AddNotFigure {
        name: " Anni ".into(),
    })
    .unwrap();
    assert_eq!(s.project().not_figures, ["ALLE", "Anni"]);
    assert!(asked(&s).is_empty());

    assert_eq!(s.not_figure_changes("alle"), [unknown("ALLE", 1)]);
    s.edit_project(ProjectEdit::RemoveNotFigure {
        name: "alle".into(),
    })
    .unwrap();
    assert_eq!(s.project().not_figures, ["Anni"]);
    assert_eq!(asked(&s), ["ALLE"]);
}

#[test]
fn a_mark_no_character_is_refused_for_a_name_that_is_taken() {
    let mut s = Session::open(&make("names-none-taken")).unwrap();

    for name in ["bert", "ANNI", "Alle", " "] {
        let err = s
            .edit_project(ProjectEdit::AddNotFigure { name: name.into() })
            .unwrap_err();
        assert!(
            matches!(
                err,
                ServiceError::BadValue {
                    field: Field::Name,
                    ..
                }
            ),
            "{name}: {err:?}"
        );
    }
    assert_eq!(s.project().not_figures, ["ALLE"]);
}

#[test]
fn the_overview_carries_the_other_names_and_the_marks() {
    let s = Session::open(&make("names-overview")).unwrap();

    let overview = s.overview();

    assert_eq!(overview.characters[0].aliases, ["ANNI"]);
    assert_eq!(overview.not_figures, ["ALLE"]);
}

#[test]
fn a_new_or_renamed_character_cannot_take_a_name_marked_no_character() {
    let mut s = Session::open(&make("names-marked")).unwrap();
    let add = ProjectEdit::AddCharacter {
        name: "Alle".into(),
        short: "AL".into(),
        color: FigureColor::Blue,
        group: None,
        size_m: None,
    };

    for refused in [add, edit("Alle", &["ANNI"])] {
        let err = s.edit_project(refused).unwrap_err();
        assert!(
            matches!(
                err,
                ServiceError::BadValue {
                    field: Field::Name,
                    ..
                }
            ),
            "{err:?}"
        );
    }
    assert_eq!(s.project().characters.len(), 2);
    assert_eq!(s.project().characters[0].name, "Anna");
}
