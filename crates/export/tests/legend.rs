//! The legend of a printed plan (LH-19): what it lists for drawings.
#![allow(clippy::unwrap_used)]

use klappe_export::{svg, DrawingTexts, Orientation, Page, Paper, ShotTexts, Texts, Theme};
use klappe_model::{Dash, FigureColor, Fill, Point};
use klappe_stellprobe::{DisplayList, Item, Role, Shape};

fn page() -> Page {
    let texts = Texts {
        figure: "Figur".into(),
        camera: "Kamera".into(),
        light: "Licht".into(),
        axis: "Achse".into(),
        path: "Weg".into(),
        mark: "Mark".into(),
        drawing: DrawingTexts {
            line: "Zeichnung".into(),
            dashed: "Gestrichelt".into(),
            dotted: "Gepunktet".into(),
            area: "Fläche".into(),
            colors: vec![
                (FigureColor::Red, "rot".into()),
                (FigureColor::Blue, "blau".into()),
                (FigureColor::Cyan, "türkis".into()),
            ],
        },
        unit: "m".into(),
        decimal_separator: ',',
        shots: ShotTexts::default(),
        lamps: Default::default(),
    };

    Page::new(Paper::A4, Orientation::Landscape, "T", texts)
}

fn drawing(color: FigureColor, dash: Dash, fill: Fill, shape: Shape) -> Item {
    Item {
        role: Role::Drawing { color, dash, fill },
        shape,
        label: None,
        source: None,
    }
}

fn line() -> Shape {
    Shape::Line {
        from: Point::new(2.0, 2.0),
        to: Point::new(8.0, 4.0),
        width_m: 0.0,
    }
}

fn boxed() -> Shape {
    Shape::Polygon {
        points: vec![
            Point::new(2.0, 2.0),
            Point::new(8.0, 2.0),
            Point::new(8.0, 4.0),
        ],
    }
}

/// The words of the legend, in order.
fn legend(items: Vec<Item>) -> Vec<String> {
    let list = DisplayList {
        items,
        lamps: Vec::new(),
        width_m: 20.0,
        depth_m: 13.0,
    };
    let out = svg(&list, &page(), Theme::Light).unwrap();

    out.lines()
        .filter(|l| l.contains("legend-label"))
        .filter_map(|l| l.rsplit_once("\">"))
        .map(|(_, text)| text.trim_end_matches("</text>").to_owned())
        .collect()
}

/// A line by its dash, a filled shape as an area, each look once; a plan without drawings
/// has the six fixed entries only.
#[test]
fn the_legend_lists_the_looks_of_drawing() {
    let red = FigureColor::Red;
    let items = vec![
        drawing(red, Dash::Dashed, Fill::Outline, line()),
        drawing(red, Dash::Dashed, Fill::Outline, line()),
        // A line that asked for a fill it cannot have is the same look.
        drawing(red, Dash::Dashed, Fill::Filled, line()),
        drawing(FigureColor::Cyan, Dash::Solid, Fill::Filled, boxed()),
        drawing(FigureColor::Cyan, Dash::Dotted, Fill::Outline, line()),
    ];

    let bare = legend(Vec::new());
    let listed = legend(items);

    assert_eq!(bare.len(), 6, "{bare:?}");
    assert_eq!(&listed[..6], &bare[..]);
    assert_eq!(
        &listed[6..],
        ["Gestrichelt rot", "Fläche türkis", "Gepunktet türkis"],
        "{listed:?}"
    );
}

/// Two lines that differ in colour only are two entries that say so in words.
#[test]
fn the_legend_names_the_colour_of_drawing() {
    let items = vec![
        drawing(FigureColor::Red, Dash::Solid, Fill::Outline, line()),
        drawing(FigureColor::Blue, Dash::Solid, Fill::Outline, line()),
    ];

    let listed = legend(items);

    assert_eq!(
        &listed[6..],
        ["Zeichnung rot", "Zeichnung blau"],
        "{listed:?}"
    );
}
