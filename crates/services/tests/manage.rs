//! Projects, scenes, locations and characters (LH-01 to LH-03), on a copy of the example
//! project: scene `szene-012` ("12") at `saengersaal`, characters `elisabeth` and `burgvogt`
//! (both in Plan A).
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_services::{Field, FigureColor, ProjectEdit, ServiceError, Session};

fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe")
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
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

fn open(name: &str) -> Session {
    let dir = scratch(name);
    copy_dir(&example(), &dir);
    Session::open(&dir).unwrap()
}

fn add_scene(number: &str, location: &str) -> ProjectEdit {
    ProjectEdit::AddScene {
        number: number.into(),
        heading: "INT. HOF – NACHT".into(),
        location: location.into(),
    }
}

fn add_location(name: &str) -> ProjectEdit {
    ProjectEdit::AddLocation {
        name: name.into(),
        width_m: 8.0,
        depth_m: 5.5,
    }
}

fn add_character(name: &str, short: &str) -> ProjectEdit {
    ProjectEdit::AddCharacter {
        name: name.into(),
        short: short.into(),
        color: FigureColor::Green,
        group: None,
    }
}

fn scene_ids(s: &Session) -> Vec<String> {
    s.project().scenes.iter().map(|x| x.id.clone()).collect()
}

#[test]
fn creates_a_project_in_a_missing_folder() {
    let dir = scratch("manage-create");

    let s = Session::create(&dir, "  Neue Burg ").unwrap();

    assert_eq!(s.project().title, "Neue Burg");
    assert!(s.project().scenes.is_empty());
    assert!(!s.is_dirty());
    assert_eq!(Session::open(&dir).unwrap().project(), s.project());
}

#[test]
fn creates_a_project_in_an_empty_folder() {
    let dir = scratch("manage-create-empty");
    std::fs::create_dir_all(&dir).unwrap();

    assert!(Session::create(&dir, "Film").is_ok());
}

#[test]
fn create_refuses_a_folder_with_files() {
    let dir = scratch("manage-create-full");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("notizen.txt"), "x").unwrap();

    let err = Session::create(&dir, "Film").unwrap_err();

    assert!(matches!(err, ServiceError::NotEmpty(p) if p == dir));
    assert!(!dir.join("projekt.toml").exists());
}

#[test]
fn create_refuses_an_empty_title() {
    let err = Session::create(&scratch("manage-create-title"), "  ").unwrap_err();

    assert!(matches!(
        err,
        ServiceError::BadValue {
            field: Field::Title,
            ..
        }
    ));
}

#[test]
fn adds_a_scene_with_an_empty_plan() {
    let mut s = open("manage-add-scene");

    s.edit_project(add_scene("13", "saengersaal")).unwrap();

    let scene = s.project().scene("szene-013").unwrap();
    assert_eq!(scene.number, "13");
    assert_eq!(scene.heading, "INT. HOF – NACHT");
    assert_eq!(scene.location, "saengersaal");
    assert_eq!(scene.plans.len(), 1);
    assert_eq!(scene.plans[0].name, "Plan A");
    assert!(scene.plans[0].figures.is_empty());
    assert!(s.is_dirty());
}

#[test]
fn scene_ids_come_from_the_number_and_stay_unique() {
    let mut s = open("manage-scene-ids");

    s.edit_project(add_scene("12A", "saengersaal")).unwrap();
    s.edit_project(add_scene("2", "saengersaal")).unwrap();

    assert_eq!(scene_ids(&s), ["szene-002", "szene-012", "szene-012a"]);
}

#[test]
fn a_number_is_taken_once_whatever_its_case() {
    let mut s = open("manage-number-taken");

    let err = s.edit_project(add_scene("12", "saengersaal")).unwrap_err();
    assert!(matches!(err, ServiceError::NumberTaken(n) if n == "12"));

    s.edit_project(add_scene("12A", "saengersaal")).unwrap();
    let err = s.edit_project(add_scene("12a", "saengersaal")).unwrap_err();
    assert!(matches!(err, ServiceError::NumberTaken(_)));
}

