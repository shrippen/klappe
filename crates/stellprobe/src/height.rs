//! How tall everything on the set is (LH-90, LH-91), in one place: what the model leaves
//! empty takes a default here, so every element already has a height when it is drawn.
//!
//! ```text
//!   wall        own height_m        else the set's wall_height_m
//!   opening     own sill / head     else its kind's (door 0 / 2.10, window 0.90 / 2.10)
//!   object      base_m + height_m   base empty: on the largest object under its centre,
//!                                   else the floor; height empty: MISSING_HEIGHT_M
//!   figure      its pose            standing, sitting, lying
//! ```

use klappe_model::{outline_area_m2, ObjectShape, Opening, Pose, SetObject, SetPlan, Wall};

use crate::display::object_corners;
use crate::hit::in_lasso;

/// What an object without a height is shown as, e.g. a free form not yet measured.
pub(crate) const MISSING_HEIGHT_M: f64 = 0.9;
const STANDING_M: f64 = 1.75;
const SITTING_M: f64 = 1.25;
const LYING_M: f64 = 0.4;

/// Where the wall ends above the floor.
pub fn wall_top(set: &SetPlan, wall: &Wall) -> f64 {
    wall.height_m.unwrap_or(set.wall_height_m)
}

/// Where the opening begins and ends above the floor: sill and head.
pub fn opening_span(opening: &Opening) -> (f64, f64) {
    opening.sill_head_m()
}

/// Underside and top of the object at `index` in `set.objects`, e.g. a chair on a podium:
/// (0.4, 0.85). Without an own underside it stands on the largest object with a height
/// that holds its centre and is larger than itself, else on the floor.
pub fn object_extent(set: &SetPlan, index: usize) -> (f64, f64) {
    let object = &set.objects[index];
    let base_m = object.base_m.unwrap_or_else(|| support_top(set, index));

    (base_m, base_m + object.height_m.unwrap_or(MISSING_HEIGHT_M))
}

/// How tall a figure is in a pose.
pub(crate) fn figure_height_m(pose: Pose) -> f64 {
    match pose {
        Pose::Standing => STANDING_M,
        Pose::Sitting => SITTING_M,
        Pose::Lying => LYING_M,
    }
}

/// The top of what the object at `index` stands on; 0 on the floor.
/// Only a larger object carries a smaller one, so stacks never go round in a circle.
fn support_top(set: &SetPlan, index: usize) -> f64 {
    let object = &set.objects[index];
    let support = set
        .objects
        .iter()
        .enumerate()
        .filter(|(other, o)| *other != index && o.height_m.is_some())
        .filter(|(other, o)| larger(o, *other, object, index))
        .filter(|(_, o)| in_lasso(&object_corners(o), object.at))
        .max_by(|(a, x), (b, y)| area_m2(x).total_cmp(&area_m2(y)).then(b.cmp(a)));

    support.map_or(0.0, |(other, _)| object_extent(set, other).1)
}

/// Whether `a` is larger than `b`; of two the same size, the earlier one.
fn larger(a: &SetObject, a_index: usize, b: &SetObject, b_index: usize) -> bool {
    let (a_m2, b_m2) = (area_m2(a), area_m2(b));

    a_m2 > b_m2 || (a_m2 == b_m2 && a_index < b_index)
}

/// The footprint of `object`: its box, but a free form's real area.
fn area_m2(object: &SetObject) -> f64 {
    match object.shape {
        ObjectShape::Polygon { .. } => outline_area_m2(&object_corners(object)),
        _ => object.width_m * object.depth_m,
    }
}

#[cfg(test)]
mod tests {
    use klappe_model::{ObjectShape, OpeningKind, Point};

    use super::*;

    fn object(at: (f64, f64), size_m: f64, height_m: Option<f64>) -> SetObject {
        SetObject {
            id: String::new(),
            kind: String::new(),
            label: String::new(),
            at: Point::new(at.0, at.1),
            width_m: size_m,
            depth_m: size_m,
            rotation_deg: 0.0,
            shape: ObjectShape::Rect,
            height_m,
            base_m: None,
            mirrored: false,
        }
    }

    fn set(objects: Vec<SetObject>) -> SetPlan {
        SetPlan {
            objects,
            ..SetPlan::default()
        }
    }

