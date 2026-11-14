//! `png` end to end: the file it writes, decoded again, against what the SVG says is where.
#![allow(clippy::unwrap_used)]

use klappe_model::Point;
use klappe_stellprobe::{DisplayList, Item, Role, Shape};

use crate::palette::{palette, Rgb};
use crate::{png, svg, Orientation, Page, Paper, ShotTexts, Texts, Theme};

const WIDTH_M: f64 = 20.0;
const DEPTH_M: f64 = 13.0;
const PT_PER_PX: f64 = 72.0 / 150.0;

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
    };

    Page::new(Paper::A4, Orientation::Landscape, "T", texts)
}

fn list() -> DisplayList {
    let wall = Item {
        role: Role::Wall,
        shape: Shape::Line {
            from: Point::new(2.0, 6.0),
            to: Point::new(18.0, 6.0),
            width_m: 0.4,
        },
        label: None,
        source: None,
    };

    DisplayList {
        items: vec![wall],
        width_m: WIDTH_M,
        depth_m: DEPTH_M,
    }
}

struct Decoded {
    width: usize,
    pixels: Vec<u8>,
}

impl Decoded {
    fn at(&self, x_pt: f64, y_pt: f64) -> Rgb {
        let x = (x_pt / PT_PER_PX) as usize;
        let y = (y_pt / PT_PER_PX) as usize;
        let i = (y * self.width + x) * 3;

        Rgb {
            r: self.pixels[i],
            g: self.pixels[i + 1],
            b: self.pixels[i + 2],
        }
    }
}

fn decode(file: &[u8], paper: Rgb) -> Decoded {
    let parsed = crate::png::parse(file).unwrap();

    Decoded {
        width: parsed.width as usize,
        pixels: parsed.rgb_over(paper).unwrap(),
    }
}

/// The wall's two end points and its width in points, as the SVG of the same page has them.
fn wall_in_svg() -> ((f64, f64), (f64, f64), f64) {
    let text = svg(&list(), &page(), Theme::Light).unwrap();
    let line = text
        .lines()
        .find(|l| l.contains("class=\"item wall\""))
        .unwrap();
    let after = |key: &str| {
        let start = line.find(key).unwrap() + key.len();
        let rest = &line[start..];

        rest[..rest.find('"').unwrap()].to_string()
    };

    let numbers: Vec<f64> = after("points=\"")
        .split([' ', ','])
        .map(|n| n.parse().unwrap())
        .collect();

    (
        (numbers[0], numbers[1]),
        (numbers[2], numbers[3]),
        after("stroke-width=\"").parse().unwrap(),
    )
}

#[test]
fn a_wall_colours_the_pixels_the_svg_puts_it_on() {
    let pal = palette(Theme::Light).unwrap();
    let image = decode(&png(&list(), &page(), Theme::Light).unwrap(), pal.paper);
    let (from, to, width_pt) = wall_in_svg();
    let middle = ((from.0 + to.0) / 2.0, from.1);

    assert_eq!(image.at(middle.0, middle.1), pal.foreground);
    // Inside the wall's width, near either side, and just beyond it.
    let inside = width_pt / 2.0 - 2.0 * PT_PER_PX;
    let beyond = width_pt / 2.0 + 2.0 * PT_PER_PX;
    assert_eq!(image.at(middle.0, middle.1 + inside), pal.foreground);
    assert_eq!(image.at(middle.0, middle.1 - inside), pal.foreground);
    assert_eq!(image.at(middle.0, middle.1 + beyond), pal.paper);
    // Butt ends: past the last point, the paper.
    assert_eq!(image.at(to.0 + 2.0 * PT_PER_PX, to.1), pal.paper);
}

#[test]
fn the_corner_of_the_sheet_is_paper() {
    let pal = palette(Theme::Dark).unwrap();
    let image = decode(&png(&list(), &page(), Theme::Dark).unwrap(), pal.paper);

    assert_eq!(image.at(1.0, 1.0), pal.paper);
    assert_eq!(image.at(840.0, 594.0), pal.paper);
}
