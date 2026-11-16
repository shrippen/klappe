//! Faded items (LH-15, LH-87): a line takes the mix of its colour and the paper, a fill turns
//! more transparent; pages of several plans in one PDF.
#![allow(clippy::unwrap_used)]

use klappe_export::{pdf, pdf_pages, svg, Orientation, Page, Paper, ShotTexts, Texts, Theme};
use klappe_model::Point;
use klappe_stellprobe::{DisplayList, Item, Role, Shape};

fn texts() -> Texts {
    Texts {
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
    }
}

fn page(fade: Vec<f64>) -> Page {
    let mut page = Page::new(Paper::A4, Orientation::Landscape, "Titel", texts());
    page.fade = fade;

    page
}

/// A wall and a camera body.
fn list() -> DisplayList {
    let body = vec![
        Point::new(2.0, 2.0),
        Point::new(3.0, 2.0),
        Point::new(3.0, 3.0),
    ];

    DisplayList {
        items: vec![
            Item {
                role: Role::Wall,
                shape: Shape::Line {
                    from: Point::new(1.0, 1.0),
                    to: Point::new(9.0, 1.0),
                    width_m: 0.2,
                },
                label: None,
                source: None,
            },
            Item {
                role: Role::Camera,
                shape: Shape::Polygon { points: body },
                label: Some("A".into()),
                source: None,
            },
        ],
        lamps: Vec::new(),
        width_m: 10.0,
        depth_m: 6.0,
    }
}

/// The element of the item with `class`.
fn element<'a>(svg: &'a str, class: &str) -> &'a str {
    let needle = format!("class=\"item {class}\"");

    svg.lines().find(|l| l.contains(&needle)).unwrap()
}

#[test]
fn a_faded_wall_is_paler_than_a_full_one() {
    let full = svg(&list(), &page(Vec::new()), Theme::Light).unwrap();
    let pale = svg(&list(), &page(vec![0.3, 1.0]), Theme::Light).unwrap();

    assert_ne!(element(&full, "wall"), element(&pale, "wall"));
    // The camera, at full strength, is as it was.
    assert_eq!(element(&full, "camera"), element(&pale, "camera"));
}

#[test]
fn a_faded_fill_turns_more_transparent() {
    let full = svg(&list(), &page(vec![1.0, 1.0]), Theme::Light).unwrap();
    let pale = svg(&list(), &page(vec![1.0, 0.5]), Theme::Light).unwrap();

    assert_ne!(element(&full, "camera"), element(&pale, "camera"));
    assert!(element(&pale, "camera").contains("opacity"));
}

#[test]
fn a_short_fade_list_leaves_the_rest_full() {
    let none = svg(&list(), &page(Vec::new()), Theme::Light).unwrap();
    let short = svg(&list(), &page(vec![1.0]), Theme::Light).unwrap();

    assert_eq!(none, short);
}

#[test]
fn every_plan_of_the_slice_is_a_page() {
    let one = pdf(&list(), &page(Vec::new()), Theme::Light).unwrap();
    let pages = [
        (list(), page(vec![1.0, 0.3])),
        (list(), page(vec![0.3, 1.0])),
        (list(), page(Vec::new())),
    ];

    let three = pdf_pages(&pages, Theme::Light).unwrap();

    let count = |bytes: &[u8]| {
        String::from_utf8_lossy(bytes)
            .matches("/Type /Page ")
            .count()
    };
    assert_eq!(count(&one), 1);
    assert_eq!(count(&three), 3);
    assert!(String::from_utf8_lossy(&three).contains("/Count 3"));
}

#[test]
fn no_page_is_no_pdf() {
    assert!(pdf_pages(&[], Theme::Light).is_err());
}
