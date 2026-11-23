//! The project lists and the tooltips the session hands the surfaces (Audit, Fund 5), on the
//! example project.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_services::{
    CharacterRow, FigureColor, Hint, Language, LocationRow, PlanEdit, PlanRow, ProjectEdit,
    SceneRow, Session, SunTexts, TrackKind,
};
use serde_json::json;

fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe")
}

fn open() -> Session {
    Session::open(&example()).unwrap()
}

/// The follow chooser lists the figure tracks by name: a group carries its head count (LH-23).
#[test]
fn a_group_track_is_named_with_its_count() {
    let mut s = open();
    s.edit_project(ProjectEdit::EditCharacter {
        id: "burgvogt".into(),
        name: "Burgvogt".into(),
        short: "B".into(),
        color: FigureColor::Blue,
        group: Some(12),
        size_m: None,
        aliases: None,
    })
    .unwrap();

    let names: Vec<String> = s
        .tracks()
        .into_iter()
        .filter(|t| t.kind == TrackKind::Figure)
        .map(|t| t.name)
        .collect();

    assert!(names.contains(&"Burgvogt (12)".to_owned()), "{names:?}");
    assert!(names.contains(&"Elisabeth".to_owned()), "{names:?}");
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
            id: "m4djbtj6".into(),
            number: "12".into(),
            heading: s.project().scenes[0].heading.clone(),
            in_book: true,
            plans: 2,
            shots: 10,
            line: Some(0),
            length: String::new(),
            location: s.project().locations[0].id.clone(),
            location_name: s.project().locations[0].name.clone(),
            own_location: false,
            place: "SÄNGERSAAL".into(),
            sun: SunTexts::default(),
        }
    );
    assert_eq!(
        o.locations[0],
        LocationRow {
            id: s.project().locations[0].id.clone(),
            name: s.project().locations[0].name.clone(),
            width_m: s.project().locations[0].set.width_m,
            depth_m: s.project().locations[0].set.depth_m,
            wall_height_m: s.project().locations[0].set.wall_height_m,
            latitude: None,
            longitude: None,
            north: 0.0,
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
        size_m: None,
        aliases: None,
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
            group: 12,
            size_m: None,
            aliases: Vec::new(),
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

/// A scene row counts the plans of its scene and the shots on all of them (D-13 markers).
#[test]
fn a_scene_row_counts_plans_and_shots() {
    let mut s = open();
    let before = serde_json::to_value(s.overview()).unwrap();
    assert_eq!(before["scenes"][0]["plans"], 1);
    assert_eq!(before["scenes"][0]["shots"], 5);

    s.copy_plan("Stand 1").unwrap();
    let after = serde_json::to_value(s.overview()).unwrap();

    assert_eq!(after["scenes"][0]["plans"], 2);
    assert_eq!(after["scenes"][0]["shots"], 10);
}

/// The set of the scene gives its notes and its background.
#[test]
fn the_set_gives_notes_and_background() {
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
        json!({ "kind": "mark", "name": "Elisabeth", "index": 2, "time": 6.0, "arrival": "walk", "departure": "stay" })
    );
    assert_eq!(
        serde_json::to_value(s.figure_hint("elisabeth", None)).unwrap(),
        json!({ "kind": "figure", "name": "Elisabeth", "index": null, "time": null })
    );
    assert_eq!(
        serde_json::to_value(s.light_hint("led-1")).unwrap(),
        json!({
            "kind": "light", "label": "LED-Panel", "type": "led-panel", "facing": 330.0,
            "beam": 60.0, "tunable": true, "gives_light": true, "temp": 5600.0, "power": 200.0,
        })
    );
    assert_eq!(s.light_hint("x"), None);
    assert_eq!(
        serde_json::to_value(s.object_hint("tisch")).unwrap(),
        json!({
            "kind": "object", "label": "Tafel", "library": "table", "width": 6.0,
            "depth": 1.2, "height": null, "base": 0.0, "baseDefault": true, "mirrored": false,
        })
    );
    assert_eq!(s.object_hint("x"), None);
    assert_eq!(
        serde_json::to_value(s.note_hint("x")).unwrap(),
        json!({ "kind": "note", "text": "", "bound": false })
    );
}

/// DB-32 in the window's scene list: each scene of the book has its length once the
/// screenplay is laid out; a layout is made again only when the text changed.
#[test]
fn the_scene_list_shows_each_scenes_length() {
    let mut s = open();
    assert!(s.overview().scenes.iter().all(|r| r.length.is_empty()));

    s.paginate_if_changed(Language::German);
    let first = s.overview().scenes[0].clone();
    assert!(first.in_book);
    assert!(!first.length.is_empty(), "{first:?}");

    let longer = format!("{}\n{}", s.script_text(), "Lange Handlung. ".repeat(400));
    s.set_script_text(&longer);
    assert_eq!(s.overview().scenes[0].length, first.length);
    s.paginate_if_changed(Language::German);
    assert_ne!(s.overview().scenes[0].length, first.length);
}

/// Review of PR 287: until the next layout a scene keeps its own last length, also when typing
/// moves its heading to the line another scene's heading had.
#[test]
fn a_scene_keeps_its_own_length_until_the_next_layout() {
    let mut s = open();
    let heading = format!("{} #12#", s.project().scenes[0].heading);
    let long = "Lange Handlung. ".repeat(300);
    s.set_script_text(&format!(
        "INT. FLUR - TAG\n\nKurz.\n\n{heading}\n\n{long}\n"
    ));
    s.paginate_if_changed(Language::German);
    let scene = s.overview().scenes[0].clone();
    assert_eq!(scene.line, Some(4));
    assert_ne!(scene.length, "1/8", "the short scene's length");

    s.set_script_text(&format!("{heading}\n\n{long}\n"));

    let moved = s.overview().scenes[0].clone();
    assert_eq!(moved.line, Some(0));
    assert_eq!(moved.length, scene.length);
}
