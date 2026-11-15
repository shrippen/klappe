//! The fill itself: the triangles set up once, then filled in bands of rows, one band per
//! core; then the edges.
//!
//! ```text
//!   triangles ──► Setup (weights, 1/depth, rows, columns)
//!                   │
//!        ┌──────────┼──────────┐      each band fills its rows from every setup
//!      band 0     band 1  …  band n
//! ```

use std::num::NonZeroUsize;
use std::thread;

use crate::{Frame, Paint, Triangle, NO_SOURCE};

/// Fewer rows than this per band are not worth a thread.
const MIN_BAND_ROWS: usize = 32;

/// The frame `width` by `height` with `triangles` filled in, nearest first wins per pixel,
/// and the edges marked.
pub fn rasterize(width: usize, height: usize, triangles: &[Triangle]) -> Frame {
    let size = width * height;
    // Zeroed memory comes cheap; what starts otherwise each band sets itself, in parallel.
    let mut frame = Frame {
        width,
        height,
        role: vec![0; size],
        shade: vec![0; size],
        source: vec![0; size],
        face: vec![0; size],
        depth: vec![0.0; size],
        edge: vec![false; size],
    };
    let setups: Vec<Setup> = triangles
        .iter()
        .filter_map(|t| Setup::of(t, &frame))
        .collect();

    let cores = thread::available_parallelism().map_or(1, NonZeroUsize::get);
    let rows = height.div_ceil(cores).max(MIN_BAND_ROWS);
    let chunk = (rows * width).max(1);
    thread::scope(|scope| {
        let bands = frame
            .role
            .chunks_mut(chunk)
            .zip(frame.shade.chunks_mut(chunk))
            .zip(frame.source.chunks_mut(chunk))
            .zip(frame.face.chunks_mut(chunk))
            .zip(frame.depth.chunks_mut(chunk))
            .enumerate();
        for (n, ((((role, shade), source), face), depth)) in bands {
            let mut band = Band {
                first_row: n * rows,
                width,
                role,
                shade,
                source,
                face,
                depth,
            };
            let setups = &setups;
            scope.spawn(move || band.paint(setups));
        }
    });

    thread::scope(|scope| {
        let (face, depth) = (&frame.face, &frame.depth);
        for (n, edge) in frame.edge.chunks_mut(chunk).enumerate() {
            scope.spawn(move || mark_edges(face, depth, width, n * chunk, edge));
        }
    });

    frame
}

/// Twice the signed area of (a, b, p): positive when p lies left of a → b on the screen.
fn side(a: (f32, f32), b: (f32, f32), p: (f32, f32)) -> f32 {
    (b.0 - a.0) * (p.1 - a.1) - (b.1 - a.1) * (p.0 - a.0)
}

/// A value that runs linearly over the screen: `dx * x + dy * y + at0` at pixel centre (x, y).
#[derive(Clone, Copy)]
struct Plane {
    dx: f32,
    dy: f32,
    at0: f32,
}

impl Plane {
    /// The plane through the three corners' values.
    fn through(corners: [(f32, f32); 3], values: [f32; 3], area: f32) -> Self {
        let [a, b, c] = corners;
        // Each corner's weight is the side of the opposite edge over the area.
        let weight = |p: (f32, f32), q: (f32, f32)| Plane {
            dx: (p.1 - q.1) / area,
            dy: (q.0 - p.0) / area,
            at0: side(p, q, (0.5, 0.5)) / area,
        };
        let w = [weight(b, c), weight(c, a), weight(a, b)];

        Plane {
            dx: w.iter().zip(values).map(|(w, v)| w.dx * v).sum(),
            dy: w.iter().zip(values).map(|(w, v)| w.dy * v).sum(),
            at0: w.iter().zip(values).map(|(w, v)| w.at0 * v).sum(),
        }
    }

    fn at(&self, x: f32, y: f32) -> f32 {
        self.dx * x + self.dy * y + self.at0
    }
}

/// A triangle ready to fill: its weights and 1/depth over the screen, the pixels it can touch.
struct Setup {
    weights: [Plane; 3],
    depth: Plane,
    columns: (usize, usize),
    rows: (usize, usize),
    paint: Paint,
}

impl Setup {
    /// `None` for a triangle without area or outside the frame.
    fn of(triangle: &Triangle, frame: &Frame) -> Option<Self> {
        let corners = triangle.corners.map(|k| (k.x, k.y));
        let [a, b, c] = corners;
        let area = side(a, b, c);
        if area == 0.0 || !area.is_finite() {
            return None;
        }
        let (x0, x1, y0, y1) = bounds(frame, corners)?;

        // The corners' weights; all of them at least 0 inside, whichever way it winds.
        Some(Self {
            weights: [
                Plane::through(corners, [1.0, 0.0, 0.0], area),
                Plane::through(corners, [0.0, 1.0, 0.0], area),
                Plane::through(corners, [0.0, 0.0, 1.0], area),
            ],
            depth: Plane::through(corners, triangle.corners.map(|k| 1.0 / k.depth), area),
            columns: (x0, x1),
            rows: (y0, y1),
            paint: triangle.paint,
        })
    }
}