#[test]
fn a_scene_needs_a_number_and_a_known_location() {
    let mut s = open("manage-scene-bad");

    let err = s.edit_project(add_scene("  ", "saengersaal")).unwrap_err();
    assert!(matches!(
        err,
        ServiceError::BadValue {
            field: Field::Number,
            ..
        }
    ));

    let err = s.edit_project(add_scene("13", "nirgends")).unwrap_err();
    assert!(matches!(err, ServiceError::NoLocation(id) if id == "nirgends"));

    assert!(!s.is_dirty());
}

#[test]
fn changing_the_heading_keeps_the_number() {
    let mut s = open("manage-heading");

    s.edit_project(ProjectEdit::EditScene {
        id: "szene-012".into(),
        number: "12".into(),
        heading: " INT. SÄNGERSAAL – NACHT ".into(),
        location: "saengersaal".into(),
    })
    .unwrap();

    let scene = s.project().scene("szene-012").unwrap();
    assert_eq!(scene.number, "12");
    assert_eq!(scene.heading, "INT. SÄNGERSAAL – NACHT");
    // The script's scene 12 still finds the plan: its lines are linked.
    assert!(!s.script_lines().is_empty());
}

#[test]
fn renumbers_a_scene_but_not_onto_another() {
    let mut s = open("manage-renumber");
    s.edit_project(add_scene("13", "saengersaal")).unwrap();

    let err = s
        .edit_project(ProjectEdit::EditScene {
            id: "szene-013".into(),
            number: "12".into(),
            heading: "x".into(),
            location: "saengersaal".into(),
        })
        .unwrap_err();
    assert!(matches!(err, ServiceError::NumberTaken(_)));

    s.edit_project(ProjectEdit::EditScene {
        id: "szene-013".into(),
        number: "14".into(),
        heading: "x".into(),
        location: "saengersaal".into(),
    })
    .unwrap();

    // The id stays: it is a file name.
    assert_eq!(s.project().scene("szene-013").unwrap().number, "14");
}

#[test]
fn edits_a_scene_and_moves_it_in_one_undo_step() {
    let mut s = open("manage-move");
    s.edit_project(add_location("Burghof")).unwrap();

    s.edit_project(ProjectEdit::EditScene {
        id: "szene-012".into(),
        number: "13".into(),
        heading: "EXT. BURGHOF".into(),
        location: "burghof".into(),
    })
    .unwrap();
    let scene = s.project().scene("szene-012").unwrap();
    assert_eq!(scene.number, "13");
    assert_eq!(scene.location, "burghof");

    s.undo();
    let scene = s.project().scene("szene-012").unwrap();
    assert_eq!(scene.number, "12");
    assert_eq!(scene.heading, "INT. SÄNGERSAAL – TAG");
    assert_eq!(scene.location, "saengersaal");

    let err = s
        .edit_project(ProjectEdit::EditScene {
            id: "szene-012".into(),
            number: "12".into(),
            heading: "x".into(),
            location: "nirgends".into(),
        })
        .unwrap_err();
    assert!(matches!(err, ServiceError::NoLocation(_)));
    assert_eq!(
        s.project().scene("szene-012").unwrap().heading,
        "INT. SÄNGERSAAL – TAG"
    );
}

#[test]
fn removes_a_scene_and_undoes() {
    let mut s = open("manage-remove-scene");

    s.edit_project(ProjectEdit::RemoveScene {
        id: "szene-012".into(),
    })
    .unwrap();
    assert!(s.project().scenes.is_empty());
    assert!(s.scene().is_none());

    s.undo();
    assert_eq!(scene_ids(&s), ["szene-012"]);
    assert!(s.scene().is_some());

    let err = s
        .edit_project(ProjectEdit::RemoveScene { id: "x".into() })
        .unwrap_err();
    assert!(matches!(err, ServiceError::NoScene(_)));
}

#[test]
fn the_view_stays_on_its_scene() {
    let mut s = open("manage-view");
    // Sorts before szene-012: the current scene moves down the list.
    s.edit_project(add_scene("2", "saengersaal")).unwrap();
    assert_eq!(s.scene().unwrap().id, "szene-012");

    s.undo();
    assert_eq!(s.scene().unwrap().id, "szene-012");

    s.redo();
    assert_eq!(s.scene().unwrap().id, "szene-012");
}

