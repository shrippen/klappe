//! Saved views (LH-119) and sketches (LH-120) in the project folder.
#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::{Path, PathBuf};

use klappe_model::{FloorPlan, Location, Point, Project, SavedView, Scene, SetPlan, Sketch};
use klappe_store::{load, save, StoreError};

const PROJECT_FILE: &str = "projekt.toml";

fn test_dir(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&dir);

    dir
}

fn view(id: &str, name: &str) -> SavedView {
    SavedView {
        id: id.into(),
        name: name.into(),
        layers: vec!["set".into(), "figures".into()],
        cameras: vec!["a".into()],
        figures: vec!["elisabeth".into()],
    }
}

#[test]
fn views_are_written_as_ansicht_tables_and_read_back() {
    let dir = test_dir("views-round-trip");
    let mut project = Project::new("Film");
    project.views = vec![view("nur-a", "Nur A"), view("weit", "Weit")];

    save(&dir, &project, None).unwrap();
    let text = fs::read_to_string(dir.join(PROJECT_FILE)).unwrap();

    assert_eq!(text.matches("[[ansicht]]").count(), 2, "{text}");
    assert_eq!(load(&dir).unwrap().project, project);
}

#[test]
fn a_project_without_views_writes_none() {
    let dir = test_dir("views-none");
    save(&dir, &Project::new("Film"), None).unwrap();

    let text = fs::read_to_string(dir.join(PROJECT_FILE)).unwrap();

    assert!(!text.contains("ansicht"), "{text}");
    assert!(load(&dir).unwrap().project.views.is_empty());
}

#[test]
fn a_view_without_a_name_or_with_a_twin_id_is_refused() {
    for (name, views) in [
        ("views-no-name", vec![view("a", " ")]),
        ("views-no-id", vec![view("", "A")]),
        ("views-twin", vec![view("a", "A"), view("a", "B")]),
        (
            "views-case-twin",
            vec![view("Weit", "A"), view("weit", "B")],
        ),
    ] {
        let dir = test_dir(name);
        let mut project = Project::new("Film");
        project.views = views;
        save(&dir, &project, None).unwrap();

        let loaded = load(&dir);

        assert!(matches!(loaded, Err(StoreError::Parse { .. })), "{name}");
    }
}

#[test]
fn sketches_are_part_of_the_plan_and_optional() {
    let dir = test_dir("sketches");
    let mut project = Project::new("Film");
    project.locations.push(Location {
        id: "saal".into(),
        name: "Saal".into(),
        site: None,
        set: SetPlan::default(),
    });
    let mut plan = FloorPlan::new("plan-a", "Plan A");
    save(&dir, &scene_with(&project, plan.clone()), None).unwrap();
    let plain = fs::read_to_string(dir.join("szenen/szene-001.json")).unwrap();
    assert!(!plain.contains("sketches"), "{plain}");

    plan.sketches.push(Sketch {
        points: vec![Point::new(1.0, 2.0), Point::new(1.5, 2.5)],
    });
    let with = scene_with(&project, plan);
    save(&dir, &with, None).unwrap();

    let text = fs::read_to_string(dir.join("szenen/szene-001.json")).unwrap();
    assert!(text.contains("sketches"), "{text}");
    assert_eq!(load(&dir).unwrap().project, with);
}

fn scene_with(project: &Project, mut plan: FloorPlan) -> Project {
    let mut project = project.clone();
    plan.location = "saal".into();
    project.scenes = vec![Scene {
        id: "szene-001".into(),
        number: "1".into(),
        heading: "INT. SAAL – TAG".into(),
        location: Some("saal".into()),
        plans: vec![plan],
        ..Default::default()
    }];

    project
}
