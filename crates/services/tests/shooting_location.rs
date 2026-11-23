//! Where a scene is shot (DB-51, D-21): its scripted location's shooting location unless it has
//! one of its own; the list of scripted locations writes `orte.toml` at once, one undo step per
//! change; a plan stays on the set it is drawn on.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_model::{BookState, FloorPlan, Location, Place, Project, Scene, SetPlan};
use klappe_services::{PlaceEdit, ProjectEdit, ServiceError, Session};

const BOOK: &str = "INT. WACHRAUM – NACHT #14#\n\nNebel.\n\n\
EXT. HAFEN – NACHT #23#\n\nWasser.\n\n";

fn location(id: &str, name: &str) -> Location {
    Location {
        id: id.into(),
        name: name.into(),
        site: None,
        set: SetPlan::default(),
    }
}

fn scene(id: &str, number: &str, heading: &str, plans: Vec<FloorPlan>) -> Scene {
    Scene {
        id: id.into(),
        number: number.into(),
        heading: heading.into(),
        state: BookState::InBook,
        plans,
        ..Scene::default()
    }
}

fn place(id: &str, name: &str, location: Option<&str>) -> Place {
    Place {
        id: id.into(),
        name: name.into(),
        variants: Vec::new(),
        location: location.map(str::to_owned),
    }
}

/// WACHRAUM is shot in the lighthouse, where scene 14 has its plan; HAFEN (scene 23) is shot
/// nowhere yet. An unused place SPEICHER is in the list too.
fn make(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    let mut project = Project::new("Test");
    project.locations = vec![
        location("leuchtturm", "Leuchtturm"),
        location("studio", "Studio 2"),
    ];
    let plan = FloorPlan {
        location: "leuchtturm".into(),
        ..FloorPlan::new("plan-a", "Plan A")
    };
    project.scenes = vec![
        scene("aaaa", "14", "INT. WACHRAUM – NACHT", vec![plan]),
        scene("bbbb", "23", "EXT. HAFEN – NACHT", Vec::new()),
    ];
    klappe_store::save(&dir, &project, Some(BOOK)).unwrap();
    klappe_store::save_places(
        &dir,
        &[
            place("wach", "WACHRAUM", Some("leuchtturm")),
            place("hafen", "HAFEN", None),
            place("speicher", "SPEICHER", None),
        ],
    )
    .unwrap();

    dir
}

fn edit(name: &str, variants: &[&str], location: Option<&str>) -> PlaceEdit {
    PlaceEdit {
        name: name.into(),
        variants: variants.iter().map(|v| (*v).into()).collect(),
        location: location.map(str::to_owned),
    }
}

fn on_disk(dir: &Path, id: &str) -> Place {
    let places = klappe_store::load_places(dir).unwrap();

    places.into_iter().find(|p| p.id == id).unwrap()
}

#[test]
fn a_scene_is_shot_where_its_scripted_location_is() {
    let s = Session::open(&make("shoot-follow")).unwrap();
    let rows = s.overview().scenes;

    assert_eq!(rows[0].location, "leuchtturm");
    assert_eq!(rows[0].place, "WACHRAUM");
    assert!(!rows[0].own_location);
    assert_eq!(rows[1].location, "");
    assert_eq!(rows[1].place, "HAFEN");

    let places = s.overview().places;
    assert_eq!(places[0].location_name, "Leuchtturm");
    assert_eq!(places[0].scenes, ["14"]);
    assert_eq!(places[1].location, "");
}

#[test]
fn a_scene_shot_nowhere_has_no_plan_until_its_place_gets_a_location() {
    let dir = make("shoot-nowhere");
    let mut s = Session::open(&dir).unwrap();
    s.select_scene("bbbb").unwrap();

    let shot = s.not_shot().unwrap();
    assert_eq!(shot.place_name, "HAFEN");
    assert!(matches!(s.add_plan(""), Err(ServiceError::NotShot)));

    s.edit_place("hafen", &edit("HAFEN", &[], Some("studio")))
        .unwrap();
    assert_eq!(on_disk(&dir, "hafen").location.as_deref(), Some("studio"));
    assert!(
        s.not_shot().is_none(),
        "no plan yet, but shot: Plan anlegen"
    );
    s.add_plan("").unwrap();
    assert_eq!(s.plan().unwrap().location, "studio");

    // Undo takes the step back; the surface writes its entry (`save_answer_step`).
    s.undo();
    s.undo();
    s.save_answer_step().unwrap();
    assert_eq!(on_disk(&dir, "hafen").location, None);
    assert!(s.not_shot().is_some());
}

