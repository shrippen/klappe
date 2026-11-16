//! A placed image (LH-18) in SVG and PDF: printed like the background, with its frame; one
//! that is missing prints as its frame with the file name.
#![allow(clippy::unwrap_used)]

use klappe_export::{pdf, svg, LampTexts, Orientation, Page, Paper, ShotTexts, Texts, Theme};
use klappe_model::Point;
use klappe_stellprobe::{DisplayList, Item, Role, Shape, Source};

const FILE: &str = "bilder/skizze.jpg";

/// SOI, SOF0 (8 bits, 20 high, 30 wide, 3 components): enough for the export.
const JPEG: [u8; 23] = [
    0xFF, 0xD8, 0xFF, 0xC0, 0x00, 0x11, 0x08, 0x00, 0x14, 0x00, 0x1E, 0x03, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0xFF, 0xD9,
];

fn p(x_m: f64, y_m: f64) -> Point {
    Point::new(x_m, y_m)
}

/// The frame of a 4 x 3 m image at (2, 1), with its picture when `picture` is set.
fn list(picture: bool) -> DisplayList {
    let mut items = Vec::new();
    if picture {
        items.push(Item {
            role: Role::Picture,
            shape: Shape::Image {
                file: FILE.into(),
                at: p(2.0, 1.0),
                width_m: 4.0,
                depth_m: 3.0,
                rotation_deg: 0.0,
                opacity: 1.0,
            },
            label: None,
            source: None,
        });
    }
    items.push(Item {
        role: Role::Picture,
        shape: Shape::Polygon {
            points: vec![p(2.0, 1.0), p(6.0, 1.0), p(6.0, 4.0), p(2.0, 4.0)],
        },
        label: (!picture).then(|| FILE.to_owned()),
        source: Some(Source::Image("i1".into())),
    });

    DisplayList {
        lamps: Vec::new(),
        items,
        width_m: 20.0,
        depth_m: 13.0,
    }
}

fn page(with_image: bool) -> Page {
    let texts = Texts {
        figure: "Figur".into(),
        camera: "Kamera".into(),
        light: "Licht".into(),
        axis: "Achse".into(),
        path: "Weg".into(),
        mark: "Mark".into(),
        drawing: Default::default(),
        unit: "m".into(),
        decimal_separator: ',',
        shots: ShotTexts::default(),
        lamps: LampTexts::default(),
    };
    let mut page = Page::new(Paper::A4, Orientation::Landscape, "T", texts);
    if with_image {
        page.images.insert(FILE.into(), JPEG.to_vec());
    }

    page
}

/// 4 x 3 m at 35.43 pt/m: 141.72 x 106.29 pt, opaque.
#[test]
fn svg_embeds_the_picture_and_draws_its_frame() {
    let svg = svg(&list(true), &page(true), Theme::Light).unwrap();

    let image = svg.lines().find(|l| l.contains("<image")).unwrap();
    assert!(image.contains("class=\"picture\""), "{image}");
    assert!(
        image.contains("width=\"141.72\" height=\"106.29\""),
        "{image}"
    );
    assert!(image.contains("opacity=\"1\""), "{image}");
    assert!(svg.contains("item picture"), "{svg}");
}

#[test]
fn pdf_prints_the_picture() {
    let bytes = pdf(&list(true), &page(true), Theme::Light).unwrap();

    assert!(bytes.windows(JPEG.len()).any(|w| w == JPEG));
}

/// Nothing is handed in for a missing file: the frame stands alone, named.
#[test]
fn a_missing_file_prints_as_its_frame_with_the_name() {
    let page = page(false);

    let svg = svg(&list(false), &page, Theme::Light).unwrap();
    assert!(!svg.contains("<image"));
    assert!(svg.contains("item picture"));
    assert!(svg.contains(FILE), "{svg}");
    assert!(pdf(&list(false), &page, Theme::Light).is_ok());
}
