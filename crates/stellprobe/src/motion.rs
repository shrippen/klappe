use klappe_model::{Camera, CameraMark, FigureTrack, Mark, Point};

/// Where something is and where it faces at one moment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placement {
    pub at: Point,
    pub facing_deg: f64,
}

/// Steps a curve's length is measured in, to walk it at an even pace.
const ARC_STEPS: usize = 64;

/// Straight pieces a curve is drawn with.
const CURVE_SEGMENTS: usize = 24;

/// The common part of figure and camera marks: arrive at `at_s`, stay `hold_s`, then walk on.
#[derive(Debug, Clone, Copy)]
struct Key {
    at_s: f64,
    hold_s: f64,
    /// Bend of the way to this mark, see `Mark::via`.
    via: Option<Point>,
    place: Placement,
}

impl From<&Mark> for Key {
    fn from(m: &Mark) -> Self {
        Key {
            at_s: m.at_s,
            hold_s: m.hold_s,
            via: m.via,
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
            via: m.via,
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

/// Before the first mark: there. Between marks: waits `hold_s`, then moves along the way to
/// the next mark (straight, or the curve through its `via`) at an even pace and turns the
/// short way round, arriving at the next mark's time. After the last: there.
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
            at: along(from.place.at, to.via, to.place.at, t),
            facing_deg: turn(from.place.facing_deg, to.place.facing_deg, t),
        });
    }

    keys.last().map(|k| k.place)
}

/// The point at `t` (0 to 1) of the quadratic Bézier curve from `a` to `b` bent towards `via`.
fn bezier(a: Point, via: Point, b: Point, t: f64) -> Point {
    let u = 1.0 - t;
    let (wa, wv, wb) = (u * u, 2.0 * u * t, t * t);

    Point::new(
        wa * a.x_m + wv * via.x_m + wb * b.x_m,
        wa * a.y_m + wv * via.y_m + wb * b.y_m,
    )
}

/// Where the way from `a` to `b` bent towards `via` is halfway along the curve (straight: the
/// middle of the leg): the point to grab to bend it (LH-21).
pub fn bend_point(a: Point, via: Option<Point>, b: Point) -> Point {
    bezier(a, via.unwrap_or_else(|| a.lerp(b, 0.5)), b, 0.5)
}

/// The `via` that makes the way from `a` to `b` pass through `bend`, halfway along: the
/// inverse of `bend_point`.
pub fn via_through(a: Point, bend: Point, b: Point) -> Point {
    let middle = a.lerp(b, 0.5);

    Point::new(2.0 * bend.x_m - middle.x_m, 2.0 * bend.y_m - middle.y_m)
}

/// The point `t` (0 to 1) of the way from `a` to `b` by length, not by curve parameter: a
/// curve's parameter runs fast where it is straight and slow where it bends, which would
/// speed a figure up and down. Measures the curve in `ARC_STEPS` pieces and walks that table.
fn along(a: Point, via: Option<Point>, b: Point, t: f64) -> Point {
    let Some(via) = via else {
        return a.lerp(b, t);
    };

    // lengths[i]: length of the curve up to parameter i / ARC_STEPS.
    let mut lengths = [0.0; ARC_STEPS + 1];
    let mut last = a;
    for i in 1..=ARC_STEPS {
        let next = bezier(a, via, b, i as f64 / ARC_STEPS as f64);
        lengths[i] = lengths[i - 1] + last.distance_m(next);
        last = next;
    }

    let total_m = lengths[ARC_STEPS];
    if total_m <= 0.0 {
        return a;
    }

    // The table step that holds the wanted length, then the share of it by linear search.
    let wanted_m = total_m * t;
    let step = lengths
        .partition_point(|l| *l < wanted_m)
        .clamp(1, ARC_STEPS);
    let step_m = lengths[step] - lengths[step - 1];
    let share = if step_m > 0.0 {
        (wanted_m - lengths[step - 1]) / step_m
    } else {
        0.0
    };

    bezier(a, via, b, ((step - 1) as f64 + share) / ARC_STEPS as f64)
}

