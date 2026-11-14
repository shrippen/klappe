use klappe_model::{Camera, CameraMark, Movement, Rig, Shot};

use crate::optics::mark_lens_at;

/// Differences below this are no movement: a millimetre, a hundredth of a degree.
const SAME_M: f64 = 0.001;
const SAME_DEG: f64 = 0.01;

/// The movement the camera's marks make during `shot` (LH-33): a changed place gives the rig
/// of that way, else a turn is a pan, else a changed tilt a tilt, else a changed lens a zoom;
/// without any of these the camera is static. Only ways that run within the shot count: from
/// the end of the pause at a mark to the next mark's time.
///
/// ```text
///  marks   1 ────────── 2 ────────── 3
///  shot         |═══════════|
///  counts            way 1→2 (way 2→3 starts after the shot)
/// ```
///
/// Example: a crane mark 2 at another place, within the shot: `Movement::Crane`.
pub fn movement_in(camera: &Camera, shot: &Shot) -> Movement {
    let ways: Vec<(&CameraMark, &CameraMark)> = camera
        .marks
        .windows(2)
        .map(|pair| (&pair[0], &pair[1]))
        .filter(|(from, to)| {
            let leave_s = (from.at_s + from.hold_s).min(to.at_s);

            leave_s < shot.to_s && to.at_s > shot.from_s
        })
        .collect();

    let moved = ways
        .iter()
        .find(|(from, to)| from.at.distance_m(to.at) > SAME_M);
    if let Some(movement) = moved.and_then(|(_, to)| rig_movement(to.rig)) {
        return movement;
    }
    if ways.iter().any(|(from, to)| turn_deg(from, to) > SAME_DEG) {
        return Movement::Pan;
    }
    if ways
        .iter()
        .any(|(from, to)| (to.tilt_deg - from.tilt_deg).abs() > SAME_DEG)
    {
        return Movement::Tilt;
    }
    if mark_lens_at(camera, shot.from_s) != mark_lens_at(camera, shot.to_s) {
        return Movement::Zoom;
    }

    Movement::Static
}

/// What a way is called in the shot list; a tripod moves nothing.
fn rig_movement(rig: Rig) -> Option<Movement> {
    match rig {
        Rig::Tripod => None,
        Rig::Dolly | Rig::Track => Some(Movement::Dolly),
        Rig::Crane => Some(Movement::Crane),
        Rig::Handheld => Some(Movement::Handheld),
        Rig::Gimbal => Some(Movement::Gimbal),
    }
}

/// The angle between two facings, 0 to 180.
fn turn_deg(from: &CameraMark, to: &CameraMark) -> f64 {
    let turn = (to.facing_deg - from.facing_deg).rem_euclid(360.0);

    turn.min(360.0 - turn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use klappe_model::{Point, Sensor, ShotSize};

    fn mark(at_s: f64, x_m: f64) -> CameraMark {
        CameraMark {
            at_s,
            at: Point::new(x_m, 0.0),
            facing_deg: 0.0,
            height_m: 1.6,
            tilt_deg: 0.0,
            hold_s: 0.0,
            via: None,
            rig: Rig::Dolly,
            lens_mm: None,
        }
    }

    fn camera(marks: Vec<CameraMark>) -> Camera {
        Camera {
            id: "a".into(),
            label: "A".into(),
            sensor: Sensor::Super35,
            lens_mm: 35.0,
            aspect: 2.39,
            marks,
            follows: None,
        }
    }

    fn shot(from_s: f64, to_s: f64) -> Shot {
        Shot {
            id: "s".into(),
            number: "1.1".into(),
            camera: "a".into(),
            size: ShotSize::Wide,
            angle: Default::default(),
            movement: Movement::Static,
            lens_mm: None,
            from_s,
            to_s,
            description: String::new(),
            props: String::new(),
            gear: String::new(),
            crew: String::new(),
            note: String::new(),
        }
    }

    fn movement(marks: Vec<CameraMark>) -> Movement {
        movement_in(&camera(marks), &shot(0.0, 10.0))
    }

    #[test]
    fn a_changed_place_gives_the_rig() {
        let with = |rig| {
            let mut to = mark(5.0, 3.0);
            to.rig = rig;

            movement(vec![mark(0.0, 0.0), to])
        };

        assert_eq!(with(Rig::Dolly), Movement::Dolly);
        assert_eq!(with(Rig::Track), Movement::Dolly);
        assert_eq!(with(Rig::Crane), Movement::Crane);
        assert_eq!(with(Rig::Handheld), Movement::Handheld);
        assert_eq!(with(Rig::Gimbal), Movement::Gimbal);
    }

    /// A place wins over a turn, a turn over a tilt, a tilt over a zoom.
    #[test]
    fn place_turn_tilt_zoom_in_that_order() {
        let mut to = mark(5.0, 0.0);
        to.lens_mm = Some(50.0);
        let mut from = mark(0.0, 0.0);
        from.lens_mm = Some(35.0);

        assert_eq!(movement(vec![from.clone(), to.clone()]), Movement::Zoom);
        to.tilt_deg = 10.0;
        assert_eq!(movement(vec![from.clone(), to.clone()]), Movement::Tilt);
        to.facing_deg = 350.0;
        assert_eq!(movement(vec![from.clone(), to.clone()]), Movement::Pan);
        to.at = Point::new(1.0, 0.0);
        assert_eq!(movement(vec![from, to]), Movement::Dolly);
    }

    #[test]
    fn nothing_changes_nothing_moves() {
        assert_eq!(
            movement(vec![mark(0.0, 0.0), mark(5.0, 0.0)]),
            Movement::Static
        );
        assert_eq!(movement(vec![mark(0.0, 0.0)]), Movement::Static);
        assert_eq!(movement(vec![]), Movement::Static);
    }

    /// A turn the short way round: 350° to 10° is 20°, 0° to 360° none.
    #[test]
    fn a_full_turn_is_no_turn() {
        let mut to = mark(5.0, 0.0);
        to.facing_deg = 360.0;

        assert_eq!(movement(vec![mark(0.0, 0.0), to]), Movement::Static);
    }

    /// A tripod does not move; a lens is stated at two marks, or it is no zoom.
    #[test]
    fn a_tripod_pans_and_a_lone_lens_is_no_zoom() {
        let mut to = mark(5.0, 0.0);
        to.rig = Rig::Tripod;
        to.facing_deg = 30.0;
        to.lens_mm = Some(50.0);

        assert_eq!(movement(vec![mark(0.0, 0.0), to]), Movement::Pan);

        let mut from = mark(0.0, 0.0);
        from.lens_mm = Some(50.0);

        assert_eq!(movement(vec![from, mark(5.0, 0.0)]), Movement::Static);
    }

    /// Only ways that run within the shot count: after the pause, before the next mark.
    #[test]
    fn only_ways_within_the_shot_count() {
        let mut held = mark(0.0, 0.0);
        held.hold_s = 6.0;
        let marks = vec![held, mark(8.0, 3.0), mark(20.0, 9.0)];
        let cam = camera(marks);

        // The way 1 to 2 runs from 6 s to 8 s.
        assert_eq!(movement_in(&cam, &shot(0.0, 6.0)), Movement::Static);
        assert_eq!(movement_in(&cam, &shot(0.0, 7.0)), Movement::Dolly);
        assert_eq!(movement_in(&cam, &shot(8.0, 9.0)), Movement::Dolly);
        assert_eq!(movement_in(&cam, &shot(0.0, 0.0)), Movement::Static);
    }
}
