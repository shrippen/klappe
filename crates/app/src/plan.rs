//! The display list as `KanteShapeCanvas` shapes: Stellprobe's roles become Kante colour names
//! and line styles. No geometry here (agent.md): positions come as they are.

use std::path::Path;

use klappe_services::{
    look, Area, Dash, DisplayList, Hue, Item, Point, Role, Shape, Source, NOTE_TEXT_M,
};
use serde_json::{json, Value};

use crate::fileurl;

mod height;
mod key;
mod picture;
mod sweep;

use height::height_shape;
pub use height::HeightTexts;
pub use key::{key, parse_key, ElementKind, ParsedKey};
pub use picture::{frame_json, ppm, Palette, IMAGE_FORMAT};
use sweep::sweep_shape;
pub use sweep::Sweep;

/// Radius of the points of the scale gesture on the plan.
const GUIDE_POINT_RADIUS_M: f64 = 0.15;

/// How a role is drawn: Kante colour name, line style, fill.
struct Style {
    label: &'static str,
    stroke: &'static str,
    fill: &'static str,
}

const SOLID: &str = "solid";
const DASHED: &str = "dashed";
const DOTTED: &str = "dotted";

/// Stellprobe says what a role looks like; this says it in Kante's words.
fn style(role: Role) -> Style {
    let look = look(role);

    Style {
        label: hue_name(look.hue),
        stroke: stroke_of(look.line),
        fill: fill_of(look.area),
    }
}

fn hue_name(hue: Hue) -> &'static str {
    match hue {
        Hue::Ground => "ground",
        Hue::Text => "text",
        Hue::Muted => "muted",
        Hue::Accent => "accent",
        Hue::Surface => "surface",
        Hue::Color(c) => c.name(),
    }
}

fn stroke_of(dash: Dash) -> &'static str {
    match dash {
        Dash::Solid => SOLID,
        Dash::Dashed => DASHED,
        Dash::Dotted => DOTTED,
    }
}

fn fill_of(area: Area) -> &'static str {
    match area {
        Area::None => "none",
        Area::Tint => "tint",
        Area::Solid => SOLID,
    }
}

fn flat(points: &[Point]) -> Vec<f64> {
    points.iter().flat_map(|p| [p.x_m, p.y_m]).collect()
}

/// A `file://` URL for `file` (relative to the project folder `dir`): the canvas wants an
/// absolute URL.
fn file_url(dir: &Path, file: &str) -> String {
    let joined = dir.join(file);
    let absolute = std::path::absolute(&joined).unwrap_or(joined);

    fileurl::from_path(&absolute)
}

fn image(dir: &Path, shape: &Shape) -> Option<Value> {
    let Shape::Image {
        file,
        at,
        width_m,
        depth_m,
        rotation_deg,
        opacity,
    } = shape
    else {
        return None;
    };

    Some(json!({
        "kind": "image",
        "source": file_url(dir, file),
        "points": flat(&[*at]),
        "width": width_m,
        "height": depth_m,
        "rotation": rotation_deg,
        "opacity": opacity,
        "key": "",
    }))
}

fn shape(dir: &Path, item: &Item, selected: &dyn Fn(&Source) -> bool) -> Option<Value> {
    if let Some(image) = image(dir, &item.shape) {
        return Some(image);
    }

    let look = style(item.role);
    let (kind, points, radius, width) = match &item.shape {
        Shape::Line { from, to, width_m } => ("line", flat(&[*from, *to]), 0.0, *width_m),
        Shape::Polyline { points } => ("polyline", flat(points), 0.0, 0.0),
        Shape::Polygon { points } => ("polygon", flat(points), 0.0, 0.0),
        Shape::Circle { center, radius_m } => ("circle", flat(&[*center]), *radius_m, 0.0),
        Shape::Image { .. } => return None,
    };

    Some(json!({
        "kind": kind,
        "points": points,
        "radius": radius,
        "width": width,
        "label": look.label,
        "stroke": look.stroke,
        "fill": look.fill,
        "text": item.label.clone().unwrap_or_default(),
        "textAnchor": if item.role == Role::Note { "topLeft" } else { "center" },
        "captionSize": if item.role == Role::Note { NOTE_TEXT_M } else { 0.0 },
        "key": item.source.as_ref().map(key).unwrap_or_default(),
        "selected": item.source.as_ref().is_some_and(selected),
    }))
}

