// Test helpers may unwrap; a failure should stop the test.
#![allow(clippy::unwrap_used)]

use klappe_export::{pdf, svg, ExportError, Orientation, Page, Paper, Texts, Theme};
use klappe_model::{
    Camera, CameraMark, Character, FigureColor, FigureTrack, FloorPlan, Light, LightKind, Mark,
    Point, Pose, Project, Sensor, SetObject, SetPlan, Wall,
};
use klappe_stellprobe::{render, DisplayList};
use serde_json::Value;

const PALETTE_JSON: &str = include_str!("../kante/palette.json");
const TITLE: &str = "Szene 12 · Sängersaal · Plan A";
const FRAME_TOLERANCE: f64 = 0.01;

fn project() -> Project {
    let mut p = Project::new("Test");
    p.characters = vec![Character {
        id: "e".into(),
        name: "Elisabeth".into(),
        short: "E".into(),
        color: FigureColor::Purple,
    }];
    p
}

fn set() -> SetPlan {
    SetPlan {
        width_m: 20.0,
        depth_m: 13.0,
        walls: vec![Wall {
            from: Point::new(0.0, 0.0),
            to: Point::new(20.0, 0.0),
            thickness_m: 0.2,
        }],
        objects: vec![SetObject {
            id: "t".into(),
            kind: "table".into(),
            label: "Tisch".into(),
            at: Point::new(10.0, 8.0),
            width_m: 2.0,
            depth_m: 1.0,
            rotation_deg: 0.0,
        }],
    }
}

fn mark(at_s: f64, x: f64, y: f64) -> Mark {
    Mark {
        at_s,
        at: Point::new(x, y),
        facing_deg: 0.0,
        pose: Pose::Standing,
        hold_s: 0.0,
    }
}

fn plan() -> FloorPlan {
    let mut p = FloorPlan::new("a", "Plan A");
    p.figures = vec![
        FigureTrack {
            character: "e".into(),
            marks: vec![mark(0.0, 2.0, 6.0), mark(8.0, 7.0, 6.0)],
        },
        FigureTrack {
            character: "e".into(),
            marks: vec![mark(0.0, 12.0, 6.0)],
        },
    ];
    p.cameras = vec![Camera {
        id: "a".into(),
        label: "A".into(),
        sensor: Sensor::Super35,
        lens_mm: 35.0,
        aspect: 2.39,
        marks: vec![CameraMark {
            at_s: 0.0,
            at: Point::new(8.0, 12.0),
            facing_deg: -90.0,
            height_m: 1.6,
            tilt_deg: 0.0,
            hold_s: 0.0,
        }],
        follows: None,
    }];
    p.lights = vec![Light {
        id: "l1".into(),
        label: "L1".into(),
        kind: LightKind::LedPanel,
        at: Point::new(18.0, 2.0),
        facing_deg: 135.0,
        beam_deg: 60.0,
        color_temp_k: 5600.0,
        power_w: 200.0,
    }];
    p
}

fn list() -> DisplayList {
    render(&project(), &set(), &plan(), 0.0)
}

fn german() -> Texts {
    Texts {
        figure: "Figur".into(),
        camera: "Kamera".into(),
        light: "Licht".into(),
        axis: "Achse".into(),
        path: "Weg".into(),
        mark: "Mark".into(),
        unit: "m".into(),
        decimal_separator: ',',
    }
}

fn page() -> Page {
    Page::new(Paper::A4, Orientation::Landscape, TITLE, german())
}