/// Some whole rows of the frame, from `first_row` on. While filling, `depth` holds 1/depth:
/// 0 is nothing, larger is nearer; it runs linearly on the screen, depth itself does not.
struct Band<'a> {
    first_row: usize,
    width: usize,
    role: &'a mut [u8],
    shade: &'a mut [u8],
    source: &'a mut [u32],
    face: &'a mut [u32],
    depth: &'a mut [f32],
}

impl Band<'_> {
    /// Fills the band from every setup, then turns 1/depth into depth.
    fn paint(&mut self, setups: &[Setup]) {
        self.source.fill(NO_SOURCE);
        self.face.fill(NO_SOURCE);
        for setup in setups {
            self.fill(setup);
        }
        for depth in self.depth.iter_mut() {
            *depth = if *depth > 0.0 {
                1.0 / *depth
            } else {
                f32::INFINITY
            };
        }
    }

    /// Writes the pixels of this band whose centre lies inside the triangle and which it is
    /// nearest for.
    fn fill(&mut self, setup: &Setup) {
        let last_row = self.first_row + self.role.len() / self.width.max(1);
        let (y0, y1) = (setup.rows.0.max(self.first_row), setup.rows.1.min(last_row));
        let paint = setup.paint;

        for y in y0..y1 {
            let Some((from, to)) = span(&setup.weights, y as f32, setup.columns) else {
                continue;
            };
            let row = (y - self.first_row) * self.width;
            for x in from..to {
                if paint.sparse && (x + y) % 2 == 1 {
                    continue;
                }
                let near = setup.depth.at(x as f32, y as f32);
                let i = row + x;
                if near <= self.depth[i] {
                    continue;
                }
                self.depth[i] = near;
                self.role[i] = paint.role;
                self.shade[i] = paint.shade;
                self.source[i] = paint.source;
                self.face[i] = paint.face;
            }
        }
    }
}

/// The columns from `x0` to before `x1` of row `y` whose centre all weights hold at least 0.
fn span(weights: &[Plane; 3], y: f32, (x0, x1): (usize, usize)) -> Option<(usize, usize)> {
    let (mut low, mut high) = (x0 as f32, x1 as f32);

    for w in weights {
        let rest = w.dy * y + w.at0;
        if w.dx == 0.0 {
            if rest < 0.0 {
                return None;
            }
            continue;
        }
        // dx * x + rest >= 0 holds from (or up to) this column.
        let edge = -rest / w.dx;
        if w.dx > 0.0 {
            low = low.max(edge.ceil());
        } else {
            high = high.min(edge.floor() + 1.0);
        }
    }

    (low < high).then_some((low as usize, high as usize))
}

/// The pixel rows and columns the triangle can touch, within the frame; `None` when outside.
fn bounds(frame: &Frame, corners: [(f32, f32); 3]) -> Option<(usize, usize, usize, usize)> {
    let xs = corners.map(|c| c.0);
    let ys = corners.map(|c| c.1);
    let low = |v: [f32; 3]| {
        v.iter()
            .copied()
            .fold(f32::INFINITY, f32::min)
            .floor()
            .max(0.0)
    };
    let high = |v: [f32; 3], limit: usize| {
        let top = v.iter().copied().fold(f32::NEG_INFINITY, f32::max).ceil();
        top.clamp(0.0, limit as f32)
    };

    let (x0, x1) = (low(xs) as usize, high(xs, frame.width) as usize);
    let (y0, y1) = (low(ys) as usize, high(ys, frame.height) as usize);

    (x0 < x1 && y0 < y1).then_some((x0, x1, y0, y1))
}