/// The points picked so far for a scale or an area, and the line or path between them: a
/// preview on top of the plan. Not part of the project.
fn guide(points: &[Point]) -> Vec<Value> {
    let mut shapes = Vec::new();

    if let [from, to] = points {
        shapes.push(json!({
            "kind": "line", "points": flat(&[*from, *to]), "radius": 0.0, "width": 0.0,
            "label": "focus", "stroke": DASHED, "fill": "none", "text": "", "key": "",
        }));
    }
    if points.len() > 2 {
        shapes.push(json!({
            "kind": "polyline", "points": flat(points), "radius": 0.0, "width": 0.0,
            "label": "focus", "stroke": DASHED, "fill": "none", "text": "", "key": "",
        }));
    }
    for point in points {
        shapes.push(json!({
            "kind": "circle", "points": flat(&[*point]), "radius": GUIDE_POINT_RADIUS_M,
            "width": 0.0, "label": "focus", "stroke": SOLID, "fill": "solid", "text": "",
            "key": "",
        }));
    }

    shapes
}

/// A measurement on the plan: its points (one or two) and, with two, the length as text.
pub struct Measure {
    pub points: Vec<Point>,
    pub caption: String,
}

/// The measuring tool's dashed line with a dot at each end, the length written on it. Not
/// part of the project.
fn measure_shapes(measure: &Measure) -> Vec<Value> {
    let mut shapes = guide(&measure.points);

    // `guide` puts the line first, when there is one; else the caption sits on the one dot.
    if let Some(first) = shapes.first_mut() {
        first["text"] = json!(measure.caption);
    }

    shapes
}

