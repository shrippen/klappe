//! The register of scenes: random ids, order, and what loading refuses (DATENMODELL §4).
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_services::{Field, ProjectEdit, ServiceError, Session};

const FIXTURE_ID: &str = "m4djbtj6";

fn copy_example(name: &str) -> PathBuf {
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
        match entry.file_type().unwrap().is_dir() {
            true => copy_dir(&entry.path(), &target),
            false => drop(std::fs::copy(entry.path(), target).unwrap()),
        }
    }
}

fn add(number: &str) -> ProjectEdit {
    ProjectEdit::AddScene {
        number: number.into(),
        heading: "INT. HOF – NACHT".into(),
        location: "saengersaal".into(),
    }
}

#[test]
fn a_new_scene_gets_a_random_id_and_goes_last() {
    let mut s = Session::open(&copy_example("reg-add")).unwrap();

    s.edit_project(add("1")).unwrap();
    s.edit_project(add("2")).unwrap();
    let ids: Vec<String> = s.project().scenes.iter().map(|x| x.id.clone()).collect();

    assert_eq!(ids[0], FIXTURE_ID);
    assert_eq!(s.project().scenes[1].number, "1");
    assert_eq!(s.project().scenes[2].number, "2");
    assert_ne!(ids[1], ids[2]);
    assert!(ids[1..]
        .iter()
        .all(|id| id.len() == 8 && !id.contains("szene")));
}

#[test]
fn renumbering_a_scene_keeps_its_id() {
    let mut s = Session::open(&copy_example("reg-renumber")).unwrap();

    s.edit_project(ProjectEdit::EditScene {
        id: FIXTURE_ID.into(),
        number: "40".into(),
        heading: "INT. SÄNGERSAAL – TAG".into(),
        location: "saengersaal".into(),
    })
    .unwrap();

    assert_eq!(s.project().scenes[0].id, FIXTURE_ID);
    assert_eq!(s.project().scenes[0].number, "40");
}

#[test]
fn a_conflicted_screenplay_is_not_opened() {
    let dir = copy_example("reg-conflict-script");
    let script = std::fs::read_to_string(dir.join("drehbuch.fountain")).unwrap();
    let conflicted = format!("<<<<<<< HEAD\n{script}\n=======\nanders\n>>>>>>> x\n");
    std::fs::write(dir.join("drehbuch.fountain"), conflicted).unwrap();

    let error = Session::open(&dir).err().unwrap();

    assert!(
        matches!(
            &error,
            ServiceError::Store(klappe_store::StoreError::Conflict { line: 1, path })
                if path == &dir.join("drehbuch.fountain")
        ),
        "{error}"
    );
}

#[test]
fn a_conflicted_register_is_not_opened() {
    let dir = copy_example("reg-conflict-register");
    std::fs::write(
        dir.join("szenen.toml"),
        "<<<<<<< HEAD\na\n=======\nb\n>>>>>>> x\n",
    )
    .unwrap();

    let error = Session::open(&dir).err().unwrap();

    assert!(
        matches!(
            &error,
            ServiceError::Store(klappe_store::StoreError::Conflict { line: 1, .. })
        ),
        "{error}"
    );
}

#[test]
fn a_conflict_is_worded_in_both_languages() {
    let dir = copy_example("reg-conflict-words");
    std::fs::write(
        dir.join("szenen.toml"),
        "<<<<<<< HEAD\n=======\n>>>>>>> x\n",
    )
    .unwrap();
    let error = Session::open(&dir).err().unwrap();

    let de = klappe_services::error_text(klappe_services::Language::German, &error);
    let en = klappe_services::error_text(klappe_services::Language::English, &error);

    assert!(
        de.contains("Konfliktmarken") && de.contains("szenen.toml:1"),
        "{de}"
    );
    assert!(
        en.contains("conflict markers") && en.contains("szenen.toml:1"),
        "{en}"
    );
}

#[test]
fn a_heading_with_a_line_break_is_refused() {
    let mut s = Session::open(&copy_example("reg-heading")).unwrap();

    let added = s.edit_project(ProjectEdit::AddScene {
        number: "1".into(),
        heading: "INT. HOF\n– NACHT".into(),
        location: "saengersaal".into(),
    });
    let edited = s.edit_project(ProjectEdit::EditScene {
        id: FIXTURE_ID.into(),
        number: "12".into(),
        heading: "INT.\r\nSAAL".into(),
        location: "saengersaal".into(),
    });

    for result in [added, edited] {
        let error = result.unwrap_err();
        assert!(
            matches!(
                error,
                ServiceError::BadValue {
                    field: Field::Heading,
                    ..
                }
            ),
            "{error}"
        );
    }
    assert_eq!(s.project().scenes.len(), 1);
}

#[test]
fn an_orphan_scene_file_is_listed_in_the_load_notes() {
    let dir = copy_example("reg-orphan-note");
    std::fs::write(dir.join("szenen.toml"), "").unwrap();

    let s = Session::open(&dir).unwrap();

    assert_eq!(s.load_notes().len(), 1);
    let text = klappe_services::load_notes_text(klappe_services::Language::English, s.load_notes());
    assert!(text.contains(FIXTURE_ID), "{text}");
    // The book's scene 12 gets an entry of its own, first; the orphan follows.
    let orphan = s.project().scenes.iter().find(|x| x.id == FIXTURE_ID);
    assert_eq!(orphan.unwrap().state, klappe_model::BookState::NotInBook);
}
