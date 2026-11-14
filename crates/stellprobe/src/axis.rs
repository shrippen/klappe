use klappe_model::{FloorPlan, Point};

use crate::motion::{camera_at, figure_at};

/// The 180° line between two figures at one moment (LH-40, LH-42).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Axis {
    pub a: Point,
    pub b: Point,
}

/// Which side of the axis a point is on, looking from `a` to `b` on screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
    On,
}

/// Points closer to the line than this count as on it (1 cm).
const ON_LINE_M: f64 = 0.01;

impl Axis {
    pub fn side(&self, p: Point) -> Side {
        let (dx, dy) = (self.b.x_m - self.a.x_m, self.b.y_m - self.a.y_m);
        let length = dx.hypot(dy);
        if length == 0.0 {
            return Side::On;
        }

        // Signed distance; y grows downwards, so positive is the right-hand side on screen.
        let cross = (dx * (p.y_m - self.a.y_m) - dy * (p.x_m - self.a.x_m)) / length;
        if cross.abs() < ON_LINE_M {
            return Side::On;
        }
        if cross > 0.0 {
            Side::Right
        } else {
            Side::Left
        }
    }
}

/// The axis at `t_s`: between the plan's axis pair, else its first two figures.
pub fn axis_at(plan: &FloorPlan, t_s: f64) -> Option<Axis> {
    let (a, b) = match &plan.axis {
        Some(pair) => (plan.figure(&pair.a)?, plan.figure(&pair.b)?),
        None => (plan.figures.first()?, plan.figures.get(1)?),
    };

    Some(Axis {
        a: figure_at(a, t_s)?.at,
        b: figure_at(b, t_s)?.at,
    })
}

/// Shots whose camera stands on the other side of the axis than in the shot before, in cut
/// order (LH-41). A camera on the line breaks no rule and keeps the side of the shot before.
///
/// Example: 12.1 and 12.2 from the left, 12.3 from the right: 12.3 jumps.
pub fn axis_jumps(plan: &FloorPlan) -> Vec<String> {
    let mut jumps = Vec::new();
    let mut last: Option<Side> = None;

    for shot in &plan.shots {
        let Some(camera) = plan.camera(&shot.camera) else {
            continue;
        };
        let Some(place) = camera_at(camera, shot.from_s) else {
            continue;
        };
        let Some(axis) = axis_at(plan, shot.from_s) else {
            continue;
        };

        let side = axis.side(place.at);
        if side == Side::On {
            continue;
        }
        if last.is_some_and(|l| l != side) {
            jumps.push(shot.id.clone());
        }
        last = Some(side);
    }

    jumps
}

#[cfg(test)]
mod tests {
    use super::*;
    use klappe_model::{
        Camera, CameraMark, FigureTrack, Mark, Movement, Pose, Sensor, Shot, ShotSize,
    };

    fn figure(id: &str, x: f64) -> FigureTrack {
        let at = Point::new(x, 5.0);
        FigureTrack {
            character: id.into(),
            marks: vec![Mark {
                at_s: 0.0,
                at,
                facing_deg: 0.0,
                pose: Pose::Standing,
                hold_s: 0.0,
            }],
        }
    }

    fn camera(id: &str, y: f64) -> Camera {
        let at = Point::new(5.0, y);
        Camera {
            id: id.into(),
            label: id.to_uppercase(),
            sensor: Sensor::Super35,
            lens_mm: 35.0,
            aspect: 2.39,
            marks: vec![CameraMark {
                at_s: 0.0,
                at,
                facing_deg: 0.0,
                height_m: 1.6,
                tilt_deg: 0.0,
                hold_s: 0.0,
            }],
            follows: None,
        }
    }

    fn shot(id: &str, camera: &str) -> Shot {
        Shot {
            id: id.into(),
            number: id.into(),
            camera: camera.into(),
            size: ShotSize::Medium,
            angle: Default::default(),
            movement: Movement::Static,
            lens_mm: None,
            from_s: 0.0,
            to_s: 1.0,
            description: String::new(),
            props: String::new(),
            gear: String::new(),
            crew: String::new(),
            note: String::new(),
        }
    }

    /// Two figures on y = 5; camera `a` above the line, `b` above too, `c` below.
    fn plan(shots: &[(&str, &str)]) -> FloorPlan {
        let mut p = FloorPlan::new("p", "Plan A");
        p.figures = vec![figure("e", 2.0), figure("v", 8.0)];
        p.cameras = vec![camera("a", 1.0), camera("b", 2.0), camera("c", 9.0)];
        p.shots = shots.iter().map(|(id, cam)| shot(id, cam)).collect();
        p
    }

    #[test]
    fn sides_seen_from_a_to_b() {
        let axis = Axis {
            a: Point::new(0.0, 0.0),
            b: Point::new(10.0, 0.0),
        };

        assert_eq!(axis.side(Point::new(5.0, 1.0)), Side::Right);
        assert_eq!(axis.side(Point::new(5.0, -1.0)), Side::Left);
        assert_eq!(axis.side(Point::new(5.0, 0.0)), Side::On);
    }

    #[test]
    fn same_side_no_jump() {
        assert!(axis_jumps(&plan(&[("1", "a"), ("2", "b")])).is_empty());
    }

    #[test]
    fn crossing_is_a_jump() {
        let jumps = axis_jumps(&plan(&[("1", "a"), ("2", "b"), ("3", "c")]));

        assert_eq!(jumps, vec!["3".to_string()]);
    }

    #[test]
    fn crossing_back_is_a_jump_too() {
        let jumps = axis_jumps(&plan(&[("1", "a"), ("2", "c"), ("3", "b")]));

        assert_eq!(jumps, vec!["2".to_string(), "3".to_string()]);
    }

    #[test]
    fn no_figures_no_axis() {
        let mut p = plan(&[("1", "a"), ("2", "c")]);
        p.figures.clear();

        assert!(axis_jumps(&p).is_empty());
    }
}
