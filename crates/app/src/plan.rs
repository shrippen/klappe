//! The display list as `KanteShapeCanvas` shapes: Stellprobe's roles become Kante colour names
//! and line styles. No geometry here (agent.md): positions come as they are.

use klappe_services::{DisplayList, FigureColor, Item, Point, Role, Shape, Source};
use serde_json::{json, Value};

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
        Role::Wall => l("text", SOLID, "none"),
        Role::Object => l("muted", SOLID, "tint"),
        Role::LightBeam => l("orange", SOLID, "tint"),
        Role::Light => l("orange", SOLID, "solid"),
        Role::Axis => l("orange", DASHED, "none"),
        Role::CameraPath => l("yellow", DOTTED, "none"),
        Role::Frustum => l("yellow", SOLID, "tint"),
        Role::Camera => l("yellow", SOLID, "solid"),
        Role::FigurePath(c) => l(color(c), DOTTED, "none"),
        Role::Mark(c) => l(color(c), SOLID, "none"),
        Role::Figure(c) => l(color(c), SOLID, "solid"),
    }
}

/// Kante's label names (`KanteStyle.labels`).
pub fn color(c: FigureColor) -> &'static str {
    match c {
        FigureColor::Purple => "purple",
        FigureColor::Blue => "blue",
        FigureColor::Green => "green",
        FigureColor::Yellow => "yellow",
        FigureColor::Orange => "orange",
        FigureColor::Red => "red",
        FigureColor::Cyan => "cyan",
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
        Source::Wall(index) => format!("wall:{index}"),
    }
}

fn flat(points: &[Point]) -> Vec<f64> {
    points.iter().flat_map(|p| [p.x_m, p.y_m]).collect()
}

fn shape(item: &Item, selected: &dyn Fn(&Source) -> bool) -> Value {
    let look = look(item.role);
    let (kind, points, radius, width) = match &item.shape {
        Shape::Line { from, to, width_m } => ("line", flat(&[*from, *to]), 0.0, *width_m),
        Shape::Polyline { points } => ("polyline", flat(points), 0.0, 0.0),
        Shape::Polygon { points } => ("polygon", flat(points), 0.0, 0.0),
        Shape::Circle { center, radius_m } => ("circle", flat(&[*center]), *radius_m, 0.0),
    };

    json!({
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
    })
}

/// `{ width, depth, shapes: [...] }` for the canvas; `selected` marks the shot's camera.
pub fn shapes_json(list: &DisplayList, selected: &dyn Fn(&Source) -> bool) -> String {
    let shapes: Vec<Value> = list.items.iter().map(|i| shape(i, selected)).collect();

    json!({ "width": list.width_m, "depth": list.depth_m, "shapes": shapes }).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

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

        let v: Value = serde_json::from_str(&shapes_json(&list, &is_a)).unwrap_or_default();

        assert_eq!(v["width"], 20.0);
        assert_eq!(v["shapes"][0]["stroke"], "dashed");
        assert_eq!(v["shapes"][0]["label"], "orange");
        assert_eq!(v["shapes"][1]["kind"], "circle");
        assert_eq!(v["shapes"][1]["points"], json!([2.0, 3.0]));
        assert_eq!(v["shapes"][1]["key"], "camera:a");
        assert_eq!(v["shapes"][1]["selected"], true);
    }

    #[test]
    fn mark_keys_name_character_and_index() {
        let k = key(&Source::Mark {
            character: "e".into(),
            index: 2,
        });

        assert_eq!(k, "mark:e:2");
    }
}
