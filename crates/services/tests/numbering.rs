//! How shots are numbered (LH-76), their short name (LH-110), picture (LH-74) and the colour
//! of their camera (LH-111), on a copy of the example: scene 12 with cameras A and B and five
//! shots (a 0-6, a 6-14, b 6-9, b 9-14, a 14-20).
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_services::{
    FigureColor, Layer, Numbering, PlanEdit, ProjectEdit, Session, ShotField, Visibility,
};

fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe")
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

fn open(name: &str) -> (Session, PathBuf) {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    copy_dir(&example(), &dir);

    (Session::open(&dir).unwrap(), dir)
}

fn numbers(s: &Session) -> Vec<String> {
    s.shots().into_iter().map(|r| r.number).collect()
}

fn scheme(s: &mut Session, numbering: Numbering) {
    s.edit_project(ProjectEdit::SetNumbering { numbering })
        .unwrap();
}

#[test]
fn the_scheme_is_a_project_setting_and_one_undo_step() {
    let (mut s, _) = open("numbering-scheme");
    assert_eq!(s.project().numbering, Numbering::Scene);
    assert_eq!(s.overview().numbering, "scene");
    assert_eq!(s.overview().numberings, ["scene", "camera", "group"]);

    scheme(&mut s, Numbering::Camera);
    assert_eq!(s.overview().numbering, "camera");
    s.undo();

    assert_eq!(s.project().numbering, Numbering::Scene);
}

#[test]
fn the_camera_scheme_counts_per_camera_and_sets_versions_off_by_a_hyphen() {
    let (mut s, _) = open("numbering-camera");
    scheme(&mut s, Numbering::Camera);

    s.renumber_shots().unwrap();
    assert_eq!(numbers(&s), ["12A.1", "12A.2", "12B.1", "12B.2", "12A.3"]);

    s.set_time(6.0);
    s.add_shot(Some("b")).unwrap();
    assert!(numbers(&s).contains(&"12B.3".to_owned()));

    let copy = s.version_shot("s12-3").unwrap();
    let again = s.version_shot(&copy).unwrap();
    let all = numbers(&s);
    assert!(all.contains(&"12B.1-A".to_owned()), "{all:?}");
    assert!(all.contains(&"12B.1-B".to_owned()), "{all:?}");
    assert_ne!(copy, again);

    // Renumbering keeps a version behind the number of its base.
    s.renumber_shots().unwrap();
    let all = numbers(&s);
    assert!(all.contains(&"12B.1-A".to_owned()), "{all:?}");
}

#[test]
fn the_group_scheme_gives_simultaneous_shots_one_number_and_a_camera_letter_each() {
    let (mut s, _) = open("numbering-group");
    scheme(&mut s, Numbering::Group);

    s.renumber_shots().unwrap();

    assert_eq!(numbers(&s), ["12.1A", "12.2A", "12.2B", "12.3B", "12.4A"]);

    // A shot of camera B over the time of 12.1A joins that group.
    s.set_time(0.0);
    let id = s.add_shot(Some("b")).unwrap();
    let row = s.shots().into_iter().find(|r| r.id == id).unwrap();
    assert_eq!(row.number, "12.1B");

    let version = s.version_shot(&id).unwrap();
    let row = s.shots().into_iter().find(|r| r.id == version).unwrap();
    assert_eq!(row.number, "12.1B-A");
}

#[test]
fn the_scene_scheme_keeps_its_letters_for_versions() {
    let (mut s, _) = open("numbering-scene");

    let copy = s.version_shot("s12-3").unwrap();

    let row = s.shots().into_iter().find(|r| r.id == copy).unwrap();
    assert_eq!(row.number, "12.3A");
}

#[test]
fn a_short_name_is_a_field_and_an_undo_step() {
    let (mut s, _) = open("numbering-short");

    s.edit_shot("s12-2", ShotField::ShortName, "Auftritt")
        .unwrap();
    let row = s.shots().into_iter().find(|r| r.id == "s12-2").unwrap();
    assert_eq!(row.short_name, "Auftritt");

    s.undo();
    let row = s.shots().into_iter().find(|r| r.id == "s12-2").unwrap();
    assert_eq!(row.short_name, "");
}

