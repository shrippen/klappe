//! Walls with their doors and windows (LH-10): the wall in pieces between the gaps, and what
//! stands in each gap.

use klappe_model::{DoorSide, Hinge, Opening, OpeningKind, Point, SetPlan, Wall};

use crate::display::{Item, Role, Shape, Source};

/// Straight pieces a door's swing is drawn with.
const SWING_STEPS: usize = 8;
/// A window's two lines stand this far from the wall's middle line, in wall thicknesses.
const WINDOW_LINE_OFFSET: f64 = 0.25;

/// The set's walls, then its openings (so a click takes the opening before the wall).
pub(crate) fn wall_items(set: &SetPlan, items: &mut Vec<Item>) {
    for (index, wall) in set.walls.iter().enumerate() {
        wall_pieces(index, wall, items);
    }

    for (wall_index, wall) in set.walls.iter().enumerate() {
        for (index, opening) in wall.openings.iter().enumerate() {
            let source = Source::Opening {
                wall: wall_index,
                index,
            };
            opening_items(wall, opening, source, items);
        }
    }
}

/// The wall as the pieces between its openings; one piece when it has none.
fn wall_pieces(index: usize, wall: &Wall, items: &mut Vec<Item>) {
    for (start_m, end_m) in solid_spans(wall) {
        items.push(Item {
            role: Role::Wall,
            shape: Shape::Line {
                from: wall.point_at(start_m),
                to: wall.point_at(end_m),
                width_m: wall.thickness_m,
            },
            label: None,
            source: Some(Source::Wall(index)),
        });
    }
}

/// Where the wall stands, in metres from `from`: the stretches between its openings.
/// Plan and camera view both draw these (e.g. 0..3 and 7..10 for a 4 m door at 5 m).
pub(crate) fn solid_spans(wall: &Wall) -> Vec<(f64, f64)> {
    let length_m = wall.length_m();
    let mut spans = wall
        .openings
        .iter()
        .map(Opening::span_m)
        .collect::<Vec<_>>();
    spans.sort_by(|a, b| a.0.total_cmp(&b.0));

    // Walks along the wall; `cursor_m` is where the next piece starts.
    let mut cursor_m = 0.0;
    let mut pieces = Vec::new();

    for (start_m, end_m) in spans {
        if start_m > cursor_m {
            pieces.push((cursor_m, start_m.min(length_m)));
        }
        cursor_m = cursor_m.max(end_m);
    }

    // A wall of length 0 keeps its (0, 0) piece, so it can still be clicked.
    if cursor_m < length_m || length_m == 0.0 {
        pieces.push((cursor_m, length_m));
    }

    pieces
}

