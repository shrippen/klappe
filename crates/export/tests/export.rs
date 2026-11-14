// Test helpers may unwrap; a failure should stop the test.
#![allow(clippy::unwrap_used)]

use klappe_export::{
    pdf, png, svg, ExportError, Orientation, Page, Paper, ShotTexts, Texts, Theme,
};
use klappe_model::{
    Arrival, Camera, CameraMark, Character, Dash, Departure, Drawing, DrawingShape, FigureColor,
    FigureTrack, Fill, FloorPlan, Light, LightKind, Mark, Note, ObjectShape, Point, Pose, Project,
    Sensor, SetObject, SetPlan, Wall,
};
use klappe_stellprobe::{render, DisplayList};
use serde_json::Value;

const PALETTE_JSON: &str = include_str!("../kante/palette.json");
const TITLE: &str = "Szene 12 · Sängersaal · Plan A";
const FRAME_TOLERANCE: f64 = 0.01;
const MIN_LABEL_PT: f64 = 8.0;

fn project() -> Project {
    let mut p = Project::new("Test");
    p.characters = vec![Character {
        id: "e".into(),
        name: "Elisabeth".into(),
        short: "E".into(),
        color: FigureColor::Purple,
        group: None,
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
            shape: ObjectShape::Rect,
            height_m: None,
        }],
        background: None,
        notes: Vec::new(),
    }
}

