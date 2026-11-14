//! Times of a figure's marks: pose, arrival, departure, hold and the pace between marks.

use klappe_model::{Arrival, Departure, FigureTrack, FloorPlan, Mark, Point, Pose};
use klappe_stellprobe::{walked_ways, way_length_m, Source};

use super::{bad_mark, current_index, figure_mark, Scope};
use crate::error::ServiceError;
use crate::field::Field;

/// How fast a figure goes along a way (LH-63).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pace {
    Walk,
    Run,
}

impl Pace {
    /// Every pace.
    pub const ALL: [Pace; 2] = [Pace::Walk, Pace::Run];

    /// The name QML and the catalogue (`pace.<name>`) use, e.g. `run`.
    pub fn name(self) -> &'static str {
        match self {
            Pace::Walk => "walk",
            Pace::Run => "run",
        }
    }

    /// The pace `name` names; `None` for any other text.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|p| p.name() == name)
    }

    fn speed_m_s(self) -> f64 {
        match self {
            Pace::Walk => WALK_SPEED_M_S,
            Pace::Run => RUN_SPEED_M_S,
        }
    }
}

/// What the plan says about a mark, for the fields that change it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MarkInfo {
    /// `None` for a camera's mark.
    pub pose: Option<Pose>,
    /// `None` for a camera's mark.
    pub arrival: Option<Arrival>,
    /// `None` for a camera's mark.
    pub departure: Option<Departure>,
    pub hold_s: f64,
    /// Lens height; `None` for a figure's mark (LH-34).
    pub height_m: Option<f64>,
    /// Tilt up from the horizon; `None` for a figure's mark (LH-34).
    pub tilt_deg: Option<f64>,
    /// The figure walks a way to the mark: not the first of its track, not one it appears at or
    /// is gone at after leaving (LH-26).
    pub has_way: bool,
    /// Speed along that way; `None` when it takes no time.
    pub pace_m_s: Option<f64>,
}

/// Walking speed that sets the time of a new mark: an easy pace on set, 5 km/h.
pub(super) const WALK_SPEED_M_S: f64 = 1.4;

/// Running speed for `Pace::Run`: 14.4 km/h, a jog to a sprint's lower end.
const RUN_SPEED_M_S: f64 = 4.0;

pub(super) fn set_pose(
    scope: &mut Scope<'_>,
    source: &Source,
    pose: Pose,
) -> Result<(), ServiceError> {
    let (character, index) =
        figure_mark(scope, source).ok_or_else(|| bad_mark(Field::Source, ""))?;
    let mark = scope
        .plan
        .figures
        .iter_mut()
        .find(|f| f.character == character)
        .and_then(|f| f.marks.get_mut(index))
        .ok_or(ServiceError::Gone)?;

    mark.pose = pose;

    Ok(())
}

pub(super) fn set_arrival(
    scope: &mut Scope<'_>,
    source: &Source,
    arrival: Arrival,
) -> Result<(), ServiceError> {
    figure_mark_mut(scope, source)?.arrival = arrival;

    Ok(())
}

pub(super) fn set_departure(
    scope: &mut Scope<'_>,
    source: &Source,
    departure: Departure,
) -> Result<(), ServiceError> {
    figure_mark_mut(scope, source)?.departure = departure;

    Ok(())
}

/// The figure mark `source` names.
fn figure_mark_mut<'a>(
    scope: &'a mut Scope<'_>,
    source: &Source,
) -> Result<&'a mut Mark, ServiceError> {
    let (character, index) =
        figure_mark(scope, source).ok_or_else(|| bad_mark(Field::Source, ""))?;

    scope
        .plan
        .figures
        .iter_mut()
        .find(|f| f.character == character)
        .and_then(|f| f.marks.get_mut(index))
        .ok_or(ServiceError::Gone)
}

/// The arrival and the pause of the mark `source` names, with those of the marks after it in
/// the same track.
struct Timing<'a> {
    /// Per mark of the track: `(at_s, hold_s)`.
    marks: Vec<(&'a mut f64, &'a mut f64)>,
    index: usize,
}

impl Timing<'_> {
    /// Moves the marks from `first` on by `delta_s`.
    fn shift_from(&mut self, first: usize, delta_s: f64) {
        for (at_s, _) in self.marks.iter_mut().skip(first) {
            **at_s += delta_s;
        }
    }
}

fn timing<'a>(scope: &'a mut Scope<'_>, source: &Source) -> Result<Timing<'a>, ServiceError> {
    let time_s = scope.time_s;
    let (index, marks) = match source {
        Source::Mark { character, index } => {
            let track = scope
                .plan
                .figures
                .iter_mut()
                .find(|f| f.character == *character)
                .ok_or(ServiceError::Gone)?;
            let marks = track.marks.iter_mut();

            (
                Some(*index),
                marks
                    .map(|m| (&mut m.at_s, &mut m.hold_s))
                    .collect::<Vec<_>>(),
            )
        }
        Source::Figure(character) => {
            let track = scope
                .plan
                .figures
                .iter_mut()
                .find(|f| f.character == *character)
                .ok_or(ServiceError::Gone)?;
            let index = current_index(track.marks.iter().map(|m| m.at_s), time_s);
            let marks = track.marks.iter_mut();

            (index, marks.map(|m| (&mut m.at_s, &mut m.hold_s)).collect())
        }
        Source::Camera(id) => {
            let camera = scope
                .plan
                .cameras
                .iter_mut()
                .find(|c| c.id == *id)
                .ok_or(ServiceError::Gone)?;
            let index = current_index(camera.marks.iter().map(|m| m.at_s), time_s);
            let marks = camera.marks.iter_mut();

            (index, marks.map(|m| (&mut m.at_s, &mut m.hold_s)).collect())
        }
        _ => return Err(bad_mark(Field::Source, "")),
    };
    let index = index
        .filter(|i| *i < marks.len())
        .ok_or(ServiceError::Gone)?;

    Ok(Timing { marks, index })
}

