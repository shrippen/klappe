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

    assert!(!header.contains("numbering"), "{header}");
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
