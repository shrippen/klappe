//! The figure rule of a change from outside: what answers added to `figuren.toml` is no
//! conflict, anything else both sides changed is.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_model::{Character, FigureColor, Project, Scene};
use klappe_services::{FigureChoice, OnOutside, ProjectEdit, Saved, ServiceError, Session};

const BOOK: &str =
    "INT. SAAL – TAG #1#\n\nAnna tritt ein.\n\nANNA\nHallo.\n\nBERT\nJa.\n\nCARL\nNa.\n\n";

fn character(id: &str, name: &str) -> Character {
    Character {
        id: id.into(),
        name: name.into(),
        short: "X".into(),
        color: FigureColor::Red,
        group: None,
        size_m: None,
        aliases: Vec::new(),
    }
}

/// A saved project with Anna and Bert, and a book that cues Carl too.
fn two(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    let mut project = Project::new("Test");
    project.characters = vec![character("anna", "Anna"), character("bert", "Bert")];
    project.scenes = vec![Scene {
        id: "aaaa".into(),
        number: "1".into(),
        heading: "INT. SAAL – TAG".into(),
        ..Scene::default()
    }];
    klappe_store::save(&dir, &project, Some(BOOK)).unwrap();

    dir
}

fn names(dir: &Path) -> Vec<String> {
    let project = klappe_store::load(dir).unwrap().project;

    project.characters.iter().map(|c| c.name.clone()).collect()
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

fn rename(id: &str, name: &str) -> ProjectEdit {
    ProjectEdit::EditCharacter {
        id: id.into(),
        name: name.into(),
        short: "X".into(),
        color: FigureColor::Red,
        group: None,
        size_m: None,
        aliases: None,
    }
}

#[test]
fn the_figure_writers_leave_nothing_for_a_check_to_find() {
    let dir = two("fig-writers-quiet");
    let mut tui = Session::open(&dir).unwrap();

    tui.resolve_figure("CARL", FigureChoice::Create).unwrap();
    assert_eq!(tui.check().unwrap(), Default::default());

    tui.undo();
    tui.save_answer().unwrap();
    assert_eq!(tui.check().unwrap(), Default::default());

    tui.redo();
    tui.save_answer().unwrap();
    assert_eq!(tui.check().unwrap(), Default::default());
    assert_eq!(names(&dir), ["Anna", "Bert", "Carl"]);
}

#[test]
fn an_answer_and_an_own_figure_change_need_no_question() {
    let dir = two("fig-rule-answer");
    let mut gui = Session::open(&dir).unwrap();
    let mut tui = Session::open(&dir).unwrap();
    gui.edit_project(add("Dora")).unwrap();
    gui.edit_project(rename("anna", "Annabelle")).unwrap();
    tui.resolve_figure("CARL", FigureChoice::Create).unwrap();

    let saved = gui.save(OnOutside::Ask).unwrap();

    assert_eq!(saved, Saved::Done);
    assert_eq!(names(&dir), ["Annabelle", "Bert", "Dora", "Carl"]);
}

#[test]
fn an_outside_rename_and_an_own_figure_change_are_a_conflict() {
    let dir = two("fig-rule-rename");
    let mut gui = Session::open(&dir).unwrap();
    gui.edit_project(add("Dora")).unwrap();
    let file = std::fs::read_to_string(dir.join("figuren.toml")).unwrap();
    std::fs::write(
        dir.join("figuren.toml"),
        file.replace("Anna", "Von draussen"),
    )
    .unwrap();

    let saved = gui.save(OnOutside::Ask).unwrap();

    assert_eq!(saved, Saved::Conflict);
    assert_eq!(gui.outside()[0].kind, klappe_store::FileKind::Figures);
    assert_eq!(names(&dir), ["Von draussen", "Bert"]);
}

#[test]
fn an_answer_alone_keeps_the_windows_undo() {
    let dir = two("fig-rule-undo");
    let mut gui = Session::open(&dir).unwrap();
    let mut tui = Session::open(&dir).unwrap();
    gui.add_plan("Plan B").ok();
    gui.edit_project(add("Dora")).unwrap();
    tui.resolve_figure("CARL", FigureChoice::Create).unwrap();

    let found = gui.check().unwrap();

    assert_eq!(found, Default::default());
    assert!(gui.can_undo());
    assert!(gui.project().characters.iter().any(|c| c.name == "Carl"));
}

#[test]
fn an_alias_for_a_figure_deleted_outside_is_refused_in_a_conflict() {
    let dir = two("fig-rule-held-alias");
    let mut gui = Session::open(&dir).unwrap();
    gui.edit_project(add("Dora")).unwrap();
    let mut outside = klappe_store::load(&dir).unwrap().project;
    outside.characters.retain(|c| c.id != "bert");
    klappe_store::save_characters(&dir, &outside).unwrap();

    // Twice: a refusal must not make the second try pass.
    for _ in 0..2 {
        let answered = gui.resolve_figure("CARL", FigureChoice::Assign("bert".into()));

        assert!(
            matches!(answered, Err(ServiceError::NoCharacter(_))),
            "{answered:?}"
        );
    }
}