pub(super) fn set_hold(
    scope: &mut Scope<'_>,
    source: &Source,
    hold_s: f64,
) -> Result<(), ServiceError> {
    let mut timing = timing(scope, source)?;
    let index = timing.index;
    let delta_s = hold_s - *timing.marks[index].1;

    *timing.marks[index].1 = hold_s;
    timing.shift_from(index + 1, delta_s);

    Ok(())
}

pub(super) fn set_pace(
    scope: &mut Scope<'_>,
    source: &Source,
    pace: Pace,
) -> Result<(), ServiceError> {
    let (character, index) =
        figure_mark(scope, source).ok_or_else(|| bad_mark(Field::Source, ""))?;
    let before = index
        .checked_sub(1)
        .ok_or_else(|| bad_mark(Field::Pace, "first mark"))?;
    let marks = &scope
        .plan
        .figure(&character)
        .ok_or(ServiceError::Gone)?
        .marks;
    let (Some(prev), Some(mark)) = (marks.get(before), marks.get(index)) else {
        return Err(ServiceError::Gone);
    };

    // A way the figure does not walk (it appears there, or left before) has no pace (LH-26).
    let track = scope.plan.figure(&character).ok_or(ServiceError::Gone)?;
    if !walked_ways(track).get(index).copied().unwrap_or(false) {
        return Err(bad_mark(Field::Pace, "no way"));
    }

    // Arrive after the pause at the mark before and the walk along the way.
    let walk_s = way_length_m(prev.at, mark.via, mark.at) / pace.speed_m_s();
    let delta_s = prev.at_s + prev.hold_s + walk_s - mark.at_s;

    timing(scope, source)?.shift_from(index, delta_s);

    Ok(())
}

/// What the fields of a mark show: `source` is a figure's mark, or a figure or camera for
/// its mark at the playhead. `None` for anything else.
pub(crate) fn mark_info(plan: &FloorPlan, time_s: f64, source: &Source) -> Option<MarkInfo> {
    match source {
        Source::Mark { character, index } => figure_info(plan.figure(character)?, *index),
        Source::Figure(character) => {
            let track = plan.figure(character)?;
            let index = current_index(track.marks.iter().map(|m| m.at_s), time_s)?;

            figure_info(track, index)
        }
        Source::Camera(id) => {
            let marks = &plan.camera(id)?.marks;
            let index = current_index(marks.iter().map(|m| m.at_s), time_s)?;
            let mark = marks.get(index)?;
            let pace_m_s = index
                .checked_sub(1)
                .and_then(|before| marks.get(before))
                .and_then(|prev| {
                    pace_to(
                        (prev.at, prev.at_s + prev.hold_s),
                        (mark.at, mark.via, mark.at_s),
                    )
                });

            Some(MarkInfo {
                pose: None,
                arrival: None,
                departure: None,
                hold_s: mark.hold_s,
                height_m: Some(mark.height_m),
                tilt_deg: Some(mark.tilt_deg),
                has_way: index > 0,
                pace_m_s,
            })
        }
        _ => None,
    }
}

fn figure_info(track: &FigureTrack, index: usize) -> Option<MarkInfo> {
    let mark = track.marks.get(index)?;
    let has_way = walked_ways(track).get(index).copied().unwrap_or(false);
    let pace_m_s = index
        .checked_sub(1)
        .filter(|_| has_way)
        .and_then(|before| track.marks.get(before))
        .and_then(|prev| {
            pace_to(
                (prev.at, prev.at_s + prev.hold_s),
                (mark.at, mark.via, mark.at_s),
            )
        });

    Some(MarkInfo {
        pose: Some(mark.pose),
        arrival: Some(mark.arrival),
        departure: Some(mark.departure),
        hold_s: mark.hold_s,
        height_m: None,
        tilt_deg: None,
        has_way,
        pace_m_s,
    })
}

/// Speed of the way from where and when a mark is left to the next mark's place, bend and
/// arrival; `None` when the way takes no time.
fn pace_to(leave: (Point, f64), arrive: (Point, Option<Point>, f64)) -> Option<f64> {
    let walk_s = arrive.2 - leave.1;
    if walk_s <= 0.0 {
        return None;
    }

    Some(way_length_m(leave.0, arrive.1, arrive.0) / walk_s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Language;

    #[test]
    fn pace_names_read_back() {
        for pace in Pace::ALL {
            assert_eq!(Pace::from_name(pace.name()), Some(pace));
        }
        assert_eq!(Pace::from_name("crawl"), None);
    }

    /// The mark card shows `pace.<name>` and sends the name back.
    #[test]
    fn every_pace_has_a_text_in_both_languages() {
        for pace in Pace::ALL {
            let key = format!("pace.{}", pace.name());
            for language in [Language::German, Language::English] {
                assert_ne!(language.text(&key), key);
            }
        }
    }
}
