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

    /// Direction from self to `other` in degrees: 0° = +x, 90° = +y (clockwise on screen).
    pub fn bearing_deg(self, other: Point) -> f64 {
        (other.y_m - self.y_m)
            .atan2(other.x_m - self.x_m)
            .to_degrees()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_and_lerp() {
        let a = Point::new(0.0, 0.0);
        let b = Point::new(3.0, 4.0);

        assert_eq!(a.distance_m(b), 5.0);
        assert_eq!(a.lerp(b, 0.5), Point::new(1.5, 2.0));
    }

    #[test]
    fn bearing_is_clockwise_on_screen() {
        let o = Point::default();

        assert_eq!(o.bearing_deg(Point::new(1.0, 0.0)), 0.0);
        assert_eq!(o.bearing_deg(Point::new(0.0, 1.0)), 90.0);
    }
}
