//! Upright sheets: the plan fits every paper.
#![allow(clippy::unwrap_used)]

use klappe_export::{svg, Orientation, Page, Paper, ShotTexts, Texts, Theme};
use klappe_stellprobe::DisplayList;

fn page(paper: Paper) -> Page {
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
        lamps: Default::default(),
    };

    Page::new(paper, Orientation::Portrait, "Titel", texts)
}

/// The numbers of the attribute `name` of the first line with `class`.
fn numbers(svg: &str, name: &str) -> Vec<f64> {
    let start = svg.find(&format!("{name}=\"")).unwrap() + name.len() + 2;
    let rest = &svg[start..];

    rest[..rest.find('"').unwrap()]
        .split(' ')
        .map(|v| v.trim_end_matches("pt").parse().unwrap())
        .collect()
}

#[test]
fn the_plan_fits_an_upright_page() {
    let list = DisplayList {
        items: Vec::new(),
        lamps: Vec::new(),
        width_m: 20.0,
        depth_m: 13.0,
    };

    for paper in Paper::ALL {
        let out = svg(&list, &page(paper), Theme::Light).unwrap();
        let view = numbers(&out, "viewBox");
        let frame = out.lines().find(|l| l.contains("class=\"frame\"")).unwrap();
        let (x, y) = (numbers(frame, "x")[0], numbers(frame, "y")[0]);
        let (w, h) = (numbers(frame, "width")[0], numbers(frame, "height")[0]);

        assert!(view[2] < view[3], "{paper:?} is upright");
        assert!(x >= 0.0 && x + w <= view[2], "{paper:?} wide");
        assert!(y >= 0.0 && y + h <= view[3], "{paper:?} high");
    }
}
