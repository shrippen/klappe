use klappe_model::{FloorPlan, Point, Shot};

use crate::motion::camera_at;

/// Cameras this close (metres) at a shot's start stand at the same place (LH-72).
pub const SETUP_TOLERANCE_M: f64 = 0.5;

/// The order the shot list is read in (LH-72).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShotOrder {
    /// As edited: the order of the film.
    Cut,
    /// Setups grouped, so the crew lights and sets up once per camera position.
    Shooting,
}

/// What a camera setup is: one camera at one place with one lens.
struct Setup {
    camera: String,
    at: Option<Point>,
    lens_mm: f64,
}

impl Setup {
    /// Where the shot's camera stands at its start, and with which lens: the shot's own,
    /// else the camera's (as the shot list prints it). A camera that is gone has no place
    /// and only the shot's own lens.
    fn of(shot: &Shot, plan: &FloorPlan) -> Self {
        let camera = plan.camera(&shot.camera);

        Setup {
            camera: shot.camera.clone(),
            at: camera.and_then(|c| camera_at(c, shot.from_s)).map(|p| p.at),
            lens_mm: match camera {
                Some(c) => shot.lens_mm.unwrap_or(c.lens_mm),
                None => shot.lens_mm.unwrap_or_default(),
            },
        }
    }

    fn matches(&self, other: &Setup) -> bool {
        let same_place = match (self.at, other.at) {
            (Some(a), Some(b)) => a.distance_m(b) <= SETUP_TOLERANCE_M,
            (None, None) => true,
            _ => false,
        };

        self.camera == other.camera && self.lens_mm == other.lens_mm && same_place
    }
}

/// The setup number (1-based) of every shot, in the plan's cut order. Setups are numbered
/// by their first shot. A shot joins the first setup it matches; the place is compared with
/// that setup's first shot, so a slow dolly cannot chain shots into one setup.
///
/// Example: A at (8; 11) with 35 mm, A at (9; 8) with 40 mm, B at (18; 4) with 50 mm, B
/// again, A at (9; 8) with 40 mm: `[1, 2, 3, 3, 2]`.
pub fn shot_setups(plan: &FloorPlan) -> Vec<usize> {
    let mut setups: Vec<Setup> = Vec::new();

    plan.shots
        .iter()
        .map(|shot| {
            let setup = Setup::of(shot, plan);
            if let Some(i) = setups.iter().position(|s| s.matches(&setup)) {
                return i + 1;
            }

            setups.push(setup);
            setups.len()
        })
        .collect()
}

/// The shots with their setup number, in the order asked for. `Shooting` lists setup 1
/// first, then 2 and so on; within a setup the cut order stays.
pub fn ordered_shots(plan: &FloorPlan, order: ShotOrder) -> Vec<(usize, &Shot)> {
    let mut rows: Vec<(usize, &Shot)> = shot_setups(plan).into_iter().zip(&plan.shots).collect();
    if order == ShotOrder::Shooting {
        // Stable: equal setups keep the cut order.
        rows.sort_by_key(|(setup, _)| *setup);
    }

    rows
}

#[cfg(test)]
mod tests {
    use super::*;
    use klappe_model::{Camera, CameraMark, Movement, Sensor, ShotSize};

    fn camera(id: &str, lens_mm: f64, places: &[(f64, f64, f64)]) -> Camera {
        Camera {
            id: id.into(),
            label: id.to_uppercase(),
            sensor: Sensor::Super35,
            lens_mm,
            aspect: 2.39,
            marks: places
                .iter()
                .map(|&(at_s, x_m, y_m)| CameraMark {
                    at_s,
                    at: Point::new(x_m, y_m),
                    facing_deg: 0.0,
                    height_m: 1.6,
                    tilt_deg: 0.0,
                    hold_s: 0.0,
                })
                .collect(),
            follows: None,
        }
    }

    fn shot(id: &str, camera: &str, lens_mm: Option<f64>, from_s: f64, to_s: f64) -> Shot {
        Shot {
            id: id.into(),
            number: id.into(),
            camera: camera.into(),
            size: ShotSize::Wide,
            angle: Default::default(),
            movement: Movement::Static,
            lens_mm,
            from_s,
            to_s,
            description: String::new(),
            props: String::new(),
            gear: String::new(),
            crew: String::new(),
            note: String::new(),
        }
    }

    fn ids(rows: &[(usize, &Shot)]) -> Vec<String> {
        rows.iter().map(|(_, s)| s.id.clone()).collect()
    }

