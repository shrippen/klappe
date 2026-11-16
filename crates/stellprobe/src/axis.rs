use klappe_model::{FigureTrack, FloorPlan, Point, Shot};

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

/// The tracks the axis runs between: the plan's pair, else its first two figures.
fn axis_figures(plan: &FloorPlan) -> Option<(&FigureTrack, &FigureTrack)> {
    match &plan.axis {
        Some(pair) => Some((plan.figure(&pair.a)?, plan.figure(&pair.b)?)),
        None => Some((plan.figures.first()?, plan.figures.get(1)?)),
    }
}

/// The characters the axis runs between: the plan's pair, else its first two figures.
pub fn axis_pair(plan: &FloorPlan) -> Option<(&str, &str)> {
    let (a, b) = axis_figures(plan)?;

    Some((&a.character, &b.character))
}

/// The axis at `t_s`: between the plan's axis pair, else its first two figures.
pub fn axis_at(plan: &FloorPlan, t_s: f64) -> Option<Axis> {
    let (a, b) = axis_figures(plan)?;

    Some(Axis {
        a: figure_at(a, t_s)?.at,
        b: figure_at(b, t_s)?.at,
    })
}

/// A shot on the smaller side of the axis (or on a tied side); `against` are the shots on the other side.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AxisJump {
    pub shot: String,
    /// Ids of the other shots of the plan on the other side, in cut order.
    pub against: Vec<String>,
}

/// Shots on the side of the axis with fewer shots than the other (LH-41), in cut order. With
/// as many shots on both sides, both sides are flagged. Each shot's side is taken at its own
/// start. A camera on the line has no side: it is flagged never and counts for no side.
///
/// Example: 12.1 and 12.2 from the left, 12.3 from the right: only 12.3 jumps, against
/// 12.1 and 12.2.
pub fn axis_jumps(plan: &FloorPlan) -> Vec<AxisJump> {
    let sides: Vec<(&str, Side)> = plan
        .shots
        .iter()
        .filter_map(|shot| Some((shot.id.as_str(), shot_side(plan, shot)?)))
        .collect();
    let count = |wanted: Side| sides.iter().filter(|(_, side)| *side == wanted).count();
    let (left, right) = (count(Side::Left), count(Side::Right));

    sides
        .iter()
        .filter_map(|(id, side)| {
            let (own, other) = match side {
                Side::Left => (left, right),
                _ => (right, left),
            };
            // Nothing on the other side, or this side is the majority.
            if other == 0 || own > other {
                return None;
            }

            let against: Vec<String> = sides
                .iter()
                .filter(|(_, o)| o != side)
                .map(|(other_id, _)| (*other_id).to_owned())
                .collect();

            Some(AxisJump {
                shot: (*id).to_owned(),
                against,
            })
        })
        .collect()
}

/// Which side of the axis the shot's camera stands on at the shot's start; `None` on the
/// line or without camera or axis.
fn shot_side(plan: &FloorPlan, shot: &Shot) -> Option<Side> {
    let camera = plan.camera(&shot.camera)?;
    let place = camera_at(camera, shot.from_s)?;
    let side = axis_at(plan, shot.from_s)?.side(place.at);

    (side != Side::On).then_some(side)
}

#[cfg(test)]
mod tests {
    use super::*;
    use klappe_model::{
        Arrival, AxisPair, Camera, CameraMark, Departure, FigureTrack, Mark, Movement, Pose,
        Sensor, Shot, ShotSize,
    };

    fn figure(id: &str, x: f64) -> FigureTrack {
        figure_at(id, x, 5.0)
    }

    fn figure_at(id: &str, x: f64, y: f64) -> FigureTrack {
        let at = Point::new(x, y);
        FigureTrack {
            character: id.into(),
            marks: vec![Mark {
                at_s: 0.0,
                at,
                facing_deg: 0.0,
                pose: Pose::Standing,
                hold_s: 0.0,
                via: Vec::new(),
                curve: Default::default(),
                arrival: Arrival::Walk,
                departure: Departure::Stay,
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
                via: Vec::new(),
                curve: Default::default(),
                rig: Default::default(),
                lens_mm: None,
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
            setup_of: None,
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

    fn jump(shot: &str, against: &[&str]) -> AxisJump {
        AxisJump {
            shot: shot.into(),
            against: against.iter().map(|s| (*s).to_owned()).collect(),
        }
    }

    #[test]
    fn same_side_no_jump() {
        assert!(axis_jumps(&plan(&[("1", "a"), ("2", "b")])).is_empty());
    }

    #[test]
    fn crossing_flags_the_minority() {
        let jumps = axis_jumps(&plan(&[("1", "a"), ("2", "b"), ("3", "c")]));

        assert_eq!(jumps, vec![jump("3", &["1", "2"])]);
    }

    #[test]
    fn ten_against_one_flags_the_one() {
        let mut shots: Vec<(String, &str)> = (1..=10).map(|n| (n.to_string(), "a")).collect();
        shots.push(("11".into(), "c"));
        let refs: Vec<(&str, &str)> = shots.iter().map(|(id, cam)| (id.as_str(), *cam)).collect();
        let against: Vec<String> = (1..=10).map(|n| n.to_string()).collect();
        let against: Vec<&str> = against.iter().map(String::as_str).collect();

        assert_eq!(axis_jumps(&plan(&refs)), vec![jump("11", &against)]);
    }

    #[test]
    fn a_tie_flags_both_sides() {
        let jumps = axis_jumps(&plan(&[("1", "a"), ("2", "c"), ("3", "b"), ("4", "c")]));

        assert_eq!(
            jumps,
            vec![
                jump("1", &["2", "4"]),
                jump("2", &["1", "3"]),
                jump("3", &["2", "4"]),
                jump("4", &["1", "3"]),
            ]
        );
    }

    #[test]
    fn nothing_on_one_side_no_jump() {
        assert!(axis_jumps(&plan(&[("1", "c"), ("2", "c")])).is_empty());
    }

    #[test]
    fn a_shot_far_away_in_the_list_counts() {
        let jumps = axis_jumps(&plan(&[("1", "a"), ("2", "b"), ("3", "b"), ("4", "c")]));

        assert_eq!(jumps, vec![jump("4", &["1", "2", "3"])]);
    }

    #[test]
    fn crossing_back_is_a_jump_too() {
        let jumps = axis_jumps(&plan(&[("1", "a"), ("2", "c"), ("3", "b")]));

        assert_eq!(jumps, vec![jump("2", &["1", "3"])]);
    }

    #[test]
    fn on_the_line_is_no_reference() {
        let mut p = plan(&[("1", "a"), ("2", "line"), ("3", "b")]);
        p.cameras.push(camera("line", 5.0));

        assert!(axis_jumps(&p).is_empty());
    }

    #[test]
    fn the_chosen_pair_sets_the_axis() {
        let mut p = plan(&[("1", "a"), ("2", "c")]);
        p.figures.push(figure_at("w", 8.0, 10.0));
        assert_eq!(axis_jumps(&p).len(), 2);

        // Vertical line x = 8: all cameras (x = 5) stand on its left.
        p.axis = Some(AxisPair {
            a: "v".into(),
            b: "w".into(),
        });

        assert_eq!(axis_pair(&p), Some(("v", "w")));
        assert!(axis_jumps(&p).is_empty());
    }

    #[test]
    fn no_figures_no_axis() {
        let mut p = plan(&[("1", "a"), ("2", "c")]);
        p.figures.clear();

        assert!(axis_jumps(&p).is_empty());
    }
}
