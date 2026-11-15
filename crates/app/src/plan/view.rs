//! The camera's view as `KanteShapeCanvas` shapes (LH-36): the same translation of a look to
//! Kante's words as the plan has. Positions come as they are.

use klappe_services::{view_look, CameraView};
use serde_json::{json, Value};

use super::{fill_of, hue_name, stroke_of};

/// `{ width, depth, shapes: [...] }` for the canvas, in image units: `depth` is the frame
/// height. Same names as the plan's JSON, so the canvas takes either.
pub fn view_json(view: &CameraView) -> String {
    let shapes: Vec<Value> = view
        .items
        .iter()
        .map(|item| {
            let look = view_look(item.role);

            json!({
                "kind": "polygon",
                "points": item.points.iter().flatten().collect::<Vec<_>>(),
                "radius": 0.0,
                "width": 0.0,
                "label": hue_name(look.hue),
                "stroke": stroke_of(look.line),
                "fill": fill_of(look.area),
                "text": item.label.clone().unwrap_or_default(),
                "key": "",
                "selected": false,
            })
        })
        .collect();

    json!({
        "width": klappe_services::FRAME_WIDTH,
        "depth": view.height,
        "shapes": shapes,
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use klappe_services::{FigureColor, ViewItem, ViewRole};

    use super::*;

    #[test]
    fn shapes_carry_the_look_of_their_role() {
        let view = CameraView {
            height: 0.5,
            horizon_y: 0.25,
            items: vec![
                ViewItem {
                    role: ViewRole::Floor,
                    points: vec![[0.0, 0.25], [1.0, 0.25], [1.0, 0.5]],
                    label: None,
                },
                ViewItem {
                    role: ViewRole::Figure(FigureColor::Red),
                    points: vec![[0.4, 0.2], [0.6, 0.2], [0.6, 0.4], [0.4, 0.4]],
                    label: Some("Elisabeth".into()),
                },
            ],
        };

        let json: Value = serde_json::from_str(&view_json(&view)).unwrap();

        assert_eq!(json["width"], 1.0);
        assert_eq!(json["depth"], 0.5);
        let shapes = json["shapes"].as_array().unwrap();
        assert_eq!(shapes[0]["label"], "muted");
        assert_eq!(shapes[0]["fill"], "tint");
        assert_eq!(shapes[1]["label"], "red");
        assert_eq!(shapes[1]["fill"], "solid");
        assert_eq!(shapes[1]["text"], "Elisabeth");
        assert_eq!(shapes[1]["points"].as_array().unwrap().len(), 8);
    }
}