fn mark(at_s: f64, x: f64, y: f64) -> Mark {
    Mark {
        at_s,
        at: Point::new(x, y),
        facing_deg: 0.0,
        pose: Pose::Standing,
        hold_s: 0.0,
        via: None,
        arrival: Arrival::Walk,
        departure: Departure::Stay,
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
            via: None,
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
    render(&project(), &set(), &plan(), 0.0, None)
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
        shots: ShotTexts::default(),
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

    assert_eq!(lines_with(&out, "legend-label").len(), 6);
    for key in ["figure", "camera", "light", "axis", "path", "mark"] {
        assert!(
            !lines_with(&out, &format!("legend {key}")).is_empty(),
            "{key}"
        );
    }
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

/// LH-26: a figure out of the scene is left out of the print; its marks stay.
#[test]
fn a_figure_that_left_is_not_printed() {
    let mut p = plan();
    p.figures[0].marks[0].departure = Departure::Leave;
    let at = |t_s| {
        let list = render(&project(), &set(), &p, t_s, None);
        svg(&list, &page(), Theme::Light).unwrap()
    };
    let figures = |out: &str| lines_with(out, "item figure purple").len();

    let before = figures(&at(0.0));
    let after = figures(&at(1.0));

    assert!(before > after, "{before} {after}");
    assert_eq!(
        lines_with(&at(0.0), "item mark purple").len(),
        lines_with(&at(1.0), "item mark purple").len()
    );
}

/// LH-16: a note shows in SVG and PDF with its text.
#[test]
fn notes_are_printed_with_their_text() {
    let mut room = set();
    room.notes = vec![Note {
        id: "n1".into(),
        at: Point::new(4.0, 3.0),
        text: "Vorhang schließen".into(),
    }];
    let list = render(&project(), &room, &plan(), 0.0, None);

    let out = svg(&list, &page(), Theme::Light).unwrap();
    let bytes = pdf(&list, &page(), Theme::Light).unwrap();

    assert_eq!(lines_with(&out, "item note").len(), 1);
    assert!(out.contains(">Vorhang schließen<"));
    // ß = 0xDF = \337 in WinAnsi.
    assert!(String::from_utf8_lossy(&bytes).contains("(Vorhang schlie\\337en) Tj"));
}

/// LH-23: a group is printed larger, with its dots and the head count in its label.
#[test]
fn a_group_is_printed_with_its_count() {
    let mut who = project();
    who.characters[0].group = Some(12);
    let grouped = render(&who, &set(), &plan(), 0.0, None);

    let out = svg(&grouped, &page(), Theme::Light).unwrap();

    assert!(out.contains(">Elisabeth (12)<"));
    // Three dots in each body (start and end of the way), in the outline colour.
    assert_eq!(lines_with(&out, "item group-dot").len(), 6);
}

/// A mark's number sits above the figure standing on it, also above a group's wider body.
#[test]
fn a_mark_number_clears_a_group() {
    let mut who = project();
    who.characters[0].group = Some(12);
    let out = svg(
        &render(&who, &set(), &plan(), 0.0, None),
        &page(),
        Theme::Light,
    )
    .unwrap();

    let body = lines_with(&out, "item figure purple")
        .into_iter()
        .find(|l| l.starts_with("<circle"))
        .unwrap();
    let body_top = attr(body, "cy") - attr(body, "r");
    let number = out
        .lines()
        .filter(|l| l.contains("class=\"label\"") && l.ends_with(">1</text>"))
        .find(|l| (attr(l, "x") - attr(body, "cx")).abs() < 0.01)
        .unwrap();

    assert!(attr(number, "y") < body_top, "{number} {body}");
}

/// LH-19: drawings show in SVG, PDF and PNG; an arrow's head is a filled area, a dash tells
/// a dashed line from a solid one, a fill tints the inside.
#[test]
fn drawings_are_printed() {
    let drawing = |id: &str, shape, dash, fill| Drawing {
        id: id.into(),
        shape,
        color: FigureColor::Red,
        dash,
        fill,
    };
    let (a, b) = (Point::new(2.0, 10.0), Point::new(8.0, 12.0));
    let mut with = plan();
    with.drawings = vec![
        drawing(
            "d1",
            DrawingShape::Arrow { from: a, to: b },
            Dash::Dashed,
            Fill::Outline,
        ),
        drawing(
            "d2",
            DrawingShape::Rect { from: a, to: b },
            Dash::Solid,
            Fill::Filled,
        ),
    ];
    let bare = render(&project(), &set(), &plan(), 0.0, None);
    let list = render(&project(), &set(), &with, 0.0, None);

    let out = svg(&list, &page(), Theme::Light).unwrap();
    let pdf_bytes = pdf(&list, &page(), Theme::Light).unwrap();
    let png_with = png(&list, &page(), Theme::Light).unwrap();
    let png_bare = png(&bare, &page(), Theme::Light).unwrap();

    let shaft = lines_with(&out, "item drawing red");
    assert_eq!(shaft.len(), 2);
    assert!(shaft[0].contains("stroke-dasharray"));
    assert!(!shaft[1].contains("stroke-dasharray"));
    assert!(!shaft[0].contains("fill-opacity"));
    assert!(shaft[1].contains("fill-opacity"));
    let head = lines_with(&out, "item arrow-head red");
    assert_eq!(head.len(), 1);
    assert!(!head[0].contains("fill=\"none\""));
    assert!(!head[0].contains("fill-opacity"));
    assert!(pdf_bytes.starts_with(b"%PDF"));
    assert_ne!(png_with, png_bare);
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

/// What a legend symbol looks like apart from colour and place.
fn signature(line: &str) -> String {
    let tag = line[1..].split(' ').next().unwrap_or_default();
    let points = line.split(" points=\"").nth(1).map_or(0, |p| {
        p.split('"').next().unwrap_or_default().split(' ').count()
    });
    let has = |name: &str| line.contains(name);

    format!(
        "{tag} r={} points={points} dash=[{}] opacity={} filled={}",
        if has(" r=\"") { attr(line, "r") } else { 0.0 },
        line.split(" stroke-dasharray=\"")
            .nth(1)
            .map_or("solid", |d| d.split('"').next().unwrap_or_default()),
        has("fill-opacity"),
        !has("fill=\"none\""),
    )
}

#[test]
fn legend_symbols_differ_in_shape_not_only_colour() {
    let out = svg(&list(), &page(), Theme::Light).unwrap();
    let mut seen: Vec<(String, String)> = Vec::new();

    for key in ["figure", "camera", "light", "axis", "path", "mark"] {
        let mut parts: Vec<String> = lines_with(&out, &format!("legend {key}"))
            .into_iter()
            .map(signature)
            .collect();
        parts.sort();
        let sig = parts.join(" | ");

        if let Some((other, _)) = seen.iter().find(|(_, s)| *s == sig) {
            panic!("legend {key} looks like legend {other}: {sig}");
        }
        seen.push((key.to_string(), sig));
    }
}

/// A4 landscape is 297 x 210 mm = 841.89 x 595.28 pt (72 / 25.4 pt per mm); margin 10 mm
/// = 28.35 pt. Content width 841.89 - 2 * 28.35 = 785.20 pt. Footer = scale row 26 +
/// one legend row 16 = 42 pt (9 legend columns fit), title band 24, two gaps of 6. Plan
/// area height 595.28 - 56.69 - 24 - 42 - 12 = 460.58 pt. Scale = min(785.20 / 20 m,
/// 460.58 / 13 m) = 35.43 pt/m. Frame 708.59 x 460.58 pt, centred: x = 28.35 + (785.20 -
/// 708.59) / 2 = 66.65, y = 28.35 + 24 + 6 = 58.35. Longest round bar <= 160 pt is 2 m.
#[test]
fn the_scale_matches_hand_worked_numbers() {
    let out = svg(&list(), &page(), Theme::Light).unwrap();
    let frame = lines_with(&out, "frame")[0];
    let near = |got: f64, want: f64| assert!((got - want).abs() < 0.011, "{got} vs {want}");

    near(attr(frame, "x"), 66.65);
    near(attr(frame, "y"), 58.35);
    near(attr(frame, "width"), 708.59);
    near(attr(frame, "height"), 460.58);

    let bar = lines_with(&out, "scale")[0];
    let points = bar
        .split(" points=\"")
        .nth(1)
        .unwrap()
        .split('"')
        .next()
        .unwrap();
    let xs: Vec<f64> = points
        .split(' ')
        .map(|p| p.split(',').next().unwrap().parse().unwrap())
        .collect();
    near(xs[1] - xs[0], 70.86);
    assert!(lines_with(&out, "scale-label")[0].contains(">2 m<"));
}

#[test]
fn bar_and_label_agree_on_tiny_plans() {
    let tiny = DisplayList {
        width_m: 0.2,
        depth_m: 0.2,
        ..DisplayList::default()
    };
    let out = svg(&tiny, &page(), Theme::Light).unwrap();
    let scale = attr(lines_with(&out, "frame")[0], "width") / 0.2;

    let bar = lines_with(&out, "scale")[0];
    let points = bar
        .split(" points=\"")
        .nth(1)
        .unwrap()
        .split('"')
        .next()
        .unwrap();
    let xs: Vec<f64> = points
        .split(' ')
        .map(|p| p.split(',').next().unwrap().parse().unwrap())
        .collect();
    let label = lines_with(&out, "scale-label")[0];
    let text = label
        .split('>')
        .nth(1)
        .unwrap()
        .split(" m<")
        .next()
        .unwrap();
    let metres: f64 = text.replace(',', ".").parse().unwrap();

    assert!(((xs[1] - xs[0]) / scale - metres).abs() < 0.001, "{label}");
}

#[test]
fn labels_under_circles_are_centred() {
    let out = svg(&list(), &page(), Theme::Light).unwrap();
    let label = out
        .lines()
        .find(|l| l.contains("class=\"label\"") && l.contains(">Elisabeth<"))
        .unwrap();

    assert!(label.contains("text-anchor=\"middle\""));
}

/// A figure on its mark: the number and the name must not sit on top of each other.
#[test]
fn mark_number_and_name_do_not_overlap() {
    let out = svg(&list(), &page(), Theme::Light).unwrap();
    let labels = lines_with(&out, "label");
    let name = labels.iter().find(|l| l.contains(">Elisabeth<")).unwrap();
    let number = labels
        .iter()
        .find(|l| l.contains(">1<") && (attr(l, "x") - attr(name, "x")).abs() < FRAME_TOLERANCE)
        .unwrap();

    let apart = (attr(number, "y") - attr(name, "y")).abs();

    assert!(apart >= MIN_LABEL_PT, "{apart} pt apart\n{number}\n{name}");
}

/// The plan draws a lamp as a circle with a cone; the legend must show the same.
#[test]
fn legend_light_is_a_circle_with_a_cone() {
    let out = svg(&list(), &page(), Theme::Light).unwrap();
    let parts = lines_with(&out, "legend light");

    let circles = parts.iter().filter(|l| l.starts_with("<circle")).count();
    let cones = parts
        .iter()
        .filter(|l| l.starts_with("<polygon") && signature(l).contains("points=3"))
        .count();

    assert_eq!((circles, cones, parts.len()), (1, 1, 2), "{parts:#?}");
}

/// A circle's centre y and radius from its svg line.
fn circle_y_r(line: &str) -> (f64, f64) {
    (attr(line, "cy"), attr(line, "r"))
}

/// The number above a mark must clear the figure standing on it, at every scale.
#[test]
fn mark_number_clears_the_figure() {
    let sizes_m = [(20.0, 13.0), (3.0, 2.0), (80.0, 50.0)];
    let papers = [Paper::A4, Paper::A3];

    for (width_m, depth_m) in sizes_m {
        for paper in papers {
            let mut room = set();
            (room.width_m, room.depth_m) = (width_m, depth_m);
            let mut plan = plan();
            plan.figures.truncate(1);
            plan.figures[0].marks = vec![mark(0.0, width_m / 2.0, depth_m / 2.0)];
            let list = render(&project(), &room, &plan, 0.0, None);
            let page = Page::new(paper, Orientation::Landscape, TITLE, german());
            let out = svg(&list, &page, Theme::Light).unwrap();

            let figure = out
                .lines()
                .find(|l| l.contains("class=\"item figure ") && l.contains("<circle"))
                .unwrap();
            let (cy, r) = circle_y_r(figure);
            let number = lines_with(&out, "label")
                .into_iter()
                .find(|l| l.contains(">1<"))
                .unwrap();

            // Digits have no descender: the baseline is the glyph's lowest point.
            let clear_pt = cy - r - attr(number, "y");
            assert!(
                clear_pt > 0.0,
                "{width_m}x{depth_m} {paper:?}: {clear_pt} pt\n{figure}\n{number}"
            );
        }
    }
}

const PNG_SIGNATURE: [u8; 8] = *b"\x89PNG\r\n\x1a\n";
/// A4 landscape at 150 dpi: 297 mm and 210 mm over 25.4 mm per inch, times 150, rounded.
const A4_LANDSCAPE_PX: (u32, u32) = (1754, 1240);
const PNG_IHDR_AT: usize = 16;
const PNG_RGB_8BIT: [u8; 2] = [8, 2];
#[cfg(not(debug_assertions))]
const PNG_BUDGET_S: f64 = 1.0;

fn be32(bytes: &[u8], at: usize) -> u32 {
    u32::from_be_bytes(bytes[at..at + 4].try_into().unwrap())
}

#[test]
fn png_is_a_150_dpi_rgb_image_of_the_sheet() {
    let bytes = png(&list(), &page(), Theme::Light).unwrap();

    assert!(bytes.starts_with(&PNG_SIGNATURE));
    assert_eq!(&bytes[12..16], b"IHDR");
    assert_eq!(
        (be32(&bytes, PNG_IHDR_AT), be32(&bytes, PNG_IHDR_AT + 4)),
        A4_LANDSCAPE_PX
    );
    assert_eq!(bytes[PNG_IHDR_AT + 8..PNG_IHDR_AT + 10], PNG_RGB_8BIT);
    assert!(bytes.ends_with(b"IEND\xAE\x42\x60\x82"));
}

#[test]
fn png_follows_the_paper_and_the_page_size() {
    let a3 = Page::new(Paper::A3, Orientation::Portrait, TITLE, german());
    let bytes = png(&list(), &a3, Theme::Dark).unwrap();

    // 297 x 420 mm.
    assert_eq!(
        (be32(&bytes, PNG_IHDR_AT), be32(&bytes, PNG_IHDR_AT + 4)),
        (1754, 2480)
    );
}

#[test]
fn png_of_a_plan_without_size_is_an_error() {
    let mut empty = list();
    empty.width_m = 0.0;

    assert!(matches!(
        png(&empty, &page(), Theme::Light),
        Err(ExportError::EmptyPlan)
    ));
}

/// The example plan must render in under a second (release build only; a debug build is
/// many times slower).
#[cfg(not(debug_assertions))]
#[test]
fn png_of_the_example_renders_within_the_budget() {
    let start = std::time::Instant::now();
    let bytes = png(&list(), &page(), Theme::Light).unwrap();
    let took_s = start.elapsed().as_secs_f64();

    eprintln!("png: {} bytes in {took_s:.3} s", bytes.len());
    assert!(took_s < PNG_BUDGET_S, "{took_s} s");
}
