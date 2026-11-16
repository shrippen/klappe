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

/// The size of a rectangle a grip sets: the right edge sets the width, the bottom edge the depth.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Extent {
    Width,
    Depth,
}

/// A rectangle on the plan around a centre, turned about it: what a furniture object and a
/// placed image both are. Its own frame has the origin at the centre, x to the right and y
/// down, unturned.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TurnedRect {
    pub at: Point,
    pub width_m: f64,
    pub depth_m: f64,
    /// Clockwise on screen about `at`.
    pub rotation_deg: f64,
}

impl TurnedRect {
    /// The point `(x_m, y_m)` of the rectangle's own frame where it lies on the plan: turned
    /// by `rotation_deg` about the centre.
    pub fn placed(&self, x_m: f64, y_m: f64) -> Point {
        Point::new(self.at.x_m + x_m, self.at.y_m + y_m).turned(self.at, self.rotation_deg)
    }

    /// The point `along_m` from the centre along the axis of `extent`, on the plan.
    fn on_axis(&self, extent: Extent, along_m: f64) -> Point {
        match extent {
            Extent::Width => self.placed(along_m, 0.0),
            Extent::Depth => self.placed(0.0, along_m),
        }
    }

    fn size_m(&self, extent: Extent) -> f64 {
        match extent {
            Extent::Width => self.width_m,
            Extent::Depth => self.depth_m,
        }
    }

    /// The middle of the edge that sets `extent` (right for the width, bottom for the depth).
    pub fn edge_middle(&self, extent: Extent) -> Point {
        self.on_axis(extent, self.size_m(extent) / 2.0)
    }

    /// The middle of that edge pushed `gap_m` outward: where its size grip sits.
    pub fn grip_place(&self, extent: Extent, gap_m: f64) -> Point {
        self.on_axis(extent, self.size_m(extent) / 2.0 + gap_m)
    }

    /// The rectangle with `extent` set so its grip, now at `grip_at`, lands on `to` as far as
    /// the axis of `extent` goes, within `min_m` to `max_m`. The opposite edge (left or top)
    /// stays, so the centre moves half the change.
    pub fn stretched(
        &self,
        extent: Extent,
        grip_at: Point,
        to: Point,
        (min_m, max_m): (f64, f64),
    ) -> TurnedRect {
        // The axis on the plan: where one metre along it lies, from the centre.
        let tip = self.on_axis(extent, 1.0);
        let axis = (tip.x_m - self.at.x_m, tip.y_m - self.at.y_m);
        let size_m = self.size_m(extent);

        // The grip sits a gap outside the edge: the size changes by how far `to` lies beyond it.
        let beyond_m = (to.x_m - grip_at.x_m) * axis.0 + (to.y_m - grip_at.y_m) * axis.1;
        let new_m = (size_m + beyond_m).clamp(min_m, max_m);
        let moved_m = (new_m - size_m) / 2.0;

        let mut rect = *self;
        rect.at = Point::new(
            self.at.x_m + moved_m * axis.0,
            self.at.y_m + moved_m * axis.1,
        );
        match extent {
            Extent::Width => rect.width_m = new_m,
            Extent::Depth => rect.depth_m = new_m,
        }

        rect
    }
}

/// A full turn; angles are kept within [0, 360).
pub const FULL_TURN_DEG: f64 = 360.0;

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

    /// A table 2 m wide, 1 m deep at (10; 5), turned a quarter clockwise: its width runs down.
    fn turned_table() -> TurnedRect {
        TurnedRect {
            at: Point::new(10.0, 5.0),
            width_m: 2.0,
            depth_m: 1.0,
            rotation_deg: 90.0,
        }
    }

    fn near(a: Point, b: Point) -> bool {
        (a.x_m - b.x_m).abs() < 1e-9 && (a.y_m - b.y_m).abs() < 1e-9
    }

    #[test]
    fn a_turned_rect_has_its_edges_and_grips_where_counted_by_hand() {
        let r = turned_table();

        // Right edge 1 m along the turned width (down), bottom edge 0.5 m along the depth (left).
        assert!(near(r.edge_middle(Extent::Width), Point::new(10.0, 6.0)));
        assert!(near(r.edge_middle(Extent::Depth), Point::new(9.5, 5.0)));
        assert!(near(
            r.grip_place(Extent::Width, 0.3),
            Point::new(10.0, 6.3)
        ));
    }

    #[test]
    fn stretching_keeps_the_far_edge_and_stays_within_bounds() {
        let r = turned_table();
        let grip = r.grip_place(Extent::Width, 0.3);

        // One metre further down: 3 m wide, the centre moves half of it.
        let wider = r.stretched(Extent::Width, grip, Point::new(10.0, 7.3), (0.1, 100.0));
        assert!((wider.width_m - 3.0).abs() < 1e-9);
        assert!(near(wider.at, Point::new(10.0, 5.5)));
        assert_eq!(wider.depth_m, 1.0);

        // Far past the other edge: held at the least width, the top edge (y 4) stays.
        let least = r.stretched(Extent::Width, grip, Point::new(10.0, 0.0), (0.1, 100.0));
        assert!((least.width_m - 0.1).abs() < 1e-9);
        assert!(near(least.at, Point::new(10.0, 4.05)));
    }

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
