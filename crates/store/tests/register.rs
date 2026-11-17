//! The register of what is shot: `szenen.toml` and `szenen/<id>.json` (DATENMODELL §4).
#![allow(clippy::unwrap_used)]

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use klappe_model::{random_id, BookState, FloorPlan, Kind, Print, Project, Scene, FORMAT_VERSION};
use klappe_store::{load, save, StoreError};

fn test_dir(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&dir);

    dir
}

fn scene(id: &str, number: &str, heading: &str) -> Scene {
    Scene {
        id: id.into(),
        number: number.into(),
        heading: heading.into(),
        location: "saal".into(),
        plans: vec![FloorPlan::new("plan-a", "Plan A")],
        ..Default::default()
    }
}

#[test]
fn the_register_round_trips_in_file_order() {
    let dir = test_dir("register-round-trip");
    let mut project = Project::new("Film");
    let mut interview = scene("zz", "", "Gespräch am Tisch");
    interview.kind = Kind::Interview;
    let mut struck = scene("aa", "2", "INT. HOF – TAG");
    struck.state = BookState::NotInBook;
    struck.prints = vec![
        Print { tag: 0, hash: 7 },
        Print {
            tag: 2,
            hash: 0xdead_beef,
        },
    ];
    // Not sorted by id: the register keeps its own order.
    project.scenes = vec![interview, scene("mm", "1", "INT. SAAL – TAG"), struck];

    save(&dir, &project, None).unwrap();
    let text = fs::read_to_string(dir.join("szenen.toml")).unwrap();

    assert_eq!(text.matches("[[szene]]").count(), 3, "{text}");
    assert!(text.contains("art = \"interview\""), "{text}");
    assert!(text.contains("state = \"not-in-book\""), "{text}");
    assert!(text.contains("\"2:deadbeef\""), "{text}");
    assert_eq!(load(&dir).unwrap().project, project);
}

#[test]
fn the_scene_file_holds_no_number_or_heading() {
    let dir = test_dir("register-slim-file");
    let mut project = Project::new("Film");
    project.scenes = vec![scene("mm", "1", "INT. SAAL – TAG")];

    save(&dir, &project, None).unwrap();
    let text = fs::read_to_string(dir.join("szenen/mm.json")).unwrap();

    assert!(
        !text.contains("number") && !text.contains("heading"),
        "{text}"
    );
    assert!(text.contains("\"location\""), "{text}");
}

#[test]
fn renumbering_keeps_the_file() {
    let dir = test_dir("register-renumber");
    let mut project = Project::new("Film");
    project.scenes = vec![scene("mm", "1", "INT. SAAL – TAG")];
    save(&dir, &project, None).unwrap();

    project.scenes[0].number = "7A".into();
    save(&dir, &project, None).unwrap();

    assert!(dir.join("szenen/mm.json").exists());
    assert_eq!(fs::read_dir(dir.join("szenen")).unwrap().count(), 1);
    assert_eq!(load(&dir).unwrap().project.scenes[0].number, "7A");
}

#[test]
fn random_ids_are_unique_and_survive_save_and_load() {
    let dir = test_dir("register-ids");
    let mut project = Project::new("Film");
    project.scenes = (0..50)
        .map(|n| scene(&random_id(), &n.to_string(), "INT. SAAL – TAG"))
        .collect();
    let ids: HashSet<&str> = project.scenes.iter().map(|s| s.id.as_str()).collect();
    assert_eq!(ids.len(), 50);

    save(&dir, &project, None).unwrap();
    let loaded = load(&dir).unwrap().project;

    assert_eq!(loaded.scenes, project.scenes);
}

#[test]
fn a_conflicted_register_is_refused() {
    let dir = test_dir("register-conflict");
    save(&dir, &Project::new("Film"), None).unwrap();
    fs::write(
        dir.join("szenen.toml"),
        "[[szene]]\n<<<<<<< HEAD\nid = \"a\"\n=======\nid = \"b\"\n>>>>>>> x\n",
    )
    .unwrap();

    let error = load(&dir).unwrap_err();

    assert!(
        matches!(error, StoreError::Conflict { line: 2, .. }),
        "{error}"
    );
}

#[test]
fn an_older_format_is_refused() {
    let dir = test_dir("register-older");
    save(&dir, &Project::new("Film"), None).unwrap();
    fs::write(
        dir.join("projekt.toml"),
        "format_version = 2\ntitle = \"x\"\n",
    )
    .unwrap();

    let error = load(&dir).unwrap_err();

    assert!(
        matches!(error, StoreError::OlderFormat { found: 2, .. }),
        "{error}"
    );
}

#[test]
fn a_scene_file_without_an_entry_is_refused() {
    let dir = test_dir("register-orphan");
    let mut project = Project::new("Film");
    project.scenes = vec![scene("mm", "1", "INT. SAAL – TAG")];
    save(&dir, &project, None).unwrap();
    fs::write(dir.join("szenen.toml"), "").unwrap();

    let error = load(&dir).unwrap_err();

    assert!(matches!(error, StoreError::Parse { .. }), "{error}");
}

#[test]
fn newer_format_wins_over_missing_title() {
    let dir = test_dir("newer-no-title");
    fs::create_dir_all(&dir).unwrap();
    let newer = FORMAT_VERSION + 1;
    fs::write(
        dir.join("projekt.toml"),
        format!("format_version = {newer}\n"),
    )
    .unwrap();

    let error = load(&dir).unwrap_err();

    assert!(matches!(error, StoreError::NewerFormat { .. }), "{error}");
}