    #[test]
    fn a_wall_takes_the_room_height_unless_it_has_its_own() {
        let mut set = set(Vec::new());
        set.wall_height_m = 3.2;
        let mut wall = Wall::new(Point::new(0.0, 0.0), Point::new(4.0, 0.0), 0.2);

        assert_eq!(wall_top(&set, &wall), 3.2);

        wall.height_m = Some(1.1);
        assert_eq!(wall_top(&set, &wall), 1.1);
    }

    #[test]
    fn an_opening_takes_its_kinds_sill_and_head_unless_it_has_its_own() {
        let mut window = Opening::new(OpeningKind::Window, 1.0, 1.2);

        assert_eq!(
            opening_span(&Opening::new(OpeningKind::Door, 1.0, 0.9)),
            (0.0, 2.1)
        );
        assert_eq!(opening_span(&window), (0.9, 2.1));

        window.sill_m = Some(1.4);
        window.head_m = Some(2.4);
        assert_eq!(opening_span(&window), (1.4, 2.4));
    }

    /// A chair (0.45 m) on a podium (0.4 m) on the floor: 0.4 to 0.85.
    #[test]
    fn an_object_stands_on_the_larger_one_under_its_centre() {
        let set = set(vec![
            object((2.0, 2.0), 0.5, Some(0.45)),
            object((2.0, 2.0), 3.0, Some(0.4)),
        ]);

        assert_eq!(object_extent(&set, 1), (0.0, 0.4));
        let (base_m, top_m) = object_extent(&set, 0);
        assert_eq!(base_m, 0.4);
        assert!((top_m - 0.85).abs() < 1e-9);
    }

    #[test]
    fn an_object_beside_another_stands_on_the_floor() {
        let set = set(vec![
            object((5.0, 5.0), 0.5, Some(0.45)),
            object((2.0, 2.0), 3.0, Some(0.4)),
        ]);

        assert_eq!(object_extent(&set, 0), (0.0, 0.45));
    }

    /// A rug has no height: what stands on it stands on the floor.
    #[test]
    fn an_object_without_height_carries_nothing() {
        let set = set(vec![
            object((2.0, 2.0), 0.5, Some(0.45)),
            object((2.0, 2.0), 3.0, None),
        ]);

        assert_eq!(object_extent(&set, 0).0, 0.0);
        assert_eq!(object_extent(&set, 1), (0.0, MISSING_HEIGHT_M));
    }

    /// Two the same size on one spot: the later stands on the earlier, not both on each other.
    #[test]
    fn two_objects_the_same_size_stack_once() {
        let set = set(vec![
            object((2.0, 2.0), 1.0, Some(0.5)),
            object((2.0, 2.0), 1.0, Some(0.5)),
        ]);

        assert_eq!(object_extent(&set, 0), (0.0, 0.5));
        assert_eq!(object_extent(&set, 1), (0.5, 1.0));
    }

    #[test]
    fn an_own_underside_wins_over_stacking() {
        let mut shelf = object((2.0, 2.0), 0.5, Some(0.3));
        shelf.base_m = Some(1.5);
        let set = set(vec![shelf, object((2.0, 2.0), 3.0, Some(0.4))]);

        assert_eq!(object_extent(&set, 0), (1.5, 1.8));
    }

    /// A thin L has a large box but a small area: a smaller square with more area carries.
    #[test]
    fn a_free_form_counts_with_its_real_area() {
        let points = [
            (-2.0, -2.0),
            (-1.0, -2.0),
            (-1.0, 1.0),
            (2.0, 1.0),
            (2.0, 2.0),
            (-2.0, 2.0),
        ];
        let mut thin_l = object((0.0, 0.0), 4.0, Some(0.5));
        thin_l.shape = ObjectShape::Polygon {
            points: points.iter().map(|&(x, y)| Point::new(x, y)).collect(),
        };
        let square = object((0.0, 0.0), 3.0, Some(1.0));
        let small = object((-1.2, 1.2), 0.5, Some(0.2));
        let set = set(vec![thin_l, square, small]);

        assert_eq!(object_extent(&set, 2), (1.0, 1.2));
    }
}
