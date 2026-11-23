//! Marking elements in a scene's text (D-23): a mark is a quote of words and their
//! occurrence; it survives a save, follows the text and says when the text lost it.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_model::{BookState, Project, Scene, TextMark};
use klappe_services::{
    BreakdownEdit, ElementCategory, Field, MarkTarget, OnOutside, ProjectEdit, ServiceError,
    Session,
};

const BOOK: &str = "INT. WACHRAUM – NACHT #14#\n\n\
Nebel. Ein Stapel Seiten liegt im Nebel.\n\n\
ANNA\nWo ist der Nebel?\n\n\
EXT. HAFEN – NACHT #23#\n\nNebel über dem Wasser.\n\n";

fn make(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    let mut project = Project::new("Test");
    let scene = |id: &str, number: &str, heading: &str| Scene {
        id: id.into(),
        number: number.into(),
        heading: heading.into(),
        state: BookState::InBook,
        ..Scene::default()
    };
    project.scenes = vec![
        scene("aaaa", "14", "INT. WACHRAUM – NACHT"),
        scene("bbbb", "23", "EXT. HAFEN – NACHT"),
    ];
    klappe_store::save(&dir, &project, Some(BOOK)).unwrap();

    dir
}

fn open(name: &str) -> (Session, PathBuf) {
    let dir = make(name);
    let mut s = Session::open(&dir).unwrap();
    s.select_scene("aaaa").unwrap();

    (s, dir)
}

fn marks(s: &Session) -> Vec<TextMark> {
    s.project().scene("aaaa").unwrap().sheet.marks.clone()
}

/// The text of each marked run of the sheet's text with its element.
fn marked(s: &Session) -> Vec<(String, String)> {
    let text = s.overview().sheet.unwrap().text;

    text.iter()
        .flat_map(|l| &l.runs)
        .filter_map(|r| Some((r.text.clone(), r.element.clone()?)))
        .collect()
}

#[test]
fn the_sheet_shows_the_scene_text_line_by_line() {
    let (s, _) = open("marks-text");
    let text = s.overview().sheet.unwrap().text;

    let lines: Vec<(&str, String)> = text
        .iter()
        .map(|l| (l.kind, l.runs.iter().map(|r| r.text.as_str()).collect()))
        .collect();
    assert_eq!(
        lines,
        [
            (
                "action",
                "Nebel. Ein Stapel Seiten liegt im Nebel.".to_owned()
            ),
            ("cue", "ANNA".to_owned()),
            ("dialogue", "Wo ist der Nebel?".to_owned()),
        ]
    );
}

/// Marking as a new element makes it, puts it in the scene and keeps the quote; one undo
/// step takes all of it back.
#[test]
fn marked_words_become_an_element_of_the_scene() {
    let (mut s, _) = open("marks-new");

    s.mark_text(
        "aaaa",
        (0, "Nebel. Ein ", "Stapel Seiten"),
        MarkTarget::New(ElementCategory::Prop),
    )
    .unwrap();

    let element = s.project().elements[0].clone();
    assert_eq!(element.name, "Stapel Seiten");
    assert_eq!(element.category, ElementCategory::Prop);
    assert_eq!(
        s.project().scene("aaaa").unwrap().sheet.elements,
        std::slice::from_ref(&element.id)
    );
    assert_eq!(
        marks(&s),
        [TextMark {
            element: element.id.clone(),
            text: "Stapel Seiten".into(),
            occurrence: 0,
        }]
    );
    assert_eq!(marked(&s), [("Stapel Seiten".to_owned(), element.id)]);
    assert_eq!(s.undo_step(), Some("MarkText"));

    s.undo();
    assert!(s.project().elements.is_empty());
    assert!(marks(&s).is_empty());
}

/// The second „Nebel“ of the line is marked, not the first; an existing element is taken
/// by id, one of the same name and category by name.
#[test]
fn a_mark_is_the_occurrence_that_was_selected() {
    let (mut s, _) = open("marks-occurrence");
    let fog = s.add_element(ElementCategory::Sfx, "Nebel").unwrap();

    s.mark_text(
        "aaaa",
        (0, "Nebel. Ein Stapel Seiten liegt im ", "Nebel"),
        MarkTarget::Element(fog.clone()),
    )
    .unwrap();
    s.mark_text(
        "aaaa",
        (2, "Wo ist der ", "Nebel"),
        MarkTarget::New(ElementCategory::Sfx),
    )
    .unwrap();

    assert_eq!(s.project().elements.len(), 1);
    let occurrences: Vec<usize> = marks(&s).iter().map(|m| m.occurrence).collect();
    assert_eq!(occurrences, [1, 2]);
    let text = s.overview().sheet.unwrap().text;
    let runs: Vec<&str> = text[0].runs.iter().map(|r| r.text.as_str()).collect();
    assert_eq!(runs, ["Nebel. Ein Stapel Seiten liegt im ", "Nebel", "."]);
    assert_eq!(text[0].runs[1].category, Some("sfx"));
}