/// `{ width, depth, shapes: [...] }` for the canvas; `selected` marks the shot's camera.
/// `dir` is the project folder the image files lie in; `picked` the points of a scale in
/// progress, `marquee` the box or lasso being pulled up, `measure` the measurement on screen.
pub fn shapes_json(
    list: &DisplayList,
    dir: &Path,
    selected: &dyn Fn(&Source) -> bool,
    picked: &[Point],
    marquee: Option<&Sweep>,
    measure: Option<&Measure>,
    heights: &HeightTexts,
) -> String {
    let mut shapes: Vec<Value> = list
        .items
        .iter()
        .filter_map(|i| match i.role {
            Role::Height { cm } => height_shape(i, cm, heights),
            _ => shape(dir, i, selected),
        })
        .collect();
    shapes.extend(guide(picked));
    shapes.extend(marquee.map(sweep_shape));
    shapes.extend(measure.map(measure_shapes).unwrap_or_default());

    json!({ "width": list.width_m, "depth": list.depth_m, "shapes": shapes }).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use klappe_services::{FigureColor, Fill};

    fn item(role: Role, shape: Shape, source: Option<Source>) -> Item {
        Item {
            role,
            shape,
            label: Some("A".into()),
            source,
        }
    }

    /// The plan's JSON with nothing picked, dragged or measured.
    fn json_of(list: &DisplayList, selected: &dyn Fn(&Source) -> bool) -> String {
        let texts = HeightTexts::default();
        shapes_json(list, Path::new("/p"), selected, &[], None, None, &texts)
    }

    /// Every role as (role, label, stroke, fill) as the canvas gets it; pins `look`.
    #[test]
    fn every_role_has_its_look() {
        let red = FigureColor::Red;
        let rows = [
            (Role::Background, "muted", "solid", "none"),
            (Role::Wall, "text", "solid", "none"),
            (Role::Object, "muted", "solid", "tint"),
            (Role::LightBeam, "orange", "solid", "tint"),
            (Role::Light, "orange", "solid", "solid"),
            (Role::Note, "text", "solid", "tint"),
            (Role::Door, "text", "solid", "none"),
            (Role::Window, "text", "solid", "none"),
            (Role::ArrowHead(red), "red", "solid", "solid"),
            (Role::Axis, "orange", "dashed", "none"),
            (
                Role::CameraPath { dash: Dash::Solid },
                "yellow",
                "solid",
                "none",
            ),
            (
                Role::CameraPath { dash: Dash::Dashed },
                "yellow",
                "dashed",
                "none",
            ),
            (
                Role::CameraPath { dash: Dash::Dotted },
                "yellow",
                "dotted",
                "none",
            ),
            (Role::Frustum, "yellow", "solid", "tint"),
            (Role::Camera, "yellow", "solid", "solid"),
            (Role::FigurePath(red), "red", "dotted", "none"),
            (Role::Mark(red), "red", "solid", "none"),
            (Role::FigureHandle(red), "red", "solid", "solid"),
            (Role::CameraHandle, "yellow", "solid", "solid"),
            (Role::DrawingHandle, "accent", "solid", "solid"),
            (Role::NoteHandle, "accent", "solid", "solid"),
            (Role::OpeningHandle, "accent", "solid", "solid"),
            (Role::LightHandle, "accent", "solid", "solid"),
            (Role::Figure(red), "red", "solid", "solid"),
            (Role::GroupDot, "ground", "solid", "solid"),
        ];
        let drawings = [
            (Dash::Solid, Fill::Outline, "solid", "none"),
            (Dash::Solid, Fill::Filled, "solid", "tint"),
            (Dash::Dashed, Fill::Outline, "dashed", "none"),
            (Dash::Dashed, Fill::Filled, "dashed", "tint"),
            (Dash::Dotted, Fill::Outline, "dotted", "none"),
            (Dash::Dotted, Fill::Filled, "dotted", "tint"),
        ];
        let mut cases: Vec<(Role, &str, &str, &str)> = rows.to_vec();
        for (dash, fill, stroke, area) in drawings {
            let role = Role::Drawing {
                color: FigureColor::Blue,
                dash,
                fill,
            };
            cases.push((role, "blue", stroke, area));
        }

        for (role, label, stroke, fill) in cases {
            let circle = Shape::Circle {
                center: Point::new(0.0, 0.0),
                radius_m: 1.0,
            };
            let v =
                shape(Path::new("/p"), &item(role, circle, None), &|_| false).unwrap_or_default();

            assert_eq!(v["label"], label, "{role:?}");
            assert_eq!(v["stroke"], stroke, "{role:?}");
            assert_eq!(v["fill"], fill, "{role:?}");
        }
    }

    #[test]
    fn roles_become_kante_names_and_styles() {
        let list = DisplayList {
            lamps: Vec::new(),
            items: vec![
                item(
                    Role::Axis,
                    Shape::Line {
                        from: Point::new(0.0, 0.0),
                        to: Point::new(1.0, 0.0),
                        width_m: 0.0,
                    },
                    None,
                ),
                item(
                    Role::Camera,
                    Shape::Circle {
                        center: Point::new(2.0, 3.0),
                        radius_m: 0.25,
                    },
                    Some(Source::Camera("a".into())),
                ),
            ],
            width_m: 20.0,
            depth_m: 13.0,
        };
        let is_a = |s: &Source| matches!(s, Source::Camera(id) if id == "a");

        let json = json_of(&list, &is_a);
        let v: Value = serde_json::from_str(&json).unwrap_or_default();

        assert_eq!(v["width"], 20.0);
        assert_eq!(v["shapes"][0]["stroke"], "dashed");
        assert_eq!(v["shapes"][0]["label"], "orange");
        assert_eq!(v["shapes"][1]["kind"], "circle");
        assert_eq!(v["shapes"][1]["points"], json!([2.0, 3.0]));
        assert_eq!(v["shapes"][1]["key"], "camera:a");
        assert_eq!(v["shapes"][1]["selected"], true);
    }

    /// The dots of a group take the text colour: in the body's colour they would vanish.
    #[test]
    fn group_dots_stand_out_from_the_body() {
        let at = Point::new(1.0, 1.0);
        let circle = |radius_m| Shape::Circle {
            center: at,
            radius_m,
        };
        let list = DisplayList {
            lamps: Vec::new(),
            items: vec![
                item(Role::Figure(FigureColor::Purple), circle(0.6), None),
                item(Role::GroupDot, circle(0.1), None),
            ],
            width_m: 20.0,
            depth_m: 13.0,
        };

        let json = json_of(&list, &|_| false);
        let v: Value = serde_json::from_str(&json).unwrap_or_default();

        assert_eq!(v["shapes"][1]["label"], "ground");
        assert_eq!(v["shapes"][1]["fill"], "solid");
    }

    #[test]
    fn a_note_is_a_card_with_its_text() {
        let list = DisplayList {
            lamps: Vec::new(),
            items: vec![Item {
                role: Role::Note,
                shape: Shape::Polygon {
                    points: vec![
                        Point::new(3.0, 4.0),
                        Point::new(5.0, 4.0),
                        Point::new(5.0, 5.0),
                        Point::new(3.0, 5.0),
                    ],
                },
                label: Some("Vorhang schließen".into()),
                source: Some(Source::Note("n1".into())),
            }],
            width_m: 20.0,
            depth_m: 13.0,
        };

        let json = json_of(&list, &|_| false);
        let v: Value = serde_json::from_str(&json).unwrap_or_default();

        assert_eq!(v["shapes"][0]["text"], "Vorhang schließen");
        assert_eq!(v["shapes"][0]["key"], "note:n1");
        assert_eq!(v["shapes"][0]["kind"], "polygon");
        assert_eq!(v["shapes"][0]["textAnchor"], "topLeft");
        assert_eq!(v["shapes"][0]["captionSize"], NOTE_TEXT_M);
        assert_eq!(v["shapes"][0]["fill"], "tint");
    }

    /// A dashed, filled drawing keeps its colour, dash and fill on the canvas; its arrow head
    /// is solid whatever the shaft is.
    #[test]
    fn a_drawing_keeps_colour_dash_and_fill() {
        let at = Some(Source::Drawing("d1".into()));
        let list = DisplayList {
            lamps: Vec::new(),
            items: vec![
                item(
                    Role::Drawing {
                        color: FigureColor::Cyan,
                        dash: Dash::Dashed,
                        fill: Fill::Filled,
                    },
                    Shape::Polygon {
                        points: vec![
                            Point::new(0.0, 0.0),
                            Point::new(1.0, 0.0),
                            Point::new(1.0, 1.0),
                        ],
                    },
                    at.clone(),
                ),
                item(
                    Role::ArrowHead(FigureColor::Cyan),
                    Shape::Polygon {
                        points: vec![
                            Point::new(0.0, 0.0),
                            Point::new(1.0, 0.0),
                            Point::new(1.0, 1.0),
                        ],
                    },
                    at,
                ),
            ],
            width_m: 20.0,
            depth_m: 13.0,
        };

        let json = json_of(&list, &|_| false);
        let v: Value = serde_json::from_str(&json).unwrap_or_default();

        assert_eq!(v["shapes"][0]["label"], "cyan");
        assert_eq!(v["shapes"][0]["stroke"], "dashed");
        assert_eq!(v["shapes"][0]["fill"], "tint");
        assert_eq!(v["shapes"][0]["key"], "drawing:d1");
        assert_eq!(v["shapes"][1]["stroke"], "solid");
        assert_eq!(v["shapes"][1]["fill"], "solid");
    }

    fn background_list() -> DisplayList {
        DisplayList {
            lamps: Vec::new(),
            items: vec![item(
                Role::Background,
                Shape::Image {
                    file: "bilder/Grundriss Nr 1 äö.png".into(),
                    at: Point::new(1.0, 2.0),
                    width_m: 10.0,
                    depth_m: 5.0,
                    rotation_deg: 15.0,
                    opacity: 0.5,
                },
                None,
            )],
            width_m: 20.0,
            depth_m: 13.0,
        }
    }

    #[test]
    fn the_background_is_an_image_with_an_absolute_url() {
        let json = shapes_json(
            &background_list(),
            Path::new("/home/a b/p#1"),
            &|_| false,
            &[],
            None,
            None,
            &HeightTexts::default(),
        );
        let v: Value = serde_json::from_str(&json).unwrap_or_default();
        let image = &v["shapes"][0];

        assert_eq!(image["kind"], "image");
        assert_eq!(
            image["source"],
            "file:///home/a%20b/p%231/bilder/Grundriss%20Nr%201%20%C3%A4%C3%B6.png"
        );
        assert_eq!(image["points"], json!([1.0, 2.0]));
        assert_eq!(image["width"], 10.0);
        assert_eq!(image["height"], 5.0);
        assert_eq!(image["rotation"], 15.0);
        assert_eq!(image["opacity"], 0.5);
        assert_eq!(image["key"], "");
        assert_eq!(v["shapes"].as_array().map(Vec::len), Some(1));
    }

    /// A relative project folder becomes an absolute URL: the canvas would resolve it
    /// against Kante.
    #[test]
    fn a_relative_project_folder_gives_an_absolute_url() {
        let url = file_url(Path::new("projekt"), "bilder/a.png");

        assert!(url.starts_with("file:///"), "{url}");
        assert!(url.ends_with("/projekt/bilder/a.png"), "{url}");
    }

    /// A project folder with a space, a `#` and an umlaut: the URL gives the same path back.
    #[test]
    fn a_file_url_reads_back_as_the_path() {
        let dir = Path::new("/home/a b/p#1");
        let file = "bilder/Grundriss Nr 1 \u{e4}\u{f6}.png";

        let url = file_url(dir, file);

        assert_eq!(fileurl::to_path(&url), dir.join(file));
    }

    #[test]
    fn a_measurement_is_a_dashed_line_with_its_length() {
        let list = DisplayList::default();
        let measure = Measure {
            points: vec![Point::new(1.0, 1.0), Point::new(4.0, 5.0)],
            caption: "5 m".into(),
        };

        let json = shapes_json(
            &list,
            Path::new("/p"),
            &|_| false,
            &[],
            None,
            Some(&measure),
            &HeightTexts::default(),
        );
        let v: Value = serde_json::from_str(&json).unwrap_or_default();
        let shapes = v["shapes"].as_array().cloned().unwrap_or_default();

        assert_eq!(shapes.len(), 3);
        assert_eq!(shapes[0]["kind"], "line");
        assert_eq!(shapes[0]["stroke"], "dashed");
        assert_eq!(shapes[0]["text"], "5 m");
        assert_eq!(shapes[0]["points"], json!([1.0, 1.0, 4.0, 5.0]));
        assert_eq!(shapes[1]["kind"], "circle");
        assert_eq!(shapes[2]["kind"], "circle");
    }

    #[test]
    fn a_first_measure_point_has_no_line_and_no_text() {
        let list = DisplayList::default();
        let measure = Measure {
            points: vec![Point::new(1.0, 1.0)],
            caption: String::new(),
        };

        let json = shapes_json(
            &list,
            Path::new("/p"),
            &|_| false,
            &[],
            None,
            Some(&measure),
            &HeightTexts::default(),
        );
        let v: Value = serde_json::from_str(&json).unwrap_or_default();

        assert_eq!(v["shapes"].as_array().map(Vec::len), Some(1));
        assert_eq!(v["shapes"][0]["kind"], "circle");
        assert_eq!(v["shapes"][0]["text"], "");
    }
}

#[cfg(test)]
mod perf;
