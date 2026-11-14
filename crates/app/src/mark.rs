//! The fields of a mark typed or picked on the card: pose, arrival, departure, pause, pace and
//! the camera's aim, each as a plan edit. Like the tools (`tools.rs`), no geometry here.

use klappe_services::{
    read_number, Arrival, Departure, Field, Pace, PlanEdit, Pose, Range, Source, Unit,
};

use crate::tools::Typed;

/// The mark of a figure, as a selection names it: itself, or the figure for its mark at the
/// playhead.
fn is_figure_mark(selected: &Source) -> bool {
    matches!(selected, Source::Mark { .. } | Source::Figure(_))
}

/// The pose `name` for the selected figure mark, as an edit. `None` for a selection that is
/// no figure mark or a name that is no pose.
pub fn pose_edit(selected: Option<&Source>, name: &str) -> Option<PlanEdit> {
    let source = selected.filter(|s| is_figure_mark(s))?;
    let pose = Pose::from_name(name)?;

    Some(PlanEdit::SetPose {
        source: source.clone(),
        pose,
    })
}

/// The arrival `name` for the selected figure mark, as an edit. `None` for a selection that
/// is no figure mark or a name that is no arrival.
pub fn arrival_edit(selected: Option<&Source>, name: &str) -> Option<PlanEdit> {
    let source = selected.filter(|s| is_figure_mark(s))?;
    let arrival = Arrival::from_name(name)?;

    Some(PlanEdit::SetArrival {
        source: source.clone(),
        arrival,
    })
}

/// The departure `name` for the selected figure mark, as an edit. `None` for a selection
/// that is no figure mark or a name that is no departure.
pub fn departure_edit(selected: Option<&Source>, name: &str) -> Option<PlanEdit> {
    let source = selected.filter(|s| is_figure_mark(s))?;
    let departure = Departure::from_name(name)?;

    Some(PlanEdit::SetDeparture {
        source: source.clone(),
        departure,
    })
}

/// The pause typed for the selected mark (figure or camera), as an edit.
pub fn hold_edit(selected: Option<&Source>, text: &str) -> Typed {
    let source = selected.filter(|s| is_figure_mark(s) || matches!(s, Source::Camera(_)))?;
    let hold_s = read_number(Field::HoldS, text, Unit::Second, Range::NotNegative);

    Some(hold_s.map(|hold_s| PlanEdit::SetHold {
        source: source.clone(),
        hold_s,
    }))
}

/// The lens height and the tilt typed for the selected camera mark, as an edit (LH-34). The
/// session checks the limits.
pub fn aim_edit(selected: Option<&Source>, height: &str, tilt: &str) -> Typed {
    let source = selected.filter(|s| matches!(s, Source::Camera(_)))?;
    let height_m = read_number(Field::Height, height, Unit::Metre, Range::Any);
    let aimed = height_m.and_then(|height_m| {
        Ok(PlanEdit::SetCameraAim {
            source: source.clone(),
            height_m,
            tilt_deg: read_number(Field::Tilt, tilt, Unit::Degree, Range::Any)?,
        })
    });

    Some(aimed)
}

