//! A line with a width as polygons to fill. Like the SVG and PDF writers draw it: butt ends,
//! round joins, dashes that run on across corners.
//!
//! ```text
//!  segment ─► quad       corner ─► disc       all wound the same way, so one
//!                                             nonzero fill unites them without seams
//! ```

use std::f64::consts::{PI, TAU};

use crate::canvas::Pt;

/// How far a flattened curve may stray from the true one, in pixels.
const CURVE_TOLERANCE_PX: f64 = 0.1;
const MIN_ARC_STEPS: usize = 8;
const MAX_ARC_STEPS: usize = 128;
/// A corner turning less than this (sine of the angle) needs no join.
const STRAIGHT_SINE: f64 = 1e-3;

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
/// (empty or with a non-positive entry: solid).
pub(crate) fn outline(
    path: &[Pt],
    closure: Closure,
    width_px: f64,
    dash_px: &[f64],
) -> Vec<Vec<Pt>> {
    let half = width_px / 2.0;
    if half <= 0.0 || !half.is_finite() || path.len() < 2 {
        return Vec::new();
    }

    let mut path = path.to_vec();
    if closure == Closure::Closed {
        path.push(path[0]);
    }

    let dashed = !dash_px.is_empty() && dash_px.iter().all(|d| *d > 0.0 && d.is_finite());
    let pieces = match dashed {
        true => dashes(&path, dash_px),
        false => vec![path],
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

/// The "on" runs of the pattern along the path, each an open polyline. The pattern starts at
/// the first point and carries over corners, as in SVG.
fn dashes(path: &[Pt], pattern: &[f64]) -> Vec<Vec<Pt>> {
    let mut out = Vec::new();
    let mut piece: Vec<Pt> = vec![path[0]];
    let mut index = 0;
    let mut left = pattern[0];

    for pair in path.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        let len = dist(a, b);
        let mut at = 0.0;

        while len - at > left {
            at += left;
            let t = at / len;
            let p = (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);

            // Even entries draw, odd ones skip.
            match index % 2 == 0 {
                true => {
                    piece.push(p);
                    out.push(std::mem::take(&mut piece));
                }
                false => piece = vec![p],
            }
            index = (index + 1) % pattern.len();
            left = pattern[index];
        }

        left -= len - at;
        if index % 2 == 0 {
            piece.push(b);
        }
    }

    if index % 2 == 0 && piece.len() > 1 {
        out.push(piece);
    }

    out
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
        let polys = outline(&[(0.0, 0.0), (10.0, 0.0)], Closure::Open, 2.0, &[]);

        assert_eq!(polys.len(), 1);
        assert!((signed_area(&polys[0]).abs() - 20.0).abs() < 1e-9);
        let xs: Vec<f64> = polys[0].iter().map(|p| p.0).collect();
        assert!(xs.iter().all(|x| *x == 0.0 || *x == 10.0));
    }

    #[test]
    fn every_polygon_runs_the_same_way() {
        let path = [(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 5.0)];
        let polys = outline(&path, Closure::Closed, 2.0, &[]);

        assert!(polys.len() > 4);
        assert!(polys.iter().all(|p| signed_area(p) < 0.0));
    }

    #[test]
    fn a_straight_continuation_gets_no_join() {
        let path = [(0.0, 0.0), (5.0, 0.0), (10.0, 0.0)];

        assert_eq!(outline(&path, Closure::Open, 2.0, &[]).len(), 2);
    }

    #[test]
    fn the_dashes_follow_the_pattern_across_corners() {
        // 6 on, 3 off along 10 + 10: on 0..6, off 6..9, on 9..15 (corner at 10), off 15..18,
        // on 18..20.
        let path = [(0.0, 0.0), (10.0, 0.0), (10.0, 10.0)];
        let pieces = dashes(&path, &[6.0, 3.0]);

        assert_eq!(pieces.len(), 3);
        assert_eq!(pieces[0], vec![(0.0, 0.0), (6.0, 0.0)]);
        assert_eq!(pieces[1], vec![(9.0, 0.0), (10.0, 0.0), (10.0, 5.0)]);
        assert_eq!(pieces[2], vec![(10.0, 8.0), (10.0, 10.0)]);
    }

    #[test]
    fn closing_a_ring_adds_its_last_edge() {
        let square = [(0.0, 0.0), (4.0, 0.0), (4.0, 4.0), (0.0, 4.0)];
        let open = outline(&square, Closure::Open, 1.0, &[]);
        let closed = outline(&square, Closure::Closed, 1.0, &[]);

        assert!(closed.len() > open.len());
    }

    #[test]
    fn a_degenerate_line_draws_nothing() {
        assert!(outline(&[(1.0, 1.0)], Closure::Open, 2.0, &[]).is_empty());
        assert!(outline(&[(0.0, 0.0), (1.0, 0.0)], Closure::Open, 0.0, &[]).is_empty());
        assert!(outline(&[(1.0, 1.0), (1.0, 1.0)], Closure::Open, 2.0, &[]).is_empty());
    }

    #[test]
    fn bigger_circles_take_more_steps() {
        assert!(arc_steps(100.0) > arc_steps(5.0));
        assert_eq!(arc_steps(0.01), MIN_ARC_STEPS);
    }
}
