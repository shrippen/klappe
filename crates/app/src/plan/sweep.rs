//! The box or lasso a drag on empty ground pulls up, as a canvas shape.

use klappe_services::Point;
use serde_json::{json, Value};

use super::{flat, DASHED};

/// What a drag on empty ground pulls up to take what lies in it (LH-08).
#[derive(Debug, Clone, PartialEq)]
pub enum Sweep {
    /// The box between two corners.
    Box(Point, Point),
    /// The freehand line so far, in metres.
    Lasso(Vec<Point>),
}

/// The box a drag on empty ground pulls up: outlined and tinted, as the other previews are;
/// the lasso is the dashed open line the pointer drew.
pub(super) fn sweep_shape(sweep: &Sweep) -> Value {
    match sweep {
        Sweep::Box(a, b) => json!({
            "kind": "polygon",
            "points": flat(&[*a, Point::new(b.x_m, a.y_m), *b, Point::new(a.x_m, b.y_m)]),
            "radius": 0.0, "width": 0.0, "label": "focus", "stroke": DASHED, "fill": "tint",
            "text": "", "key": "",
        }),
        Sweep::Lasso(points) => json!({
            "kind": "polyline", "points": flat(points), "radius": 0.0, "width": 0.0,
            "label": "focus", "stroke": DASHED, "fill": "none", "text": "", "key": "",
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plan::shapes_json;
    use klappe_services::DisplayList;
    use std::path::Path;

    #[test]
    fn a_box_being_pulled_is_a_tinted_outline() {
        let list = DisplayList::default();
        let corners = Sweep::Box(Point::new(1.0, 2.0), Point::new(4.0, 3.0));

        let json = shapes_json(
            &list,
            Path::new("/p"),
            &|_| false,
            &[],
            Some(&corners),
            None,
        );
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
    fn a_lasso_being_drawn_is_a_dashed_open_line() {
        let list = DisplayList::default();
        let lasso = Sweep::Lasso(vec![
            Point::new(1.0, 2.0),
            Point::new(4.0, 2.0),
            Point::new(3.0, 5.0),
        ]);

        let json = shapes_json(&list, Path::new("/p"), &|_| false, &[], Some(&lasso), None);
        let v: Value = serde_json::from_str(&json).unwrap_or_default();

        assert_eq!(v["shapes"][0]["kind"], "polyline");
        assert_eq!(
            v["shapes"][0]["points"],
            json!([1.0, 2.0, 4.0, 2.0, 3.0, 5.0])
        );
        assert_eq!(v["shapes"][0]["stroke"], "dashed");
        assert_eq!(v["shapes"][0]["fill"], "none");
    }
}
