use klappe_model::{Arrival, Camera, CameraMark, Departure, FigureTrack, Mark, Point};

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
    /// Always `Walk` and `Stay` for a camera.
    arrival: Arrival,
    departure: Departure,
    place: Placement,
}

impl From<&Mark> for Key {
    fn from(m: &Mark) -> Self {
        Key {
            at_s: m.at_s,
            hold_s: m.hold_s,
            via: m.via,
            arrival: m.arrival,
            departure: m.departure,
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
            arrival: Arrival::Walk,
            departure: Departure::Stay,
            place: Placement {
                at: m.at,
                facing_deg: m.facing_deg,
            },
        }
    }
}

/// A figure at time `t_s` (LH-60); `None` without marks, and while it is not in the scene
/// (LH-26).
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
///
/// A figure that is not in the scene (LH-26) has no place: before the first mark if it
/// appears there, after the pause at a mark it leaves, and from the pause at the mark
/// before one it appears at, until that mark's time (it shows up instead of walking).
fn place_at(keys: &[Key], t_s: f64) -> Option<Placement> {
    let first = keys.first()?;
    if t_s < first.at_s {
        return (first.arrival == Arrival::Walk).then_some(first.place);
    }

    let gone = gone_on_arrival(keys);

    for (i, pair) in keys.windows(2).enumerate() {
        let (from, to) = (pair[0], pair[1]);
        if t_s >= to.at_s {
            continue;
        }

        let leave_s = (from.at_s + from.hold_s).min(to.at_s);
        if t_s <= leave_s {
            return (!gone[i]).then_some(from.place);
        }

        let stays = from.departure == Departure::Stay && !gone[i];
        if !stays || to.arrival == Arrival::Appear {
            return None;
        }

        let t = (t_s - leave_s) / (to.at_s - leave_s);
        return Some(Placement {
            at: along(from.place.at, to.via, to.place.at, t),
            facing_deg: turn(from.place.facing_deg, to.place.facing_deg, t),
        });
    }

    let last = keys.last()?;
    let left = last.departure == Departure::Leave && t_s > last.at_s + last.hold_s;
    let absent = left || gone.last().copied().unwrap_or(false);

    (!absent).then_some(last.place)
}

