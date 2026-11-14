//! The display list as `KanteShapeCanvas` shapes: Stellprobe's roles become Kante colour names
//! and line styles. No geometry here (agent.md): positions come as they are.

use std::path::Path;

use klappe_services::{Dash, DisplayList, Fill, Item, Point, Role, Shape, Source};
use serde_json::{json, Value};

/// Radius of the points of the scale gesture on the plan.
const GUIDE_POINT_RADIUS_M: f64 = 0.15;

/// How a role is drawn: Kante colour name, line style, fill.
struct Look {
    label: &'static str,
    stroke: &'static str,
    fill: &'static str,
}

const SOLID: &str = "solid";
const DASHED: &str = "dashed";
const DOTTED: &str = "dotted";

fn look(role: Role) -> Look {
    let l = |label, stroke, fill| Look {
        label,
        stroke,
        fill,
    };

    match role {
        // The image takes no colour; the canvas draws it as it is.
        Role::Background => l("muted", SOLID, "none"),
        Role::Wall => l("text", SOLID, "none"),
        Role::Object => l("muted", SOLID, "tint"),
        Role::LightBeam => l("orange", SOLID, "tint"),
        Role::Light => l("orange", SOLID, "solid"),
        Role::Note => l("accent", SOLID, "solid"),
        Role::Drawing {
            color: c,
            dash,
            fill,
        } => l(c.name(), stroke_of(dash), fill_of(fill)),
        Role::ArrowHead(c) => l(c.name(), SOLID, "solid"),
        Role::Axis => l("orange", DASHED, "none"),
        Role::CameraPath => l("yellow", DOTTED, "none"),
        Role::Frustum => l("yellow", SOLID, "tint"),
        Role::Camera => l("yellow", SOLID, "solid"),
        Role::FigurePath(c) => l(c.name(), DOTTED, "none"),
        Role::Mark(c) => l(c.name(), SOLID, "none"),
        Role::FigureHandle(c) => l(c.name(), SOLID, "solid"),
        Role::CameraHandle => l("yellow", SOLID, "solid"),
        Role::Figure(c) => l(c.name(), SOLID, "solid"),
        // The ground colour stands out from every figure colour (the text colour did not).
        Role::GroupDot => l("ground", SOLID, "solid"),
    }
}

fn stroke_of(dash: Dash) -> &'static str {
    match dash {
        Dash::Solid => SOLID,
        Dash::Dashed => DASHED,
        Dash::Dotted => DOTTED,
    }
}

/// A filled drawing is tinted, like the view cone: the plan below stays readable.
fn fill_of(fill: Fill) -> &'static str {
    match fill {
        Fill::Outline => "none",
        Fill::Filled => "tint",
    }
}

/// `camera:a`, `figure:elisabeth`, `mark:elisabeth:0` …; the app's tooltips read it back.
pub fn key(source: &Source) -> String {
    match source {
        Source::Object(id) => format!("object:{id}"),
        Source::Light(id) => format!("light:{id}"),
        Source::Camera(id) => format!("camera:{id}"),
        Source::Figure(id) => format!("figure:{id}"),
        Source::Mark { character, index } => format!("mark:{character}:{index}"),
        Source::FigureVia { character, index } => format!("figurevia:{character}:{index}"),
        Source::CameraVia { camera, index } => format!("cameravia:{camera}:{index}"),
        Source::Wall(index) => format!("wall:{index}"),
        Source::Note(id) => format!("note:{id}"),
        Source::Drawing(id) => format!("drawing:{id}"),
    }
}

fn flat(points: &[Point]) -> Vec<f64> {
    points.iter().flat_map(|p| [p.x_m, p.y_m]).collect()
}

/// A `file://` URL for `file` (relative to the project folder `dir`): the canvas wants an
/// absolute URL. Anything but unreserved characters and `/` is percent-encoded.
fn file_url(dir: &Path, file: &str) -> String {
    let joined = dir.join(file);
    let absolute = std::path::absolute(&joined).unwrap_or(joined);
    let path = absolute.to_string_lossy().replace('\\', "/");

    // A Windows path (`C:/…`) needs the slash after `file://`.
    let mut url = String::from("file://");
    if !path.starts_with('/') {
        url.push('/');
    }
    for byte in path.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' | b'/' | b':' => {
                url.push(char::from(byte));
            }
            _ => url.push_str(&format!("%{byte:02X}")),
        }
    }

    url
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

    let look = look(item.role);
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

/// The box a drag on empty ground pulls up to take what lies in it (LH-08): outlined and
/// tinted, as the other previews are.
fn marquee_shape((a, b): (Point, Point)) -> Value {
    json!({
        "kind": "polygon",
        "points": flat(&[a, Point::new(b.x_m, a.y_m), b, Point::new(a.x_m, b.y_m)]),
        "radius": 0.0, "width": 0.0, "label": "focus", "stroke": DASHED, "fill": "tint",
        "text": "", "key": "",
    })
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

    // `guide` puts the line first, when there is one.
    if let Some(line) = shapes.first_mut().filter(|s| s["kind"] == "line") {
        line["text"] = json!(measure.caption);
    }

    shapes
}