#[test]
fn a_storyboard_picture_is_copied_in_shown_removed_and_undone() {
    let (mut s, dir) = open("numbering-image");
    let picture = example().join("../skizze.png");

    s.set_shot_image("s12-1", Some(&picture)).unwrap();

    let row = s.shots().into_iter().find(|r| r.id == "s12-1").unwrap();
    assert_eq!(row.image, "bilder/skizze.png");
    assert!(dir.join("bilder/skizze.png").exists());

    s.undo();
    let row = s.shots().into_iter().find(|r| r.id == "s12-1").unwrap();
    assert_eq!(row.image, "");
    s.redo();

    s.set_shot_image("s12-1", None).unwrap();
    let row = s.shots().into_iter().find(|r| r.id == "s12-1").unwrap();
    assert_eq!(row.image, "");

    assert!(s
        .set_shot_image("s12-1", Some(&dir.join("projekt.toml")))
        .is_err());
    assert!(s.set_shot_image("nope", Some(&picture)).is_err());
}

#[test]
fn a_camera_has_a_colour_by_its_place_or_its_own() {
    let (mut s, _) = open("numbering-color");
    let color_of = |s: &Session, id: &str| {
        s.shots()
            .into_iter()
            .find(|r| r.id == id)
            .unwrap()
            .camera_color
    };

    assert_eq!(color_of(&s, "s12-1"), "red");
    assert_eq!(color_of(&s, "s12-3"), "orange");

    s.edit_plan(PlanEdit::SetCameraColor {
        camera: "a".into(),
        color: Some(FigureColor::Blue),
    })
    .unwrap();
    assert_eq!(color_of(&s, "s12-1"), "blue");

    s.edit_plan(PlanEdit::SetCameraColor {
        camera: "a".into(),
        color: None,
    })
    .unwrap();
    assert_eq!(color_of(&s, "s12-1"), "red");
}

#[test]
fn the_new_fields_and_the_scheme_survive_saving() {
    let (mut s, dir) = open("numbering-save");
    scheme(&mut s, Numbering::Group);
    s.edit_shot("s12-2", ShotField::ShortName, "Auftritt")
        .unwrap();
    s.set_shot_image("s12-2", Some(&example().join("../skizze.png")))
        .unwrap();
    s.edit_plan(PlanEdit::SetCameraColor {
        camera: "b".into(),
        color: Some(FigureColor::Green),
    })
    .unwrap();
    s.save().unwrap();

    let again = Session::open(&dir).unwrap();

    assert_eq!(again.project().numbering, Numbering::Group);
    let row = again.shots().into_iter().find(|r| r.id == "s12-2").unwrap();
    assert_eq!(
        (row.short_name.as_str(), row.image.as_str()),
        ("Auftritt", "bilder/skizze.png")
    );
    assert_eq!(
        again
            .shots()
            .into_iter()
            .find(|r| r.id == "s12-3")
            .unwrap()
            .camera_color,
        "green"
    );
    let header = std::fs::read_to_string(dir.join("projekt.toml")).unwrap();
    assert!(header.contains("numbering = \"group\""), "{header}");
}

#[test]
fn the_default_scheme_is_not_written() {
    let (mut s, dir) = open("numbering-default");
    s.edit_shot("s12-2", ShotField::Note, "x").unwrap();
    s.save().unwrap();

    let header = std::fs::read_to_string(dir.join("projekt.toml")).unwrap();

    // `scene_numbering` (DB-33) is another key.
    assert!(
        !header.lines().any(|l| l.starts_with("numbering")),
        "{header}"
    );
}

#[test]
fn the_plan_shows_a_picture_spot_for_each_camera_in_view() {
    let (mut s, _) = open("numbering-spots");
    s.set_time(7.0);

    let spots = s.camera_spots();
    assert_eq!(
        spots.iter().map(|p| p.id.as_str()).collect::<Vec<_>>(),
        ["a", "b"]
    );
    assert!(s.camera_picture_of("a", 96).is_some());
    assert!(s.camera_picture_of("nope", 96).is_none());

    s.set_track("a", Visibility::Hidden);
    assert_eq!(s.camera_spots().len(), 1);

    s.set_layer(Layer::Cameras, Visibility::Hidden);
    assert!(s.camera_spots().is_empty());
}

