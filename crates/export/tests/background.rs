//! The background image in SVG and PDF.
#![allow(clippy::unwrap_used)]

use klappe_export::{pdf, svg, ExportError, Orientation, Page, Paper, ShotTexts, Texts, Theme};
use klappe_model::Point;
use klappe_stellprobe::{DisplayList, Item, Role, Shape};

const FILE: &str = "bilder/saal.jpg";

/// SOI, SOF0 (8 bits, 20 high, 30 wide, 3 components): enough for the export, which only
/// copies a JPEG's bytes.
const JPEG: [u8; 23] = [
    0xFF, 0xD8, 0xFF, 0xC0, 0x00, 0x11, 0x08, 0x00, 0x14, 0x00, 0x1E, 0x03, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0xFF, 0xD9,
];

fn list(rotation_deg: f64) -> DisplayList {
    DisplayList {
        items: vec![Item {
            role: Role::Background,
            shape: Shape::Image {
                file: FILE.into(),
                at: Point::new(2.0, 1.0),
                width_m: 10.0,
                depth_m: 5.0,
                rotation_deg,
                opacity: 0.5,
            },
            label: None,
            source: None,
        }],
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
        unit: "m".into(),
        decimal_separator: ',',
        shots: ShotTexts::default(),
    };
    let mut page = Page::new(Paper::A4, Orientation::Landscape, "T", texts);
    if with_image {
        page.images.insert(FILE.into(), JPEG.to_vec());
    }

    page
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// A4 landscape, 10 mm margin: the plan area is 708.59 x 460.58 pt, so 20 x 13 m fit at
/// 35.43 pt/m, filling it. The image is 10 x 5 m: 354.29 x 177.15 pt, at 2 m right and 1 m
/// below the frame's corner (66.65, 58.35): (137.51, 93.78).
#[test]
fn svg_embeds_the_image_turned_about_its_corner() {
    let svg = svg(&list(30.0), &page(true), Theme::Light).unwrap();

    let image = svg.lines().find(|l| l.contains("<image")).unwrap();
    assert!(image.contains("class=\"background\""));
    assert!(image.contains("width=\"354.29\" height=\"177.15\""));
    assert!(image.contains("opacity=\"0.5\""));
    assert!(image.contains("transform=\"rotate(30 "));
    assert!(image.contains("href=\"data:image/jpeg;base64,/9j/wAARCAAUAB4D"));
}

/// The image comes first inside the clipped plan group: behind everything else.
#[test]
fn svg_draws_the_image_behind_the_plan() {
    let svg = svg(&list(0.0), &page(true), Theme::Light).unwrap();

    let group = svg.find("<g clip-path").unwrap();
    let image = svg.find("<image").unwrap();
    assert!(image > group);
    assert!(!svg.contains("transform=\"rotate"));
}

#[test]
fn pdf_takes_a_jpeg_as_it_is() {
    let bytes = pdf(&list(0.0), &page(true), Theme::Light).unwrap();
    let body = text(&bytes);

    assert!(body.contains("/Subtype /Image /Width 30 /Height 20 /ColorSpace /DeviceRGB"));
    assert!(body.contains("/Filter /DCTDecode"));
    assert!(body.contains("/XObject << /Im0 "));
    // Opacity 0.5 is a graphics state, then the placement; the PDF's y runs up from the
    // page's bottom (595.28 pt high): the bottom left corner is at 595.28 - 93.78 - 177.15.
    assert!(body.contains("/ca 0.5"));
    assert!(body.contains("354.29 0 0 177.15 137.51 324.35 cm /Im0 Do"));
    assert!(bytes.windows(JPEG.len()).any(|w| w == JPEG));
}

/// Turned 90° clockwise about its corner, the image's width runs down the page.
#[test]
fn pdf_turns_clockwise() {
    let body = text(&pdf(&list(90.0), &page(true), Theme::Light).unwrap());

    // x axis: (0, -354.29); y axis: (177.15, 0); the bottom left corner swings out to the
    // left of the top left one: x = 137.51 - 177.15 = -39.64, y stays 595.28 - 93.78 = 501.5.
    assert!(body.contains("0 -354.29 177.15 0 -39.64 501.5 cm /Im0 Do"));
}

#[test]
fn an_image_not_handed_in_is_an_error() {
    let missing = |r: Result<String, ExportError>| matches!(r, Err(ExportError::Image { file, .. }) if file == FILE);

    assert!(missing(svg(&list(0.0), &page(false), Theme::Light)));
}

#[test]
fn a_file_that_is_no_image_is_an_error() {
    let mut p = page(false);
    p.images.insert(FILE.into(), b"nothing".to_vec());

    assert!(matches!(
        pdf(&list(0.0), &p, Theme::Light),
        Err(ExportError::Image { .. })
    ));
}