/// A colour of the embedded palette, looked up the way a designer would.
fn colour(theme: Theme, group: &str, key: &str) -> String {
    let root: Value = serde_json::from_str(PALETTE_JSON).unwrap();
    let (scope, dark) = match theme {
        Theme::Light => (&root["light"], false),
        Theme::Dark => (&root, true),
    };
    let entry = &scope[group][key];
    let text = match (dark, entry.get("bright")) {
        (true, Some(bright)) => bright,
        _ => entry,
    };

    text.as_str().unwrap().to_string()
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

#[test]
fn svg_has_one_element_per_item() {
    let list = list();
    let out = svg(&list, &page(), Theme::Light).unwrap();

    assert_eq!(out.matches("class=\"item ").count(), list.items.len());
    assert!(out.starts_with("<svg "));
}

#[test]
fn svg_colours_come_from_the_palette() {
    let out = svg(&list(), &page(), Theme::Light).unwrap();
    let find = |class: &str| lines_with(&out, &format!("item {class}"))[0].to_string();
    let fg0 = colour(Theme::Light, "foregrounds", "fg0");
    let fg3 = colour(Theme::Light, "foregrounds", "fg3");
    let bg2 = colour(Theme::Light, "backgrounds", "bg2");
    let purple = colour(Theme::Light, "semantic", "purple");
    let yellow = colour(Theme::Light, "semantic", "yellow");
    let orange = colour(Theme::Light, "semantic", "orange");

    assert!(find("wall").contains(&format!("stroke=\"{fg0}\"")));
    let object = find("object");
    assert!(object.contains(&format!("fill=\"{bg2}\"")));
    assert!(object.contains(&format!("stroke=\"{fg3}\"")));
    assert!(find("figure purple").contains(&format!("fill=\"{purple}\"")));
    assert!(find("camera").contains(&format!("fill=\"{yellow}\"")));
    assert!(find("light").contains(&format!("fill=\"{orange}\"")));
    assert!(find("axis").contains(&format!("stroke=\"{orange}\"")));
}

#[test]
fn dark_theme_uses_the_bright_tones() {
    let out = svg(&list(), &page(), Theme::Dark).unwrap();
    let line = lines_with(&out, "item figure purple")[0];
    let paper = colour(Theme::Dark, "backgrounds", "bg-panel");

    assert!(line.contains(&format!(
        "fill=\"{}\"",
        colour(Theme::Dark, "semantic", "purple")
    )));
    assert!(lines_with(&out, "paper")[0].contains(&format!("fill=\"{paper}\"")));
}

#[test]
fn information_is_not_only_colour() {
    let out = svg(&list(), &page(), Theme::Light).unwrap();

    assert!(lines_with(&out, "item axis")[0].contains("stroke-dasharray"));
    assert!(lines_with(&out, "item figure-path purple")[0].contains("stroke-dasharray"));
    // Frustum and beam: translucent fill and an outline.
    for class in ["item frustum", "item beam"] {
        let line = lines_with(&out, class)[0];
        assert!(line.contains("fill-opacity"), "{class}");
        assert!(line.contains("stroke=\""), "{class}");
    }
}

#[test]
fn legend_and_scale_bar_are_present() {
    let out = svg(&list(), &page(), Theme::Light).unwrap();

    assert_eq!(lines_with(&out, "legend").len(), 6);
    for name in ["Figur", "Kamera", "Licht", "Achse", "Weg", "Mark"] {
        assert!(out.contains(&format!(">{name}</text>")), "{name}");
    }
    assert_eq!(lines_with(&out, "scale").len(), 3);
    let label = lines_with(&out, "scale-label")[0];
    assert!(label.contains(" m</text>"));
}

#[test]
fn the_plan_keeps_its_aspect_ratio() {
    let list = list();
    let ratio = list.width_m / list.depth_m;

    for (paper, orientation) in [
        (Paper::A4, Orientation::Landscape),
        (Paper::A4, Orientation::Portrait),
        (Paper::A3, Orientation::Landscape),
    ] {
        let out = svg(
            &list,
            &Page::new(paper, orientation, TITLE, german()),
            Theme::Light,
        )
        .unwrap();
        let frame = lines_with(&out, "frame")[0];

        let got = attr(frame, "width") / attr(frame, "height");
        assert!((got - ratio).abs() < FRAME_TOLERANCE, "{got} vs {ratio}");
    }
}

#[test]
fn text_is_at_least_eight_points() {
    let out = svg(&list(), &page(), Theme::Light).unwrap();

    let sizes: Vec<f64> = out
        .lines()
        .filter(|l| l.starts_with("<text "))
        .map(|l| attr(l, "font-size"))
        .collect();
    assert!(sizes.len() > 6);
    assert!(sizes.iter().all(|s| *s >= 8.0));
}

#[test]
fn the_title_is_written_escaped() {
    let mut other = page();
    other.title = "Tom & <Jerry>".into();
    let out = svg(&list(), &other, Theme::Light).unwrap();

    assert!(out.contains("Tom &amp; &lt;Jerry&gt;"));
    assert!(svg(&list(), &page(), Theme::Light).unwrap().contains(TITLE));
}

#[test]
fn pdf_is_well_formed() {
    let bytes = pdf(&list(), &page(), Theme::Light).unwrap();
    let text = String::from_utf8_lossy(&bytes).to_string();

    assert!(bytes.starts_with(b"%PDF-"));
    assert!(text.ends_with("%%EOF\n"));

    // startxref points at "xref"; each entry points at "n 0 obj".
    let at = |marker: &str| text.rfind(marker).unwrap();
    let xref_at: usize = text[at("startxref") + 10..]
        .lines()
        .next()
        .unwrap()
        .parse()
        .unwrap();
    assert!(bytes[xref_at..].starts_with(b"xref\n"));

    let tail = String::from_utf8_lossy(&bytes[xref_at..]).to_string();
    let mut lines = tail.lines().skip(1);
    let count: usize = lines
        .next()
        .unwrap()
        .split(' ')
        .nth(1)
        .unwrap()
        .parse()
        .unwrap();
    assert_eq!(lines.next().unwrap(), "0000000000 65535 f ");
    for n in 1..count {
        let entry = lines.next().unwrap();
        assert_eq!(entry.len(), 19);
        let offset: usize = entry[..10].parse().unwrap();
        assert!(
            bytes[offset..].starts_with(format!("{n} 0 obj\n").as_bytes()),
            "{n}"
        );
    }
}

#[test]
fn pdf_contains_title_and_umlauts() {
    let bytes = pdf(&list(), &page(), Theme::Light).unwrap();
    let text = String::from_utf8_lossy(&bytes);

    // ä = 0xE4 = \344, · = 0xB7 = \267 in WinAnsi.
    assert!(text.contains("(Szene 12 \\267 S\\344ngersaal \\267 Plan A) Tj"));
    assert!(text.contains("/BaseFont /Helvetica "));
    assert!(text.contains("/WinAnsiEncoding"));
}

#[test]
fn pdf_page_size_and_opacity() {
    let page = Page::new(Paper::A3, Orientation::Portrait, TITLE, german());
    let bytes = pdf(&list(), &page, Theme::Light).unwrap();
    let text = String::from_utf8_lossy(&bytes);

    assert!(text.contains("/MediaBox [0 0 841.89 1190.55]"));
    assert!(text.contains("/Type /ExtGState /ca 0.15"));
    assert!(text.contains(" c h"));
}

#[test]
fn a_plan_without_size_is_an_error() {
    let empty = DisplayList::default();

    assert!(matches!(
        svg(&empty, &page(), Theme::Light),
        Err(ExportError::EmptyPlan)
    ));
    assert!(matches!(
        pdf(&empty, &page(), Theme::Light),
        Err(ExportError::EmptyPlan)
    ));
}

#[test]
fn a_page_without_room_is_an_error() {
    let mut page = page();
    page.margin_mm = 200.0;

    assert!(matches!(
        svg(&list(), &page, Theme::Light),
        Err(ExportError::PageTooSmall)
    ));
}

#[test]
fn texts_come_from_the_caller() {
    let mut other = page();
    other.texts = Texts {
        figure: "Actor".into(),
        unit: "metres".into(),
        decimal_separator: '.',
        ..german()
    };
    let out = svg(&list(), &other, Theme::Light).unwrap();

    assert!(out.contains(">Actor</text>"));
    assert!(!out.contains(">Figur</text>"));
    assert!(lines_with(&out, "scale-label")[0].contains(" metres</text>"));
}
