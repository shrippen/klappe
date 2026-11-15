//! Dragging marks and shot edges on the timeline (LH-61).

use super::qobject;
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use klappe_services::{Grab, TimeSnap};

const FIGURE_MARK: &str = "figure";
const CAMERA_MARK: &str = "camera";
const SHOT_FROM: &str = "shot-from";
const SHOT_TO: &str = "shot-to";

impl qobject::Controller {
    pub(super) fn drag_time(&self, grab: &QString, seconds: f64, snap: &QString) -> f64 {
        let (Some(grab), Some(snap)) = (grab_of(grab), snap_of(snap)) else {
            return seconds;
        };
        let Some(s) = self.rust().session.as_ref() else {
            return seconds;
        };

        s.drag_time(&grab, seconds, snap).unwrap_or(seconds)
    }

    pub(super) fn drop_time(
        mut self: Pin<&mut Self>,
        grab: &QString,
        seconds: f64,
        snap: &QString,
    ) -> bool {
        let (Some(grab), Some(snap)) = (grab_of(grab), snap_of(snap)) else {
            return false;
        };
        let Some(s) = self.as_mut().session() else {
            return false;
        };
        let result = s.drop_time(&grab, seconds, snap);

        self.report(Some(result))
    }
}

/// What the timeline holds, as `kind:index:id`: `figure` or `camera` (mark number `index` of
/// track `id`), `shot-from` or `shot-to` (an edge of shot `id`, any index). E.g. `figure:1:anna`.
fn grab_of(text: &QString) -> Option<Grab> {
    let text = text.to_string();
    let mut parts = text.splitn(3, ':');
    let (kind, index, id) = (parts.next()?, parts.next()?, parts.next()?.to_owned());
    let index = index.parse::<usize>().ok()?;

    match kind {
        FIGURE_MARK => Some(Grab::FigureMark {
            character: id,
            index,
        }),
        CAMERA_MARK => Some(Grab::CameraMark { camera: id, index }),
        SHOT_FROM => Some(Grab::ShotFrom(id)),
        SHOT_TO => Some(Grab::ShotTo(id)),
        _ => None,
    }
}

fn snap_of(text: &QString) -> Option<TimeSnap> {
    TimeSnap::from_name(&text.to_string())
}