/// The way through `stops` (a mark's place and its `via`), as the points to draw: straight
/// legs are their two ends, curved ones `CURVE_SEGMENTS` pieces.
pub(crate) fn path(stops: impl IntoIterator<Item = (Point, Option<Point>)>) -> Vec<Point> {
    let stops: Vec<_> = stops.into_iter().collect();
    let mut points: Vec<Point> = stops.first().map(|(at, _)| *at).into_iter().collect();

    for pair in stops.windows(2) {
        let (from, (to, via)) = (pair[0].0, pair[1]);
        let Some(via) = via else {
            points.push(to);
            continue;
        };

        for i in 1..=CURVE_SEGMENTS {
            points.push(bezier(from, via, to, i as f64 / CURVE_SEGMENTS as f64));
        }
    }

    points
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
            via: None,
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

    fn curved(at_s: f64, at: Point, via: Point) -> Mark {
        Mark {
            at,
            via: Some(via),
            ..mark(at_s, 0.0, 0.0)
        }
    }

    /// The middle of the time is the middle of the length (the top of this symmetric arch,
    /// B(0.5) = (5, 4)), and equal times cover equal lengths: the steps between samples every
    /// 0.1 s are as long as each other, which steps of the curve parameter would not be.
    #[test]
    fn a_curve_is_walked_at_even_pace() {
        let t = track(vec![
            mark(0.0, 0.0, 0.0),
            curved(8.0, Point::new(10.0, 0.0), Point::new(5.0, 8.0)),
        ]);
        let at = |t_s: f64| figure_at(&t, t_s).unwrap().at;

        assert!(
            at(4.0).distance_m(Point::new(5.0, 4.0)) < 1e-6,
            "{:?}",
            at(4.0)
        );

        let steps_m: Vec<f64> = (0..80)
            .map(|i| at(f64::from(i) * 0.1).distance_m(at(f64::from(i + 1) * 0.1)))
            .collect();
        let mean_m = steps_m.iter().sum::<f64>() / steps_m.len() as f64;
        for step_m in &steps_m {
            assert!(
                (step_m - mean_m).abs() < 0.01 * mean_m,
                "{step_m} vs {mean_m}"
            );
        }
    }

    /// No `via`: the leg stays straight, with the same numbers as before curves existed.
    #[test]
    fn without_via_the_leg_is_straight() {
        let t = track(vec![mark(0.0, 0.0, 0.0), mark(10.0, 10.0, 0.0)]);

        assert_eq!(figure_at(&t, 2.5).map(|p| p.at), Some(Point::new(2.5, 0.0)));
    }

    /// Ends of a curve are the marks, before and after it.
    #[test]
    fn a_curve_ends_at_its_marks() {
        let t = track(vec![
            mark(0.0, 0.0, 0.0),
            curved(8.0, Point::new(10.0, 0.0), Point::new(5.0, 8.0)),
        ]);

        assert_eq!(figure_at(&t, 0.0).map(|p| p.at), Some(Point::new(0.0, 0.0)));
        assert_eq!(
            figure_at(&t, 8.0).map(|p| p.at),
            Some(Point::new(10.0, 0.0))
        );
    }

    /// The first mark has nothing to come from: its `via` is not used.
    #[test]
    fn via_on_the_first_mark_is_ignored() {
        let mut first = mark(0.0, 0.0, 0.0);
        first.via = Some(Point::new(0.0, 9.0));
        let t = track(vec![first, mark(10.0, 10.0, 0.0)]);

        assert_eq!(figure_at(&t, 5.0).map(|p| p.at), Some(Point::new(5.0, 0.0)));
        assert_eq!(
            path([(Point::new(0.0, 0.0), Some(Point::new(0.0, 9.0)))]).len(),
            1
        );
    }

    #[test]
    fn a_drawn_path_follows_the_curve() {
        let a = Point::new(0.0, 0.0);
        let b = Point::new(10.0, 0.0);
        let straight = path([(a, None), (b, None)]);
        let bent = path([(a, None), (b, Some(Point::new(5.0, 8.0)))]);

        assert_eq!(straight, vec![a, b]);
        assert_eq!(bent.len(), CURVE_SEGMENTS + 1);
        assert_eq!(bent.first(), Some(&a));
        assert_eq!(bent.last(), Some(&b));
        assert!(bent.iter().any(|p| (p.y_m - 4.0).abs() < 1e-9));
    }

    #[test]
    fn the_bend_point_is_on_the_curve() {
        let a = Point::new(0.0, 0.0);
        let b = Point::new(10.0, 0.0);
        let via = Point::new(5.0, 8.0);

        assert_eq!(bend_point(a, None, b), Point::new(5.0, 0.0));
        assert_eq!(bend_point(a, Some(via), b), Point::new(5.0, 4.0));
        assert_eq!(via_through(a, Point::new(5.0, 4.0), b), via);
    }
}
