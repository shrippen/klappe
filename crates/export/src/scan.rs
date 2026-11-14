//! Polygon coverage with the nonzero rule.
//!
//! Anti-aliasing is exact across (the covered part of each pixel along x is computed from the
//! edge crossings) and sampled down (`ROW_SAMPLES` rows per pixel row). Exact area coverage
//! would need the nonzero rule resolved inside a pixel, which overlapping contours (a stroke's
//! segments and joins, glyphs that overlap) make hard; sampling rows resolves it per row with
//! a plain sorted-crossings walk. Near-horizontal edges get `ROW_SAMPLES + 1` grey levels.

use crate::canvas::Pt;

pub(crate) const ROW_SAMPLES: usize = 16;
/// Coverage below this is not worth a write.
const MIN_COVERAGE: f32 = 1.0 / 512.0;

/// The rectangle drawing may touch, in device pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Bounds {
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
}

impl Bounds {
    pub(crate) fn intersect(self, other: Bounds) -> Bounds {
        Bounds {
            left: self.left.max(other.left),
            top: self.top.max(other.top),
            right: self.right.min(other.right),
            bottom: self.bottom.min(other.bottom),
        }
    }
}

struct Edge {
    x_top: f64,
    y_top: f64,
    y_bottom: f64,
    /// x per unit of y.
    slope: f64,
    /// +1 when the polygon runs down here, -1 up.
    winding: i32,
}

/// Every polygon is closed implicitly; horizontal and non-finite edges add nothing.
fn edges(polys: &[Vec<Pt>]) -> Vec<Edge> {
    let mut out = Vec::new();

    for poly in polys {
        if poly.len() < 3 {
            continue;
        }

        for (i, &a) in poly.iter().enumerate() {
            let b = poly[(i + 1) % poly.len()];
            let finite = [a.0, a.1, b.0, b.1].iter().all(|v| v.is_finite());
            if a.1 == b.1 || !finite {
                continue;
            }

            let (top, bottom, winding) = match a.1 < b.1 {
                true => (a, b, 1),
                false => (b, a, -1),
            };
            out.push(Edge {
                x_top: top.0,
                y_top: top.1,
                y_bottom: bottom.1,
                slope: (bottom.0 - top.0) / (bottom.1 - top.1),
                winding,
            });
        }
    }

    out
}

/// Calls `emit(row, column, coverage)` for every pixel of `clip` the polygons touch;
/// coverage is 0..1. Rows run top to bottom.
pub(crate) fn scan(polys: &[Vec<Pt>], clip: Bounds, emit: &mut dyn FnMut(usize, usize, f32)) {
    let mut edges = edges(polys);
    if edges.is_empty() {
        return;
    }
    edges.sort_by(|a, b| a.y_top.total_cmp(&b.y_top));

    let mut shape = Bounds {
        left: f64::INFINITY,
        top: edges[0].y_top,
        right: f64::NEG_INFINITY,
        bottom: f64::NEG_INFINITY,
    };
    for e in &edges {
        let x_bottom = e.x_top + (e.y_bottom - e.y_top) * e.slope;
        shape.left = shape.left.min(e.x_top).min(x_bottom);
        shape.right = shape.right.max(e.x_top).max(x_bottom);
        shape.bottom = shape.bottom.max(e.y_bottom);
    }

    let area = shape.intersect(clip);
    let area = area.intersect(Bounds {
        left: 0.0,
        top: 0.0,
        right: f64::INFINITY,
        bottom: f64::INFINITY,
    });
    if area.left >= area.right || area.top >= area.bottom {
        return;
    }

    let col0 = area.left.floor() as usize;
    let col1 = area.right.ceil() as usize;
    let width = col1 - col0;
    let span_left = clip.left.max(col0 as f64) - col0 as f64;
    let span_right = clip.right.min(col1 as f64) - col0 as f64;

    // Per pixel: the partial coverage at span ends, plus the +1/-1 marks of whole pixels
    // between them, summed on the way across.
    let mut partial = vec![0.0_f32; width + 2];
    let mut runs = vec![0.0_f32; width + 2];
    let mut crossings: Vec<(f64, i32)> = Vec::new();
    let mut active: Vec<usize> = Vec::new();
    let mut next = 0;

    for row in area.top.floor() as usize..area.bottom.ceil() as usize {
        let row_top = row as f64;
        while next < edges.len() && edges[next].y_top < row_top + 1.0 {
            active.push(next);
            next += 1;
        }
        active.retain(|&i| edges[i].y_bottom > row_top);
        if active.is_empty() {
            continue;
        }

        partial.fill(0.0);
        runs.fill(0.0);
        for sample in 0..ROW_SAMPLES {
            let y = row_top + (sample as f64 + 0.5) / ROW_SAMPLES as f64;
            if y < clip.top || y >= clip.bottom {
                continue;
            }

            crossings.clear();
            for &i in &active {
                let e = &edges[i];
                if e.y_top <= y && y < e.y_bottom {
                    crossings.push((e.x_top + (y - e.y_top) * e.slope, e.winding));
                }
            }
            crossings.sort_by(|a, b| a.0.total_cmp(&b.0));

            let (mut winding, mut start) = (0, 0.0);
            for &(x, direction) in &crossings {
                let before = winding;
                winding += direction;
                if before == 0 && winding != 0 {
                    start = x;
                }
                if before != 0 && winding == 0 {
                    let from = (start - col0 as f64).clamp(span_left, span_right);
                    let to = (x - col0 as f64).clamp(span_left, span_right);
                    add_span(&mut partial, &mut runs, from, to);
                }
            }
        }

        let mut whole = 0.0_f32;
        for col in 0..width {
            whole += runs[col];
            let coverage = (partial[col] + whole) / ROW_SAMPLES as f32;
            if coverage > MIN_COVERAGE {
                emit(row, col0 + col, coverage.min(1.0));
            }
        }
    }
}