#[test]
fn a_plan_stays_on_its_set_and_says_so() {
    let dir = make("shoot-elsewhere");
    let mut s = Session::open(&dir).unwrap();
    assert!(s.plan_elsewhere().is_none());

    s.edit_place("wach", &edit("WACHRAUM", &[], Some("studio")))
        .unwrap();
    let elsewhere = s.plan_elsewhere().unwrap();
    assert_eq!(elsewhere.drawn_on, "leuchtturm");
    assert_eq!(elsewhere.shot_at_name, "Studio 2");
    assert_eq!(s.plan().unwrap().location, "leuchtturm");

    s.shoot_scene_at(Some("leuchtturm")).unwrap();
    assert!(s.plan_elsewhere().is_none());
    assert!(s.overview().scenes[0].own_location);
}

#[test]
fn a_rename_keeps_the_name_the_headings_use() {
    let dir = make("shoot-rename");
    let mut s = Session::open(&dir).unwrap();

    s.edit_place("wach", &edit("Wachraum oben", &[], Some("leuchtturm")))
        .unwrap();

    let place = on_disk(&dir, "wach");
    assert_eq!(place.name, "Wachraum oben");
    assert_eq!(place.variants, ["WACHRAUM"]);
    assert!(s.pending_places().is_empty(), "the book asks nothing");
}

#[test]
fn a_name_of_another_place_or_a_missing_location_is_refused() {
    let dir = make("shoot-refused");
    let mut s = Session::open(&dir).unwrap();

    let err = s
        .edit_place("wach", &edit("WACHRAUM", &["hafen"], None))
        .unwrap_err();
    assert!(matches!(err, ServiceError::BadValue { .. }), "{err:?}");
    let err = s
        .edit_place("wach", &edit("WACHRAUM", &[], Some("keller")))
        .unwrap_err();
    assert!(matches!(err, ServiceError::NoLocation(_)), "{err:?}");
    assert_eq!(
        on_disk(&dir, "wach").location.as_deref(),
        Some("leuchtturm")
    );
}

#[test]
fn only_a_place_no_heading_names_goes() {
    let dir = make("shoot-remove");
    let mut s = Session::open(&dir).unwrap();

    let err = s.remove_place("hafen").unwrap_err();
    assert!(
        matches!(err, ServiceError::PlaceInUse { headings: 1 }),
        "{err:?}"
    );

    s.remove_place("speicher").unwrap();
    assert_eq!(klappe_store::load_places(&dir).unwrap().len(), 2);
    s.undo();
    s.save_answer_step().unwrap();
    assert_eq!(on_disk(&dir, "speicher").name, "SPEICHER");
}

#[test]
fn a_new_scene_gets_plan_a_only_where_it_is_shot() {
    let mut s = Session::open(&make("shoot-new")).unwrap();
    let add = |number: &str, heading: &str| ProjectEdit::AddScene {
        number: number.into(),
        heading: heading.into(),
        location: None,
    };

    s.edit_project(add("15", "INT. WACHRAUM – TAG")).unwrap();
    s.edit_project(add("24", "EXT. HAFEN – TAG")).unwrap();

    let scenes = &s.project().scenes;
    assert_eq!(scenes[2].plans[0].location, "leuchtturm");
    assert!(scenes[3].plans.is_empty());
}

#[test]
fn a_location_with_a_plan_on_it_stays() {
    let mut s = Session::open(&make("shoot-keep")).unwrap();

    let err = s
        .edit_project(ProjectEdit::RemoveLocation {
            id: "leuchtturm".into(),
        })
        .unwrap_err();
    assert!(
        matches!(err, ServiceError::LocationInUse { scenes: 1, .. }),
        "{err:?}"
    );
}
