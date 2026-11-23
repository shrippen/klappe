//! The breakdown: `breakdown/elemente.toml` and `breakdown/<scene-id>.toml` (DB-60, FORMAT.md).
#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::{Path, PathBuf};

use klappe_model::{
    Element, ElementCategory, ElementStatus, Location, Point, Project, Scene, SetObject, SetPlan,
    Sheet,
};
use klappe_store::{encode_tree, kind_of, load, read_tree, save, FileKind, StoreError};

fn dir(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&dir);

    dir
}

fn element(id: &str, name: &str) -> Element {
    Element {
        id: id.into(),
        category: ElementCategory::SetDressing,
        name: name.into(),
        description: "Messing, alt".into(),
        status: ElementStatus::Working,
    }
}

fn scene(id: &str, number: &str, sheet: Sheet) -> Scene {
    Scene {
        id: id.into(),
        number: number.into(),
        heading: "INT. SAAL – TAG".into(),
        sheet,
        ..Scene::default()
    }
}

fn project() -> Project {
    let mut project = Project::new("T");
    project.elements = vec![element("e1", "Kronleuchter"), element("e2", "Teppich")];
    project.scenes = vec![
        scene(
            "aaaa",
            "1",
            Sheet {
                story_day: "Tag 3".into(),
                prestop_s: Some(95),
                planned_min: Some(240),
                continuity: "Mantel offen".into(),
                note: "Nebel".into(),
                elements: vec!["e2".into(), "e1".into()],
                excluded: vec!["e3".into()],
                characters: vec!["bob".into()],
            },
        ),
        scene("bbbb", "2", Sheet::default()),
    ];

    project
}

#[test]
fn the_breakdown_survives_a_save() {
    let dir = dir("breakdown-roundtrip");
    let project = project();

    save(&dir, &project, None).unwrap();
    let loaded = load(&dir).unwrap().project;

    assert_eq!(loaded.elements, project.elements);
    assert_eq!(loaded.scenes[0].sheet, project.scenes[0].sheet);
    assert!(loaded.scenes[1].sheet.is_empty());
}

#[test]
fn the_files_exist_only_with_content() {
    let dir = dir("breakdown-files");
    let mut project = project();

    save(&dir, &project, None).unwrap();
    let toml = fs::read_to_string(dir.join("breakdown/elemente.toml")).unwrap();
    assert!(toml.contains("[[element]]"));
    assert!(toml.contains("category = \"set-dressing\""));
    assert!(toml.contains("status = \"working\""));
    assert!(dir.join("breakdown/szenen/aaaa.toml").is_file());
    assert!(!dir.join("breakdown/szenen/bbbb.toml").exists());

    project.scenes[0].sheet = Sheet::default();
    project.elements.clear();
    save(&dir, &project, None).unwrap();

    assert!(!dir.join("breakdown/elemente.toml").exists());
    assert!(!dir.join("breakdown/szenen/aaaa.toml").exists());
}

#[test]
fn a_project_without_breakdown_has_no_folder() {
    let dir = dir("breakdown-none");

    save(&dir, &Project::new("T"), None).unwrap();

    assert!(!dir.join("breakdown").exists());
    assert!(load(&dir).unwrap().project.elements.is_empty());
}

#[test]
fn a_sheet_of_a_gone_scene_is_not_kept() {
    let dir = dir("breakdown-orphan");
    save(&dir, &project(), None).unwrap();
    fs::write(
        dir.join("breakdown/szenen/zzzz.toml"),
        "story_day = \"x\"\n",
    )
    .unwrap();

    let loaded = load(&dir).unwrap().project;
    assert_eq!(loaded.scenes.len(), 2);

    save(&dir, &loaded, None).unwrap();
    assert!(!dir.join("breakdown/szenen/zzzz.toml").exists());
}

#[test]
fn a_sheet_naming_unknown_ids_loads() {
    let dir = dir("breakdown-dangling");
    let mut project = project();
    project.elements.clear();
    save(&dir, &project, None).unwrap();

    let loaded = load(&dir).unwrap().project;

    assert_eq!(loaded.scenes[0].sheet.elements, ["e2", "e1"]);
}

#[test]
fn an_element_id_used_twice_does_not_load_or_save() {
    let dir = dir("breakdown-duplicate");
    save(&dir, &project(), None).unwrap();
    fs::write(
        dir.join("breakdown/elemente.toml"),
        "[[element]]\nid = \"e1\"\ncategory = \"prop\"\nname = \"A\"\n\n\
         [[element]]\nid = \"e1\"\ncategory = \"prop\"\nname = \"B\"\n",
    )
    .unwrap();

    assert!(matches!(load(&dir), Err(StoreError::Parse { .. })));

    let mut twice = project();
    twice.elements.push(element("e1", "Noch einer"));
    assert!(matches!(
        save(&dir, &twice, None),
        Err(StoreError::Parse { .. })
    ));
}

#[test]
fn an_unknown_category_does_not_load() {
    let dir = dir("breakdown-category");
    save(&dir, &project(), None).unwrap();
    fs::write(
        dir.join("breakdown/elemente.toml"),
        "[[element]]\nid = \"e1\"\ncategory = \"nonsense\"\nname = \"A\"\n",
    )
    .unwrap();

    assert!(load(&dir).is_err());
}

#[test]
fn the_files_are_part_of_the_tree() {
    let dir = dir("breakdown-tree");
    let project = project();
    save(&dir, &project, None).unwrap();

    let tree = read_tree(&dir).unwrap();
    assert!(tree.contains_key("breakdown/elemente.toml"));
    assert!(tree.contains_key("breakdown/szenen/aaaa.toml"));
    assert_eq!(tree, encode_tree(&project, None).unwrap());

    assert_eq!(
        kind_of("breakdown/elemente.toml"),
        Some(FileKind::Breakdown)
    );
    assert_eq!(
        kind_of("breakdown/szenen/aaaa.toml"),
        Some(FileKind::Breakdown)
    );
    assert_eq!(kind_of("breakdown/a/b.toml"), None);
}

#[test]
fn a_set_object_keeps_its_element() {
    let dir = dir("breakdown-object");
    let mut project = Project::new("T");
    let set = SetPlan {
        objects: vec![
            SetObject {
                element: Some("e1".into()),
                ..object("o1", "Tisch")
            },
            object("o2", "Stuhl"),
        ],
        ..SetPlan::default()
    };
    project.locations = vec![Location {
        id: "saal".into(),
        name: "Saal".into(),
        site: None,
        set,
    }];

    save(&dir, &project, None).unwrap();
    let text = fs::read_to_string(dir.join("drehorte/saal.toml")).unwrap();
    assert_eq!(text.matches("element = ").count(), 1);

    let objects = load(&dir).unwrap().project.locations[0].set.objects.clone();
    assert_eq!(objects[0].element.as_deref(), Some("e1"));
    assert_eq!(objects[1].element, None);
}

fn object(id: &str, label: &str) -> SetObject {
    SetObject {
        id: id.into(),
        kind: "table".into(),
        label: label.into(),
        at: Point::new(1.0, 1.0),
        width_m: 1.0,
        depth_m: 1.0,
        rotation_deg: 0.0,
        shape: Default::default(),
        height_m: None,
        base_m: None,
        mirrored: false,
        element: None,
    }
}
