//! A character is renamed in the whole book (DB-04): every cue it answers to, its id and
//! everything on it stay.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_model::{Character, FigureColor, Project, Scene};
use klappe_services::{Field, OnOutside, ServiceError, Session};

const BOOK: &str = "INT. SAAL – TAG #1#\n\nANNA\nHallo.\n\nANNI\nLange her.\n\n\
BERT\nIch bin nicht Anna.\n\nANNA (V.O.)\nNoch einmal.\n\n@Anni ^\nUnd noch einmal.\n\n";

fn character(id: &str, name: &str, aliases: &[&str]) -> Character {
    Character {
        id: id.into(),
        name: name.into(),
        short: "AN".into(),
        color: FigureColor::Green,
        group: None,
        size_m: None,
        aliases: aliases.iter().map(|a| (*a).into()).collect(),
    }
}

/// A saved project: Anna (also called ANNI) and Bert.
fn make(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    let mut project = Project::new("Test");
    project.characters = vec![
        character("anna", "Anna", &["ANNI"]),
        character("bert", "Bert", &[]),
    ];
    project.scenes = vec![Scene {
        id: "aaaa".into(),
        number: "1".into(),
        heading: "INT. SAAL – TAG".into(),
        ..Scene::default()
    }];
    klappe_store::save(&dir, &project, Some(BOOK)).unwrap();

    dir
}

fn example(name: &str) -> PathBuf {
    let from = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe");
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    copy_dir(&from, &dir);

    dir
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
            continue;
        }
        std::fs::copy(entry.path(), target).unwrap();
    }
}

#[test]
fn every_cue_of_the_figure_is_renamed_and_nothing_else() {
    let mut s = Session::open(&make("rename-fig-cues")).unwrap();

    let count = s.rename_figure("Anna", "Eva").unwrap();

    assert_eq!(count, 4);
    assert_eq!(
        s.script_text(),
        BOOK.replace("ANNA (V.O.)", "EVA (V.O.)")
            .replace("ANNA\nHallo", "EVA\nHallo")
            .replace("ANNI\nLange", "EVA\nLange")
            .replace("@Anni ^", "@Eva ^")
    );
    assert!(s.script_text().contains("Ich bin nicht Anna."));
}

#[test]
fn the_id_and_what_hangs_on_it_stay() {
    let mut s = Session::open(&make("rename-fig-id")).unwrap();

    s.rename_figure("Anna", "Eva").unwrap();

    let eva = s.project().character("anna").unwrap();
    assert_eq!(eva.name, "Eva");
    assert_eq!((eva.short.as_str(), eva.color), ("AN", FigureColor::Green));
    assert_eq!(s.project().characters.len(), 2);
    // The other names stay: the book does not cue them any more, they cost nothing.
    assert_eq!(eva.aliases, ["ANNI"]);
}

#[test]
fn the_figure_is_found_by_an_alias_too() {
    let mut s = Session::open(&make("rename-fig-alias")).unwrap();

    s.rename_figure("anni", "Eva").unwrap();

    assert_eq!(s.project().character("anna").unwrap().name, "Eva");
}

#[test]
fn text_and_figure_come_back_in_one_undo_step() {
    let mut s = Session::open(&make("rename-fig-undo")).unwrap();
    let before = s.script_text().to_owned();

    s.rename_figure("Anna", "Eva").unwrap();
    s.undo();
    assert_eq!(s.script_text(), before);
    assert_eq!(s.project().character("anna").unwrap().name, "Anna");
    assert!(!s.can_undo());

    s.redo();
    assert!(s.script_text().contains("EVA (V.O.)"));
    assert_eq!(s.project().character("anna").unwrap().name, "Eva");
}

#[test]
fn the_new_name_is_saved_with_the_same_id() {
    let dir = make("rename-fig-save");
    let mut s = Session::open(&dir).unwrap();

    s.rename_figure("Anna", "Eva").unwrap();
    s.save(OnOutside::Ask).unwrap();

    let again = Session::open(&dir).unwrap();
    assert_eq!(again.project().character("anna").unwrap().name, "Eva");
    assert!(again.pending_figures().is_empty());
    assert!(again.script_text().contains("EVA\nHallo."));
}

#[test]
fn an_empty_equal_or_taken_name_is_refused() {
    let mut s = Session::open(&make("rename-fig-refused")).unwrap();

    for name in ["", "  "] {
        let err = s.rename_figure("Anna", name).unwrap_err();
        assert!(matches!(
            err,
            ServiceError::BadValue {
                field: Field::Name,
                ..
            }
        ));
    }

    let same = s.rename_figure("Anna", " Anna ").unwrap_err();
    assert!(matches!(same, ServiceError::NameUnchanged(_)));

    // Another character's name, in any case.
    let taken = s.rename_figure("Anna", "bert").unwrap_err();
    assert!(matches!(
        taken,
        ServiceError::BadValue {
            field: Field::Name,
            ..
        }
    ));

    let none = s.rename_figure("Zora", "Eva").unwrap_err();
    assert!(matches!(none, ServiceError::NoCharacter(_)));

    assert_eq!(s.script_text(), BOOK);
    assert!(!s.can_undo());
}

/// DB-64: the links of the plans point at elements of the scene, and a rename keeps their
/// count and order.
#[test]
fn links_from_the_plans_to_the_script_stay_valid() {
    let mut s = Session::open(&example("rename-fig-links")).unwrap();
    let chips = |s: &Session| -> Vec<_> {
        s.script_lines()
            .iter()
            .map(|l| (l.index, l.chips.clone()))
            .collect()
    };
    let before = chips(&s);
    let links = s.project().scenes[0].plans[0].links.clone();
    assert!(!links.is_empty());

    s.rename_figure("Elisabeth", "Elise").unwrap();

    assert_eq!(s.project().scenes[0].plans[0].links, links);
    let after = chips(&s);
    assert_eq!(after.len(), before.len());
    assert_eq!(
        after.iter().map(|l| l.1.len()).sum::<usize>(),
        before.iter().map(|l| l.1.len()).sum::<usize>()
    );
    let cue = s
        .script_lines()
        .into_iter()
        .find(|l| l.kind == "character")
        .unwrap();
    assert_eq!(
        (cue.text.as_str(), cue.character.as_str()),
        ("ELISE", "elisabeth")
    );
}
