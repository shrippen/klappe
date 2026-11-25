//! The breakdown (DB-60, D-22): elements, the scene sheet, suggestions from the set, the
//! characters without a line; each edit is one undo step and the sheet survives a save.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_model::{
    BookState, Character, FigureColor, FloorPlan, Location, Place, Point, Project, Scene,
    SetObject, SetPlan,
};
use klappe_services::{
    BreakdownEdit, ElementCategory, ElementStatus, Field, ProjectEdit, ServiceError, Session,
};

const BOOK: &str = "INT. WACHRAUM – NACHT #14#\n\nANNA\nHallo.\n\n\
EXT. HAFEN – NACHT #23#\n\nWasser.\n\n";

fn character(id: &str, group: Option<u32>) -> Character {
    Character {
        id: id.into(),
        name: id.to_uppercase(),
        short: id[..1].to_uppercase(),
        color: FigureColor::ALL[0],
        group,
        size_m: None,
        aliases: Vec::new(),
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

/// Scene 14 (`aaaa`, WACHRAUM) is shot in the lighthouse, whose set has a lamp and a table;
/// ANNA speaks in it. Scene 23 (`bbbb`, HAFEN) is shot nowhere. BOB is silent, CROWD is a group
/// of 12.
fn make(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    let mut project = Project::new("Test");
    let set = SetPlan {
        objects: vec![object("o1", "Öllampe"), object("o2", "Tisch")],
        ..SetPlan::default()
    };
    project.locations = vec![Location {
        id: "leuchtturm".into(),
        name: "Leuchtturm".into(),
        site: None,
        set,
    }];
    project.characters = vec![
        character("anna", None),
        character("bob", None),
        character("crowd", Some(12)),
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
    let place = Place {
        id: "wach".into(),
        name: "WACHRAUM".into(),
        variants: Vec::new(),
        location: Some("leuchtturm".into()),
    };
    klappe_store::save_places(&dir, &[place]).unwrap();

    dir
}

fn open(name: &str) -> Session {
    let mut s = Session::open(&make(name)).unwrap();
    s.select_scene("aaaa").unwrap();

    s
}

fn add(s: &mut Session, category: ElementCategory, name: &str) -> String {
    s.add_element(category, name).unwrap()
}

fn to_scene(s: &mut Session, scene: &str, element: &str) {
    s.edit_project(ProjectEdit::Breakdown(BreakdownEdit::AddSceneElement {
        scene: scene.into(),
        element: element.into(),
    }))
    .unwrap();
}

fn sheet(s: &Session, scene: &str) -> klappe_model::Sheet {
    s.project().scene(scene).unwrap().sheet.clone()
}

/// The numbers of the scenes the first element is in.
fn numbers(s: &Session) -> Vec<String> {
    let elements = s.overview().elements;

    elements[0]
        .scenes
        .iter()
        .map(|r| r.number.clone())
        .collect()
}

fn point_at(s: &mut Session, object: &str, element: &str) {
    s.edit_project(ProjectEdit::Breakdown(BreakdownEdit::SetObjectElement {
        location: "leuchtturm".into(),
        object: object.into(),
        element: Some(element.into()),
    }))
    .unwrap();
}

#[test]
fn an_element_is_added_trimmed_and_undone() {
    let mut s = open("breakdown-add");

    let id = add(&mut s, ElementCategory::Prop, "  Öllampe ");
    let element = &s.project().elements[0];

    assert_eq!(element.id, id);
    assert_eq!(element.name, "Öllampe");
    assert_eq!(element.status, ElementStatus::Open);
    assert_eq!(s.undo_step(), Some("AddElement"));

    s.undo();
    assert!(s.project().elements.is_empty());
}

#[test]
fn a_name_is_not_empty_and_not_used_twice_in_a_category() {
    let mut s = open("breakdown-names");
    add(&mut s, ElementCategory::Prop, "Öllampe");
    let again = |s: &mut Session, category, name: &str| {
        s.edit_project(ProjectEdit::Breakdown(BreakdownEdit::AddElement {
            category,
            name: name.into(),
        }))
    };

    assert!(matches!(
        again(&mut s, ElementCategory::Prop, " öllampe "),
        Err(ServiceError::BadValue { .. })
    ));
    assert!(matches!(
        again(&mut s, ElementCategory::Prop, "  "),
        Err(ServiceError::BadValue { .. })
    ));
    again(&mut s, ElementCategory::SetDressing, "Öllampe").unwrap();
    assert_eq!(s.project().elements.len(), 2);
}

#[test]
fn an_element_is_edited_and_undone() {
    let mut s = open("breakdown-edit");
    let id = add(&mut s, ElementCategory::Prop, "Lampe");
    let lamp = add(&mut s, ElementCategory::Prop, "Laterne");

    s.edit_project(ProjectEdit::Breakdown(BreakdownEdit::EditElement {
        id: id.clone(),
        category: ElementCategory::Vehicle,
        name: "Boot".into(),
        description: " rot ".into(),
        status: ElementStatus::Working,
        responsible: None,
    }))
    .unwrap();
    let element = s.project().elements[0].clone();
    assert_eq!(element.category, ElementCategory::Vehicle);
    assert_eq!(element.name, "Boot");
    assert_eq!(element.description, "rot");
    assert_eq!(element.status, ElementStatus::Working);
    assert_eq!(s.undo_step(), Some("EditElement"));

    // Renaming into a name of the same category is refused.
    let clash = s.edit_project(ProjectEdit::Breakdown(BreakdownEdit::EditElement {
        id: lamp,
        category: ElementCategory::Vehicle,
        name: "boot".into(),
        description: String::new(),
        status: ElementStatus::Open,
        responsible: None,
    }));
    assert!(matches!(clash, Err(ServiceError::BadValue { .. })));

    s.undo();
    assert_eq!(s.project().elements[0].name, "Lampe");
}

#[test]
fn the_sheet_is_set_and_undone() {
    let mut s = open("breakdown-sheet");

    s.edit_project(ProjectEdit::Breakdown(BreakdownEdit::SetSheet {
        scene: "aaaa".into(),
        story_day: " Tag 3 ".into(),
        prestop: "1:35".into(),
        planned: "240".into(),
        continuity: "Anna trägt den Mantel".into(),
        note: "Nebelmaschine".into(),
    }))
    .unwrap();
    let sheet = sheet(&s, "aaaa");

    assert_eq!(sheet.story_day, "Tag 3");
    assert_eq!(sheet.prestop_s, Some(95));
    assert_eq!(sheet.planned_min, Some(240));
    let view = s.overview().sheet.unwrap();
    assert_eq!(view.prestop_text, "1:35");
    assert_eq!(view.planned_text, "4:00");
    assert_eq!(view.continuity, "Anna trägt den Mantel");
    assert_eq!(view.note, "Nebelmaschine");

    s.undo();
    assert!(self::sheet(&s, "aaaa").is_empty());
}

#[test]
fn a_sheet_time_that_is_none_is_refused_by_its_field() {
    let mut s = open("breakdown-sheet-time");
    let mut set = |prestop: &str, planned: &str| {
        s.edit_project(ProjectEdit::Breakdown(BreakdownEdit::SetSheet {
            scene: "aaaa".into(),
            story_day: String::new(),
            prestop: prestop.into(),
            planned: planned.into(),
            continuity: String::new(),
            note: String::new(),
        }))
    };

    assert!(matches!(
        set("1:7x", ""),
        Err(ServiceError::BadValue {
            field: Field::Prestop,
            ..
        })
    ));
    assert!(matches!(
        set("", "1:60"),
        Err(ServiceError::BadValue {
            field: Field::Planned,
            ..
        })
    ));
}

#[test]
fn an_element_comes_into_a_scene_and_goes_again() {
    let mut s = open("breakdown-scene-element");
    let id = add(&mut s, ElementCategory::Prop, "Öllampe");

    to_scene(&mut s, "aaaa", &id);
    to_scene(&mut s, "aaaa", &id);
    assert_eq!(sheet(&s, "aaaa").elements, std::slice::from_ref(&id));

    let view = s.overview().sheet.unwrap();
    assert_eq!(view.categories.len(), 1);
    assert_eq!(view.categories[0].category, "prop");
    assert_eq!(view.categories[0].elements[0].name, "Öllampe");
    assert_eq!(numbers(&s), ["14"]);

    s.edit_project(ProjectEdit::Breakdown(BreakdownEdit::RemoveSceneElement {
        scene: "aaaa".into(),
        element: id,
    }))
    .unwrap();
    assert!(s.overview().sheet.unwrap().categories.is_empty());
    assert!(s.overview().elements[0].scenes.is_empty());
}

#[test]
fn the_field_with_suggestions_finds_or_makes_the_element_in_one_step() {
    let mut s = open("breakdown-to-scene");
    let add_to = |s: &mut Session, scene: &str, name: &str| {
        s.edit_project(ProjectEdit::Breakdown(BreakdownEdit::AddElementToScene {
            scene: scene.into(),
            category: ElementCategory::Costume,
            name: name.into(),
        }))
        .unwrap();
    };

    add_to(&mut s, "aaaa", "Mantel");
    assert_eq!(s.project().elements.len(), 1);
    assert_eq!(s.undo_step(), Some("AddElementToScene"));

    // The same name in another scene is the same element.
    add_to(&mut s, "bbbb", " mantel ");
    assert_eq!(s.project().elements.len(), 1);
    assert_eq!(numbers(&s), ["14", "23"]);

    s.undo();
    assert_eq!(s.project().elements.len(), 1);
    s.undo();
    assert!(s.project().elements.is_empty());
    assert!(sheet(&s, "aaaa").is_empty());
}

#[test]
fn a_suggestion_comes_from_the_set_and_is_taken_or_turned_down() {
    let mut s = open("breakdown-suggest");
    let lamp = add(&mut s, ElementCategory::Prop, "Öllampe");
    let table = add(&mut s, ElementCategory::SetDressing, "Tisch");
    point_at(&mut s, "o1", &lamp);
    point_at(&mut s, "o2", &table);
    assert_eq!(s.undo_step(), Some("SetObjectElement"));

    let found = s.overview().sheet.unwrap().suggestions;
    assert_eq!(found.len(), 2);
    assert_eq!(found[0].element, lamp);
    assert_eq!(found[0].name, "Öllampe");
    assert_eq!(found[0].category, "prop");
    assert_eq!(found[0].object.as_deref(), Some("Öllampe"));

    s.edit_project(ProjectEdit::Breakdown(BreakdownEdit::TakeSuggestion {
        scene: "aaaa".into(),
        element: lamp.clone(),
    }))
    .unwrap();
    s.edit_project(ProjectEdit::Breakdown(BreakdownEdit::ExcludeSuggestion {
        scene: "aaaa".into(),
        element: table.clone(),
    }))
    .unwrap();

    let view = s.overview().sheet.unwrap();
    assert!(view.suggestions.is_empty());
    assert_eq!(view.categories[0].elements[0].id, lamp);
    assert_eq!(sheet(&s, "aaaa").excluded, std::slice::from_ref(&table));

    // Taking a turned down one lists it and drops the turning down.
    s.edit_project(ProjectEdit::Breakdown(BreakdownEdit::TakeSuggestion {
        scene: "aaaa".into(),
        element: table.clone(),
    }))
    .unwrap();
    assert!(sheet(&s, "aaaa").excluded.is_empty());
    assert!(sheet(&s, "aaaa").elements.contains(&table));

    // A scene shot nowhere is offered nothing.
    s.select_scene("bbbb").unwrap();
    assert!(s.overview().sheet.unwrap().suggestions.is_empty());
}

#[test]
fn removing_an_element_cleans_every_reference() {
    let mut s = open("breakdown-remove");
    let lamp = add(&mut s, ElementCategory::Prop, "Öllampe");
    let table = add(&mut s, ElementCategory::SetDressing, "Tisch");
    point_at(&mut s, "o1", &lamp);
    point_at(&mut s, "o2", &table);
    to_scene(&mut s, "aaaa", &lamp);
    to_scene(&mut s, "bbbb", &lamp);
    s.edit_project(ProjectEdit::Breakdown(BreakdownEdit::ExcludeSuggestion {
        scene: "aaaa".into(),
        element: table.clone(),
    }))
    .unwrap();

    s.edit_project(ProjectEdit::Breakdown(BreakdownEdit::RemoveElement {
        id: lamp.clone(),
    }))
    .unwrap();
    assert_eq!(s.undo_step(), Some("RemoveElement"));
    s.edit_project(ProjectEdit::Breakdown(BreakdownEdit::RemoveElement {
        id: table.clone(),
    }))
    .unwrap();

    let project = s.project();
    assert!(project.elements.is_empty());
    assert!(project.scenes.iter().all(|s| s.sheet.is_empty()));
    let objects = &project.locations[0].set.objects;
    assert!(objects.iter().all(|o| o.element.is_none()));

    s.undo();
    s.undo();
    assert_eq!(sheet(&s, "aaaa").elements, std::slice::from_ref(&lamp));
    assert_eq!(s.project().locations[0].set.objects[0].element, Some(lamp));
}

#[test]
fn an_unknown_element_or_scene_is_refused() {
    let mut s = open("breakdown-unknown");

    let no_element = s.edit_project(ProjectEdit::Breakdown(BreakdownEdit::AddSceneElement {
        scene: "aaaa".into(),
        element: "weg".into(),
    }));
    assert!(matches!(no_element, Err(ServiceError::NoElement(_))));

    let id = add(&mut s, ElementCategory::Prop, "Lampe");
    let no_scene = s.edit_project(ProjectEdit::Breakdown(BreakdownEdit::AddSceneElement {
        scene: "weg".into(),
        element: id,
    }));
    assert!(matches!(no_scene, Err(ServiceError::NoScene(_))));
}

#[test]
fn speaking_characters_come_from_the_book_and_others_are_added() {
    let mut s = open("breakdown-characters");
    let add_character = |s: &mut Session, scene: &str, id: &str| {
        s.edit_project(ProjectEdit::Breakdown(BreakdownEdit::AddSceneCharacter {
            scene: scene.into(),
            character: id.into(),
        }))
    };

    let view = s.overview().sheet.unwrap();
    assert_eq!(view.speaking.len(), 1);
    assert_eq!(view.speaking[0].id, "anna");
    assert!(view.present.is_empty());

    // Anna speaks in scene 14: nothing to add. In scene 23 she is silent company.
    assert!(matches!(
        add_character(&mut s, "aaaa", "anna"),
        Err(ServiceError::BadValue { .. })
    ));
    add_character(&mut s, "bbbb", "anna").unwrap();

    add_character(&mut s, "aaaa", "bob").unwrap();
    add_character(&mut s, "aaaa", "crowd").unwrap();
    assert_eq!(s.undo_step(), Some("AddSceneCharacter"));
    assert!(matches!(
        add_character(&mut s, "aaaa", "nobody"),
        Err(ServiceError::NoCharacter(_))
    ));

    let view = s.overview().sheet.unwrap();
    assert_eq!(view.speaking[0].id, "anna");
    let present: Vec<_> = view
        .present
        .iter()
        .map(|f| (f.id.as_str(), f.group))
        .collect();
    assert_eq!(present, [("bob", 0), ("crowd", 12)]);

    s.edit_project(ProjectEdit::Breakdown(
        BreakdownEdit::RemoveSceneCharacter {
            scene: "aaaa".into(),
            character: "bob".into(),
        },
    ))
    .unwrap();
    assert_eq!(s.overview().sheet.unwrap().present.len(), 1);
}

#[test]
fn the_breakdown_survives_a_save_and_its_files_follow_the_content() {
    let dir = make("breakdown-save");
    let mut s = Session::open(&dir).unwrap();
    assert!(!dir.join("breakdown").exists());

    let lamp = add(&mut s, ElementCategory::Prop, "Öllampe");
    point_at(&mut s, "o1", &lamp);
    to_scene(&mut s, "aaaa", &lamp);
    s.edit_project(ProjectEdit::Breakdown(BreakdownEdit::AddSceneCharacter {
        scene: "aaaa".into(),
        character: "crowd".into(),
    }))
    .unwrap();
    s.save(klappe_services::OnOutside::Ask).unwrap();

    assert!(dir.join("breakdown/elemente.toml").is_file());
    assert!(dir.join("breakdown/szenen/aaaa.toml").is_file());
    assert!(!dir.join("breakdown/szenen/bbbb.toml").exists());

    let again = Session::open(&dir).unwrap();
    assert_eq!(again.project().elements, s.project().elements);
    assert_eq!(sheet(&again, "aaaa"), sheet(&s, "aaaa"));
    assert_eq!(
        again.project().locations[0].set.objects[0].element,
        Some(lamp.clone())
    );

    // Empty again: the files go.
    let mut s = again;
    s.edit_project(ProjectEdit::Breakdown(BreakdownEdit::RemoveElement {
        id: lamp,
    }))
    .unwrap();
    s.edit_project(ProjectEdit::Breakdown(
        BreakdownEdit::RemoveSceneCharacter {
            scene: "aaaa".into(),
            character: "crowd".into(),
        },
    ))
    .unwrap();
    s.save(klappe_services::OnOutside::Ask).unwrap();
    assert!(!dir.join("breakdown/elemente.toml").exists());
    assert!(!dir.join("breakdown/szenen/aaaa.toml").exists());
}

#[test]
fn a_breakdown_file_changed_outside_is_taken_over() {
    let dir = make("breakdown-outside");
    let mut s = Session::open(&dir).unwrap();

    std::fs::create_dir_all(dir.join("breakdown")).unwrap();
    std::fs::write(
        dir.join("breakdown/elemente.toml"),
        "[[element]]\nid = \"x1\"\ncategory = \"prop\"\nname = \"Krug\"\n",
    )
    .unwrap();
    s.check().unwrap();

    assert_eq!(s.project().elements.len(), 1);
    assert_eq!(s.project().elements[0].name, "Krug");
}

/// Character ids come from the name: a deleted one must not linger in a sheet, or a later
/// character of that name would stand in the scene unasked.
#[test]
fn removing_a_character_takes_it_out_of_every_sheet() {
    let mut s = open("breakdown-character-gone");
    s.edit_project(ProjectEdit::Breakdown(BreakdownEdit::AddSceneCharacter {
        scene: "aaaa".into(),
        character: "crowd".into(),
    }))
    .unwrap();

    s.edit_project(ProjectEdit::RemoveCharacter { id: "crowd".into() })
        .unwrap();
    assert!(sheet(&s, "aaaa").characters.is_empty());

    s.undo();
    assert_eq!(sheet(&s, "aaaa").characters, ["crowd"]);
}

#[test]
fn adding_an_element_returns_its_id() {
    let mut s = open("add-id");

    let id = s.add_element(ElementCategory::Prop, "Brief").unwrap();

    let found = s.project().elements.iter().find(|e| e.id == id).unwrap();
    assert_eq!(found.name, "Brief");
    assert_eq!(s.undo_step(), Some("AddElement"));
}

#[test]
fn a_refused_element_returns_no_id() {
    let mut s = open("add-id-refused");
    s.add_element(ElementCategory::Prop, "Brief").unwrap();

    assert!(s.add_element(ElementCategory::Prop, "brief").is_err());
    assert!(s.add_element(ElementCategory::Prop, " ").is_err());
}

#[test]
fn adding_a_location_returns_its_id() {
    let mut s = open("add-loc-id");

    let id = s.add_location("Halle", (8.0, 5.5), 2.6, None).unwrap();

    let found = s.project().locations.iter().find(|l| l.id == id).unwrap();
    assert_eq!(found.name, "Halle");
}
