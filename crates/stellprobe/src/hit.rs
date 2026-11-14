use klappe_model::Point;

use crate::display::{Item, Shape};

/// Whether `at` is on `item`, give or take `tolerance_m` (selection by click or touch).
/// Circles and polygons count with their area, lines with their width, paths by their stroke;
/// images are never hit.
pub fn hit(item: &Item, at: Point, tolerance_m: f64) -> bool {
    match &item.shape {
        Shape::Circle { center, radius_m } => center.distance_m(at) <= radius_m + tolerance_m,
        Shape::Line { from, to, width_m } => {
            segment_distance_m(at, *from, *to) <= width_m / 2.0 + tolerance_m
        }
        Shape::Polyline { points } => path_distance_m(points, at, Path::Open) <= tolerance_m,
        Shape::Polygon { points } => {
            inside(points, at) || path_distance_m(points, at, Path::Closed) <= tolerance_m
        }
        // A background is only looked at: clicking on it must not take it.
        Shape::Image { .. } => false,
    }
}

/// Whether `item` lies wholly in the box with corners `a` and `b` (selection by rectangle,
/// LH-08). The corners come in any order. Shapes without points and images are never in.
pub fn within(item: &Item, a: Point, b: Point) -> bool {
    let (min_x, max_x) = (a.x_m.min(b.x_m), a.x_m.max(b.x_m));
    let (min_y, max_y) = (a.y_m.min(b.y_m), a.y_m.max(b.y_m));
    let inside = |p: &Point| (min_x..=max_x).contains(&p.x_m) && (min_y..=max_y).contains(&p.y_m);

    match &item.shape {
        Shape::Circle { center, radius_m } => {
            inside(&Point::new(center.x_m - radius_m, center.y_m - radius_m))
                && inside(&Point::new(center.x_m + radius_m, center.y_m + radius_m))
        }
        Shape::Line { from, to, .. } => inside(from) && inside(to),
        Shape::Polyline { points } | Shape::Polygon { points } => {
            !points.is_empty() && points.iter().all(inside)
        }
        Shape::Image { .. } => false,
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Path {
    Open,
    Closed,
}

/// Distance from `at` to the nearest segment of the path; infinite without segments.
fn path_distance_m(points: &[Point], at: Point, path: Path) -> f64 {
    let mut nearest = f64::INFINITY;

    for pair in points.windows(2) {
        nearest = nearest.min(segment_distance_m(at, pair[0], pair[1]));
    }

    let closing = match (path, points.first(), points.last()) {
        (Path::Closed, Some(first), Some(last)) => segment_distance_m(at, *last, *first),
        _ => f64::INFINITY,
    };
    nearest.min(closing)
}

fn segment_distance_m(at: Point, from: Point, to: Point) -> f64 {
    let (dx, dy) = (to.x_m - from.x_m, to.y_m - from.y_m);
    let length_sq = dx * dx + dy * dy;
    if length_sq == 0.0 {
        return at.distance_m(from);
    }

    // Where the foot of the perpendicular falls on the segment, kept between its ends.
    let t = ((at.x_m - from.x_m) * dx + (at.y_m - from.y_m) * dy) / length_sq;
    at.distance_m(from.lerp(to, t.clamp(0.0, 1.0)))
}

/// Even-odd rule: a ray to the right crosses the edges an odd number of times from inside.
fn inside(points: &[Point], at: Point) -> bool {
    let mut inside = false;

    for (i, a) in points.iter().enumerate() {
        let b = points[(i + 1) % points.len()];
        if (a.y_m > at.y_m) == (b.y_m > at.y_m) {
            continue;
        }

        let x_at_y = a.x_m + (at.y_m - a.y_m) / (b.y_m - a.y_m) * (b.x_m - a.x_m);
        if at.x_m < x_at_y {
            inside = !inside;
        }
    }

    inside
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::display::Role;

    fn item(shape: Shape) -> Item {
        Item {
            role: Role::Object,
            shape,
            label: None,
            source: None,
        }
    }

    fn square() -> Item {
        item(Shape::Polygon {
            points: vec![
                Point::new(0.0, 0.0),
                Point::new(2.0, 0.0),
                Point::new(2.0, 2.0),
                Point::new(0.0, 2.0),
            ],
        })
    }

    #[test]
    fn circle_counts_with_its_tolerance() {
        let c = item(Shape::Circle {
            center: Point::new(1.0, 1.0),
            radius_m: 0.5,
        });

        assert!(hit(&c, Point::new(1.4, 1.0), 0.0));
        assert!(!hit(&c, Point::new(1.55, 1.0), 0.0));
        assert!(hit(&c, Point::new(1.55, 1.0), 0.1));
    }

    #[test]
    fn polygon_inside_and_near_its_edge() {
        let s = square();

        assert!(hit(&s, Point::new(1.0, 1.0), 0.0));
        assert!(!hit(&s, Point::new(2.1, 1.0), 0.0));
        assert!(hit(&s, Point::new(2.1, 1.0), 0.15));
        assert!(!hit(&s, Point::new(-0.3, -0.3), 0.15));
    }

    /// A line is as wide as it is drawn: 0.2 m thick, 0.1 m each side.
    #[test]
    fn line_counts_with_half_its_width() {
        let l = item(Shape::Line {
            from: Point::new(0.0, 0.0),
            to: Point::new(4.0, 0.0),
            width_m: 0.2,
        });

        assert!(hit(&l, Point::new(2.0, 0.1), 0.0));
        assert!(!hit(&l, Point::new(2.0, 0.2), 0.0));
        assert!(hit(&l, Point::new(2.0, 0.25), 0.15));
        // Beyond the end, measured to the end point: 0.3 from (4, 0).
        assert!(!hit(&l, Point::new(4.3, 0.0), 0.1));
    }

    /// An open path is not closed: the middle of its loop is not on it.
    #[test]
    fn polyline_is_open_and_hollow() {
        let p = item(Shape::Polyline {
            points: vec![
                Point::new(0.0, 0.0),
                Point::new(2.0, 0.0),
                Point::new(2.0, 2.0),
            ],
        });

        assert!(hit(&p, Point::new(1.0, 0.05), 0.1));
        assert!(hit(&p, Point::new(2.05, 1.0), 0.1));
        assert!(!hit(&p, Point::new(1.0, 1.0), 0.1));
        assert!(!hit(&p, Point::new(1.0, 2.0), 0.1));
    }

    #[test]
    fn images_are_never_hit() {
        let image = item(Shape::Image {
            file: "bilder/saal.png".into(),
            at: Point::new(0.0, 0.0),
            width_m: 4.0,
            depth_m: 4.0,
            rotation_deg: 0.0,
            opacity: 1.0,
        });

        assert!(!hit(&image, Point::new(2.0, 2.0), 1.0));
    }

    #[test]
    fn a_box_takes_what_lies_wholly_in_it() {
        let circle = item(Shape::Circle {
            center: Point::new(1.0, 1.0),
            radius_m: 0.5,
        });
        let square = square();

        // Corners in any order.
        assert!(within(&circle, Point::new(2.0, 2.0), Point::new(0.0, 0.0)));
        // A circle that pokes out by its radius is not in.
        assert!(!within(&circle, Point::new(0.8, 0.0), Point::new(2.0, 2.0)));
        assert!(within(&square, Point::new(0.0, 0.0), Point::new(2.0, 2.0)));
        assert!(!within(&square, Point::new(0.0, 0.0), Point::new(1.9, 2.0)));
    }

    #[test]
    fn a_box_takes_no_image_and_no_empty_shape() {
        let image = item(Shape::Image {
            file: "bilder/saal.png".into(),
            at: Point::new(1.0, 1.0),
            width_m: 1.0,
            depth_m: 1.0,
            rotation_deg: 0.0,
            opacity: 1.0,
        });
        let empty = item(Shape::Polygon { points: vec![] });
        let (a, b) = (Point::new(0.0, 0.0), Point::new(9.0, 9.0));

        assert!(!within(&image, a, b));
        assert!(!within(&empty, a, b));
    }

    #[test]
    fn degenerate_shapes_do_not_panic() {
        let empty = item(Shape::Polygon { points: vec![] });
        let dot = item(Shape::Polyline {
            points: vec![Point::new(1.0, 1.0)],
        });

        assert!(!hit(&empty, Point::new(0.0, 0.0), 1.0));
        assert!(!hit(&dot, Point::new(1.0, 1.0), 1.0));
    }
}