/// The pace `name` for the way to the selected figure mark, as an edit. `None` for a
/// selection that is no figure mark or a name that is no pace.
pub fn pace_edit(selected: Option<&Source>, name: &str) -> Option<PlanEdit> {
    let source = selected.filter(|s| is_figure_mark(s))?;
    let pace = Pace::from_name(name)?;

    Some(PlanEdit::SetPace {
        source: source.clone(),
        pace,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::edit_of;

    #[test]
    fn a_pose_name_for_a_figure_mark_is_a_set_pose() {
        let mark = Source::Mark {
            character: "elisabeth".into(),
            index: 1,
        };

        assert_eq!(
            pose_edit(Some(&mark), "sitting"),
            Some(PlanEdit::SetPose {
                source: mark.clone(),
                pose: Pose::Sitting
            })
        );
        assert!(pose_edit(Some(&Source::Figure("elisabeth".into())), "lying").is_some());
        assert_eq!(pose_edit(Some(&mark), "flying"), None);
        assert_eq!(pose_edit(Some(&Source::Camera("a".into())), "lying"), None);
        assert_eq!(pose_edit(None, "lying"), None);
    }

    #[test]
    fn arrival_and_departure_names_for_a_figure_mark_are_edits() {
        let mark = Source::Mark {
            character: "elisabeth".into(),
            index: 1,
        };

        assert_eq!(
            arrival_edit(Some(&mark), "appear"),
            Some(PlanEdit::SetArrival {
                source: mark.clone(),
                arrival: Arrival::Appear
            })
        );
        assert_eq!(
            departure_edit(Some(&mark), "leave"),
            Some(PlanEdit::SetDeparture {
                source: mark.clone(),
                departure: Departure::Leave
            })
        );
        assert!(arrival_edit(Some(&Source::Figure("elisabeth".into())), "walk").is_some());
        assert_eq!(arrival_edit(Some(&mark), "fly"), None);
        assert_eq!(departure_edit(Some(&mark), "fly"), None);
        assert_eq!(
            arrival_edit(Some(&Source::Camera("a".into())), "walk"),
            None
        );
        assert_eq!(departure_edit(None, "stay"), None);
        assert_eq!(Arrival::Appear.name(), "appear");
        assert_eq!(Departure::Leave.name(), "leave");
    }

    #[test]
    fn a_pause_typed_for_a_mark_is_a_set_hold() {
        let camera = Source::Camera("a".into());

        assert_eq!(
            edit_of(hold_edit(Some(&camera), " 2,5 ")),
            Some(PlanEdit::SetHold {
                source: camera.clone(),
                hold_s: 2.5
            })
        );
        assert!(edit_of(hold_edit(Some(&camera), "0")).is_some());
        assert_eq!(edit_of(hold_edit(Some(&camera), "-1")), None);
        assert_eq!(edit_of(hold_edit(Some(&camera), "lang")), None);
        assert_eq!(edit_of(hold_edit(Some(&camera), "")), None);
        assert_eq!(edit_of(hold_edit(Some(&Source::Wall(0)), "1")), None);
        assert_eq!(edit_of(hold_edit(None, "1")), None);
    }

    #[test]
    fn a_height_and_tilt_typed_for_a_camera_mark_are_a_set_camera_aim() {
        let camera = Source::Camera("a".into());

        assert_eq!(
            edit_of(aim_edit(Some(&camera), " 2,4 ", "-15.5")),
            Some(PlanEdit::SetCameraAim {
                source: camera.clone(),
                height_m: 2.4,
                tilt_deg: -15.5
            })
        );
        assert!(edit_of(aim_edit(Some(&camera), "0", "0")).is_some());
        assert_eq!(edit_of(aim_edit(Some(&camera), "", "0")), None);
        assert_eq!(edit_of(aim_edit(Some(&camera), "1", "")), None);
        assert_eq!(edit_of(aim_edit(Some(&camera), "hoch", "0")), None);
        assert_eq!(edit_of(aim_edit(Some(&camera), "1", "NaN")), None);
        assert_eq!(edit_of(aim_edit(Some(&camera), "inf", "0")), None);
        let figure = Source::Figure("elisabeth".into());
        assert_eq!(edit_of(aim_edit(Some(&figure), "1", "0")), None);
        assert_eq!(edit_of(aim_edit(None, "1", "0")), None);
    }

    #[test]
    fn a_pace_name_for_a_figure_mark_is_a_set_pace() {
        let mark = Source::Mark {
            character: "elisabeth".into(),
            index: 1,
        };

        assert_eq!(
            pace_edit(Some(&mark), "run"),
            Some(PlanEdit::SetPace {
                source: mark.clone(),
                pace: Pace::Run
            })
        );
        assert!(pace_edit(Some(&mark), "walk").is_some());
        assert_eq!(pace_edit(Some(&mark), "crawl"), None);
        assert_eq!(pace_edit(Some(&Source::Camera("a".into())), "run"), None);
        assert_eq!(pace_edit(None, "run"), None);
    }
}
