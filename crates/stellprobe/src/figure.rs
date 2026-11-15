//! How big a figure's body is on the plan (LH-23, LH-25).

use klappe_model::{Character, Point};

use crate::display::FIGURE_RADIUS_M;

/// A group's body is larger than a single figure; inside it stand `GROUP_DOTS` small
/// circles, however many people the group counts.
pub(crate) const GROUP_RADIUS_M: f64 = 0.6;
pub(crate) const GROUP_DOT_RADIUS_M: f64 = 0.17;
const GROUP_DOT_RING_M: f64 = 0.3;
pub(crate) const GROUP_DOTS: usize = 3;

/// The radius of the body: the figure's own size or the standard, a group's wider by the
/// same ratio as a standard group is to a standard figure.
pub(crate) fn body_radius_m(character: &Character) -> f64 {
    let single_m = character.size_m.map_or(FIGURE_RADIUS_M, |d| d / 2.0);
    match character.group {
        Some(_) => single_m * GROUP_RADIUS_M / FIGURE_RADIUS_M,
        None => single_m,
    }
}

/// The centres of the dots in a group's body: evenly on a ring, one straight up. `scale` is
/// the body's size against a standard group's.
pub(crate) fn group_dots(at: Point, scale: f64) -> Vec<Point> {
    (0..GROUP_DOTS)
        .map(|i| {
            let deg = -90.0 + 360.0 * i as f64 / GROUP_DOTS as f64;
            let a = deg.to_radians();
            Point::new(
                at.x_m + GROUP_DOT_RING_M * scale * a.cos(),
                at.y_m + GROUP_DOT_RING_M * scale * a.sin(),
            )
        })
        .collect()
}
