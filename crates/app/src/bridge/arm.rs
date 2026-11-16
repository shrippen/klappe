//! What a key or the command palette arms for the next click on the plan (LH-104, LH-105,
//! LH-109), and what is done to the selected way (LH-107, LH-108).

use super::q;
use super::qobject;
use crate::arm::{self, Armed};
use crate::way;
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use klappe_services::Point;

impl qobject::Controller {
    pub(super) fn arm(mut self: Pin<&mut Self>, what: &QString) -> bool {
        let Some(armed) = Armed::from_name(&what.to_string()) else {
            return false;
        };
        if !arm::can_arm(armed, self.rust().selected.only()) {
            return false;
        }

        self.as_mut().rust_mut().get_mut().arm = armed;
        self.refresh();
        true
    }

    pub(super) fn can_arm(&self, what: &QString) -> bool {
        Armed::from_name(&what.to_string())
            .is_some_and(|armed| arm::can_arm(armed, self.rust().selected.only()))
    }

    pub(super) fn cancel_arm(mut self: Pin<&mut Self>) {
        if self.rust().arm == Armed::Idle {
            return;
        }

        self.as_mut().rust_mut().get_mut().arm = Armed::Idle;
        self.refresh();
    }

    /// The click at (`x`, `y`) while something is armed. The axis waits for a click on a
    /// figure; any other click is the mark's place, snapped to the grid.
    pub(super) fn place_armed(mut self: Pin<&mut Self>, x: f64, y: f64) -> bool {
        let armed = self.rust().arm;
        let selected = self.rust().selected.clone();
        let Some(s) = self.as_mut().session() else {
            return false;
        };
        let at = Point::new(x, y);

        if armed == Armed::AxisTo {
            let pair = arm::axis_pair(selected.only(), s.pick(at).as_ref());
            let Some((a, b)) = pair else {
                return false;
            };

            self.as_mut().cancel_arm();
            return self.set_axis(&q(&a), &q(&b));
        }

        let snapped = s.snap(at);
        let Some(edit) = arm::click_edit(armed, selected.only(), snapped) else {
            return false;
        };

        self.as_mut().cancel_arm();
        self.commit_edit(edit)
    }

    pub(super) fn nudge_way(self: Pin<&mut Self>, right: i32, down: i32) -> bool {
        let Some(s) = self.rust().session.as_ref() else {
            return false;
        };
        let step_m = s.grid().step_m();
        let Some(edit) = way::nudge_edit(self.rust().selected.only(), step_m, (right, down)) else {
            return false;
        };

        self.commit_edit(edit)
    }

    pub(super) fn way_curve(&self) -> QString {
        let curve = self
            .rust()
            .session
            .as_ref()
            .zip(self.rust().selected.only())
            .and_then(|(s, selected)| s.way_curve(selected));

        q(curve.map(|c| c.name()).unwrap_or_default())
    }

    pub(super) fn set_way_curve(self: Pin<&mut Self>, name: &QString) -> bool {
        let Some(edit) = way::curve_edit(self.rust().selected.only(), &name.to_string()) else {
            return false;
        };

        self.commit_edit(edit)
    }
}