/// A door: the hinge at the start (or end) of the gap, the leaf standing open at a right angle
/// to the right (or left) of the wall, and the swing back to the other end of the gap. A window: two lines across the gap.
fn opening_items(wall: &Wall, opening: &Opening, source: Source, items: &mut Vec<Item>) {
    let (start_m, end_m) = opening.span_m();
    let (a, b) = (wall.point_at(start_m), wall.point_at(end_m));
    let length_m = wall.length_m();
    if length_m == 0.0 {
        return;
    }

    // `unit` runs along the wall, `side` is a right angle to it.
    let unit = (
        (wall.to.x_m - wall.from.x_m) / length_m,
        (wall.to.y_m - wall.from.y_m) / length_m,
    );
    let mut side = (-unit.1, unit.0);
    if opening.side == DoorSide::Left {
        side = (-side.0, -side.1);
    }

    match opening.kind {
        OpeningKind::Door => {
            // The leaf closes towards the gap's other end, along `unit` or against it.
            let (hinge, shut) = match opening.hinge {
                Hinge::Start => (a, unit),
                Hinge::End => (b, (-unit.0, -unit.1)),
            };

            let mut points = vec![hinge];
            for step in 0..=SWING_STEPS {
                let turn = std::f64::consts::FRAC_PI_2 * step as f64 / SWING_STEPS as f64;
                let (sin, cos) = turn.sin_cos();
                points.push(Point::new(
                    hinge.x_m + opening.width_m * (side.0 * cos + shut.0 * sin),
                    hinge.y_m + opening.width_m * (side.1 * cos + shut.1 * sin),
                ));
            }

            items.push(Item {
                role: Role::Door,
                shape: Shape::Polygon { points },
                label: None,
                source: Some(source),
            });
        }
        OpeningKind::Window => {
            let off = wall.thickness_m * WINDOW_LINE_OFFSET;

            for sign in [-1.0, 1.0] {
                let shift =
                    |p: Point| Point::new(p.x_m + side.0 * off * sign, p.y_m + side.1 * off * sign);

                items.push(Item {
                    role: Role::Window,
                    shape: Shape::Polyline {
                        points: vec![shift(a), shift(b)],
                    },
                    label: None,
                    source: Some(source.clone()),
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 10 m wall along x at y = 2, 0.2 m thick.
    fn wall_with(openings: Vec<Opening>) -> SetPlan {
        let mut wall = Wall::new(Point::new(0.0, 2.0), Point::new(10.0, 2.0), 0.2);
        wall.openings = openings;

        SetPlan {
            walls: vec![wall],
            ..SetPlan::default()
        }
    }

    fn opening(kind: OpeningKind, at_m: f64, width_m: f64) -> Opening {
        Opening {
            kind,
            at_m,
            width_m,
            head_m: None,
            sill_m: None,
            side: DoorSide::Right,
            hinge: Hinge::Start,
        }
    }

    fn items(set: &SetPlan) -> Vec<Item> {
        let mut items = Vec::new();
        wall_items(set, &mut items);
        items
    }

    fn pieces(items: &[Item]) -> Vec<(f64, f64)> {
        items
            .iter()
            .filter_map(|i| match (&i.role, &i.shape) {
                (Role::Wall, Shape::Line { from, to, .. }) => Some((from.x_m, to.x_m)),
                _ => None,
            })
            .collect()
    }

    /// A wall of length 0 stays a piece, so it can still be clicked.
    #[test]
    fn a_zero_length_wall_keeps_its_piece() {
        let mut set = wall_with(vec![]);
        set.walls[0].to = set.walls[0].from;

        assert_eq!(pieces(&items(&set)), [(0.0, 0.0)]);
    }

    #[test]
    fn a_wall_without_openings_is_one_piece() {
        let list = items(&wall_with(vec![]));

        assert_eq!(pieces(&list), [(0.0, 10.0)]);
        assert_eq!(list.len(), 1);
    }

    /// A door 0.9 m wide centred at 4 m: the wall stops at 3.55 and goes on at 4.45.
    #[test]
    fn the_wall_has_a_gap_where_an_opening_is() {
        let list = items(&wall_with(vec![opening(OpeningKind::Door, 4.0, 0.9)]));
        let found = pieces(&list);

        assert_eq!(found.len(), 2);
        assert!((found[0].1 - 3.55).abs() < 1e-9);
        assert!((found[1].0 - 4.45).abs() < 1e-9);
        assert_eq!((found[0].0, found[1].1), (0.0, 10.0));
        assert!(list
            .iter()
            .filter(|i| i.role == Role::Wall)
            .all(|i| i.source == Some(Source::Wall(0))));
    }

    /// An opening at the very end leaves one piece; the order they are listed in does not
    /// matter.
    #[test]
    fn gaps_at_the_end_and_in_any_order() {
        let set = wall_with(vec![
            opening(OpeningKind::Window, 6.0, 1.0),
            opening(OpeningKind::Door, 0.45, 0.9),
        ]);

        let found = pieces(&items(&set));

        assert_eq!(found.len(), 2);
        assert!((found[0].0 - 0.9).abs() < 1e-9 && (found[0].1 - 5.5).abs() < 1e-9);
        assert!((found[1].0 - 6.5).abs() < 1e-9 && (found[1].1 - 10.0).abs() < 1e-9);
    }

    #[test]
    fn a_door_is_a_leaf_and_a_swing_from_the_hinge() {
        let list = items(&wall_with(vec![opening(OpeningKind::Door, 4.0, 1.0)]));
        let door = list.iter().find(|i| i.role == Role::Door).unwrap();
        let Shape::Polygon { points } = &door.shape else {
            panic!("a door is a polygon: {:?}", door.shape);
        };

        assert_eq!(door.source, Some(Source::Opening { wall: 0, index: 0 }));
        // Hinge at 3.5, the leaf tip a door's width to the side, the swing ends at 4.5.
        let (hinge, tip, last) = (points[0], points[1], points[points.len() - 1]);
        assert_eq!((hinge.x_m, hinge.y_m), (3.5, 2.0));
        assert!((tip.x_m - 3.5).abs() < 1e-9 && ((tip.y_m - 2.0).abs() - 1.0).abs() < 1e-9);
        assert!((last.x_m - 4.5).abs() < 1e-9 && (last.y_m - 2.0).abs() < 1e-9);
        // Every point of the swing is a door's width from the hinge.
        assert!(points[1..]
            .iter()
            .all(|p| (hinge.distance_m(*p) - 1.0).abs() < 1e-9));
    }

    /// A 1 m door in the gap 3.5..4.5 of the wall along x at y = 2 (y runs down, so right of
    /// the wall is y = 3): the hinge, the leaf tip and the end of the swing of each swing.
    #[test]
    fn a_door_swings_by_its_side_and_hinge() {
        let cases = [
            (
                DoorSide::Right,
                Hinge::Start,
                [(3.5, 2.0), (3.5, 3.0), (4.5, 2.0)],
            ),
            (
                DoorSide::Left,
                Hinge::Start,
                [(3.5, 2.0), (3.5, 1.0), (4.5, 2.0)],
            ),
            (
                DoorSide::Right,
                Hinge::End,
                [(4.5, 2.0), (4.5, 3.0), (3.5, 2.0)],
            ),
            (
                DoorSide::Left,
                Hinge::End,
                [(4.5, 2.0), (4.5, 1.0), (3.5, 2.0)],
            ),
        ];

        for (side, hinge, [want_hinge, want_tip, want_last]) in cases {
            let mut door = opening(OpeningKind::Door, 4.0, 1.0);
            door.side = side;
            door.hinge = hinge;
            let list = items(&wall_with(vec![door]));
            let found = list.iter().find(|i| i.role == Role::Door).unwrap();
            let Shape::Polygon { points } = &found.shape else {
                panic!("a door is a polygon");
            };
            let at = |p: Point| (p.x_m, p.y_m);
            let near = |p: Point, w: (f64, f64)| (p.x_m - w.0).abs() + (p.y_m - w.1).abs() < 1e-9;

            assert!(
                near(points[0], want_hinge),
                "{side:?} {hinge:?} hinge {:?}",
                at(points[0])
            );
            assert!(
                near(points[1], want_tip),
                "{side:?} {hinge:?} tip {:?}",
                at(points[1])
            );
            let last = points[points.len() - 1];
            assert!(
                near(last, want_last),
                "{side:?} {hinge:?} end {:?}",
                at(last)
            );
        }
    }

    #[test]
    fn a_window_is_two_lines_in_the_gap() {
        let list = items(&wall_with(vec![opening(OpeningKind::Window, 4.0, 1.2)]));
        let lines: Vec<_> = list.iter().filter(|i| i.role == Role::Window).collect();

        assert_eq!(lines.len(), 2);
        for line in &lines {
            let Shape::Polyline { points } = &line.shape else {
                panic!("a window line is a polyline");
            };
            assert!((points[0].x_m - 3.4).abs() < 1e-9 && (points[1].x_m - 4.6).abs() < 1e-9);
            // Inside the wall's thickness, either side of its middle line.
            assert!((points[0].y_m - 2.0).abs() < 0.1);
        }
        assert!(lines[0].shape != lines[1].shape);
    }

    #[test]
    fn openings_come_after_the_walls_so_a_click_finds_them_first() {
        let list = items(&wall_with(vec![opening(OpeningKind::Door, 4.0, 1.0)]));
        let first_opening = list.iter().position(|i| i.role == Role::Door).unwrap();
        let last_wall = list.iter().rposition(|i| i.role == Role::Wall).unwrap();

        assert!(last_wall < first_opening);
    }
}
