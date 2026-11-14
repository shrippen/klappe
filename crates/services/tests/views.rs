//! The project lists and the tooltips the session hands the surfaces (Audit, Fund 5), on the
//! example project.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_services::{
    CharacterRow, FigureColor, Hint, LocationRow, PlanEdit, PlanRow, ProjectEdit, SceneRow, Session,
};
use serde_json::json;

fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe")
}

fn open() -> Session {
    Session::open(&example()).unwrap()
}

/// The lists name the location of each scene and count its scenes.
#[test]
fn the_overview_lists_scenes_locations_and_plans() {
    let mut s = open();
    s.copy_plan("Stand 1").unwrap();

    let o = s.overview();

    assert_eq!(
        o.scenes[0],
        SceneRow {
            id: "szene-012".into(),
            number: "12".into(),
            heading: s.project().scenes[0].heading.clone(),
            location: s.project().scenes[0].location.clone(),
            location_name: s.project().locations[0].name.clone(),
        }
    );
    assert_eq!(
        o.locations[0],
        LocationRow {
            id: s.project().locations[0].id.clone(),
            name: s.project().locations[0].name.clone(),
            width_m: s.project().locations[0].set.width_m,
            depth_m: s.project().locations[0].set.depth_m,
            scenes: s.project().scenes.len(),
        }
    );
    assert_eq!(
        o.plans,
        [
            PlanRow {
                id: "plan-a".into(),
                name: "Plan A".into(),
                shots: 5
            },
            PlanRow {
                id: "stand-1".into(),
                name: "Stand 1".into(),
                shots: 5
            },
        ]
    );
    assert_eq!(o.plan, "stand-1");
}

/// A group has its head count, a single person 0; the colour is the role's name.
#[test]
fn the_overview_gives_groups_their_count() {
    let mut s = open();
    s.edit_project(ProjectEdit::EditCharacter {
        id: "burgvogt".into(),
        name: "Burgvogt".into(),
        short: "B".into(),
        color: FigureColor::Blue,
        group: Some(12),
    })
    .unwrap();

    let characters = s.overview().characters;
    let burgvogt = characters.iter().find(|c| c.id == "burgvogt").unwrap();
    let elisabeth = characters.iter().find(|c| c.id == "elisabeth").unwrap();

    assert_eq!(
        burgvogt,
        &CharacterRow {
            id: "burgvogt".into(),
            name: "Burgvogt".into(),
            short: "B".into(),
            color: "blue",
            group: 12
        }
    );
    assert_eq!(elisabeth.group, 0);
}

/// The overview is the JSON the project view reads: camel case, units dropped.
#[test]
fn the_overview_serialises_as_the_view_reads_it() {
    let value = serde_json::to_value(open().overview()).unwrap();

    assert_eq!(value["scenes"][0]["locationName"], "Sängersaal");
    assert!(value["locations"][0]["width"].is_f64());
    assert_eq!(
        value["plans"][0],
        json!({ "id": "plan-a", "name": "Plan A", "shots": 5 })
    );
}

/// The set of the scene gives the location, its notes and its background.
#[test]
fn the_scene_location_gives_the_set() {
    let s = open();

    assert!(!s.has_background());
    assert_eq!(s.note_text("n1").as_deref(), Some("Vorhang schließen"));
    assert_eq!(s.note_text("x"), None);
}

/// A camera's hint has its shots, the character it follows and the angle to a tenth.
#[test]
fn a_camera_hint_names_shots_and_follow() {
    let s = open();

    let Some(Hint::Camera {
        shots,
        follows,
        fov_deg,
        body,
        ..
    }) = s.camera_hint("b")
    else {
        panic!("no camera hint");
    };

    assert_eq!(shots, "12.3, 12.4");
    assert_eq!(follows, "Burgvogt");
    assert_eq!(fov_deg, 28.0);
    assert_eq!(body, "super35");
    assert_eq!(s.camera_hint("x"), None);
}

/// A sensor of no body's width is a custom one: no body id.
#[test]
fn a_custom_sensor_has_no_body() {
    let mut s = open();
    s.edit_plan(PlanEdit::SetSensor {
        camera: "a".into(),
        width_mm: 23.4,
        aspect: None,
    })
    .unwrap();

    let Some(Hint::Camera { body, .. }) = s.camera_hint("a") else {
        panic!("no camera hint");
    };

    assert_eq!(body, "");
}

/// Figure, mark, light, object and note hints, and the JSON each makes.
#[test]
fn the_other_hints_carry_their_fields() {
    let s = open();

    assert_eq!(
        serde_json::to_value(s.mark_hint("elisabeth", Some(1))).unwrap(),
        json!({ "kind": "mark", "name": "Elisabeth", "index": 2, "time": 6.0 })
    );
    assert_eq!(
        serde_json::to_value(s.figure_hint("elisabeth", None)).unwrap(),
        json!({ "kind": "figure", "name": "Elisabeth", "index": null, "time": null })
    );
    assert_eq!(
        serde_json::to_value(s.light_hint("led-1")).unwrap(),
        json!({ "kind": "light", "label": "LED-Panel", "temp": 5600.0, "power": 200.0 })
    );
    assert_eq!(s.light_hint("x"), None);
    assert_eq!(
        serde_json::to_value(s.object_hint("tisch")).unwrap(),
        json!({
            "kind": "object", "label": "Tafel", "library": "table", "width": 6.0,
            "depth": 1.2, "height": null,
        })
    );
    assert_eq!(s.object_hint("x"), None);
    assert_eq!(
        serde_json::to_value(s.note_hint("x")).unwrap(),
        json!({ "kind": "note", "text": "" })
    );
}
