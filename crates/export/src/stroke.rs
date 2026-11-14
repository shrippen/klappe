//! A line with a width as polygons to fill. Like the SVG and PDF writers draw it: butt ends,
//! round joins, dashes that run on across corners.
//!
//! ```text
//!  segment ─► quad       corner ─► disc       all wound the same way, so one
//!                                             nonzero fill unites them without seams
//! ```

use std::f64::consts::{PI, TAU};

use crate::canvas::Pt;
use crate::scan::Bounds;

/// How far a flattened curve may stray from the true one, in pixels.
const CURVE_TOLERANCE_PX: f64 = 0.1;
const MIN_ARC_STEPS: usize = 8;
const MAX_ARC_STEPS: usize = 128;
/// A corner turning less than this (sine of the angle) needs no join.
const STRAIGHT_SINE: f64 = 1e-3;
/// Slack around the clip for anti-aliased edges, in pixels.
const EDGE_MARGIN_PX: f64 = 1.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Closure {
    Open,
    Closed,
}

/// Points per full circle of `radius_px`, enough to stay within `CURVE_TOLERANCE_PX`.
pub(crate) fn arc_steps(radius_px: f64) -> usize {
    if radius_px <= CURVE_TOLERANCE_PX {
        return MIN_ARC_STEPS;
    }

    let step = 2.0 * (1.0 - CURVE_TOLERANCE_PX / radius_px).acos();

    ((PI / (step / 2.0)).ceil() as usize).clamp(MIN_ARC_STEPS, MAX_ARC_STEPS)
}

/// The polygons that paint `path` with a line of `width_px`; `dash_px` is the on/off pattern
/// (empty or with a non-positive entry: solid). Only what can reach `clip` is made, so the work
/// follows the size of the image, not the length of the path; points that are not finite are
/// dropped.
pub(crate) fn outline(
    path: &[Pt],
    closure: Closure,
    width_px: f64,
    dash_px: &[f64],
    clip: Bounds,
) -> Vec<Vec<Pt>> {
    let half = width_px / 2.0;
    let mut path: Vec<Pt> = path
        .iter()
        .copied()
        .filter(|p| p.0.is_finite() && p.1.is_finite())
        .collect();
    if half <= 0.0 || !half.is_finite() || path.len() < 2 {
        return Vec::new();
    }

    if closure == Closure::Closed {
        path.push(path[0]);
    }

    // What lies further than the half width from the clip cannot colour a pixel of it.
    let reach = half + EDGE_MARGIN_PX;
    let area = Bounds {
        left: clip.left - reach,
        top: clip.top - reach,
        right: clip.right + reach,
        bottom: clip.bottom + reach,
    };
    let dashed = !dash_px.is_empty() && dash_px.iter().all(|d| *d > 0.0 && d.is_finite());
    let pieces = match dashed {
        true => dashes(&path, dash_px, area),
        false => runs(&path, area),
    };

    let mut out = Vec::new();
    for piece in &pieces {
        piece_polygons(piece, half, &mut out);
    }

    out
}

fn dist(a: Pt, b: Pt) -> f64 {
    (b.0 - a.0).hypot(b.1 - a.1)
}

/// The part of a segment inside the clip: where it enters and leaves, as fractions along it
/// and as points. A point is set on the clip's edge itself, not by the fraction, which could not
/// tell apart places a long segment's length away.
struct Span {
    from: f64,
    to: f64,
    start: Pt,
    end: Pt,
}

fn lerp(a: Pt, b: Pt, t: f64) -> Pt {
    (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t)
}