/// An edge is where the face changes between two neighbours; it goes on the nearer one (the
/// earlier of two as near), so an outline stays on the thing in front, e.g. a table against
/// the wall behind it. Marks the pixels from `first` on that `edge` holds.
fn mark_edges(face: &[u32], depth: &[f32], width: usize, first: usize, edge: &mut [bool]) {
    let size = face.len();

    for (k, mark) in edge.iter_mut().enumerate() {
        let i = first + k;
        let x = i % width;
        let left = (x > 0).then(|| i - 1);
        let right = (x + 1 < width).then_some(i + 1);
        let above = i.checked_sub(width);
        let below = (i + width < size).then_some(i + width);

        *mark = [left, right, above, below].into_iter().flatten().any(|j| {
            face[i] != face[j] && (depth[i] < depth[j] || (depth[i] == depth[j] && i < j))
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Corner, Paint, EMPTY};

    fn paint(role: u8, face: u32) -> Paint {
        Paint {
            role,
            shade: 200,
            source: face,
            face,
            sparse: false,
        }
    }

    fn corner(x: f32, y: f32, depth: f32) -> Corner {
        Corner { x, y, depth }
    }

    /// Two triangles: an upright rectangle from (`x0`, `y0`) to (`x1`, `y1`) at one depth.
    fn rect(x0: f32, y0: f32, x1: f32, y1: f32, depth: f32, paint: Paint) -> [Triangle; 2] {
        let (a, b) = (corner(x0, y0, depth), corner(x1, y0, depth));
        let (c, d) = (corner(x1, y1, depth), corner(x0, y1, depth));

        [
            Triangle {
                corners: [a, b, c],
                paint,
            },
            Triangle {
                corners: [a, c, d],
                paint,
            },
        ]
    }

    /// The frame as text, a row per line: `.` empty, else the role as a digit; `#` an edge.
    fn ascii(frame: &Frame, edges: bool) -> String {
        let mut text = String::new();
        for y in 0..frame.height {
            for x in 0..frame.width {
                let i = y * frame.width + x;
                let c = match frame.role[i] {
                    EMPTY => '.',
                    _ if edges && frame.edge[i] => '#',
                    role => char::from_digit(u32::from(role), 10).unwrap_or('?'),
                };
                text.push(c);
            }
            text.push('\n');
        }

        text
    }

    #[test]
    fn a_rectangle_fills_the_pixels_whose_centre_it_holds() {
        let frame = rasterize(6, 4, &rect(1.0, 1.0, 4.0, 3.0, 5.0, paint(1, 0)));

        assert_eq!(ascii(&frame, false), "......\n.111..\n.111..\n......\n");
    }

    #[test]
    fn the_nearer_one_covers_whatever_order_they_come_in() {
        let far = rect(0.0, 0.0, 6.0, 4.0, 9.0, paint(1, 0));
        let near = rect(2.0, 1.0, 4.0, 3.0, 3.0, paint(2, 1));
        let expected = "111111\n112211\n112211\n111111\n";

        let first = [far, near].concat();
        let second = [near, far].concat();

        assert_eq!(ascii(&rasterize(6, 4, &first), false), expected);
        assert_eq!(ascii(&rasterize(6, 4, &second), false), expected);
    }

    /// The outline sits on the near rectangle and on the frame of the far one where it ends.
    #[test]
    fn edges_lie_on_the_nearer_side_of_a_change() {
        let triangles = [
            rect(0.0, 0.0, 8.0, 5.0, 9.0, paint(1, 0)),
            rect(2.0, 1.0, 6.0, 4.0, 3.0, paint(2, 1)),
        ]
        .concat();

        let frame = rasterize(8, 5, &triangles);

        assert_eq!(
            ascii(&frame, true),
            "11111111\n11####11\n11#22#11\n11####11\n11111111\n"
        );
    }

    /// One face of two triangles has no edge along its diagonal.
    #[test]
    fn a_face_split_in_triangles_shows_no_diagonal() {
        let frame = rasterize(6, 6, &rect(0.0, 0.0, 6.0, 6.0, 4.0, paint(1, 7)));

        assert!(frame.edge.iter().all(|e| !e));
    }

    /// A slanted floor: depth 2 at the bottom, 10 at the top, so the middle of the screen is
    /// not the middle of the depth (perspective), but 1 / (mean of 1/2 and 1/10) = 3.33.
    #[test]
    fn depth_runs_with_the_perspective() {
        let (a, b) = (corner(0.0, 0.0, 10.0), corner(2.0, 0.0, 10.0));
        let (c, d) = (corner(2.0, 2.0, 2.0), corner(0.0, 2.0, 2.0));
        let floor = paint(1, 0);
        let triangles = [
            Triangle {
                corners: [a, b, c],
                paint: floor,
            },
            Triangle {
                corners: [a, c, d],
                paint: floor,
            },
        ];

        let frame = rasterize(2, 2, &triangles);
        // Pixel centres at y 0.5 and 1.5: a quarter and three quarters of the way down.
        let expect = |t: f32| 1.0 / ((1.0 - t) / 10.0 + t / 2.0);

        assert!(
            (frame.depth[0] - expect(0.25)).abs() < 1e-4,
            "{}",
            frame.depth[0]
        );
        assert!(
            (frame.depth[2] - expect(0.75)).abs() < 1e-4,
            "{}",
            frame.depth[2]
        );
    }

    #[test]
    fn a_sparse_face_fills_every_other_pixel_and_shows_what_is_behind() {
        let mut glass = paint(2, 1);
        glass.sparse = true;
        let triangles = [
            rect(0.0, 0.0, 4.0, 2.0, 9.0, paint(1, 0)),
            rect(0.0, 0.0, 4.0, 2.0, 3.0, glass),
        ]
        .concat();

        assert_eq!(ascii(&rasterize(4, 2, &triangles), false), "2121\n1212\n");
    }

    #[test]
    fn triangles_beyond_the_frame_are_cut_off() {
        let frame = rasterize(4, 3, &rect(-5.0, -5.0, 2.0, 9.0, 1.0, paint(3, 0)));

        assert_eq!(ascii(&frame, false), "33..\n33..\n33..\n");
    }

    #[test]
    fn source_and_shade_come_with_the_winner() {
        let mut lamp = paint(4, 12);
        lamp.shade = 90;
        let frame = rasterize(2, 1, &rect(0.0, 0.0, 1.0, 1.0, 1.0, lamp));

        assert_eq!((frame.source[0], frame.shade[0]), (12, 90));
        assert_eq!((frame.source[1], frame.role[1]), (crate::NO_SOURCE, EMPTY));
    }
}
