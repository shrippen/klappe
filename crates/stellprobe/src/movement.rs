use klappe_model::{Camera, CameraMark, Movement, Rig, Shot};

use crate::optics::mark_lens_at;

/// Differences below this are no movement: a millimetre, a hundredth of a degree.
const SAME_M: f64 = 0.001;
const SAME_DEG: f64 = 0.01;

/// The movement the camera's marks make during `shot` (LH-33): a changed place gives the rig
/// of that way, else a turn is a pan, else a changed tilt a tilt, else a changed height a
/// crane, else a changed lens (also within the shot) a zoom; without any of these the camera
/// is static. Only ways that run within the shot count: from
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
    if ways
        .iter()
        .any(|(from, to)| (to.height_m - from.height_m).abs() > SAME_M)
    {
        return Movement::Crane;
    }
    if zooms_in(camera, shot) {
        return Movement::Zoom;
    }

    Movement::Static
}

/// Whether the lens changes anywhere in the shot: at its ends or at a mark within it, so a
/// zoom in and out counts although the ends agree.
fn zooms_in(camera: &Camera, shot: &Shot) -> bool {
    let within = camera
        .marks
        .iter()
        .filter(|m| m.at_s > shot.from_s && m.at_s < shot.to_s)
        .filter_map(|m| m.lens_mm);
    let lenses = [
        mark_lens_at(camera, shot.from_s),
        mark_lens_at(camera, shot.to_s),
    ]
    .into_iter()
    .flatten()
    .chain(within)
    .collect::<Vec<f64>>();

    lenses.windows(2).any(|pair| pair[0] != pair[1])
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
            via: Vec::new(),
            curve: Default::default(),
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
            color: None,
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
            setup_of: None,
            short_name: String::new(),
            image: None,
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

    /// A change of height alone is a crane; it wins over a zoom, loses to a tilt.
    #[test]
    fn a_change_of_height_is_a_crane() {
        let mut to = mark(5.0, 0.0);
        to.height_m = 4.0;
        let mut from = mark(0.0, 0.0);
        from.lens_mm = Some(35.0);
        to.lens_mm = Some(50.0);

        assert_eq!(movement(vec![from.clone(), to.clone()]), Movement::Crane);
        to.tilt_deg = 10.0;
        assert_eq!(movement(vec![from, to]), Movement::Tilt);
    }

    /// 35 to 85 and back is a zoom although the shot's ends agree.
    #[test]
    fn a_zoom_in_and_out_is_a_zoom() {
        let marks = [(0.0, 35.0), (5.0, 85.0), (10.0, 35.0)].map(|(at_s, lens_mm)| {
            let mut m = mark(at_s, 0.0);
            m.lens_mm = Some(lens_mm);
            m
        });

        assert_eq!(movement(marks.to_vec()), Movement::Zoom);
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
