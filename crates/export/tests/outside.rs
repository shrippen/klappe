//! Q1-07: the printout holds everything drawn, also what lies outside the room, and a note's
//! text in full.

// Test helpers may unwrap; a failure should stop the test.
#![allow(clippy::unwrap_used)]

use klappe_export::{svg, LampTexts, Orientation, Page, Paper, ShotTexts, Texts, Theme};
use klappe_model::{Camera, CameraMark, FloorPlan, Note, Point, Project, Sensor, SetPlan};
use klappe_stellprobe::render;

fn page() -> Page {
    let texts = Texts {
        figure: "Figur".into(),
        camera: "Kamera".into(),
        light: "Licht".into(),
        axis: "Achse".into(),
        path: "Weg".into(),
        mark: "Mark".into(),
        arrival: "Arrival".into(),
        departure: "Departure".into(),
        unit: "m".into(),
        decimal_separator: ',',
        shots: ShotTexts::default(),
        drawing: Default::default(),
        lamps: LampTexts::default(),
    };

    Page::new(Paper::A4, Orientation::Landscape, "Aussen", texts)
}

fn number_after(text: &str, key: &str) -> f64 {
    let start = text.find(key).unwrap() + key.len();
    let rest = &text[start..];

    rest[..rest.find('"').unwrap()].parse().unwrap()
}

/// `x0 y0 x1 y1` of the clip rectangle: where the plan is drawn.
fn clip(out: &str) -> (f64, f64, f64, f64) {
    let line = out.lines().find(|l| l.contains("<clipPath")).unwrap();
    let (x, y) = (number_after(line, " x=\""), number_after(line, " y=\""));

    (
        x,
        y,
        x + number_after(line, "width=\""),
        y + number_after(line, "height=\""),
    )
}

/// `x0 y0 x1 y1` of the card of the first note.
fn card(out: &str) -> (f64, f64, f64, f64) {
    let line = out
        .lines()
        .find(|l| l.contains("class=\"item note\""))
        .unwrap();
    let start = line.find("points=\"").unwrap() + "points=\"".len();
    let pts: Vec<(f64, f64)> = line[start..]
        .split('"')
        .next()
        .unwrap()
        .split(' ')
        .map(|p| {
            let (x, y) = p.split_once(',').unwrap();
            (x.parse().unwrap(), y.parse().unwrap())
        })
        .collect();
    let fold = |f: fn(f64, f64) -> f64, init: f64, pick: fn(&(f64, f64)) -> f64| {
        pts.iter().map(pick).fold(init, f)
    };

    (
        fold(f64::min, f64::MAX, |p| p.0),
        fold(f64::min, f64::MAX, |p| p.1),
        fold(f64::max, f64::MIN, |p| p.0),
        fold(f64::max, f64::MIN, |p| p.1),
    )
}

fn note(id: &str, at: Point, text: &str) -> Note {
    Note {
        id: id.into(),
        at,
        text: text.into(),
        width_m: 2.4,
        rotation_deg: 0.0,
        bound: None,
    }
}

fn exported(notes: Vec<Note>) -> String {
    let room = SetPlan {
        width_m: 20.0,
        depth_m: 13.0,
        notes,
        ..Default::default()
    };
    let list = render(
        &Project::new("Test"),
        &room,
        &FloorPlan::new("a", "Plan A"),
        0.0,
        &|_| None,
    );

    svg(&list, &page(), Theme::Light).unwrap()
}

/// A note below and left of the room (0 to 20 m by 0 to 13 m) lies inside the drawn plan.
#[test]
fn a_note_outside_the_room_is_on_the_page() {
    let out = exported(vec![
        note("a", Point::new(-1.5, 13.5), "Weg frei halten, min. 80 cm"),
        note("b", Point::new(4.0, 3.0), "innen"),
    ]);
    let (left, top, right, bottom) = card(&out);
    let (cx0, cy0, cx1, cy1) = clip(&out);
    let eps = 1e-6;

    assert!(
        left >= cx0 - eps && top >= cy0 - eps && right <= cx1 + eps && bottom <= cy1 + eps,
        "card {left},{top},{right},{bottom} outside clip {cx0},{cy0},{cx1},{cy1}"
    );
}

