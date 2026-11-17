//! Moving marks and shot edges in time by dragging them on the timeline (LH-61).

use klappe_model::FloorPlan;

use super::{Layer, Lock, Session};
use crate::error::ServiceError;
use crate::field::Field;

/// Snapping to tenths of a second, the step of the timeline's readout.
const TENTHS_PER_S: f64 = 10.0;

/// A shot keeps at least this length: its edges do not meet.
const MIN_SHOT_S: f64 = 0.1;

/// What a drag on the timeline holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Grab {
    /// The arrival of mark `index` of a character's track.
    FigureMark { character: String, index: usize },
    /// The arrival of mark `index` of a camera.
    CameraMark { camera: String, index: usize },
    /// The start of shot `id`.
    ShotFrom(String),
    /// The end of shot `id`.
    ShotTo(String),
}

impl Grab {
    /// The layer a drag changes; a locked one refuses it.
    fn layer(&self) -> Layer {
        match self {
            Grab::FigureMark { .. } => Layer::Figures,
            Grab::CameraMark { .. } | Grab::ShotFrom(_) | Grab::ShotTo(_) => Layer::Cameras,
        }
    }
}

/// Whether a dragged time snaps to tenths of a second.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeSnap {
    Tenths,
    Free,
}

impl TimeSnap {
    /// The snap `name` (`tenths`, `free`) names; `None` for any other text.
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "tenths" => Some(TimeSnap::Tenths),
            "free" => Some(TimeSnap::Free),
            _ => None,
        }
    }
}

impl Session {
    /// The time a drag of `grab` to `t_s` ends at: snapped, and kept between the neighbours
    /// (marks keep their order, a shot its edges' order). The timeline shows it while the
    /// button is down.
    pub fn drag_time(&self, grab: &Grab, t_s: f64, snap: TimeSnap) -> Result<f64, ServiceError> {
        let plan = self.plan().ok_or(ServiceError::Gone)?;
        let (min_s, max_s) = limits(plan, grab)?;
        let snapped = match snap {
            TimeSnap::Tenths => (t_s * TENTHS_PER_S).round() / TENTHS_PER_S,
            TimeSnap::Free => t_s,
        };

        Ok(snapped.max(min_s).min(max_s))
    }

    /// Ends a drag: moves what `grab` holds to `t_s` as `drag_time` shows it; one undo step.
    /// Refused while the layer is locked.
    pub fn drop_time(&mut self, grab: &Grab, t_s: f64, snap: TimeSnap) -> Result<(), ServiceError> {
        if self.lock(grab.layer()) == Lock::Locked {
            return Err(ServiceError::Locked(grab.layer()));
        }

        let t_s = self.drag_time(grab, t_s, snap)?;
        let mut next = self.project.clone();
        set(self.plan_in(&mut next)?, grab, t_s)?;
        if next == self.project {
            return Ok(());
        }

        self.commit_step(next, "Retime");
        Ok(())
    }

    /// The keyboard's way to move what `grab` holds: `delta_s` later (negative: earlier) from
    /// where it is, snapped to the tenth and kept between its neighbours; one undo step.
    pub fn nudge_time(&mut self, grab: &Grab, delta_s: f64) -> Result<(), ServiceError> {
        let now_s = time_of(self.plan().ok_or(ServiceError::Gone)?, grab)?;

        self.drop_time(grab, now_s + delta_s, TimeSnap::Tenths)
    }
}

/// The time `grab` holds now.
fn time_of(plan: &FloorPlan, grab: &Grab) -> Result<f64, ServiceError> {
    let time_s = match grab {
        Grab::FigureMark { character, index } => plan
            .figure(character)
            .and_then(|f| f.marks.get(*index))
            .map(|m| m.at_s),
        Grab::CameraMark { camera, index } => plan
            .camera(camera)
            .and_then(|c| c.marks.get(*index))
            .map(|m| m.at_s),
        Grab::ShotFrom(id) => Some(shot(plan, id)?.from_s),
        Grab::ShotTo(id) => Some(shot(plan, id)?.to_s),
    };

    time_s.ok_or_else(|| bad_grab(grab))
}

/// The earliest and latest time `grab` may be moved to. A mark lies between the marks around
/// it, the last one up to the scene's end; a shot's start between 0 and its end, its end
/// between its start and the scene's end.
fn limits(plan: &FloorPlan, grab: &Grab) -> Result<(f64, f64), ServiceError> {
    match grab {
        Grab::FigureMark { character, index } => {
            let track = plan.figure(character).ok_or(ServiceError::Gone)?;

            around(
                track.marks.iter().map(|m| m.at_s),
                *index,
                plan.duration_s(),
            )
        }
        Grab::CameraMark { camera, index } => {
            let camera = plan.camera(camera).ok_or(ServiceError::Gone)?;

            around(
                camera.marks.iter().map(|m| m.at_s),
                *index,
                plan.duration_s(),
            )
        }
        Grab::ShotFrom(id) => Ok((0.0, shot(plan, id)?.to_s - MIN_SHOT_S)),
        Grab::ShotTo(id) => Ok((shot(plan, id)?.from_s + MIN_SHOT_S, plan.duration_s())),
    }
}

/// The times of the marks before and after mark `index`; 0 and `end_s` where there is none.
fn around(
    times_s: impl Iterator<Item = f64>,
    index: usize,
    end_s: f64,
) -> Result<(f64, f64), ServiceError> {
    let times: Vec<f64> = times_s.collect();
    if index >= times.len() {
        return Err(ServiceError::Gone);
    }

    let before = index.checked_sub(1).map_or(0.0, |i| times[i]);
    let after = times.get(index + 1).copied().unwrap_or(end_s);

    Ok((before, after))
}

fn shot<'a>(plan: &'a FloorPlan, id: &str) -> Result<&'a klappe_model::Shot, ServiceError> {
    plan.shots
        .iter()
        .find(|s| s.id == id)
        .ok_or_else(|| ServiceError::NoShot(id.to_owned()))
}

fn set(plan: &mut FloorPlan, grab: &Grab, t_s: f64) -> Result<(), ServiceError> {
    let slot = match grab {
        Grab::FigureMark { character, index } => plan
            .figures
            .iter_mut()
            .find(|f| f.character == *character)
            .and_then(|f| f.marks.get_mut(*index))
            .map(|m| &mut m.at_s),
        Grab::CameraMark { camera, index } => plan
            .cameras
            .iter_mut()
            .find(|c| c.id == *camera)
            .and_then(|c| c.marks.get_mut(*index))
            .map(|m| &mut m.at_s),
        Grab::ShotFrom(id) => plan
            .shots
            .iter_mut()
            .find(|s| s.id == *id)
            .map(|s| &mut s.from_s),
        Grab::ShotTo(id) => plan
            .shots
            .iter_mut()
            .find(|s| s.id == *id)
            .map(|s| &mut s.to_s),
    };

    *slot.ok_or_else(|| bad_grab(grab))? = t_s;
    Ok(())
}

fn bad_grab(grab: &Grab) -> ServiceError {
    ServiceError::BadValue {
        field: Field::Source,
        value: format!("{grab:?}"),
    }
}