/// The part of the segment inside `area`, or `None`. A segment whose extent overflows counts as
/// outside.
fn visible(a: Pt, b: Pt, area: Bounds) -> Option<Span> {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    if !dx.is_finite() || !dy.is_finite() {
        return None;
    }

    // Liang-Barsky: per side, the distance moved towards it and the room left on it.
    let sides = [
        (-dx, a.0 - area.left),
        (dx, area.right - a.0),
        (-dy, a.1 - area.top),
        (dy, area.bottom - a.1),
    ];
    let (mut from, mut to) = (0.0_f64, 1.0_f64);
    let (mut from_side, mut to_side) = (None, None);
    for (side, (towards, room)) in sides.into_iter().enumerate() {
        if towards == 0.0 {
            if room < 0.0 {
                return None;
            }
            continue;
        }

        let t = room / towards;
        if towards < 0.0 && t > from {
            (from, from_side) = (t, Some(side));
        }
        if towards > 0.0 && t < to {
            (to, to_side) = (t, Some(side));
        }
    }
    if from > to {
        return None;
    }

    let on_edge = |t: f64, side: Option<usize>| {
        let mut p = lerp(a, b, t);
        match side {
            Some(0) => p.0 = area.left,
            Some(1) => p.0 = area.right,
            Some(2) => p.1 = area.top,
            Some(3) => p.1 = area.bottom,
            _ => {}
        }

        p
    };
    let start = match from_side {
        Some(_) => on_edge(from, from_side),
        None => a,
    };
    let end = match to_side {
        Some(_) => on_edge(to, to_side),
        None => b,
    };

    Some(Span {
        from,
        to,
        start,
        end,
    })
}

/// The runs of the path that are inside `area`, each an open polyline.
fn runs(path: &[Pt], area: Bounds) -> Vec<Vec<Pt>> {
    let mut out = Vec::new();
    let mut run: Vec<Pt> = Vec::new();

    for pair in path.windows(2) {
        let Some(span) = visible(pair[0], pair[1], area) else {
            end_run(&mut run, &mut out);
            continue;
        };

        // A run goes on only where the last one stopped.
        if span.from > 0.0 || run.is_empty() {
            end_run(&mut run, &mut out);
            run.push(span.start);
        }
        run.push(span.end);
        if span.to < 1.0 {
            end_run(&mut run, &mut out);
        }
    }
    end_run(&mut run, &mut out);

    out
}

fn end_run(run: &mut Vec<Pt>, out: &mut Vec<Vec<Pt>>) {
    if run.len() > 1 {
        out.push(std::mem::take(run));
    }
    run.clear();
}

/// Where in the on/off pattern a walk along the path is.
struct Phase<'a> {
    pattern: &'a [f64],
    /// Entry of the pattern; even ones draw, odd ones skip.
    index: usize,
    /// Left of that entry.
    left: f64,
}

impl Phase<'_> {
    fn on(&self) -> bool {
        self.index.is_multiple_of(2)
    }

    fn next(&mut self) {
        self.index = (self.index + 1) % self.pattern.len();
        self.left = self.pattern[self.index];
    }

    /// Moves on by `dist` without keeping the dashes on the way; whole periods are dropped
    /// at once, so a long way costs no more than a short one.
    fn skip(&mut self, mut dist: f64) {
        if dist > self.left {
            dist -= self.left;
            self.next();
            dist %= self.pattern.iter().sum::<f64>();
            while dist > self.left {
                dist -= self.left;
                self.next();
            }
        }

        self.left -= dist;
    }
}

/// The "on" runs of the pattern along the path, each an open polyline. The pattern starts at
/// the first point and carries over corners, as in SVG. Only the part inside `area` is walked
/// dash by dash; the rest is skipped with the pattern's phase kept.
fn dashes(path: &[Pt], pattern: &[f64], area: Bounds) -> Vec<Vec<Pt>> {
    let mut out = Vec::new();
    let mut piece: Vec<Pt> = vec![path[0]];
    let mut phase = Phase {
        pattern,
        index: 0,
        left: pattern[0],
    };

    for pair in path.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        let len = dist(a, b);
        if !len.is_finite() {
            end_dash(&mut piece, &phase, &mut out);
            continue;
        }

        let Some(span) = visible(a, b, area) else {
            end_dash(&mut piece, &phase, &mut out);
            phase.skip(len);
            piece = skip_end(&phase, b);
            continue;
        };

        if span.from > 0.0 {
            end_dash(&mut piece, &phase, &mut out);
            phase.skip(dist(a, span.start));
            piece = skip_end(&phase, span.start);
        }

        // Dashes are placed from where the segment enters, which is exact, not from `a`.
        let inside = dist(span.start, span.end);
        let dir = match len > 0.0 {
            true => ((b.0 - a.0) / len, (b.1 - a.1) / len),
            false => (0.0, 0.0),
        };
        let mut at = 0.0;
        while inside - at > phase.left {
            at += phase.left;
            let p = (span.start.0 + dir.0 * at, span.start.1 + dir.1 * at);

            // Even entries draw, odd ones skip.
            match phase.on() {
                true => {
                    piece.push(p);
                    out.push(std::mem::take(&mut piece));
                }
                false => piece = vec![p],
            }
            phase.next();
        }

        phase.left -= inside - at;
        if phase.on() {
            piece.push(span.end);
        }

        if span.to < 1.0 {
            end_dash(&mut piece, &phase, &mut out);
            phase.skip(dist(span.end, b));
            piece = skip_end(&phase, b);
        }
    }

    end_dash(&mut piece, &phase, &mut out);

    out
}