/// Adds the span `from..to` (columns, `from <= to`, inside the buffers) as one sample row.
fn add_span(partial: &mut [f32], runs: &mut [f32], from: f64, to: f64) {
    if to <= from {
        return;
    }

    let first = from.floor() as usize;
    let last = to.floor() as usize;
    if first == last {
        partial[first] += (to - from) as f32;
        return;
    }

    partial[first] += ((first + 1) as f64 - from) as f32;
    partial[last] += (to - last as f64) as f32;
    runs[first + 1] += 1.0;
    runs[last] -= 1.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    const WIDE: Bounds = Bounds {
        left: 0.0,
        top: 0.0,
        right: 100.0,
        bottom: 100.0,
    };

    fn coverage(polys: &[Vec<Pt>], clip: Bounds) -> Vec<(usize, usize, f32)> {
        let mut out = Vec::new();
        scan(polys, clip, &mut |r, c, v| out.push((r, c, v)));

        out
    }

    fn square(x0: f64, y0: f64, x1: f64, y1: f64) -> Vec<Pt> {
        vec![(x0, y0), (x1, y0), (x1, y1), (x0, y1)]
    }

    #[test]
    fn a_pixel_aligned_square_is_fully_covered() {
        let got = coverage(&[square(2.0, 3.0, 5.0, 4.0)], WIDE);

        assert_eq!(got, vec![(3, 2, 1.0), (3, 3, 1.0), (3, 4, 1.0)]);
    }

    #[test]
    fn a_half_pixel_edge_is_half_covered() {
        let got = coverage(&[square(1.5, 0.0, 3.25, 1.0)], WIDE);

        assert_eq!(got, vec![(0, 1, 0.5), (0, 2, 1.0), (0, 3, 0.25)]);
    }

    #[test]
    fn rows_are_sampled_vertically() {
        let got = coverage(&[square(0.0, 0.25, 1.0, 1.0)], WIDE);

        assert_eq!(got, vec![(0, 0, 0.75)]);
    }

    #[test]
    fn nonzero_unites_overlaps_of_the_same_direction() {
        let a = square(0.0, 0.0, 3.0, 1.0);
        let b = square(2.0, 0.0, 5.0, 1.0);
        let got = coverage(&[a, b], WIDE);

        assert_eq!(got.len(), 5);
        assert!(got.iter().all(|(_, _, c)| *c == 1.0));
    }

    #[test]
    fn the_opposite_direction_makes_a_hole() {
        let outer = square(0.0, 0.0, 5.0, 1.0);
        let mut hole = square(2.0, 0.0, 3.0, 1.0);
        hole.reverse();
        let got = coverage(&[outer, hole], WIDE);
        let columns: Vec<usize> = got.iter().map(|g| g.1).collect();

        assert_eq!(columns, vec![0, 1, 3, 4]);
    }

    #[test]
    fn the_clip_cuts_with_a_soft_edge() {
        let clip = Bounds {
            left: 1.5,
            top: 0.0,
            right: 3.0,
            bottom: 1.0,
        };
        let got = coverage(&[square(0.0, 0.0, 10.0, 1.0)], clip);

        assert_eq!(got, vec![(0, 1, 0.5), (0, 2, 1.0)]);
    }

    #[test]
    fn a_triangle_covers_half_its_box() {
        let triangle = vec![(0.0, 0.0), (4.0, 0.0), (0.0, 4.0)];
        let total: f32 = coverage(&[triangle], WIDE).iter().map(|g| g.2).sum();

        assert!((total - 8.0).abs() < 0.05, "{total}");
    }

    #[test]
    fn broken_input_draws_nothing() {
        let nan = vec![(f64::NAN, 0.0), (1.0, 0.0), (0.0, 1.0)];

        assert!(coverage(&[nan], WIDE).is_empty());
        assert!(coverage(&[vec![(0.0, 0.0), (1.0, 1.0)]], WIDE).is_empty());
    }
}
