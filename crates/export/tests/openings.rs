//! LH-10: a wall with a door and a window prints in pieces, with the door's leaf and swing
//! and the window's two lines in the gaps, as the screen shows them.

// Test helpers may unwrap; a failure should stop the test.
#![allow(clippy::unwrap_used)]

use klappe_export::{pdf, png, svg, LampTexts, Orientation, Page, Paper, ShotTexts, Texts, Theme};
use klappe_model::{FloorPlan, Opening, OpeningKind, Point, Project, SetPlan, Wall};
use klappe_stellprobe::render;

fn page() -> Page {
    let texts = Texts {
        figure: "Figur".into(),
        camera: "Kamera".into(),
        light: "Licht".into(),
        axis: "Achse".into(),
        path: "Weg".into(),
        mark: "Mark".into(),
        unit: "m".into(),
        decimal_separator: ',',
        shots: ShotTexts::default(),
        drawing: Default::default(),
        lamps: LampTexts::default(),
    };

    Page::new(Paper::A4, Orientation::Landscape, "Öffnungen", texts)
}

fn lines_with<'a>(svg: &'a str, class: &str) -> Vec<&'a str> {
    let needle = format!("class=\"{class}\"");
    svg.lines().filter(|l| l.contains(&needle)).collect()
}

fn set(openings: Vec<Opening>) -> SetPlan {
    let mut wall = Wall::new(Point::new(0.0, 0.0), Point::new(20.0, 0.0), 0.2);
    wall.openings = openings;

    SetPlan {
        width_m: 20.0,
        depth_m: 13.0,
        walls: vec![wall],
        ..SetPlan::default()
    }
}

#[test]
fn openings_are_printed_in_the_gaps() {
    let with = set(vec![
        Opening::new(OpeningKind::Door, 5.0, 0.9),
        Opening::new(OpeningKind::Window, 12.0, 1.2),
    ]);
    let project = Project::new("Test");
    let plan = FloorPlan::new("a", "Plan A");
    let bare = render(&project, &set(vec![]), &plan, 0.0, None, &|_| None);
    let list = render(&project, &with, &plan, 0.0, None, &|_| None);

    let out = svg(&list, &page(), Theme::Light).unwrap();
    let pdf_bytes = pdf(&list, &page(), Theme::Light).unwrap();
    let png_with = png(&list, &page(), Theme::Light).unwrap();
    let png_bare = png(&bare, &page(), Theme::Light).unwrap();

    assert_eq!(lines_with(&out, "item wall").len(), 3);
    assert_eq!(lines_with(&out, "item door").len(), 1);
    assert_eq!(lines_with(&out, "item window").len(), 2);
    assert!(pdf_bytes.starts_with(b"%PDF"));
    assert_ne!(png_with, png_bare);
}