/// Closes the dash being drawn, if there is one.
fn end_dash(piece: &mut Vec<Pt>, phase: &Phase, out: &mut Vec<Vec<Pt>>) {
    if phase.on() && piece.len() > 1 {
        out.push(std::mem::take(piece));
    }
    piece.clear();
}

/// The dash that goes on at `p` after a skip, if the pattern is on there.
fn skip_end(phase: &Phase, p: Pt) -> Vec<Pt> {
    match phase.on() {
        true => vec![p],
        false => Vec::new(),
    }
}

/// A quad per segment and a disc at each corner that turns.
fn piece_polygons(piece: &[Pt], half: f64, out: &mut Vec<Vec<Pt>>) {
    let mut previous: Option<(f64, f64)> = None;

    for (i, pair) in piece.windows(2).enumerate() {
        let (a, b) = (pair[0], pair[1]);
        let len = dist(a, b);
        if len <= 0.0 {
            continue;
        }

        let dir = ((b.0 - a.0) / len, (b.1 - a.1) / len);
        let n = (-dir.1 * half, dir.0 * half);
        out.push(vec![
            (a.0 + n.0, a.1 + n.1),
            (b.0 + n.0, b.1 + n.1),
            (b.0 - n.0, b.1 - n.1),
            (a.0 - n.0, a.1 - n.1),
        ]);

        if let Some(before) = previous {
            let sine = before.0 * dir.1 - before.1 * dir.0;
            let cosine = before.0 * dir.0 + before.1 * dir.1;
            if sine.abs() > STRAIGHT_SINE || cosine < 0.0 {
                out.push(disc(piece[i], half));
            }
        }
        previous = Some(dir);
    }
}