/// A version keeps its tail through a change of scheme and "Renumber", as the hint of the
/// project dialog advises: `12.3A` becomes `12B.1-A`, and back.
#[test]
fn versions_survive_a_change_of_scheme() {
    let (mut s, _) = open("numbering-switch");
    s.version_shot("s12-3").unwrap();
    assert!(numbers(&s).contains(&"12.3A".to_owned()));

    scheme(&mut s, Numbering::Camera);
    s.renumber_shots().unwrap();
    let all = numbers(&s);
    assert!(all.contains(&"12B.1".to_owned()), "{all:?}");
    assert!(all.contains(&"12B.1-A".to_owned()), "{all:?}");
    assert_eq!(all.len(), 6);

    scheme(&mut s, Numbering::Scene);
    s.renumber_shots().unwrap();
    let all = numbers(&s);
    assert!(all.contains(&"12.3".to_owned()), "{all:?}");
    assert!(all.contains(&"12.3A".to_owned()), "{all:?}");
}

#[test]
fn only_a_file_of_bilder_has_a_path() {
    let (s, dir) = open("numbering-path");

    assert_eq!(
        s.image_file("bilder/skizze.png"),
        Some(dir.join("bilder/skizze.png"))
    );
    assert_eq!(s.image_file("../x.png"), None);
    assert_eq!(s.image_file("/etc/passwd"), None);
}

#[test]
fn naming_a_shot_sets_both_fields_in_one_undo_step() {
    let (mut s, _) = open("numbering-name");

    s.name_shot("s12-2", "Auftritt", "Sie kommt herein")
        .unwrap();
    let row = s.shots().into_iter().find(|r| r.id == "s12-2").unwrap();
    assert_eq!(
        (row.short_name.as_str(), row.description.as_str()),
        ("Auftritt", "Sie kommt herein")
    );

    s.undo();
    let row = s.shots().into_iter().find(|r| r.id == "s12-2").unwrap();
    assert_eq!(row.short_name, "");
    assert_ne!(row.description, "Sie kommt herein");
    assert!(s.name_shot("nope", "a", "b").is_err());
}

/// In the group scheme the trailing letter is the camera, a version has a hyphen: leaving the
/// scheme must not read the camera as a version.
#[test]
fn leaving_the_group_scheme_keeps_cameras_and_versions_apart() {
    let (mut s, _) = open("numbering-from-group");
    scheme(&mut s, Numbering::Group);
    s.renumber_shots().unwrap();
    s.version_shot("s12-3").unwrap();
    assert_eq!(
        numbers(&s),
        ["12.1A", "12.2A", "12.2B", "12.2B-A", "12.3B", "12.4A"]
    );

    scheme(&mut s, Numbering::Camera);
    s.renumber_shots().unwrap();
    assert_eq!(
        numbers(&s),
        ["12A.1", "12A.2", "12B.1", "12B.1-A", "12B.2", "12A.3"]
    );

    scheme(&mut s, Numbering::Group);
    s.renumber_shots().unwrap();
    assert_eq!(
        numbers(&s),
        ["12.1A", "12.2A", "12.2B", "12.2B-A", "12.3B", "12.4A"]
    );

    scheme(&mut s, Numbering::Scene);
    s.renumber_shots().unwrap();
    assert_eq!(
        numbers(&s),
        ["12.1", "12.2", "12.3", "12.3A", "12.4", "12.5"]
    );
}

/// The first version of a camera A shot is `12.1A` in the scene scheme: not a camera.
#[test]
fn a_scene_version_with_the_letter_of_its_camera_stays_a_version() {
    let (mut s, _) = open("numbering-letter");
    s.version_shot("s12-1").unwrap();
    assert!(numbers(&s).contains(&"12.1A".to_owned()));

    scheme(&mut s, Numbering::Group);
    s.renumber_shots().unwrap();

    let all = numbers(&s);
    assert!(all.iter().any(|n| n.ends_with("-A")), "{all:?}");
}

/// A shot added in the group scheme and numbered again stays a shot of its own.
#[test]
fn a_shot_added_in_group_is_no_version_after_renumbering() {
    let (mut s, _) = open("numbering-reviewer");
    scheme(&mut s, Numbering::Group);
    s.set_time(0.0);
    s.add_shot(Some("b")).unwrap();

    s.renumber_shots().unwrap();

    let all = numbers(&s);
    assert!(all.contains(&"12.1A".to_owned()), "{all:?}");
    assert!(all.contains(&"12.1B".to_owned()), "{all:?}");
    assert!(all.iter().all(|n| !n.contains('-')), "{all:?}");
}

