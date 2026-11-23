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
    // 0,8 m in from the top left corner of the 20 x 13 m set, 1 m apart.
    assert_eq!(
        places,
        [
            ("elisabeth".to_owned(), 0.8, 0.8),
            ("burgvogt".to_owned(), 1.8, 0.8)
        ]
    );
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
        location: Some("saengersaal".into()),
    })
    .unwrap();
    let id = s.project().scenes.last().unwrap().id.clone();
    s.select_scene(&id).unwrap();

    let names: Vec<_> = standing(&s).into_iter().map(|p| p.0).collect();
    assert_eq!(names, ["burgvogt"]);
}

#[test]
fn a_scene_the_script_does_not_have_starts_empty() {
    let mut s = example();

    s.edit_project(ProjectEdit::AddScene {
        number: "14".into(),
        heading: "EXT. HOF – TAG".into(),
        location: Some("saengersaal".into()),
    })
    .unwrap();
    let id = s.project().scenes.last().unwrap().id.clone();
    s.select_scene(&id).unwrap();

    assert!(s.plan().unwrap().figures.is_empty());
}

/// A set 2.6 m wide leaves 1 m between the edge marks, so three
/// speakers close up to 0,5 m: 0,8 / 1,3 / 1,8.
#[test]
fn a_narrow_set_closes_the_row_up() {
    let mut s = example();
    let script = format!(
        "{}\n\nINT. FLUR – TAG #15#\n\nELISABETH\nA\n\nBURGVOGT\nB\n\nGRAF\nC\n",
        s.script_text()
    );
    s.set_script_text(&script);
    s.edit_project(ProjectEdit::AddCharacter {
        name: "Graf".into(),
        short: "G".into(),
        color: klappe_services::FigureColor::Green,
        group: None,
        size_m: None,
    })
    .unwrap();
    s.edit_project(ProjectEdit::AddLocation {
        name: "Flur".into(),
        width_m: 2.6,
        depth_m: 5.0,
        wall_height_m: 2.6,
        site: None,
    })
    .unwrap();
    s.edit_project(ProjectEdit::AddScene {
        number: "15".into(),
        heading: "INT. FLUR – TAG".into(),
        location: Some("flur".into()),
    })
    .unwrap();
    let id = s.project().scenes.last().unwrap().id.clone();
    s.select_scene(&id).unwrap();

    let xs: Vec<f64> = standing(&s).iter().map(|p| p.1).collect();

    assert_eq!(xs.len(), 3);
    for (x, want) in xs.iter().zip([0.8, 1.3, 1.8]) {
        assert!((x - want).abs() < 1e-9, "{xs:?}");
    }
}