#[test]
fn removing_the_scene_on_view_leaves_a_valid_view() {
    let mut s = open("manage-view-gone");
    s.edit_project(add_scene("13", "saengersaal")).unwrap();

    s.edit_project(ProjectEdit::RemoveScene {
        id: "szene-012".into(),
    })
    .unwrap();

    assert_eq!(s.scene().unwrap().id, "szene-013");
    assert!(s.plan().is_some());
}

#[test]
fn adds_a_location_with_an_id_from_its_name() {
    let mut s = open("manage-add-location");

    s.edit_project(add_location("Große Halle")).unwrap();
    s.edit_project(add_location("Grosse Halle")).unwrap();

    let ids: Vec<_> = s
        .project()
        .locations
        .iter()
        .map(|l| l.id.as_str())
        .collect();
    assert_eq!(ids, ["grosse-halle", "grosse-halle-2", "saengersaal"]);

    let hall = s.project().location("grosse-halle").unwrap();
    assert_eq!(hall.name, "Große Halle");
    assert_eq!((hall.set.width_m, hall.set.depth_m), (8.0, 5.5));
    assert!(hall.set.walls.is_empty());
}

#[test]
fn a_location_gets_a_name_and_a_size() {
    let mut s = open("manage-location-bad");

    let err = s.edit_project(add_location(" ")).unwrap_err();
    assert!(matches!(
        err,
        ServiceError::BadValue {
            field: Field::Name,
            ..
        }
    ));

    for (width_m, depth_m, field) in [
        (0.0, 5.0, Field::Width),
        (5.0, -1.0, Field::Depth),
        (f64::NAN, 5.0, Field::Width),
        (5.0, f64::INFINITY, Field::Depth),
        (5000.0, 5.0, Field::Width),
    ] {
        let err = s
            .edit_project(ProjectEdit::AddLocation {
                name: "Halle".into(),
                width_m,
                depth_m,
            })
            .unwrap_err();
        assert!(matches!(err, ServiceError::BadValue { field: f, .. } if f == field));
    }

    assert_eq!(s.project().locations.len(), 1);
}

#[test]
fn renames_and_resizes_a_location_keeping_id_and_set() {
    let mut s = open("manage-edit-location");
    let walls = s.project().location("saengersaal").unwrap().set.walls.len();

    s.edit_project(ProjectEdit::EditLocation {
        id: "saengersaal".into(),
        name: "Rittersaal".into(),
        width_m: 22.0,
        depth_m: 14.0,
    })
    .unwrap();

    let l = s.project().location("saengersaal").unwrap();
    assert_eq!(l.name, "Rittersaal");
    assert_eq!((l.set.width_m, l.set.depth_m), (22.0, 14.0));
    assert_eq!(l.set.walls.len(), walls);

    s.undo();
    assert_eq!(
        s.project().location("saengersaal").unwrap().name,
        "Sängersaal"
    );
}

#[test]
fn a_location_in_use_cannot_go() {
    let mut s = open("manage-location-in-use");

    let err = s
        .edit_project(ProjectEdit::RemoveLocation {
            id: "saengersaal".into(),
        })
        .unwrap_err();

    assert!(matches!(
        err,
        ServiceError::LocationInUse { id, scenes: 1 } if id == "saengersaal"
    ));
    assert!(s.project().location("saengersaal").is_some());
}

#[test]
fn an_unused_location_goes_and_comes_back_with_undo() {
    let mut s = open("manage-location-remove");
    s.edit_project(add_location("Burghof")).unwrap();

    s.edit_project(ProjectEdit::RemoveLocation {
        id: "burghof".into(),
    })
    .unwrap();
    assert!(s.project().location("burghof").is_none());

    s.undo();
    assert!(s.project().location("burghof").is_some());

    s.redo();
    assert!(s.project().location("burghof").is_none());
}