/// Versions on both cameras go through every scheme and back without changing what they are.
#[test]
fn versions_on_several_cameras_survive_every_scheme_change() {
    let (mut s, _) = open("numbering-all");
    s.version_shot("s12-1").unwrap();
    s.version_shot("s12-3").unwrap();
    s.version_shot("s12-3").unwrap();
    let was = |s: &Session| -> Vec<(String, String)> {
        s.shots().into_iter().map(|r| (r.id, r.camera)).collect()
    };
    let before = was(&s);

    let schemes = [Numbering::Camera, Numbering::Group, Numbering::Scene];
    for round in schemes
        .iter()
        .chain(&[Numbering::Group, Numbering::Camera, Numbering::Scene])
    {
        scheme(&mut s, *round);
        s.renumber_shots().unwrap();
        let all = numbers(&s);
        let versions = all
            .iter()
            .filter(|n| {
                n.ends_with(char::is_uppercase) && *round == Numbering::Scene || n.contains('-')
            })
            .count();
        assert_eq!(all.len(), 8, "{round:?} {all:?}");
        assert_eq!(versions, 3, "{round:?} {all:?}");
    }

    assert_eq!(was(&s), before);
    assert_eq!(
        numbers(&s),
        ["12.1", "12.1A", "12.2", "12.3", "12.3A", "12.3B", "12.4", "12.5"]
    );
}

/// A scene number with a letter does not confuse any scheme.
#[test]
fn a_scene_number_with_a_letter_numbers_every_scheme() {
    let (mut s, _) = open("numbering-12a");
    s.edit_project(ProjectEdit::EditScene {
        id: "m4djbtj6".into(),
        number: "12A".into(),
        heading: s.scene().unwrap().heading.clone(),
        location: s.scene().unwrap().location.clone(),
    })
    .unwrap();
    s.version_shot("s12-3").unwrap();

    s.renumber_shots().unwrap();
    assert_eq!(
        numbers(&s),
        ["12A.1", "12A.2", "12A.3", "12A.3A", "12A.4", "12A.5"]
    );

    scheme(&mut s, Numbering::Camera);
    s.renumber_shots().unwrap();
    assert_eq!(
        numbers(&s),
        ["12AA.1", "12AA.2", "12AB.1", "12AB.1-A", "12AB.2", "12AA.3"]
    );

    scheme(&mut s, Numbering::Group);
    s.renumber_shots().unwrap();
    assert_eq!(
        numbers(&s),
        ["12A.1A", "12A.2A", "12A.2B", "12A.2B-A", "12A.3B", "12A.4A"]
    );

    scheme(&mut s, Numbering::Scene);
    s.renumber_shots().unwrap();
    assert_eq!(
        numbers(&s),
        ["12A.1", "12A.2", "12A.3", "12A.3A", "12A.4", "12A.5"]
    );
}

/// What a version is lives in the shot, not in its number: it survives saving, and a version
/// whose base was deleted is numbered as a base.
#[test]
fn a_version_is_stored_as_a_reference_and_an_orphan_becomes_a_base() {
    let (mut s, dir) = open("numbering-ref");
    let id = s.version_shot("s12-3").unwrap();
    s.save().unwrap();
    let text = std::fs::read_to_string(dir.join("szenen/m4djbtj6.json")).unwrap();
    assert!(text.contains("\"version_of\""), "{text}");

    let mut s = Session::open(&dir).unwrap();
    scheme(&mut s, Numbering::Camera);
    s.renumber_shots().unwrap();
    assert!(numbers(&s).contains(&"12B.1-A".to_owned()));

    s.delete_shot("s12-3").unwrap();
    s.renumber_shots().unwrap();
    let all = numbers(&s);
    assert_eq!(all, ["12A.1", "12A.2", "12B.1", "12B.2", "12A.3"], "{id}");
}

/// Deleting a base shot promotes its first version, so a version of a survivor hangs on a real
/// base and is no shot of its own after renumbering.
#[test]
fn a_version_of_a_survivor_of_a_deleted_base_is_still_a_version() {
    let (mut s, _) = open("numbering-orphan");
    let first = s.version_shot("s12-3").unwrap();
    let second = s.version_shot("s12-3").unwrap();
    s.delete_shot("s12-3").unwrap();
    s.renumber_shots().unwrap();
    assert_eq!(
        numbers(&s),
        ["12.1", "12.2", "12.3", "12.3A", "12.4", "12.5"]
    );

    s.version_shot(&second).unwrap();
    s.renumber_shots().unwrap();

    assert_eq!(
        numbers(&s),
        ["12.1", "12.2", "12.3", "12.3A", "12.3B", "12.4", "12.5"],
        "{first}"
    );
}