/// Wound like the quads (clockwise in y-up terms), so both add up under the nonzero rule.
fn disc(center: Pt, radius: f64) -> Vec<Pt> {
    let steps = arc_steps(radius);

    (0..steps)
        .map(|k| {
            let angle = -TAU * k as f64 / steps as f64;
            (
                center.0 + radius * angle.cos(),
                center.1 + radius * angle.sin(),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const VIEW: Bounds = Bounds {
        left: -1000.0,
        top: -1000.0,
        right: 1000.0,
        bottom: 1000.0,
    };

    fn signed_area(poly: &[Pt]) -> f64 {
        let mut sum = 0.0;
        for (i, a) in poly.iter().enumerate() {
            let b = poly[(i + 1) % poly.len()];
            sum += a.0 * b.1 - b.0 * a.1;
        }

        sum / 2.0
    }

    #[test]
    fn a_segment_is_one_quad_with_butt_ends() {
        let polys = outline(&[(0.0, 0.0), (10.0, 0.0)], Closure::Open, 2.0, &[], VIEW);

        assert_eq!(polys.len(), 1);
        assert!((signed_area(&polys[0]).abs() - 20.0).abs() < 1e-9);
        let xs: Vec<f64> = polys[0].iter().map(|p| p.0).collect();
        assert!(xs.iter().all(|x| *x == 0.0 || *x == 10.0));
    }

    #[test]
    fn every_polygon_runs_the_same_way() {
        let path = [(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 5.0)];
        let polys = outline(&path, Closure::Closed, 2.0, &[], VIEW);

        assert!(polys.len() > 4);
        assert!(polys.iter().all(|p| signed_area(p) < 0.0));
    }

    #[test]
    fn a_straight_continuation_gets_no_join() {
        let path = [(0.0, 0.0), (5.0, 0.0), (10.0, 0.0)];

        assert_eq!(outline(&path, Closure::Open, 2.0, &[], VIEW).len(), 2);
    }

    #[test]
    fn the_dashes_follow_the_pattern_across_corners() {
        // 6 on, 3 off along 10 + 10: on 0..6, off 6..9, on 9..15 (corner at 10), off 15..18,
        // on 18..20.
        let path = [(0.0, 0.0), (10.0, 0.0), (10.0, 10.0)];
        let pieces = dashes(&path, &[6.0, 3.0], VIEW);

        assert_eq!(pieces.len(), 3);
        assert_eq!(pieces[0], vec![(0.0, 0.0), (6.0, 0.0)]);
        assert_eq!(pieces[1], vec![(9.0, 0.0), (10.0, 0.0), (10.0, 5.0)]);
        assert_eq!(pieces[2], vec![(10.0, 8.0), (10.0, 10.0)]);
    }

    #[test]
    fn closing_a_ring_adds_its_last_edge() {
        let square = [(0.0, 0.0), (4.0, 0.0), (4.0, 4.0), (0.0, 4.0)];
        let open = outline(&square, Closure::Open, 1.0, &[], VIEW);
        let closed = outline(&square, Closure::Closed, 1.0, &[], VIEW);

        assert!(closed.len() > open.len());
    }

    #[test]
    fn a_degenerate_line_draws_nothing() {
        assert!(outline(&[(1.0, 1.0)], Closure::Open, 2.0, &[], VIEW).is_empty());
        assert!(outline(&[(0.0, 0.0), (1.0, 0.0)], Closure::Open, 0.0, &[], VIEW).is_empty());
        assert!(outline(&[(1.0, 1.0), (1.0, 1.0)], Closure::Open, 2.0, &[], VIEW).is_empty());
    }

    #[test]
    fn bigger_circles_take_more_steps() {
        assert!(arc_steps(100.0) > arc_steps(5.0));
        assert_eq!(arc_steps(0.01), MIN_ARC_STEPS);
    }

    #[test]
    fn dashes_far_outside_the_clip_are_not_made() {
        let clip = Bounds {
            left: 0.0,
            top: 0.0,
            right: 200.0,
            bottom: 100.0,
        };
        let path = [(10.0, 50.0), (1e6, 50.0)];
        let polys = outline(&path, Closure::Open, 4.0, &[6.0, 3.0], clip);

        // 200 px hold 22 periods of 9 px.
        assert!(polys.len() < 100, "{}", polys.len());
        assert!(!polys.is_empty());
    }

    #[test]
    fn a_dash_in_the_clip_keeps_its_phase_from_the_start() {
        // On from 0, 9, 18 ... for 6, so 99..105, 108..114, 117..123, 126..132.
        let area = Bounds {
            left: 100.0,
            top: -5.0,
            right: 130.0,
            bottom: 5.0,
        };
        let pieces = dashes(&[(0.0, 0.0), (1e9, 0.0)], &[6.0, 3.0], area);

        let xs: Vec<(f64, f64)> = pieces.iter().map(|p| (p[0].0, p[p.len() - 1].0)).collect();
        assert_eq!(
            xs,
            [
                (100.0, 105.0),
                (108.0, 114.0),
                (117.0, 123.0),
                (126.0, 130.0)
            ]
        );
    }

    #[test]
    fn points_that_are_not_finite_are_dropped() {
        let path = [
            (0.0, 0.0),
            (f64::NAN, 5.0),
            (10.0, 0.0),
            (f64::INFINITY, 0.0),
        ];
        let polys = outline(&path, Closure::Open, 2.0, &[], VIEW);

        assert_eq!(polys.len(), 1);
    }
}