#[test]
fn adds_and_changes_a_character() {
    let mut s = open("manage-character");

    s.edit_project(add_character("Graf Dietrich", "GD"))
        .unwrap();
    s.edit_project(add_character("Graf Dietrich", "G2"))
        .unwrap();

    let ids: Vec<_> = s
        .project()
        .characters
        .iter()
        .map(|c| c.id.as_str())
        .collect();
    assert_eq!(
        ids,
        ["elisabeth", "burgvogt", "graf-dietrich", "graf-dietrich-2"]
    );

    s.edit_project(ProjectEdit::EditCharacter {
        id: "graf-dietrich".into(),
        name: "Dietrich".into(),
        short: "D".into(),
        color: FigureColor::Red,
        group: None,
    })
    .unwrap();

    let c = s.project().character("graf-dietrich").unwrap();
    assert_eq!((c.name.as_str(), c.short.as_str()), ("Dietrich", "D"));
    assert_eq!(c.color, FigureColor::Red);
}

#[test]
fn a_character_needs_a_name_and_a_short_label() {
    let mut s = open("manage-character-bad");

    let err = s.edit_project(add_character("", "X")).unwrap_err();
    assert!(matches!(
        err,
        ServiceError::BadValue {
            field: Field::Name,
            ..
        }
    ));

    for short in ["", "  ", "ABCD"] {
        let err = s.edit_project(add_character("Graf", short)).unwrap_err();
        assert!(matches!(
            err,
            ServiceError::BadValue {
                field: Field::Short,
                ..
            }
        ));
    }

    let err = s
        .edit_project(ProjectEdit::EditCharacter {
            id: "nobody".into(),
            name: "x".into(),
            short: "x".into(),
            color: FigureColor::Blue,
            group: None,
        })
        .unwrap_err();
    assert!(matches!(err, ServiceError::NoCharacter(_)));
}

fn group_edit(id: &str, group: Option<u32>) -> ProjectEdit {
    ProjectEdit::EditCharacter {
        id: id.into(),
        name: "Gäste".into(),
        short: "G".into(),
        color: FigureColor::Cyan,
        group,
    }
}

#[test]
fn a_group_is_set_changed_and_cleared_with_undo() {
    let mut s = open("manage-group");

    s.edit_project(ProjectEdit::AddCharacter {
        name: "Gäste".into(),
        short: "G".into(),
        color: FigureColor::Cyan,
        group: Some(12),
    })
    .unwrap();
    assert_eq!(s.project().character("gaeste").unwrap().group, Some(12));

    s.edit_project(group_edit("gaeste", Some(999))).unwrap();
    assert_eq!(s.project().character("gaeste").unwrap().group, Some(999));

    s.edit_project(group_edit("gaeste", None)).unwrap();
    assert_eq!(s.project().character("gaeste").unwrap().group, None);

    s.undo();
    assert_eq!(s.project().character("gaeste").unwrap().group, Some(999));
}

#[test]
fn a_group_has_two_to_999_people() {
    let mut s = open("manage-group-bounds");
    s.edit_project(add_character("Gäste", "G")).unwrap();

    for count in [0, 1, 1000] {
        let err = s
            .edit_project(group_edit("gaeste", Some(count)))
            .unwrap_err();
        assert!(matches!(
            err,
            ServiceError::BadValue {
                field: Field::Group,
                ..
            }
        ));
    }
    assert_eq!(s.project().character("gaeste").unwrap().group, None);

    s.edit_project(group_edit("gaeste", Some(2))).unwrap();
    assert_eq!(s.project().character("gaeste").unwrap().group, Some(2));

    let err = s
        .edit_project(ProjectEdit::AddCharacter {
            name: "Wachen".into(),
            short: "W".into(),
            color: FigureColor::Red,
            group: Some(1),
        })
        .unwrap_err();
    assert!(matches!(
        err,
        ServiceError::BadValue {
            field: Field::Group,
            ..
        }
    ));
}

#[test]
fn a_character_in_a_plan_cannot_go() {
    let mut s = open("manage-character-in-use");

    let err = s
        .edit_project(ProjectEdit::RemoveCharacter {
            id: "elisabeth".into(),
        })
        .unwrap_err();

    assert!(matches!(
        err,
        ServiceError::CharacterInUse { id, plans: 1 } if id == "elisabeth"
    ));
    assert!(s.project().character("elisabeth").is_some());
}

#[test]
fn an_unused_character_goes() {
    let mut s = open("manage-character-remove");
    s.edit_project(add_character("Graf", "G")).unwrap();

    s.edit_project(ProjectEdit::RemoveCharacter { id: "graf".into() })
        .unwrap();
    assert!(s.project().character("graf").is_none());

    s.undo();
    assert!(s.project().character("graf").is_some());
}