#[test]
fn words_that_do_not_stand_there_are_refused() {
    let (mut s, _) = open("marks-refused");
    let wrong = |s: &mut Session, line, prefix, text| {
        s.mark_text(
            "aaaa",
            (line, prefix, text),
            MarkTarget::New(ElementCategory::Prop),
        )
    };

    for (line, prefix, text) in [(0, "Nebel. ", "Stapel"), (0, "", " "), (9, "", "Nebel")] {
        assert!(matches!(
            wrong(&mut s, line, prefix, text),
            Err(ServiceError::BadValue {
                field: Field::Mark,
                ..
            })
        ));
    }
    assert!(matches!(
        s.mark_text(
            "weg",
            (0, "", "Nebel"),
            MarkTarget::New(ElementCategory::Prop)
        ),
        Err(ServiceError::NoScene(_))
    ));
    assert!(s.project().elements.is_empty());
}

#[test]
fn a_mark_survives_a_save() {
    let (mut s, dir) = open("marks-save");
    s.mark_text(
        "aaaa",
        (0, "Nebel. Ein ", "Stapel Seiten"),
        MarkTarget::New(ElementCategory::Prop),
    )
    .unwrap();
    s.save(OnOutside::Ask).unwrap();

    let again = Session::open(&dir).unwrap();
    assert_eq!(
        again.project().scene("aaaa").unwrap().sheet.marks,
        marks(&s)
    );
}

/// Rewritten words leave the element in the scene with a hint; forgetting the lost mark
/// drops the hint, not the element. A mark found again shows no hint.
#[test]
fn a_lost_mark_keeps_the_element_and_says_so() {
    let (mut s, _) = open("marks-lost");
    s.mark_text(
        "aaaa",
        (0, "Nebel. Ein ", "Stapel Seiten"),
        MarkTarget::New(ElementCategory::Prop),
    )
    .unwrap();
    let id = s.project().elements[0].id.clone();
    let lost = |s: &Session| s.overview().sheet.unwrap().categories[0].elements[0].lost;
    assert!(!lost(&s));

    let text = s.script_text().replace("Stapel Seiten", "Blätterhaufen");
    s.set_script_text(&text);
    assert!(lost(&s));
    assert!(marked(&s).is_empty());

    s.forget_lost("aaaa", &id).unwrap();
    assert_eq!(s.undo_step(), Some("ForgetMarks"));
    assert!(marks(&s).is_empty());
    assert!(!lost(&s));
    assert_eq!(s.project().scene("aaaa").unwrap().sheet.elements, [id]);
}

/// Taking the element out of the scene, turning it down or removing it drops its marks.
#[test]
fn an_element_leaving_the_scene_takes_its_marks() {
    let (mut s, _) = open("marks-leave");
    let mark = |s: &mut Session| {
        s.mark_text(
            "aaaa",
            (1, "", "ANNA"),
            MarkTarget::New(ElementCategory::Costume),
        )
        .unwrap();
        s.project().elements[0].id.clone()
    };
    let id = mark(&mut s);

    let edits = [
        BreakdownEdit::RemoveSceneElement {
            scene: "aaaa".into(),
            element: id.clone(),
        },
        BreakdownEdit::ExcludeSuggestion {
            scene: "aaaa".into(),
            element: id.clone(),
        },
        BreakdownEdit::RemoveElement { id: id.clone() },
    ];
    for edit in edits {
        mark(&mut s);
        s.edit_project(ProjectEdit::Breakdown(edit)).unwrap();
        assert!(marks(&s).is_empty());
    }
}

/// An element whose name stands in the text is suggested, the case aside, but not one that
/// is only part of a word, one the sheet lists or one it turned down.
#[test]
fn an_element_named_in_the_text_is_suggested() {
    let (mut s, _) = open("marks-named");
    let fog = s.add_element(ElementCategory::Sfx, "nebel").unwrap();
    s.add_element(ElementCategory::Prop, "Stapel").unwrap();
    s.add_element(ElementCategory::Prop, "Seite").unwrap();

    let found = s.overview().sheet.unwrap().suggestions;
    let names: Vec<&str> = found.iter().map(|f| f.name.as_str()).collect();
    assert_eq!(names, ["nebel", "Stapel"]);
    assert_eq!(found[0].object, None);

    s.edit_project(ProjectEdit::Breakdown(BreakdownEdit::ExcludeSuggestion {
        scene: "aaaa".into(),
        element: fog,
    }))
    .unwrap();
    let found = s.overview().sheet.unwrap().suggestions;
    assert_eq!(found.len(), 1);
}
