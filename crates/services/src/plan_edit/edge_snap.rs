use klappe_model::Point;
use klappe_stellprobe::{Item, Role, Shape};

/// How far from a corner or an edge a point still snaps to it, in metres.
const REACH_M: f64 = 0.2;

/// The corners and sides of the walls and objects in `items`, as closed outlines. A wall is
/// the rectangle its thickness spans; a circle has no corners and is left out.
fn outlines(items: &[Item]) -> Vec<Vec<Point>> {
    let mut found = Vec::new();

    for item in items
        .iter()
        .filter(|i| matches!(i.role, Role::Wall | Role::Object))
    {
        match &item.shape {
            Shape::Line { from, to, width_m } => found.push(wall_rectangle(*from, *to, *width_m)),
            Shape::Polygon { points } => found.push(points.clone()),
            _ => {}
        }
    }

    found
}

/// The four corners of the wall from `from` to `to`, `width_m` thick.
fn wall_rectangle(from: Point, to: Point, width_m: f64) -> Vec<Point> {
    let length = from.distance_m(to);
    if length == 0.0 {
        return vec![from];
    }

    // Half the thickness, turned a quarter to the side of the wall.
    let half = width_m / 2.0 / length;
    let (dx, dy) = ((to.x_m - from.x_m) * half, (to.y_m - from.y_m) * half);

    vec![
        Point::new(from.x_m - dy, from.y_m + dx),
        Point::new(to.x_m - dy, to.y_m + dx),
        Point::new(to.x_m + dy, to.y_m - dx),
        Point::new(from.x_m + dy, from.y_m - dx),
    ]
}

/// The point of the segment `a` to `b` nearest to `at`.
fn on_segment(at: Point, a: Point, b: Point) -> Point {
    let (dx, dy) = (b.x_m - a.x_m, b.y_m - a.y_m);
    let square = dx * dx + dy * dy;
    if square == 0.0 {
        return a;
    }

    let t = (((at.x_m - a.x_m) * dx + (at.y_m - a.y_m) * dy) / square).clamp(0.0, 1.0);

    Point::new(a.x_m + t * dx, a.y_m + t * dy)
}

/// The corner of a wall or object within `REACH_M` of `at`, else the nearest point on a side
/// within it; `None` when there is neither.
pub(crate) fn snap_to_edges(at: Point, items: &[Item]) -> Option<Point> {
    let mut corner: Option<Point> = None;
    let mut side: Option<Point> = None;
    let closer = |best: Option<Point>, found: Point| {
        let near = found.distance_m(at) <= REACH_M;
        let better = best.is_none_or(|b| found.distance_m(at) < b.distance_m(at));

        if near && better {
            Some(found)
        } else {
            best
        }
    };

    for outline in outlines(items) {
        for (i, &a) in outline.iter().enumerate() {
            let b = outline[(i + 1) % outline.len()];

            corner = closer(corner, a);
            side = closer(side, on_segment(at, a, b));
        }
    }

    corner.or(side)
}
