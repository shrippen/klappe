//! A new plan starts with the characters who speak in the scene (LH-09).
#![allow(clippy::unwrap_used)]

use std::path::Path;

use klappe_services::{ProjectEdit, Session};

const EXAMPLE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../testdata/beispiel.klappe"
);

fn example() -> Session {
    Session::open(Path::new(EXAMPLE)).unwrap()
}

/// The characters of the plan on view with the place of their one mark.
fn standing(s: &Session) -> Vec<(String, f64, f64)> {
    s.plan()
        .unwrap()
        .figures
        .iter()
        .map(|f| {
            assert_eq!(f.marks.len(), 1, "one mark each: {}", f.character);
            assert!(f.marks[0].via.is_empty(), "no way");
            (f.character.clone(), f.marks[0].at.x_m, f.marks[0].at.y_m)
        })
        .collect()
}

#[test]
fn a_new_plan_holds_the_speakers_side_by_side_at_the_edge() {
    let mut s = example();

    s.add_plan("").unwrap();

    let places = standing(&s);
    let names: Vec<_> = places.iter().map(|p| p.0.as_str()).collect();
    assert_eq!(names, ["elisabeth", "burgvogt"]);
    // In a row: the same distance from the edge, one metre apart, inside the set.
    assert_eq!(places[0].2, places[1].2);
    assert!((places[1].1 - places[0].1 - 1.0).abs() < 1e-9, "{places:?}");
    assert!(places.iter().all(|p| p.1 > 0.0 && p.1 < 20.0 && p.2 < 13.0));
    assert!(s.plan().unwrap().cameras.is_empty());
}

#[test]
fn it_is_one_undo_step() {
    let mut s = example();

    s.add_plan("").unwrap();
    s.undo();

    assert_eq!(s.scene().unwrap().plans.len(), 1);
}

#[test]
fn without_a_screenplay_the_plan_stays_empty() {
    let mut s = example();
    s.set_script_text("");

    s.add_plan("").unwrap();

    assert!(s.plan().unwrap().figures.is_empty());
}

/// A name that no character of the project has is not put, a known one is, once.
#[test]
fn the_first_plan_of_a_new_scene_holds_the_matched_speakers_once() {
    let mut s = example();
    let script = format!(
        "{}\n\nINT. HOF – NACHT #13#\n\nBURGVOGT\nHalt!\n\nUNBEKANNT\nWer da?\n\nburgvogt\nNoch einmal.\n",
        s.script_text()
    );
    s.set_script_text(&script);

    s.edit_project(ProjectEdit::AddScene {
        number: "13".into(),
        heading: "INT. HOF – NACHT".into(),
        location: "saengersaal".into(),
    })
    .unwrap();
    s.select_scene("szene-013").unwrap();

    let names: Vec<_> = standing(&s).into_iter().map(|p| p.0).collect();
    assert_eq!(names, ["burgvogt"]);
}

#[test]
fn a_scene_the_script_does_not_have_starts_empty() {
    let mut s = example();

    s.edit_project(ProjectEdit::AddScene {
        number: "14".into(),
        heading: "EXT. HOF – TAG".into(),
        location: "saengersaal".into(),
    })
    .unwrap();
    s.select_scene("szene-014").unwrap();

    assert!(s.plan().unwrap().figures.is_empty());
}
