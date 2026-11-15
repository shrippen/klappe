use serde::{Deserialize, Serialize};

use crate::geometry::Point;

/// Largest side of an object: a stadium fits; 1e300 m would not draw.
pub const MAX_OBJECT_M: f64 = 1000.0;

/// Whether `size_m` is a side an object can have: a number, more than nothing, not absurd.
pub fn is_object_size_m(size_m: f64) -> bool {
    size_m.is_finite() && size_m > 0.0 && size_m <= MAX_OBJECT_M
}

/// Tallest an object can be: a church tower fits; 1e300 m would not make sense.
pub const MAX_OBJECT_HEIGHT_M: f64 = 100.0;

/// Whether `height_m` is a height an object can have: a number, more than nothing, not absurd.
pub fn is_object_height_m(height_m: f64) -> bool {
    height_m.is_finite() && height_m > 0.0 && height_m <= MAX_OBJECT_HEIGHT_M
}

/// Whether `height_m` is a height above the floor something can begin at (a sill, an
/// underside): a number from the floor up, not absurd.
pub fn is_floor_offset_m(height_m: f64) -> bool {
    (0.0..=MAX_OBJECT_HEIGHT_M).contains(&height_m)
}

/// How an object is drawn within its width and depth.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ObjectShape {
    #[default]
    Rect,
    Ellipse,
    /// A free form (LH-92): the outline in metres from the object's centre `at`, before the
    /// rotation. `width_m` and `depth_m` are the size of the outline's bounding box.
    Polygon {
        points: Vec<Point>,
    },
}

/// The bounding box of `points`: its centre, width and depth in metres. A free form's size
/// (LH-92); empty `points` give a box of nothing at the origin.
pub fn outline_box(points: &[Point]) -> (Point, f64, f64) {
    let fold = |pick: fn(f64, f64) -> f64, of: fn(&Point) -> f64, start: f64| {
        points.iter().map(of).fold(start, pick)
    };
    let (min_x, max_x) = (
        fold(f64::min, |p| p.x_m, f64::INFINITY),
        fold(f64::max, |p| p.x_m, f64::NEG_INFINITY),
    );
    let (min_y, max_y) = (
        fold(f64::min, |p| p.y_m, f64::INFINITY),
        fold(f64::max, |p| p.y_m, f64::NEG_INFINITY),
    );
    if points.is_empty() {
        return (Point::default(), 0.0, 0.0);
    }

    let centre = Point::new((min_x + max_x) / 2.0, (min_y + max_y) / 2.0);

    (centre, max_x - min_x, max_y - min_y)
}

/// The area of the closed outline `points` in square metres (shoelace formula; the sign of the
/// turn does not matter).
pub fn outline_area_m2(points: &[Point]) -> f64 {
    let next = points.iter().cycle().skip(1);
    let twice: f64 = points
        .iter()
        .zip(next)
        .map(|(a, b)| a.x_m * b.y_m - b.x_m * a.y_m)
        .sum();

    twice.abs() / 2.0
}

/// Fewest points of a free form's outline.
const MIN_OUTLINE_POINTS: usize = 3;

/// Whether `points` can be the outline of a free form (LH-92): three or more finite points that
/// enclose an area, within the size of an object.
pub fn is_outline(points: &[Point]) -> bool {
    if points.len() < MIN_OUTLINE_POINTS {
        return false;
    }
    if !points
        .iter()
        .all(|p| p.x_m.is_finite() && p.y_m.is_finite())
    {
        return false;
    }

    let (_, width_m, depth_m) = outline_box(points);

    is_object_size_m(width_m) && is_object_size_m(depth_m) && outline_area_m2(points) > 0.0
}

/// An entry of the object library (LH-11, LH-12): what a new object of this `kind` looks like.
/// Built-in entries are named by the catalogue (`objectKind.<kind>`); an own entry carries
/// its `label`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LibraryItem {
    /// What `SetObject::kind` names, e.g. `table`.
    pub kind: String,
    /// The name of an own entry; empty for a built-in one.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub label: String,
    /// The group the list shows the entry under: a key for the catalogue
    /// (`objectGroup.<group>`) in a built-in entry, free text in an own one; empty for none.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub group: String,
    pub width_m: f64,
    pub depth_m: f64,
    #[serde(default)]
    pub shape: ObjectShape,
    /// Height of a new object of this kind (LH-17); `None` for flat ones like a rug.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height_m: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn l_shape() -> Vec<Point> {
        [
            (0.0, 0.0),
            (2.0, 0.0),
            (2.0, 1.0),
            (1.0, 1.0),
            (1.0, 3.0),
            (0.0, 3.0),
        ]
        .iter()
        .map(|&(x, y)| Point::new(x, y))
        .collect()
    }

    /// An L of 2 x 3 m with a 1 x 2 m notch has the area 4 and a box of 2 x 3 around (1, 1.5).
    #[test]
    fn an_outline_has_a_box_and_an_area() {
        let (centre, width_m, depth_m) = outline_box(&l_shape());

        assert_eq!((centre, width_m, depth_m), (Point::new(1.0, 1.5), 2.0, 3.0));
        assert_eq!(outline_area_m2(&l_shape()), 4.0);

        let mut backwards = l_shape();
        backwards.reverse();
        assert_eq!(outline_area_m2(&backwards), 4.0);
        assert_eq!(outline_box(&[]), (Point::default(), 0.0, 0.0));
        assert_eq!(outline_area_m2(&[]), 0.0);
    }

    /// An outline has three finite points round an area that fits an object.
    #[test]
    fn an_outline_needs_an_area() {
        let p = Point::new;

        assert!(is_outline(&l_shape()));
        assert!(!is_outline(&[p(0.0, 0.0), p(1.0, 1.0)]));
        assert!(!is_outline(&[p(0.0, 0.0), p(1.0, 1.0), p(2.0, 2.0)]));
        assert!(!is_outline(&[p(0.0, 0.0), p(1.0, 0.0), p(f64::NAN, 1.0)]));
        assert!(!is_outline(&[p(0.0, 0.0), p(1e6, 0.0), p(0.0, 1.0)]));
    }
}