/// A version has the camera of its base: the camera of a version is refused, the base's moves
/// all its versions.
#[test]
fn a_version_keeps_the_camera_of_its_base() {
    let (mut s, _) = open("numbering-camera-of-version");
    let version = s.version_shot("s12-3").unwrap();
    let camera_of =
        |s: &Session, id: &str| s.shots().into_iter().find(|r| r.id == id).unwrap().camera;

    assert!(s.edit_shot(&version, ShotField::Camera, "a").is_err());
    assert_eq!(camera_of(&s, &version), "b");

    s.edit_shot("s12-3", ShotField::Camera, "a").unwrap();
    assert_eq!(camera_of(&s, "s12-3"), "a");
    assert_eq!(camera_of(&s, &version), "a");
}

/// FORMAT.md: a `version_of` that names no shot makes the shot a base. Loading says so in the
/// data, so nothing else reads a base as a version.
#[test]
fn a_version_of_no_shot_is_a_base_after_loading() {
    let (mut s, dir) = open("numbering-orphan-load");
    s.version_shot("s12-3").unwrap();
    s.save().unwrap();

    let file = dir.join("szenen/m4djbtj6.json");
    let text = std::fs::read_to_string(&file).unwrap();
    assert!(text.contains("\"shot\": \"s12-3\""), "{text}");
    std::fs::write(
        &file,
        text.replace("\"shot\": \"s12-3\"", "\"shot\": \"gone\""),
    )
    .unwrap();

    let mut s = Session::open(&dir).unwrap();
    s.save().unwrap();
    let saved = std::fs::read_to_string(&file).unwrap();

    assert!(!saved.contains("version_of"), "{saved}");
}

/// The promoted base takes the number of the deleted one, its versions follow it: a new
/// version then carries on from them, without renumbering first.
#[test]
fn a_promoted_base_takes_the_number_of_the_deleted_one() {
    let (mut s, _) = open("numbering-promote");
    s.version_shot("s12-3").unwrap();
    let second = s.version_shot("s12-3").unwrap();
    assert_eq!(
        numbers(&s),
        ["12.1", "12.2", "12.3", "12.3A", "12.3B", "12.4", "12.5"]
    );

    s.delete_shot("s12-3").unwrap();
    assert_eq!(
        numbers(&s),
        ["12.1", "12.2", "12.3", "12.3A", "12.4", "12.5"]
    );

    s.version_shot(&second).unwrap();
    assert_eq!(
        numbers(&s),
        ["12.1", "12.2", "12.3", "12.3A", "12.3B", "12.4", "12.5"]
    );
}

/// The refusal of a version's camera says why, not the id of the shot.
#[test]
fn a_version_says_why_its_camera_is_refused() {
    let (mut s, _) = open("numbering-camera-reason");
    let version = s.version_shot("s12-3").unwrap();
    let row = |s: &Session| s.shots().into_iter().find(|r| r.id == version).unwrap();

    assert!(row(&s).is_version);
    assert!(
        !s.shots()
            .into_iter()
            .find(|r| r.id == "s12-3")
            .unwrap()
            .is_version
    );

    let refused = s.edit_shot(&version, ShotField::Camera, "a").unwrap_err();
    let text = klappe_services::error_text(klappe_services::Language::English, &refused);
    assert!(!text.contains(&version), "{text}");
    assert!(text.contains("base"), "{text}");
}

/// A restored backup is a load like opening: a `version_of` naming no shot is cleared, so a
/// shot added later with that id does not turn the shot into its version.
#[test]
fn a_restored_backup_clears_a_version_of_no_shot() {
    let (mut s, dir) = open("numbering-restore");
    let version = s.version_shot("s12-3").unwrap();
    s.autosave().unwrap();
    drop(s);

    let file = dir.join(".sicherung/szenen/m4djbtj6.json");
    let text = std::fs::read_to_string(&file).unwrap();
    assert!(text.contains("\"shot\": \"s12-3\""), "{text}");
    let gone = text.replace("\"shot\": \"s12-3\"", "\"shot\": \"s12-7\"");
    std::fs::write(&file, gone).unwrap();

    let mut s = Session::open(&dir).unwrap();
    s.restore_backup().unwrap();
    assert_eq!(s.add_shot(None).unwrap(), "s12-7");

    let row = s.shots().into_iter().find(|r| r.id == version).unwrap();
    assert!(!row.is_version);
}
