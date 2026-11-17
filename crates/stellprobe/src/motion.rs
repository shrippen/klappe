use klappe_model::{Arrival, Camera, CameraMark, Curve, Departure, FigureTrack, Mark, Point};

/// Where something is and where it faces at one moment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placement {
    pub at: Point,
    pub facing_deg: f64,
}

/// Steps each piece of a curve (between two points it passes) is measured in, to walk it at an
/// even pace.
const ARC_STEPS: usize = 64;

/// Straight pieces each piece of a curve is drawn with.
const CURVE_SEGMENTS: usize = 24;

/// The common part of figure and camera marks: arrive at `at_s`, stay `hold_s`, then walk on.
#[derive(Debug, Clone, Copy)]
struct Key<'a> {
    at_s: f64,
    hold_s: f64,
    /// Bends of the way to this mark, see `Mark::via`.
    via: &'a [Point],
    /// Shape of the way, see `Mark::curve`.
    curve: Curve,
    /// Always `Walk` and `Stay` for a camera.
    arrival: Arrival,
    departure: Departure,
    place: Placement,
}

impl<'a> From<&'a Mark> for Key<'a> {
    fn from(m: &'a Mark) -> Self {
        Key {
            at_s: m.at_s,
            hold_s: m.hold_s,
            via: &m.via,
            curve: m.curve,
            arrival: m.arrival,
            departure: m.departure,
            place: Placement {
                at: m.at,
                facing_deg: m.facing_deg,
            },
        }
    }
}