#[test]
fn every_change_undoes_redoes_and_marks_dirty() {
    let mut s = open("manage-undo");
    assert!(!s.can_undo());

    s.edit_project(add_location("Burghof")).unwrap();
    s.edit_project(add_character("Graf", "G")).unwrap();
    s.edit_project(add_scene("13", "burghof")).unwrap();
    assert!(s.is_dirty());

    for _ in 0..3 {
        s.undo();
    }
    assert!(!s.can_undo());
    assert_eq!(s.project().locations.len(), 1);
    assert_eq!(s.project().characters.len(), 2);
    assert_eq!(s.project().scenes.len(), 1);

    for _ in 0..3 {
        s.redo();
    }
    assert!(!s.can_redo());
    assert!(s.project().scene("szene-013").is_some());
}

#[test]
fn a_refused_edit_takes_no_undo_step() {
    let mut s = open("manage-refused");

    s.edit_project(add_scene("12", "saengersaal")).unwrap_err();

    assert!(!s.can_undo());
    assert!(!s.is_dirty());
}

#[test]
fn saves_and_loads_again() {
    let mut s = open("manage-save");
    s.edit_project(add_location("Burghof")).unwrap();
    s.edit_project(add_character("Graf", "G")).unwrap();
    s.edit_project(add_scene("13", "burghof")).unwrap();
    s.edit_project(ProjectEdit::RemoveScene {
        id: "szene-012".into(),
    })
    .unwrap();

    s.save().unwrap();

    let again = Session::open(s.dir()).unwrap();
    assert_eq!(again.project(), s.project());
    assert_eq!(scene_ids(&again), ["szene-013"]);
    assert!(!s.dir().join("szenen/szene-012.json").exists());
}

#[test]
fn a_new_project_fills_up_and_reloads() {
    let dir = scratch("manage-from-scratch");
    let mut s = Session::create(&dir, "Film").unwrap();

    s.edit_project(add_location("Halle")).unwrap();
    s.edit_project(add_scene("1", "halle")).unwrap();
    s.edit_project(add_character("Anna", "A")).unwrap();
    s.save().unwrap();

    let again = Session::open(&dir).unwrap();
    assert_eq!(again.project(), s.project());
    assert_eq!(again.scene().unwrap().number, "1");
}

/// The example project plus a location `Hof` and a scene `Szene-099` (ids that differ from
/// the generated ones only in case).
fn open_with_capitals(name: &str) -> Session {
    let dir = scratch(name);
    copy_dir(&example(), &dir);

    let location = std::fs::read_to_string(dir.join("drehorte/saengersaal.toml")).unwrap();
    let location = location.replace("id = \"saengersaal\"", "id = \"Hof\"");
    std::fs::write(dir.join("drehorte/Hof.toml"), location).unwrap();

    let scene = std::fs::read_to_string(dir.join("szenen/szene-012.json")).unwrap();
    let scene = scene
        .replace("\"id\": \"szene-012\"", "\"id\": \"Szene-099\"")
        .replace("\"number\": \"12\"", "\"number\": \"x\"");
    std::fs::write(dir.join("szenen/Szene-099.json"), scene).unwrap();

    Session::open(&dir).unwrap()
}

#[test]
fn a_new_location_id_differs_from_others_in_more_than_case() {
    let mut s = open_with_capitals("manage-case-location");

    s.edit_project(add_location("Hof")).unwrap();

    let ids: Vec<_> = s
        .project()
        .locations
        .iter()
        .map(|l| l.id.to_lowercase())
        .collect();
    assert_eq!(ids, ["hof", "hof-2", "saengersaal"]);
    s.save().unwrap();
    assert_eq!(s.autosave().unwrap(), klappe_services::Autosave::Clean);
}

#[test]
fn a_new_scene_id_differs_from_others_in_more_than_case() {
    let mut s = open_with_capitals("manage-case-scene");

    s.edit_project(add_scene("99", "saengersaal")).unwrap();

    let ids: Vec<_> = scene_ids(&s).iter().map(|i| i.to_lowercase()).collect();
    assert_eq!(ids, ["szene-099", "szene-012", "szene-099-2"]);
    s.save().unwrap();
}