/// `{ width, depth, shapes: [...] }` for the canvas; `selected` marks the shot's camera.
/// `dir` is the project folder the image files lie in; `picked` the points of a scale in
/// progress, `marquee` the box being pulled up, `measure` the measurement on screen.
pub fn shapes_json(
    list: &DisplayList,
    dir: &Path,
    selected: &dyn Fn(&Source) -> bool,
    picked: &[Point],
    marquee: Option<(Point, Point)>,
    measure: Option<&Measure>,
) -> String {
    let mut shapes: Vec<Value> = list
        .items
        .iter()
        .filter_map(|i| shape(dir, i, selected))
        .collect();
    shapes.extend(guide(picked));
    shapes.extend(marquee.map(marquee_shape));
    shapes.extend(measure.map(measure_shapes).unwrap_or_default());

    json!({ "width": list.width_m, "depth": list.depth_m, "shapes": shapes }).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use klappe_services::FigureColor;

    fn item(role: Role, shape: Shape, source: Option<Source>) -> Item {
        Item {
            role,
            shape,
            label: Some("A".into()),
            source,
        }
    }

    #[test]
    fn roles_become_kante_names_and_styles() {
        let list = DisplayList {
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

        let json = shapes_json(&list, Path::new("/p"), &is_a, &[], None, None);
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
            items: vec![
                item(Role::Figure(FigureColor::Purple), circle(0.6), None),
                item(Role::GroupDot, circle(0.1), None),
            ],
            width_m: 20.0,
            depth_m: 13.0,
        };

        let json = shapes_json(&list, Path::new("/p"), &|_| false, &[], None, None);
        let v: Value = serde_json::from_str(&json).unwrap_or_default();

        assert_eq!(v["shapes"][1]["label"], "ground");
        assert_eq!(v["shapes"][1]["fill"], "solid");
    }

    #[test]
    fn a_note_is_a_marker_with_its_text() {
        let list = DisplayList {
            items: vec![Item {
                role: Role::Note,
                shape: Shape::Circle {
                    center: Point::new(3.0, 4.0),
                    radius_m: 0.15,
                },
                label: Some("Vorhang schließen".into()),
                source: Some(Source::Note("n1".into())),
            }],
            width_m: 20.0,
            depth_m: 13.0,
        };

        let json = shapes_json(&list, Path::new("/p"), &|_| false, &[], None, None);
        let v: Value = serde_json::from_str(&json).unwrap_or_default();

        assert_eq!(v["shapes"][0]["text"], "Vorhang schließen");
        assert_eq!(v["shapes"][0]["key"], "note:n1");
        assert_eq!(v["shapes"][0]["label"], "accent");
    }

    /// A dashed, filled drawing keeps its colour, dash and fill on the canvas; its arrow head
    /// is solid whatever the shaft is.
    #[test]
    fn a_drawing_keeps_colour_dash_and_fill() {
        let at = Some(Source::Drawing("d1".into()));
        let list = DisplayList {
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

        let json = shapes_json(&list, Path::new("/p"), &|_| false, &[], None, None);
        let v: Value = serde_json::from_str(&json).unwrap_or_default();

        assert_eq!(v["shapes"][0]["label"], "cyan");
        assert_eq!(v["shapes"][0]["stroke"], "dashed");
        assert_eq!(v["shapes"][0]["fill"], "tint");
        assert_eq!(v["shapes"][0]["key"], "drawing:d1");
        assert_eq!(v["shapes"][1]["stroke"], "solid");
        assert_eq!(v["shapes"][1]["fill"], "solid");
    }

    #[test]
    fn grip_keys_name_track_and_index() {
        let figure = Source::FigureVia {
            character: "e".into(),
            index: 1,
        };
        let camera = Source::CameraVia {
            camera: "a".into(),
            index: 1,
        };

        assert_eq!(key(&figure), "figurevia:e:1");
        assert_eq!(key(&camera), "cameravia:a:1");
    }

    #[test]
    fn mark_keys_name_character_and_index() {
        let k = key(&Source::Mark {
            character: "e".into(),
            index: 2,
        });

        assert_eq!(k, "mark:e:2");
    }

    fn background_list() -> DisplayList {
        DisplayList {
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

    #[test]
    fn a_box_being_pulled_is_a_tinted_outline() {
        let list = DisplayList::default();
        let corners = Some((Point::new(1.0, 2.0), Point::new(4.0, 3.0)));

        let json = shapes_json(&list, Path::new("/p"), &|_| false, &[], corners, None);
        let v: Value = serde_json::from_str(&json).unwrap_or_default();

        assert_eq!(v["shapes"][0]["kind"], "polygon");
        assert_eq!(
            v["shapes"][0]["points"],
            json!([1.0, 2.0, 4.0, 2.0, 4.0, 3.0, 1.0, 3.0])
        );
        assert_eq!(v["shapes"][0]["fill"], "tint");
        assert_eq!(v["shapes"][0]["key"], "");
    }

    #[test]
    fn a_scale_in_progress_shows_its_points_and_line() {
        let list = DisplayList::default();
        let one = [Point::new(1.0, 1.0)];
        let two = [Point::new(1.0, 1.0), Point::new(4.0, 5.0)];

        let shapes = |picked: &[Point]| -> Vec<Value> {
            let json = shapes_json(&list, Path::new("/p"), &|_| false, picked, None, None);
            serde_json::from_str::<Value>(&json).unwrap_or_default()["shapes"]
                .as_array()
                .cloned()
                .unwrap_or_default()
        };

        assert!(shapes(&[]).is_empty());
        assert_eq!(shapes(&one).len(), 1);
        assert_eq!(shapes(&one)[0]["kind"], "circle");
        let both = shapes(&two);
        assert_eq!(both.len(), 3);
        assert_eq!(both[0]["kind"], "line");
        assert_eq!(both[0]["points"], json!([1.0, 1.0, 4.0, 5.0]));
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
        );
        let v: Value = serde_json::from_str(&json).unwrap_or_default();

        assert_eq!(v["shapes"].as_array().map(Vec::len), Some(1));
        assert_eq!(v["shapes"][0]["kind"], "circle");
        assert_eq!(v["shapes"][0]["text"], "");
    }
}