/// Per mark: the figure is not in the scene when it gets there, because it left at an earlier
/// mark and no mark between appeared again.
fn gone_on_arrival(keys: &[Key]) -> Vec<bool> {
    let mut gone_after = false;

    keys.iter()
        .map(|k| {
            let gone = gone_after && k.arrival == Arrival::Walk;
            gone_after = gone || k.departure == Departure::Leave;

            gone
        })
        .collect()
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

/// Length of the curve from `a` to `b` bent towards `via`, up to parameter i / `ARC_STEPS`.
fn arc_lengths(a: Point, via: Point, b: Point) -> [f64; ARC_STEPS + 1] {
    let mut lengths = [0.0; ARC_STEPS + 1];
    let mut last = a;
    for i in 1..=ARC_STEPS {
        let next = bezier(a, via, b, i as f64 / ARC_STEPS as f64);
        lengths[i] = lengths[i - 1] + last.distance_m(next);
        last = next;
    }

    lengths
}

/// How long the way from `a` to `b` is (straight, or the curve through its `via`), in metres:
/// what a figure walks, and so what its pace is measured on.
pub fn way_length_m(a: Point, via: Option<Point>, b: Point) -> f64 {
    match via {
        Some(via) => arc_lengths(a, via, b)[ARC_STEPS],
        None => a.distance_m(b),
    }
}

/// The point `t` (0 to 1) of the way from `a` to `b` by length, not by curve parameter: a
/// curve's parameter runs fast where it is straight and slow where it bends, which would
/// speed a figure up and down. Measures the curve in `ARC_STEPS` pieces and walks that table.
fn along(a: Point, via: Option<Point>, b: Point, t: f64) -> Point {
    let Some(via) = via else {
        return a.lerp(b, t);
    };

    let lengths = arc_lengths(a, via, b);

    let total_m = lengths[ARC_STEPS];
    if total_m <= 0.0 {
        return a;
    }

    // The table step that holds the wanted length (binary search), then the share of it
    // within that step, linearly.
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
            arrival: Arrival::Walk,
            departure: Departure::Stay,
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

    /// The parabola y = 16 t (1 - t), x = 10 t: length 16 * integral of sqrt(u^2 + 0.390625)
    /// over 0 to 1, about 13.337 m.
    #[test]
    fn a_way_is_as_long_as_its_curve() {
        let a = Point::new(0.0, 0.0);
        let b = Point::new(10.0, 0.0);

        assert_eq!(way_length_m(a, None, b), 10.0);
        let curved_m = way_length_m(a, Some(Point::new(5.0, 8.0)), b);
        assert!((curved_m - 13.337).abs() < 0.01, "{curved_m}");
    }

    fn appearing(at_s: f64, x: f64) -> Mark {
        Mark {
            arrival: Arrival::Appear,
            ..mark(at_s, x, 0.0)
        }
    }

    fn leaving(at_s: f64, x: f64, hold_s: f64) -> Mark {
        Mark {
            departure: Departure::Leave,
            ..mark(at_s, x, hold_s)
        }
    }

    fn x_at(t: &FigureTrack, t_s: f64) -> Option<f64> {
        figure_at(t, t_s).map(|p| p.at.x_m)
    }

    /// A first mark that appears: nobody there before it, there from its time on.
    #[test]
    fn a_first_mark_that_appears_is_not_there_before() {
        let t = track(vec![appearing(4.0, 2.0), mark(10.0, 8.0, 0.0)]);

        assert_eq!(x_at(&t, 0.0), None);
        assert_eq!(x_at(&t, 3.9), None);
        assert_eq!(x_at(&t, 4.0), Some(2.0));
        assert_eq!(x_at(&t, 7.0), Some(5.0));
    }

    /// Leaving at a mark: there during the pause, gone after it, also after the last mark.
    #[test]
    fn a_figure_that_leaves_is_gone_after_the_pause() {
        let t = track(vec![mark(0.0, 0.0, 0.0), leaving(10.0, 10.0, 2.0)]);

        assert_eq!(x_at(&t, 5.0), Some(5.0));
        assert_eq!(x_at(&t, 11.0), Some(10.0));
        assert_eq!(x_at(&t, 12.0), Some(10.0));
        assert_eq!(x_at(&t, 12.1), None);
        assert_eq!(x_at(&t, 99.0), None);
    }

    /// Leave at mark 2, appear at mark 3: nothing between the pause and mark 3, instead of
    /// a walk across.
    #[test]
    fn a_figure_comes_back_where_it_appears() {
        let t = track(vec![
            mark(0.0, 0.0, 0.0),
            leaving(4.0, 4.0, 1.0),
            appearing(10.0, 9.0),
            mark(14.0, 11.0, 0.0),
        ]);

        assert_eq!(x_at(&t, 5.0), Some(4.0));
        assert_eq!(x_at(&t, 7.0), None);
        assert_eq!(x_at(&t, 9.9), None);
        assert_eq!(x_at(&t, 10.0), Some(9.0));
        assert_eq!(x_at(&t, 12.0), Some(10.0));
    }

    /// An appearing mark after a staying one: the figure waits out its pause, then is gone
    /// until the new mark, which it does not walk to.
    #[test]
    fn an_appearing_mark_is_not_walked_to() {
        let t = track(vec![mark(0.0, 0.0, 1.0), appearing(10.0, 9.0)]);

        assert_eq!(x_at(&t, 1.0), Some(0.0));
        assert_eq!(x_at(&t, 5.0), None);
        assert_eq!(x_at(&t, 10.0), Some(9.0));
        assert_eq!(x_at(&t, 50.0), Some(9.0));
    }

    /// Gone stays gone over marks that only walk, until one appears.
    #[test]
    fn gone_lasts_until_a_mark_appears() {
        let t = track(vec![
            leaving(0.0, 0.0, 0.0),
            mark(5.0, 5.0, 0.0),
            appearing(10.0, 9.0),
        ]);

        assert_eq!(x_at(&t, 0.0), Some(0.0));
        assert_eq!(x_at(&t, 5.0), None);
        assert_eq!(x_at(&t, 7.0), None);
        assert_eq!(x_at(&t, 10.0), Some(9.0));
    }
}