/// The text of a note comes out in full, wrapped like the card.
#[test]
fn a_note_text_is_printed_in_full() {
    let out = exported(vec![note(
        "a",
        Point::new(4.0, 3.0),
        "Nebelmaschine aussen, Weg frei halten",
    )]);
    let printed: String = out
        .lines()
        .filter(|l| l.contains("class=\"label\""))
        .filter_map(|l| l.split('>').nth(1))
        .map(|t| t.trim_end_matches("</text"))
        .collect::<Vec<_>>()
        .join(" ");

    assert!(
        printed.contains("Nebelmaschine aussen,") && printed.contains("Weg frei halten"),
        "{printed}"
    );
}

/// `x0 y0 x1 y1` of the room frame.
fn frame(out: &str) -> (f64, f64, f64, f64) {
    let line = out.lines().find(|l| l.contains("class=\"frame\"")).unwrap();
    let (x, y) = (number_after(line, " x=\""), number_after(line, " y=\""));

    (
        x,
        y,
        x + number_after(line, "width=\""),
        y + number_after(line, "height=\""),
    )
}

/// A camera looking left, beside a note outside the room: its view cone is cut at the room
/// even though the plan area is wider.
#[test]
fn the_view_cone_stays_in_the_room_beside_an_outside_note() {
    let project = Project::new("Test");
    let room = SetPlan {
        width_m: 20.0,
        depth_m: 13.0,
        notes: vec![note("a", Point::new(-1.5, 13.5), "Weg frei")],
        ..Default::default()
    };
    let mut plan = FloorPlan::new("a", "Plan A");
    plan.cameras = vec![Camera {
        id: "c".into(),
        label: "A".into(),
        sensor: Sensor::Super35,
        lens_mm: 35.0,
        aspect: 2.39,
        marks: vec![CameraMark {
            at_s: 0.0,
            at: Point::new(2.0, 6.5),
            facing_deg: 180.0,
            height_m: 1.6,
            tilt_deg: 0.0,
            hold_s: 0.0,
            via: Vec::new(),
            curve: Default::default(),
            rig: Default::default(),
            lens_mm: None,
        }],
        follows: None,
        color: None,
    }];
    let list = render(&project, &room, &plan, 0.0, &|_| None);
    let out = svg(&list, &page(), Theme::Light).unwrap();
    let (rx0, ry0, rx1, ry1) = frame(&out);
    let eps = 1e-6;

    let wide = clip(&out);
    assert!(
        wide.0 < rx0 - 1.0,
        "the plan area should be wider than the room"
    );

    // The cone's clip is the room, not the plan area.
    let g = out
        .lines()
        .position(|l| l.contains("class=\"item frustum\""))
        .unwrap();
    let before = &out.lines().collect::<Vec<_>>()[..g];
    let group = before
        .iter()
        .rev()
        .find(|l| l.starts_with("<g clip-path"))
        .unwrap();
    let id = group
        .split("url(#")
        .nth(1)
        .unwrap()
        .split(')')
        .next()
        .unwrap();
    let pathline = out
        .lines()
        .find(|l| l.contains(&format!("<clipPath id=\"{id}\"")))
        .unwrap();
    let x = number_after(pathline, " x=\"");
    let y = number_after(pathline, " y=\"");
    let w = number_after(pathline, "width=\"");
    let h = number_after(pathline, "height=\"");

    assert!(
        (x - rx0).abs() < eps
            && (y - ry0).abs() < eps
            && (x + w - rx1).abs() < eps
            && (y + h - ry1).abs() < eps,
        "cone clip {x},{y},{w},{h} is not the room {rx0},{ry0},{rx1},{ry1}"
    );
}
