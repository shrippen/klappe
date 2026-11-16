//! LH-16: a note's text lies inside its card at any scale of the plan.

// Test helpers may unwrap; a failure should stop the test.
#![allow(clippy::unwrap_used)]

use klappe_export::{svg, LampTexts, Orientation, Page, Paper, ShotTexts, Texts, Theme};
use klappe_model::{FloorPlan, Note, Point, Project, SetPlan};
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

    Page::new(Paper::A4, Orientation::Landscape, "Notizen", texts)
}

fn lines_with<'a>(svg: &'a str, class: &str) -> Vec<&'a str> {
    let needle = format!("class=\"{class}\"");
    svg.lines().filter(|l| l.contains(&needle)).collect()
}

fn attr(line: &str, name: &str) -> f64 {
    let start = line.find(&format!(" {name}=\"")).unwrap() + name.len() + 3;
    let rest = &line[start..];

    rest[..rest.find('"').unwrap()].parse().unwrap()
}

/// Width of an average character, in font sizes; an upper bound for Helvetica text.
const AVG_CHAR_EM: f64 = 0.6;
const NOTE_LINES: [&str; 2] = ["Der Vorhang wird erst", "nach dem dritten Gong"];

/// `x0 y0 x1 y1` of the corners in an SVG `points` attribute.
fn bounds(line: &str) -> (f64, f64, f64, f64) {
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
    let min = |f: fn(&(f64, f64)) -> f64| pts.iter().map(f).fold(f64::MAX, f64::min);
    let max = |f: fn(&(f64, f64)) -> f64| pts.iter().map(f).fold(f64::MIN, f64::max);

    (min(|p| p.0), min(|p| p.1), max(|p| p.0), max(|p| p.1))
}

/// The text of a note lies inside its card, at any room size: the card is measured in metres,
/// so the text must scale with the plan.
fn assert_note_text_inside_card(width_m: f64, depth_m: f64) {
    let room = SetPlan {
        width_m,
        depth_m,
        walls: Vec::new(),
        objects: Vec::new(),
        background: None,
        notes: vec![Note {
            id: "n".into(),
            at: Point::new(4.0, 3.0),
            text: "Der Vorhang wird erst nach dem dritten Gong geschlossen".into(),
            width_m: 2.4,
        }],
        ..Default::default()
    };
    let list = render(
        &Project::new("Test"),
        &room,
        &FloorPlan::new("a", "Plan A"),
        0.0,
        &|_| None,
    );
    let out = svg(&list, &page(), Theme::Light).unwrap();

    let (left, top, right, bottom) = bounds(lines_with(&out, "item note")[0]);
    let texts: Vec<&str> = lines_with(&out, "label")
        .into_iter()
        .filter(|l| NOTE_LINES.iter().any(|t| l.contains(&format!(">{t}<"))))
        .collect();
    assert_eq!(texts.len(), NOTE_LINES.len());

    for line in texts {
        let size = attr(line, "font-size");
        let chars = line
            .split('>')
            .nth(1)
            .unwrap()
            .trim_end_matches("</text")
            .chars()
            .count();

        assert!(
            attr(line, "y") <= bottom,
            "baseline below the card at {width_m} m"
        );
        assert!(
            attr(line, "y") - size >= top,
            "text above the card at {width_m} m"
        );
        assert!(
            attr(line, "x") + chars as f64 * size * AVG_CHAR_EM <= right,
            "text wider than the card at {width_m} m"
        );
        assert!(attr(line, "x") >= left);
    }
}

#[test]
fn note_text_stays_in_its_card_in_a_small_room() {
    assert_note_text_inside_card(20.0, 13.0);
}

#[test]
fn note_text_stays_in_its_card_in_a_large_room() {
    assert_note_text_inside_card(40.0, 25.0);
}
