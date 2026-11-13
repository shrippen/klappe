use klappe_model::{Camera, CameraMark, FigureTrack, Mark, Point};

/// Where something is and where it faces at one moment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placement {
    pub at: Point,
    pub facing_deg: f64,
}

/// The common part of figure and camera marks: arrive at `at_s`, stay `hold_s`, then walk on.
#[derive(Debug, Clone, Copy)]
struct Key {
    at_s: f64,
    hold_s: f64,
    place: Placement,
}

impl From<&Mark> for Key {
    fn from(m: &Mark) -> Self {
        Key {
            at_s: m.at_s,
            hold_s: m.hold_s,
            place: Placement {
                at: m.at,
                facing_deg: m.facing_deg,
            },
        }
    }
}

impl From<&CameraMark> for Key {
    fn from(m: &CameraMark) -> Self {
        Key {
            at_s: m.at_s,
            hold_s: m.hold_s,
            place: Placement {
                at: m.at,
                facing_deg: m.facing_deg,
            },
        }
    }
}

/// A figure at time `t_s` (LH-60); `None` without marks.
pub fn figure_at(track: &FigureTrack, t_s: f64) -> Option<Placement> {
    let keys: Vec<Key> = track.marks.iter().map(Key::from).collect();
    place_at(&keys, t_s)
}

/// A camera at time `t_s`; `None` without marks.
pub fn camera_at(camera: &Camera, t_s: f64) -> Option<Placement> {
    let keys: Vec<Key> = camera.marks.iter().map(Key::from).collect();
    place_at(&keys, t_s)
}

/// Before the first mark: there. Between marks: waits `hold_s`, then moves in a straight line
/// and turns the short way round, arriving at the next mark's time. After the last: there.
///
/// ```text
///  mark 1      hold      walk        mark 2
///  |──────────|░░░░░░░░░|─────────────|
///  at_s(1)    +hold_s(1)              at_s(2)
/// ```
fn place_at(keys: &[Key], t_s: f64) -> Option<Placement> {
    let first = keys.first()?;
    if t_s <= first.at_s {
        return Some(first.place);
    }

    for pair in keys.windows(2) {
        let (from, to) = (pair[0], pair[1]);
        if t_s >= to.at_s {
            continue;
        }

        let leave_s = (from.at_s + from.hold_s).min(to.at_s);
        if t_s <= leave_s {
            return Some(from.place);
        }

        let t = (t_s - leave_s) / (to.at_s - leave_s);
        return Some(Placement {
            at: from.place.at.lerp(to.place.at, t),
            facing_deg: turn(from.place.facing_deg, to.place.facing_deg, t),
        });
    }

    keys.last().map(|k| k.place)
}

/// Turns from `a` towards `b` the short way, `t` of the way (e.g. 350° → 10° passes 0°).
fn turn(a: f64, b: f64, t: f64) -> f64 {
    let delta = (b - a + 540.0).rem_euclid(360.0) - 180.0;
    (a + delta * t).rem_euclid(360.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use klappe_model::Pose;

    fn mark(at_s: f64, x: f64, hold_s: f64) -> Mark {
        Mark {
            at_s,
            at: Point::new(x, 0.0),
            facing_deg: 0.0,
            pose: Pose::Standing,
            hold_s,
        }
    }

    fn track(marks: Vec<Mark>) -> FigureTrack {
        FigureTrack {
            character: "e".into(),
            marks,
        }
    }

    #[test]
    fn before_between_and_after_marks() {
        let t = track(vec![mark(0.0, 0.0, 0.0), mark(10.0, 10.0, 0.0)]);

        assert_eq!(figure_at(&t, -1.0).map(|p| p.at.x_m), Some(0.0));
        assert_eq!(figure_at(&t, 5.0).map(|p| p.at.x_m), Some(5.0));
        assert_eq!(figure_at(&t, 20.0).map(|p| p.at.x_m), Some(10.0));
    }

    #[test]
    fn hold_waits_then_walks() {
        let t = track(vec![mark(0.0, 0.0, 4.0), mark(10.0, 6.0, 0.0)]);

        assert_eq!(figure_at(&t, 4.0).map(|p| p.at.x_m), Some(0.0));
        assert_eq!(figure_at(&t, 7.0).map(|p| p.at.x_m), Some(3.0));
    }

    #[test]
    fn no_marks_no_place() {
        assert_eq!(figure_at(&track(vec![]), 1.0), None);
    }

    #[test]
    fn turns_the_short_way() {
        assert!((turn(350.0, 10.0, 0.5) - 0.0).abs() < 1e-9);
        assert!((turn(10.0, 350.0, 0.5) - 0.0).abs() < 1e-9);
        assert!((turn(0.0, 90.0, 0.5) - 45.0).abs() < 1e-9);
    }
}
