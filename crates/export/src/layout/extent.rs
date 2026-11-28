//! The part of the plan the printout shows: the room and everything drawn beyond it.

use klappe_model::Point;
use klappe_stellprobe::{DisplayList, Item, Role, Shape};

/// How far past the room an item may reach without widening the printout: a wall stands on
/// the room's edge with half its thickness outside.
const EDGE_SLACK_M: f64 = 0.25;

/// Corners of the box round the room and every item that belongs to the plan: a note or a
/// sofa half outside the room is still on the page.
pub(super) fn of(list: &DisplayList) -> (Point, Point) {
    let room_max = Point::new(list.width_m, list.depth_m);
    let mut min = Point::new(0.0, 0.0);
    let mut max = room_max;

    for p in list
        .items
        .iter()
        .filter(|i| shown(i.role))
        .flat_map(corners)
    {
        if !p.x_m.is_finite() || !p.y_m.is_finite() {
            continue;
        }

        let beyond = |v: f64, edge: f64| (v - edge).abs() > EDGE_SLACK_M;
        if p.x_m < 0.0 && beyond(p.x_m, 0.0) {
            min.x_m = min.x_m.min(p.x_m);
        }
        if p.y_m < 0.0 && beyond(p.y_m, 0.0) {
            min.y_m = min.y_m.min(p.y_m);
        }
        if p.x_m > room_max.x_m && beyond(p.x_m, room_max.x_m) {
            max.x_m = max.x_m.max(p.x_m);
        }
        if p.y_m > room_max.y_m && beyond(p.y_m, room_max.y_m) {
            max.y_m = max.y_m.max(p.y_m);
        }
    }

    (min, max)
}

/// Whether an item widens the printout. Lines that run on past the set (the axis, the view
/// cone, a beam, the sun) are cut at the room, as are the background and the grips.
fn shown(role: Role) -> bool {
    !matches!(
        role,
        Role::Background
            | Role::Axis
            | Role::Frustum
            | Role::LightBeam
            | Role::SunPatch
            | Role::SunRay
            | Role::North
            | Role::Handle
    )
}

/// The points that bound an item.
fn corners(item: &Item) -> Vec<Point> {
    match &item.shape {
        Shape::Line { from, to, width_m } => {
            let pad = width_m / 2.0;

            [*from, *to]
                .iter()
                .flat_map(|p| {
                    [
                        Point::new(p.x_m - pad, p.y_m - pad),
                        Point::new(p.x_m + pad, p.y_m + pad),
                    ]
                })
                .collect()
        }
        Shape::Polyline { points } | Shape::Polygon { points } => points.clone(),
        Shape::Circle { center, radius_m } => vec![
            Point::new(center.x_m - radius_m, center.y_m - radius_m),
            Point::new(center.x_m + radius_m, center.y_m + radius_m),
        ],
        Shape::Image {
            at,
            width_m,
            depth_m,
            rotation_deg,
            ..
        } => [
            (0.0, 0.0),
            (*width_m, 0.0),
            (*width_m, *depth_m),
            (0.0, *depth_m),
        ]
        .iter()
        .map(|(dx, dy)| Point::new(at.x_m + dx, at.y_m + dy).turned(*at, *rotation_deg))
        .collect(),
    }
}