    /// The example project's numbers: A moves (8; 11) to (9; 8) at 6 s, B stands at (18; 4).
    fn example() -> FloorPlan {
        let mut plan = FloorPlan::new("p", "Plan A");
        plan.cameras = vec![
            camera("a", 35.0, &[(0.0, 8.0, 11.0), (6.0, 9.0, 8.0)]),
            camera("b", 50.0, &[(0.0, 18.0, 4.0)]),
        ];
        plan.shots = vec![
            shot("1", "a", None, 0.0, 6.0),
            shot("2", "a", Some(40.0), 6.0, 14.0),
            shot("3", "b", None, 6.0, 9.0),
            shot("4", "b", None, 9.0, 14.0),
            shot("5", "a", Some(40.0), 14.0, 20.0),
        ];

        plan
    }

    #[test]
    fn same_camera_place_and_lens_share_a_setup() {
        // 1: A (8; 11) 35. 2: A (9; 8) 40, 3 and 4: B (18; 4) 50, 5: A (9; 8) 40 again.
        assert_eq!(shot_setups(&example()), [1, 2, 3, 3, 2]);
    }

    #[test]
    fn shooting_order_groups_setups() {
        let plan = example();

        let shooting = ordered_shots(&plan, ShotOrder::Shooting);

        assert_eq!(ids(&shooting), ["1", "2", "5", "3", "4"]);
        let setups: Vec<usize> = shooting.iter().map(|(n, _)| *n).collect();
        assert_eq!(setups, [1, 2, 2, 3, 3]);
    }

    #[test]
    fn cut_order_keeps_the_plan_order() {
        let plan = example();
        let cut = ordered_shots(&plan, ShotOrder::Cut);

        assert_eq!(ids(&cut), ["1", "2", "3", "4", "5"]);
        assert_eq!(cut[4].0, 2);
    }

    /// 0.4 m away is the same place, 0.6 m is not (tolerance 0.5 m).
    #[test]
    fn the_tolerance_is_half_a_metre() {
        let mut plan = FloorPlan::new("p", "Plan A");
        plan.cameras = vec![camera(
            "a",
            35.0,
            &[(0.0, 0.0, 0.0), (10.0, 0.4, 0.0), (20.0, 1.0, 0.0)],
        )];
        plan.shots = vec![
            shot("1", "a", None, 0.0, 5.0),
            shot("2", "a", None, 10.0, 15.0),
            shot("3", "a", None, 20.0, 25.0),
        ];

        // 3 is 1.0 m from 1: its own setup.
        assert_eq!(shot_setups(&plan), [1, 1, 2]);
    }

    /// Shots 0.4 m apart in a row are compared with the first of the setup, not chained.
    #[test]
    fn a_slow_move_does_not_chain() {
        let mut plan = FloorPlan::new("p", "Plan A");
        plan.cameras = vec![camera(
            "a",
            35.0,
            &[(0.0, 0.0, 0.0), (10.0, 0.4, 0.0), (20.0, 0.8, 0.0)],
        )];
        plan.shots = vec![
            shot("1", "a", None, 0.0, 5.0),
            shot("2", "a", None, 10.0, 15.0),
            shot("3", "a", None, 20.0, 25.0),
        ];

        assert_eq!(shot_setups(&plan), [1, 1, 2]);
    }

    #[test]
    fn another_lens_is_another_setup() {
        let mut plan = FloorPlan::new("p", "Plan A");
        plan.cameras = vec![camera("a", 35.0, &[(0.0, 0.0, 0.0)])];
        plan.shots = vec![
            shot("1", "a", None, 0.0, 5.0),
            shot("2", "a", Some(85.0), 5.0, 10.0),
            shot("3", "a", Some(35.0), 10.0, 15.0),
        ];

        // 3's own 35 mm is the camera's lens: same as 1.
        assert_eq!(shot_setups(&plan), [1, 2, 1]);
    }

    #[test]
    fn overlapping_shots_keep_their_own_lens() {
        // The same action twice: once wide on the camera's lens, once on 85 mm.
        let mut plan = FloorPlan::new("p", "Plan A");
        plan.cameras = vec![camera("a", 35.0, &[(0.0, 0.0, 0.0)])];
        plan.shots = vec![
            shot("1", "a", None, 0.0, 10.0),
            shot("2", "a", Some(85.0), 0.0, 10.0),
        ];

        assert_eq!(shot_setups(&plan), [1, 2]);
    }

    #[test]
    fn a_missing_camera_has_no_place() {
        let mut plan = FloorPlan::new("p", "Plan A");
        plan.shots = vec![
            shot("1", "gone", Some(35.0), 0.0, 5.0),
            shot("2", "gone", Some(35.0), 5.0, 10.0),
            shot("3", "gone", Some(50.0), 10.0, 15.0),
        ];

        assert_eq!(shot_setups(&plan), [1, 1, 2]);
    }

    #[test]
    fn no_shots_no_setups() {
        let plan = FloorPlan::new("p", "Plan A");

        assert!(shot_setups(&plan).is_empty());
        assert!(ordered_shots(&plan, ShotOrder::Shooting).is_empty());
    }
}
