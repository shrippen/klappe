//! The labels of the sun and north arrows stay on their arrows inside the set (LH-52).
#![allow(clippy::unwrap_used)]

use klappe_export::{svg, Orientation, Page, Paper, Texts, Theme};
use klappe_model::{SetPlan, Site, SunTime};
use klappe_stellprobe::{add_sun, DisplayList};

fn out(width_m: f64, depth_m: f64) -> String {
    let set = SetPlan {
        width_m,
        depth_m,
        ..SetPlan::default()
    };
    let site = Site {
        latitude_deg: 52.52,
        longitude_deg: 13.405,
        north_deg: 0.0,
    };
    let time = SunTime {
        year: 2026,
        month: 6,
        day: 21,
        hour: 7,
        minute: 30,
        utc_offset_min: Some(120),
    };
    let mut list = DisplayList {
        items: Vec::new(),
        lamps: Vec::new(),
        width_m,
        depth_m,
    };
    add_sun(&mut list, &set, &site, &time);
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
        shots: Default::default(),
        drawing: Default::default(),
        lamps: Default::default(),
    };

    svg(
        &list,
        &Page::new(Paper::A4, Orientation::Landscape, "T", texts),
        Theme::Light,
    )
    .unwrap()
}

/// A rectangle in page points: left, top, right, bottom.
#[derive(Debug, Clone, Copy)]
struct Area(f64, f64, f64, f64);

impl Area {
    fn overlaps(&self, o: &Area) -> bool {
        self.0 < o.2 && o.0 < self.2 && self.1 < o.3 && o.1 < self.3
    }

    fn inside(&self, o: &Area) -> bool {
        self.0 >= o.0 && self.1 >= o.1 && self.2 <= o.2 && self.3 <= o.3
    }
}

const FONT_PT: f64 = 8.0;
fn attr(line: &str, name: &str) -> String {
    let key = format!(" {name}=\"");
    let start = line.find(&key).unwrap() + key.len();

    line[start..].split('"').next().unwrap().to_owned()
}

fn number(line: &str, name: &str) -> f64 {
    attr(line, name).parse().unwrap()
}

fn arrow_area(line: &str) -> Area {
    let points: Vec<Vec<f64>> = attr(line, "points")
        .split(' ')
        .map(|p| p.split(',').map(|v| v.parse().unwrap()).collect())
        .collect();
    let fold = |i: usize, pick: fn(f64, f64) -> f64, start: f64| {
        points.iter().map(|p| p[i]).fold(start, pick)
    };

    Area(
        fold(0, f64::min, f64::MAX),
        fold(1, f64::min, f64::MAX),
        fold(0, f64::max, f64::MIN),
        fold(1, f64::max, f64::MIN),
    )
}

/// Helvetica advances in em of the characters a label has.
fn advance_em(c: char) -> f64 {
    match c {
        '0'..='9' => 0.556,
        '°' => 0.4,
        ' ' | '/' => 0.278,
        _ => 0.722,
    }
}

/// The boxes of the label lines that follow an arrow.
fn label_areas(lines: &[&str]) -> Vec<Area> {
    lines
        .iter()
        .take_while(|l| l.contains("class=\"label\""))
        .map(|line| {
            let text = line.split('>').nth(1).unwrap().split('<').next().unwrap();
            let width = text.chars().map(advance_em).sum::<f64>() * FONT_PT;
            let (x, y) = (number(line, "x"), number(line, "y"));

            Area(
                x - width / 2.0,
                y - FONT_PT * 0.75,
                x + width / 2.0,
                y + FONT_PT * 0.25,
            )
        })
        .collect()
}

/// The label of each arrow lies on the page inside the set's clip and not over the other arrow.
fn check(width_m: f64, depth_m: f64) {
    let svg = out(width_m, depth_m);
    let lines: Vec<&str> = svg.lines().collect();
    let clip = {
        let line = lines.iter().find(|l| l.contains("<clipPath")).unwrap();
        let (x, y) = (number(line, "x"), number(line, "y"));

        Area(x, y, x + number(line, "width"), y + number(line, "height"))
    };
    let arrow = |class: &str| {
        let at = lines
            .iter()
            .position(|l| l.contains(&format!("class=\"item {class}\"")))
            .unwrap();

        (arrow_area(lines[at]), label_areas(&lines[at + 1..]))
    };
    let (north, north_label) = arrow("north");
    let (sun, sun_label) = arrow("sun-ray");
    let set = format!("{width_m} x {depth_m} m");

    for (name, labels, other) in [("N", &north_label, &sun), ("sun", &sun_label, &north)] {
        assert!(!labels.is_empty(), "{set}: {name} has no label");
        for label in labels {
            assert!(label.inside(&clip), "{set}: {name} {label:?} clip {clip:?}");
            assert!(
                !label.overlaps(other),
                "{set}: {name} {label:?} over {other:?}"
            );
        }
    }
}

#[test]
fn labels_fit_a_wide_set() {
    check(20.0, 13.0);
}

#[test]
fn labels_fit_a_deep_set() {
    check(3.0, 40.0);
}

#[test]
fn labels_fit_a_large_set() {
    check(100.0, 100.0);
}

#[test]
fn labels_fit_a_flat_set() {
    check(40.0, 3.0);
}

#[test]
fn labels_fit_a_small_set() {
    check(2.0, 2.0);
}
