//! Grips on points picked together: a box over a selected way's or drawing's grips (LH-08).

use klappe_services::Source;

use super::Selection;

/// The way, drawing or free form a grip on a point belongs to; `None` for anything else.
fn grip_way(source: &Source) -> Option<Source> {
    match source {
        Source::FigureVia { character, .. } => Some(Source::Figure(character.clone())),
        Source::CameraVia { camera, .. } => Some(Source::Camera(camera.clone())),
        grip if grip.selected_itself() => grip.owner(),
        _ => None,
    }
}

impl Selection {
    /// The source whose grips show: the one element picked, or, for several grips of one way,
    /// drawing or free form picked together (a box over its grips), one of them.
    pub fn handled(&self) -> Option<&Source> {
        match self.0.as_slice() {
            [] => None,
            [only] => Some(only),
            [first, rest @ ..] => {
                let way = grip_way(first)?;

                rest.iter()
                    .all(|s| grip_way(s).as_ref() == Some(&way))
                    .then_some(first)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::{group_drag_edit, Pick};
    use super::*;
    use klappe_services::{Bend, PlanEdit, Point};

    fn p(x_m: f64, y_m: f64) -> Point {
        Point::new(x_m, y_m)
    }

    fn bend(n: usize) -> Source {
        Source::FigureVia {
            character: "elisabeth".into(),
            index: 1,
            bend: Bend::At(n),
        }
    }

    /// A box over the grips of the one way selected takes them together; their grips go on
    /// showing.
    #[test]
    fn grips_of_one_way_are_picked_together() {
        let way = Selection::of(Some(Source::Figure("elisabeth".into())));
        assert_eq!(way.handled(), Some(&Source::Figure("elisabeth".into())));

        let grips = way.boxed(vec![bend(0), bend(1)], Pick::Replace);

        assert_eq!(grips.len(), 2);
        assert_eq!(grips.handled(), Some(&bend(0)));

        // Another way's grip, or an element, makes it a selection of several.
        let other = Source::FigureVia {
            character: "burgvogt".into(),
            index: 1,
            bend: Bend::At(0),
        };
        let mixed = grips.clone().boxed(vec![other], Pick::Extend);
        assert_eq!(mixed.handled(), None);
        assert_eq!(mixed.sources(), []);
        let wall = grips.boxed(vec![Source::Wall(0)], Pick::Extend);
        assert_eq!(wall.sources(), [Source::Wall(0)]);
    }

    #[test]
    fn dragging_one_of_the_grips_moves_all() {
        let grips = Selection::default().boxed(vec![bend(0), bend(1)], Pick::Replace);

        let edit = group_drag_edit(&grips, &bend(1), p(0.0, 0.0), p(1.0, 0.0));

        assert_eq!(
            edit,
            Some(PlanEdit::ShiftGroup {
                sources: vec![bend(1), bend(0)],
                dx_m: 1.0,
                dy_m: 0.0
            })
        );
    }
}
