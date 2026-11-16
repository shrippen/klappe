//! What is done to a whole way, the path to a mark: moved with Ctrl held or by the arrow keys
//! (LH-107), made smooth or straight (LH-108). A way is selected through a grip of it; the
//! mark fields (`mark.rs`) take the others. Like the tools (`tools.rs`), no geometry here.

use klappe_services::{Curve, PlanEdit, Point, Source};

/// Whether `source` is a grip on a way, which selects that way.
fn is_way_grip(source: &Source) -> bool {
    matches!(source, Source::FigureVia { .. } | Source::CameraVia { .. })
}

/// A drag from `from` to `to` that grabbed `grabbed` with Ctrl held: on a grip of a way it
/// moves the whole way by the distance (the grip only would bend it).
pub fn drag_edit(grabbed: &Source, from: Point, to: Point) -> Option<PlanEdit> {
    if !is_way_grip(grabbed) {
        return None;
    }

    Some(PlanEdit::ShiftWay {
        source: grabbed.clone(),
        dx_m: to.x_m - from.x_m,
        dy_m: to.y_m - from.y_m,
    })
}

/// An arrow key on the selected way: it moves by `step_m` towards (`right`, `down`), each
/// -1, 0 or 1. `None` unless a grip of a way is the whole selection.
pub fn nudge_edit(
    selected: Option<&Source>,
    step_m: f64,
    (right, down): (i32, i32),
) -> Option<PlanEdit> {
    let source = selected.filter(|s| is_way_grip(s))?;

    Some(PlanEdit::ShiftWay {
        source: source.clone(),
        dx_m: step_m * f64::from(right),
        dy_m: step_m * f64::from(down),
    })
}

/// The shape `name` (`smooth`, `straight`) for the selected way. `None` for a selection that
/// has no way or a name that is no shape.
pub fn curve_edit(selected: Option<&Source>, name: &str) -> Option<PlanEdit> {
    let source = selected.filter(|s| {
        is_way_grip(s)
            || matches!(
                s,
                Source::Mark { .. } | Source::Figure(_) | Source::Camera(_)
            )
    })?;

    Some(PlanEdit::SetCurve {
        source: source.clone(),
        curve: Curve::from_name(name)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use klappe_services::Bend;

    fn grip() -> Source {
        Source::FigureVia {
            character: "e".into(),
            index: 1,
            bend: Bend::New(0),
        }
    }

    #[test]
    fn a_ctrl_drag_of_a_grip_moves_the_way() {
        let (from, to) = (Point::new(1.0, 1.0), Point::new(3.0, 0.5));

        assert_eq!(
            drag_edit(&grip(), from, to),
            Some(PlanEdit::ShiftWay {
                source: grip(),
                dx_m: 2.0,
                dy_m: -0.5
            })
        );
        assert_eq!(drag_edit(&Source::Figure("e".into()), from, to), None);
    }

    #[test]
    fn an_arrow_key_moves_the_selected_way_by_a_step() {
        let camera = Source::CameraVia {
            camera: "a".into(),
            index: 1,
            bend: Bend::At(0),
        };

        assert_eq!(
            nudge_edit(Some(&grip()), 0.25, (-1, 0)),
            Some(PlanEdit::ShiftWay {
                source: grip(),
                dx_m: -0.25,
                dy_m: 0.0
            })
        );
        assert!(nudge_edit(Some(&camera), 0.25, (0, 1)).is_some());
        assert_eq!(
            nudge_edit(Some(&Source::Figure("e".into())), 0.25, (0, 1)),
            None
        );
        assert_eq!(nudge_edit(None, 0.25, (0, 1)), None);
    }

    #[test]
    fn a_shape_name_for_a_way_is_a_set_curve() {
        let mark = Source::Mark {
            character: "e".into(),
            index: 1,
        };

        assert_eq!(
            curve_edit(Some(&mark), "straight"),
            Some(PlanEdit::SetCurve {
                source: mark.clone(),
                curve: Curve::Straight
            })
        );
        assert!(curve_edit(Some(&grip()), "smooth").is_some());
        assert!(curve_edit(Some(&Source::Camera("a".into())), "smooth").is_some());
        assert_eq!(curve_edit(Some(&mark), "wavy"), None);
        assert_eq!(curve_edit(Some(&Source::Wall(0)), "smooth"), None);
        assert_eq!(curve_edit(None, "smooth"), None);
    }
}
