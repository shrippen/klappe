use serde::{Deserialize, Serialize};

/// A place on the floor plan in metres from its top left corner (y grows downwards).
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Point {
    pub x_m: f64,
    pub y_m: f64,
}

impl Point {
    pub const fn new(x_m: f64, y_m: f64) -> Self {
        Self { x_m, y_m }
    }

    /// Straight-line distance in metres.
    pub fn distance_m(self, other: Point) -> f64 {
        (other.x_m - self.x_m).hypot(other.y_m - self.y_m)
    }

    /// The point a fraction `t` (0 = self, 1 = other) of the way to `other`.
    pub fn lerp(self, other: Point, t: f64) -> Point {
        Point::new(
            self.x_m + (other.x_m - self.x_m) * t,
            self.y_m + (other.y_m - self.y_m) * t,
        )
    }

    /// The point turned clockwise on the plan by `deg` about `centre`.
    pub fn turned(self, centre: Point, deg: f64) -> Point {
        if deg == 0.0 {
            return self;
        }

        let (sin, cos) = deg.to_radians().sin_cos();
        let (x_m, y_m) = (self.x_m - centre.x_m, self.y_m - centre.y_m);

        Point::new(
            centre.x_m + x_m * cos - y_m * sin,
            centre.y_m + x_m * sin + y_m * cos,
        )
    }

    /// Direction from self to `other` in degrees: 0° = +x, 90° = +y (clockwise on screen).
    pub fn bearing_deg(self, other: Point) -> f64 {
        (other.y_m - self.y_m)
            .atan2(other.x_m - self.x_m)
            .to_degrees()
    }
}

/// A full turn; angles are kept within [0, 360).
const FULL_TURN_DEG: f64 = 360.0;

/// An angle within [0, 360): -90 becomes 270. `rem_euclid` can return 360 for a tiny negative
/// number, so that case is folded back.
pub fn normalize_deg(deg: f64) -> f64 {
    let n = deg.rem_euclid(FULL_TURN_DEG);
    if n >= FULL_TURN_DEG {
        return 0.0;
    }
    n
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn angles_stay_below_a_full_turn() {
        assert_eq!(normalize_deg(-1e-20), 0.0);
        assert_eq!(normalize_deg(-90.0), 270.0);
        assert_eq!(normalize_deg(720.0), 0.0);
    }

    #[test]
    fn distance_and_lerp() {
        let a = Point::new(0.0, 0.0);
        let b = Point::new(3.0, 4.0);

        assert_eq!(a.distance_m(b), 5.0);
        assert_eq!(a.lerp(b, 0.5), Point::new(1.5, 2.0));
    }

    #[test]
    fn a_turn_is_clockwise_on_screen() {
        let centre = Point::new(1.0, 1.0);

        let turned = Point::new(2.0, 1.0).turned(centre, 90.0);

        assert!((turned.x_m - 1.0).abs() < 1e-9 && (turned.y_m - 2.0).abs() < 1e-9);
        assert_eq!(
            Point::new(5.0, 5.0).turned(centre, 0.0),
            Point::new(5.0, 5.0)
        );
    }

    #[test]
    fn bearing_is_clockwise_on_screen() {
        let o = Point::default();

        assert_eq!(o.bearing_deg(Point::new(1.0, 0.0)), 0.0);
        assert_eq!(o.bearing_deg(Point::new(0.0, 1.0)), 90.0);
    }
}