impl<'a> From<&'a CameraMark> for Key<'a> {
    fn from(m: &'a CameraMark) -> Self {
        Key {
            at_s: m.at_s,
            hold_s: m.hold_s,
            via: &m.via,
            curve: m.curve,
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
    let keys: Vec<Key<'_>> = track.marks.iter().map(Key::from).collect();
    place_at(&keys, t_s)
}

/// For each mark of `track`, whether the figure walks the way to it (LH-26): not to the
/// first mark, nor to one it appears at, nor to one it is gone at after leaving earlier.
///
/// ```text
///  marks     1      2 (leave)   3 (walk)   4 (appear)   5
///  walked    no     yes         no         no           yes
/// ```
pub fn walked_ways(track: &FigureTrack) -> Vec<bool> {
    let keys: Vec<Key<'_>> = track.marks.iter().map(Key::from).collect();
    let gone = gone_on_arrival(&keys);

    keys.iter()
        .zip(gone)
        .enumerate()
        .map(|(i, (k, gone))| i > 0 && k.arrival == Arrival::Walk && !gone)
        .collect()
}

/// A camera at time `t_s`; `None` without marks.
pub fn camera_at(camera: &Camera, t_s: f64) -> Option<Placement> {
    let keys: Vec<Key<'_>> = camera.marks.iter().map(Key::from).collect();
    place_at(&keys, t_s)
}

/// Lens height and tilt of a camera at `t_s`, in that order (LH-34, LH-33); `None` without
/// marks. They run linearly between two marks on the clock of the place: after the pause at
/// the first mark until the arrival at the next; before the first and after the last they hold.
pub fn aim_at(camera: &Camera, t_s: f64) -> Option<(f64, f64)> {
    let first = camera.marks.first()?;
    if t_s < first.at_s {
        return Some((first.height_m, first.tilt_deg));
    }

    for pair in camera.marks.windows(2) {
        let (from, to) = (&pair[0], &pair[1]);
        if t_s >= to.at_s {
            continue;
        }

        let t = progress(from, to, t_s);
        return Some((
            from.height_m + (to.height_m - from.height_m) * t,
            from.tilt_deg + (to.tilt_deg - from.tilt_deg) * t,
        ));
    }

    camera.marks.last().map(|m| (m.height_m, m.tilt_deg))
}

/// How far a camera is from mark `from` to mark `to` at `t_s`, 0 to 1: 0 during the pause.
pub(crate) fn progress(from: &CameraMark, to: &CameraMark, t_s: f64) -> f64 {
    let leave_s = (from.at_s + from.hold_s).min(to.at_s);
    if t_s <= leave_s {
        return 0.0;
    }

    (t_s - leave_s) / (to.at_s - leave_s)
}

/// Before the first mark: there. Between marks: waits `hold_s`, then moves along the way to
/// the next mark (straight, or the curve through its `via` points) at an even pace and turns the
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
fn place_at(keys: &[Key<'_>], t_s: f64) -> Option<Placement> {
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
            at: along(from.place.at, (to.via, to.curve), to.place.at, t),
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
fn gone_on_arrival(keys: &[Key<'_>]) -> Vec<bool> {
    let mut gone_after = false;

    keys.iter()
        .map(|k| {
            let gone = gone_after && k.arrival == Arrival::Walk;
            gone_after = gone || k.departure == Departure::Leave;

            gone
        })
        .collect()
}

/// The points a way passes: its start, the bends, its end.
fn anchors(a: Point, via: &[Point], b: Point) -> Vec<Point> {
    std::iter::once(a)
        .chain(via.iter().copied())
        .chain(std::iter::once(b))
        .collect()
}

/// The point at `u` (0 to the number of pieces) of the Catmull-Rom curve through `anchors`:
/// piece `i` runs from anchor `i` to `i + 1`, `u - i` into it. The curve is smooth where it
/// passes an anchor; at the two ends the first and last anchor stand in for their missing
/// neighbour.
fn spline(anchors: &[Point], u: f64) -> Point {
    let last = anchors.len() - 1;
    let piece = (u.floor().max(0.0) as usize).min(last - 1);
    let t = u - piece as f64;
    let at = |i: usize| anchors[i.min(last)];
    let (p0, p1) = (at(piece.saturating_sub(1)), at(piece));
    let (p2, p3) = (at(piece + 1), at(piece + 2));

    let weigh = |c0: f64, c1: f64, c2: f64, c3: f64| {
        0.5 * (2.0 * c1
            + (c2 - c0) * t
            + (2.0 * c0 - 5.0 * c1 + 4.0 * c2 - c3) * t * t
            + (3.0 * c1 - c0 - 3.0 * c2 + c3) * t * t * t)
    };

    Point::new(
        weigh(p0.x_m, p1.x_m, p2.x_m, p3.x_m),
        weigh(p0.y_m, p1.y_m, p2.y_m, p3.y_m),
    )
}

/// The point at `u` of the way through `anchors` in the shape `curve` gives it: the smooth
/// curve, or straight pieces from anchor to anchor (LH-108).
fn shaped(anchors: &[Point], curve: Curve, u: f64) -> Point {
    if curve == Curve::Smooth {
        return spline(anchors, u);
    }

    let piece = (u.floor().max(0.0) as usize).min(anchors.len() - 2);

    anchors[piece].lerp(anchors[piece + 1], u - piece as f64)
}

/// The grips of the way from `a` to `b` through `via`: one in the middle of each piece (a
/// straight leg: its middle). Dragging one makes a new bend there (LH-21).
pub fn bend_grips(a: Point, (via, curve): (&[Point], Curve), b: Point) -> Vec<Point> {
    let anchors = anchors(a, via, b);

    (0..anchors.len() - 1)
        .map(|piece| shaped(&anchors, curve, piece as f64 + 0.5))
        .collect()
}

/// Where `via` goes when the way's ends move from `old` to `new` (start, end): it keeps its
/// place relative to the leg, so the curve keeps its shape through a move, a turn or a stretch
/// (LH-21). A leg that has no length has no frame; `via` then only moves with its middle.
pub fn bend_follow(via: Point, old: (Point, Point), new: (Point, Point)) -> Point {
    let (old_mid, new_mid) = (old.0.lerp(old.1, 0.5), new.0.lerp(new.1, 0.5));
    let (ex, ey) = (old.1.x_m - old.0.x_m, old.1.y_m - old.0.y_m);
    let (fx, fy) = (new.1.x_m - new.0.x_m, new.1.y_m - new.0.y_m);
    let len_sq = ex * ex + ey * ey;
    let (zx, zy) = (via.x_m - old_mid.x_m, via.y_m - old_mid.y_m);

    if len_sq <= f64::EPSILON {
        return Point::new(new_mid.x_m + zx, new_mid.y_m + zy);
    }

    // z * (f / e) as complex numbers: the offset from the middle turns and scales with the leg.
    let (rx, ry) = ((fx * ex + fy * ey) / len_sq, (fy * ex - fx * ey) / len_sq);

    Point::new(
        new_mid.x_m + zx * rx - zy * ry,
        new_mid.y_m + zx * ry + zy * rx,
    )
}

/// Length of the curve through `anchors` up to parameter i / `ARC_STEPS`.
fn arc_lengths(anchors: &[Point], curve: Curve) -> Vec<f64> {
    let steps = (anchors.len() - 1) * ARC_STEPS;
    let mut lengths = vec![0.0; steps + 1];
    let mut last = anchors[0];
    for i in 1..=steps {
        let next = shaped(anchors, curve, i as f64 / ARC_STEPS as f64);
        lengths[i] = lengths[i - 1] + last.distance_m(next);
        last = next;
    }

    lengths
}

/// How long the way from `a` to `b` is (straight, or the curve through `via`), in metres:
/// what a figure walks, and so what its pace is measured on.
pub fn way_length_m(a: Point, (via, curve): (&[Point], Curve), b: Point) -> f64 {
    if via.is_empty() {
        return a.distance_m(b);
    }

    arc_lengths(&anchors(a, via, b), curve)
        .last()
        .copied()
        .unwrap_or(0.0)
}

/// The point `t` (0 to 1) of the way from `a` to `b` by length, not by curve parameter: a
/// curve's parameter runs fast where it is straight and slow where it bends, which would
/// speed a figure up and down. Measures the curve in `ARC_STEPS` pieces and walks that table.
fn along(a: Point, (via, curve): (&[Point], Curve), b: Point, t: f64) -> Point {
    if via.is_empty() {
        return a.lerp(b, t);
    }

    let anchors = anchors(a, via, b);
    let lengths = arc_lengths(&anchors, curve);
    let steps = lengths.len() - 1;

    let total_m = lengths[steps];
    if total_m <= 0.0 {
        return a;
    }

    // The table step that holds the wanted length (binary search), then the share of it
    // within that step, linearly.
    let wanted_m = total_m * t;
    let step = lengths.partition_point(|l| *l < wanted_m).clamp(1, steps);
    let step_m = lengths[step] - lengths[step - 1];
    let share = if step_m > 0.0 {
        (wanted_m - lengths[step - 1]) / step_m
    } else {
        0.0
    };

    shaped(
        &anchors,
        curve,
        ((step - 1) as f64 + share) / ARC_STEPS as f64,
    )
}

/// The way through `stops` (a mark's place, its `via` and curve), as the points to draw:
/// straight legs and straight ways are their points, smooth ones `CURVE_SEGMENTS` pieces for
/// each piece of the curve.
pub(crate) fn path<'a>(stops: impl IntoIterator<Item = (Point, &'a [Point], Curve)>) -> Vec<Point> {
    let stops: Vec<_> = stops.into_iter().collect();
    let mut points: Vec<Point> = stops.first().map(|(at, ..)| *at).into_iter().collect();

    for pair in stops.windows(2) {
        let (from, (to, via, curve)) = (pair[0].0, pair[1]);
        if via.is_empty() {
            points.push(to);
            continue;
        }
        if curve == Curve::Straight {
            points.extend(via.iter().copied());
            points.push(to);
            continue;
        }

        let anchors = anchors(from, via, to);
        let segments = (anchors.len() - 1) * CURVE_SEGMENTS;
        for i in 1..=segments {
            points.push(spline(&anchors, i as f64 / CURVE_SEGMENTS as f64));
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
            via: Vec::new(),
            curve: Default::default(),
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

    fn aimed(at_s: f64, hold_s: f64, height_m: f64, tilt_deg: f64) -> CameraMark {
        CameraMark {
            at_s,
            at: Point::new(0.0, 0.0),
            facing_deg: 0.0,
            height_m,
            tilt_deg,
            hold_s,
            via: Vec::new(),
            curve: Default::default(),
            rig: Default::default(),
            lens_mm: None,
        }
    }

    /// Height and tilt wait out the pause, then run to the next mark; they hold outside.
    #[test]
    fn height_and_tilt_run_between_marks() {
        let camera = Camera {
            id: "a".into(),
            label: "A".into(),
            sensor: klappe_model::Sensor::Super35,
            lens_mm: 35.0,
            aspect: 2.39,
            marks: vec![aimed(2.0, 2.0, 1.0, 0.0), aimed(10.0, 0.0, 3.0, -20.0)],
            follows: None,
            color: None,
        };

        assert_eq!(aim_at(&camera, 0.0), Some((1.0, 0.0)));
        assert_eq!(aim_at(&camera, 4.0), Some((1.0, 0.0)));
        assert_eq!(aim_at(&camera, 7.0), Some((2.0, -10.0)));
        assert_eq!(aim_at(&camera, 10.0), Some((3.0, -20.0)));
        assert_eq!(aim_at(&camera, 99.0), Some((3.0, -20.0)));
        assert_eq!(
            aim_at(
                &Camera {
                    marks: vec![],
                    ..camera
                },
                1.0
            ),
            None
        );
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

    /// Hand-worked: leg (17, 9.5)-(12, 6.5) with `via` (14.5, 4) becomes (17, 9.5)-(11, 7).
    #[test]
    fn the_bend_follows_a_moved_end() {
        let (a, b) = (Point::new(17.0, 9.5), Point::new(12.0, 6.5));
        let moved = Point::new(11.0, 7.0);
        let via = bend_follow(Point::new(14.5, 4.0), (a, b), (a, moved));

        assert!((via.x_m - (14.0 - 11.0 / 17.0)).abs() < 1e-9, "{via:?}");
        assert!((via.y_m - (8.25 - 75.0 / 17.0)).abs() < 1e-9, "{via:?}");
    }

    #[test]
    fn the_bend_follows_a_turned_leg() {
        let (a, b) = (Point::new(0.0, 0.0), Point::new(4.0, 0.0));
        let via = bend_follow(Point::new(2.0, 3.0), (a, b), (a, Point::new(0.0, 4.0)));

        assert!(via.distance_m(Point::new(-3.0, 2.0)) < 1e-9, "{via:?}");
    }

    #[test]
    fn a_leg_without_length_moves_the_bend_with_its_middle() {
        let a = Point::new(1.0, 1.0);
        let via = bend_follow(Point::new(2.0, 1.0), (a, a), (a, Point::new(3.0, 1.0)));

        assert!(via.distance_m(Point::new(3.0, 1.0)) < 1e-9, "{via:?}");
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

    fn curved(at_s: f64, at: Point, via: &[Point]) -> Mark {
        Mark {
            at,
            via: via.to_vec(),
            ..mark(at_s, 0.0, 0.0)
        }
    }

    /// An arch from (0, 0) to (10, 0) over one bend at its top, (5, 4).
    fn arch() -> Mark {
        curved(8.0, Point::new(10.0, 0.0), &[Point::new(5.0, 4.0)])
    }

    /// The middle of the time is the middle of the length (the top of this symmetric arch),
    /// and equal times cover equal lengths: the steps between samples every 0.1 s are as long
    /// as each other, which steps of the curve parameter would not be.
    #[test]
    fn a_curve_is_walked_at_even_pace() {
        let t = track(vec![mark(0.0, 0.0, 0.0), arch()]);
        let at = |t_s: f64| figure_at(&t, t_s).unwrap().at;

        assert!(
            at(4.0).distance_m(Point::new(5.0, 4.0)) < 0.05,
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

    /// Bends on a line, the first twice as far from the start as from the end: the spline
    /// runs fast and slow in its parameter, the figure is where the length says. Half the time
    /// is half the 10 m, x = 5, whichever the parameter does.
    #[test]
    fn even_pace_holds_with_uneven_bends() {
        let via = [Point::new(2.0, 0.0), Point::new(9.0, 0.0)];
        let t = track(vec![
            mark(0.0, 0.0, 0.0),
            curved(10.0, Point::new(10.0, 0.0), &via),
        ]);

        for second in 0..=10 {
            let x_m = figure_at(&t, f64::from(second)).unwrap().at.x_m;
            assert!((x_m - f64::from(second)).abs() < 0.01, "{second}: {x_m}");
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
        let t = track(vec![mark(0.0, 0.0, 0.0), arch()]);

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
        first.via = vec![Point::new(0.0, 9.0)];
        let t = track(vec![first, mark(10.0, 10.0, 0.0)]);

        assert_eq!(figure_at(&t, 5.0).map(|p| p.at), Some(Point::new(5.0, 0.0)));
        assert_eq!(
            path([(
                Point::new(0.0, 0.0),
                &[Point::new(0.0, 9.0)][..],
                Curve::Smooth
            )])
            .len(),
            1
        );
    }

    /// The drawn way passes every bend, at the point where one piece ends and the next starts.
    #[test]
    fn a_drawn_path_passes_its_bends() {
        let a = Point::new(0.0, 0.0);
        let b = Point::new(10.0, 0.0);
        let bends = [Point::new(3.0, 4.0), Point::new(7.0, -4.0)];
        let straight = path([(a, &[][..], Curve::Smooth), (b, &[][..], Curve::Smooth)]);
        let bent = path([(a, &[][..], Curve::Smooth), (b, &bends[..], Curve::Smooth)]);

        assert_eq!(straight, vec![a, b]);
        assert_eq!(bent.len(), 3 * CURVE_SEGMENTS + 1);
        assert_eq!(bent.first(), Some(&a));
        assert_eq!(bent.last(), Some(&b));
        assert_eq!(bent[CURVE_SEGMENTS], bends[0]);
        assert_eq!(bent[2 * CURVE_SEGMENTS], bends[1]);
    }

    /// Hand-worked: over (0, 0), (5, 4), (10, 0) the first piece has the neighbours (0, 0) and
    /// (10, 0); at its middle x is (2.5 + 2.5 - 0.625) / 2 = 2.1875 and y (2 + 4 - 1.5) / 2
    /// = 2.25. The second piece mirrors it.
    #[test]
    fn grips_sit_in_the_middle_of_each_piece() {
        let a = Point::new(0.0, 0.0);
        let b = Point::new(10.0, 0.0);

        assert_eq!(
            bend_grips(a, (&[], Curve::Smooth), b),
            vec![Point::new(5.0, 0.0)]
        );

        let grips = bend_grips(a, (&[Point::new(5.0, 4.0)], Curve::Smooth), b);
        assert_eq!(grips.len(), 2);
        assert!((grips[0].x_m - 2.1875).abs() < 1e-9, "{grips:?}");
        assert!((grips[0].y_m - 2.25).abs() < 1e-9, "{grips:?}");
        assert!((grips[1].x_m - 7.8125).abs() < 1e-9, "{grips:?}");
        assert!((grips[1].y_m - 2.25).abs() < 1e-9, "{grips:?}");
    }

    /// A way over a bend on its line is as long as the line; over one off it, a smooth curve
    /// is longer than the two straight legs to the bend, but not by much.
    #[test]
    fn a_way_is_as_long_as_its_curve() {
        let a = Point::new(0.0, 0.0);
        let b = Point::new(10.0, 0.0);

        assert_eq!(way_length_m(a, (&[], Curve::Smooth), b), 10.0);

        let on_line_m = way_length_m(a, (&[Point::new(5.0, 0.0)], Curve::Smooth), b);
        assert!((on_line_m - 10.0).abs() < 1e-6, "{on_line_m}");

        let bend = Point::new(5.0, 4.0);
        let legs_m = a.distance_m(bend) + bend.distance_m(b);
        let curved_m = way_length_m(a, (&[bend], Curve::Smooth), b);
        assert!(curved_m > legs_m && curved_m < 1.1 * legs_m, "{curved_m}");
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

    /// LH-108: a straight way runs from bend to bend, with the corner at the bend, at an even
    /// pace; its length is the legs', its drawing the points, its grips the leg middles.
    #[test]
    fn a_straight_way_has_corners() {
        let (a, b, bend) = (
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(5.0, 4.0),
        );
        let way = (&[bend][..], Curve::Straight);

        let legs_m = a.distance_m(bend) + bend.distance_m(b);
        assert!((way_length_m(a, way, b) - legs_m).abs() < 1e-6);
        assert_eq!(
            path([(a, &[][..], Curve::Smooth), (b, way.0, Curve::Straight)]),
            vec![a, bend, b]
        );
        assert_eq!(
            bend_grips(a, way, b),
            vec![a.lerp(bend, 0.5), bend.lerp(b, 0.5)]
        );

        let mut arch = Mark {
            curve: Curve::Straight,
            ..arch()
        };
        arch.via = vec![bend];
        let t = track(vec![mark(0.0, 0.0, 0.0), arch]);
        let top = figure_at(&t, 4.0).unwrap().at;
        assert!(top.distance_m(bend) < 0.01, "{top:?}");
        let quarter = figure_at(&t, 2.0).unwrap().at;
        assert!(quarter.distance_m(a.lerp(bend, 0.5)) < 0.01, "{quarter:?}");
    }
}
