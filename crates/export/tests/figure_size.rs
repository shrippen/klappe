//! A figure is printed in its size (LH-25).
#![allow(clippy::unwrap_used)]

use klappe_export::{svg, Orientation, Page, Paper, Texts, Theme};
use klappe_model::{
    Arrival, Character, Departure, FigureColor, FigureTrack, FloorPlan, Mark, Point, Pose, Project,
    SetPlan,
};
use klappe_stellprobe::{render, FIGURE_RADIUS_M};

/// The printed radius of the one figure, in the unit of the SVG.
fn printed_radius(size_m: Option<f64>) -> f64 {
    let mut project = Project::new("Test");
    project.characters = vec![Character {
        id: "e".into(),
        name: "Elisabeth".into(),
        short: "E".into(),
        color: FigureColor::Purple,
        group: None,
        size_m,
        aliases: Vec::new(),
    }];
    let mut plan = FloorPlan::new("a", "Plan A");
    plan.figures = vec![FigureTrack {
        character: "e".into(),
        marks: vec![Mark {
            at_s: 0.0,
            at: Point::new(2.0, 6.0),
            facing_deg: 0.0,
            pose: Pose::Standing,
            hold_s: 0.0,
            via: Vec::new(),
            curve: Default::default(),
            arrival: Arrival::Walk,
            departure: Departure::Stay,
        }],
    }];
    let set = SetPlan {
        width_m: 20.0,
        depth_m: 13.0,
        walls: Vec::new(),
        objects: Vec::new(),
        background: None,
        notes: Vec::new(),
        ..Default::default()
    };
    let texts = Texts {
        figure: "Figur".into(),
        camera: "Kamera".into(),
        light: "Licht".into(),
        axis: "Achse".into(),
        path: "Weg".into(),
        mark: "Mark".into(),
        unit: "m".into(),
        decimal_separator: ',',
        shots: Default::default(),
        drawing: Default::default(),
        lamps: Default::default(),
    };
    let page = Page::new(Paper::A4, Orientation::Landscape, "Titel", texts);
    let out = svg(
        &render(&project, &set, &plan, 0.0, &|_| None),
        &page,
        Theme::Light,
    )
    .unwrap();
    let body = out
        .lines()
        .find(|l| l.starts_with("<circle") && l.contains("class=\"item figure purple\""))
        .unwrap();
    let start = body.find(" r=\"").unwrap() + 4;

    body[start..].split('"').next().unwrap().parse().unwrap()
}

#[test]
fn a_sized_figure_is_printed_in_its_size() {
    let standard = printed_radius(None);
    let horse = printed_radius(Some(8.0 * FIGURE_RADIUS_M));

    // A body of eight standard radii across is four times as wide as the standard one.
    assert!((horse - 4.0 * standard).abs() < 0.01, "{horse} {standard}");
}
