//! Splits a flat face, seen on the screen, into triangles (LH-92: a free form's top is not
//! convex, so a fan from one corner would cover what lies outside it).

/// A corner on the screen.
type Spot = [f64; 2];

/// The triangles of the polygon `poly` as index triples, in the winding of `poly`. A convex
/// polygon is a fan from its first corner; any other simple polygon is cut by ear clipping.
/// A polygon with no area, or under three corners, gives none.
pub(crate) fn triangulate(poly: &[Spot]) -> Vec<[usize; 3]> {
    if poly.len() < 3 {
        return Vec::new();
    }

    let turn = winding(poly);
    if turn == 0.0 {
        return Vec::new();
    }
    if is_convex(poly, turn) {
        return (1..poly.len() - 1).map(|i| [0, i, i + 1]).collect();
    }

    clip_ears(poly, turn)
}

/// Twice the area of the triangle `a`, `b`, `c`: positive when it turns the way the screen's
/// y axis (downwards) makes positive.
fn cross(a: Spot, b: Spot, c: Spot) -> f64 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}

/// Twice the signed area of `poly`: the sign is its winding.
fn winding(poly: &[Spot]) -> f64 {
    let next = poly.iter().cycle().skip(1);

    poly.iter()
        .zip(next)
        .map(|(a, b)| a[0] * b[1] - b[0] * a[1])
        .sum()
}

/// Whether no corner turns against the winding `turn`.
fn is_convex(poly: &[Spot], turn: f64) -> bool {
    let n = poly.len();

    (0..n).all(|i| cross(poly[i], poly[(i + 1) % n], poly[(i + 2) % n]) * turn >= 0.0)
}

/// Whether `p` lies inside or on the triangle `a`, `b`, `c` that turns as `turn` does.
fn inside(p: Spot, [a, b, c]: [Spot; 3], turn: f64) -> bool {
    [cross(a, b, p), cross(b, c, p), cross(c, a, p)]
        .iter()
        .all(|side| side * turn >= 0.0)
}

/// Cuts off one ear at a time: a corner that turns with the polygon and whose triangle holds no
/// other corner. A polygon that crosses itself has no such ear at some point; its first
/// corner goes then, so the loop ends.
fn clip_ears(poly: &[Spot], turn: f64) -> Vec<[usize; 3]> {
    let mut left: Vec<usize> = (0..poly.len()).collect();
    let mut out = Vec::new();

    while left.len() > 3 {
        let n = left.len();
        let ear = (0..n).find(|&i| is_ear(poly, &left, i, turn)).unwrap_or(0);

        out.push([left[(ear + n - 1) % n], left[ear], left[(ear + 1) % n]]);
        left.remove(ear);
    }
    out.push([left[0], left[1], left[2]]);

    out
}

/// Whether corner `i` of the remaining `left` is an ear.
fn is_ear(poly: &[Spot], left: &[usize], i: usize, turn: f64) -> bool {
    let n = left.len();
    let (prev, this, next) = (left[(i + n - 1) % n], left[i], left[(i + 1) % n]);
    let tri = [poly[prev], poly[this], poly[next]];
    if cross(tri[0], tri[1], tri[2]) * turn <= 0.0 {
        return false;
    }

    // A corner at the same place as one of the triangle's is a touch, not a blocker.
    let blocks = |&k: &usize| {
        let p = poly[k];
        k != prev && k != this && k != next && !tri.contains(&p) && inside(p, tri, turn)
    };

    !left.iter().any(blocks)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The area of the triangles on the screen, summed.
    fn covered(poly: &[Spot], tris: &[[usize; 3]]) -> f64 {
        tris.iter()
            .map(|t| cross(poly[t[0]], poly[t[1]], poly[t[2]]).abs() / 2.0)
            .sum()
    }

    /// An L: 2 wide, 3 deep, with a 1 x 2 notch at the top right. Area 4.
    fn l_shape() -> Vec<Spot> {
        vec![
            [0.0, 0.0],
            [1.0, 0.0],
            [1.0, 2.0],
            [2.0, 2.0],
            [2.0, 3.0],
            [0.0, 3.0],
        ]
    }

    /// A convex polygon stays the fan from its first corner.
    #[test]
    fn a_convex_polygon_is_a_fan() {
        let square = [[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0]];

        assert_eq!(triangulate(&square), vec![[0, 1, 2], [0, 2, 3]]);
        assert_eq!(triangulate(&square[..3]), vec![[0, 1, 2]]);
    }

    /// The triangles of an L cover its area exactly and none leaves it: every triangle's centre
    /// is inside the L, and the notch's middle is in none.
    #[test]
    fn an_l_is_covered_exactly() {
        let l = l_shape();

        for poly in [l.clone(), l.iter().rev().copied().collect::<Vec<_>>()] {
            let tris = triangulate(&poly);

            assert_eq!(tris.len(), 4);
            assert_eq!(covered(&poly, &tris), 4.0);

            let notch = [1.5, 0.5];
            for t in &tris {
                let corners = [poly[t[0]], poly[t[1]], poly[t[2]]];
                let turn = cross(corners[0], corners[1], corners[2]);

                assert!(!inside(notch, corners, turn), "{t:?} covers the notch");
            }
        }
    }

    /// A fan from the first corner of the L would cover the notch; the ear clipping does not.
    #[test]
    fn a_fan_would_not_do_for_an_l() {
        let mut l = l_shape();
        l.rotate_left(4);
        let notch = [1.5, 1.8];
        let fan: Vec<[usize; 3]> = (1..l.len() - 1).map(|i| [0, i, i + 1]).collect();

        let tris = triangulate(&l);
        let covers = |t: &[usize; 3]| {
            let corners = [l[t[0]], l[t[1]], l[t[2]]];

            inside(notch, corners, cross(corners[0], corners[1], corners[2]))
        };

        assert!(!tris.iter().any(covers));
        assert!(fan.iter().any(covers));
    }

    /// A polygon with no area gives nothing; so does a line.
    #[test]
    fn a_flat_polygon_has_no_triangles() {
        assert!(triangulate(&[[0.0, 0.0], [1.0, 1.0], [2.0, 2.0]]).is_empty());
        assert!(triangulate(&[[0.0, 0.0], [1.0, 1.0]]).is_empty());
    }

    /// A U-shape (two ears far apart) and a polygon with a pointed notch terminate and cover
    /// their area.
    #[test]
    fn other_hollow_polygons_are_covered() {
        let u = [
            [0.0, 0.0],
            [1.0, 0.0],
            [1.0, 2.0],
            [2.0, 2.0],
            [2.0, 0.0],
            [3.0, 0.0],
            [3.0, 3.0],
            [0.0, 3.0],
        ];
        let arrow = [[0.0, 0.0], [4.0, 0.0], [2.0, 1.0], [4.0, 2.0], [0.0, 2.0]];

        assert_eq!(covered(&u, &triangulate(&u)), 9.0 - 2.0);
        assert_eq!(covered(&arrow, &triangulate(&arrow)), 8.0 - 2.0);
    }
}
