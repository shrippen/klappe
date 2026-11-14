//! Measuring and the scale: background image, calibration, the tape measure.

use super::qobject;
use crate::fileurl;
use crate::tools;
use crate::tools::{MeasureStep, ScaleStep, Tool};
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use klappe_services::{PlanEdit, Point, Session};

impl qobject::Controller {
    pub(super) fn set_background(mut self: Pin<&mut Self>, path: &QString) -> bool {
        let source = fileurl::to_path(&path.to_string());
        if !self
            .as_mut()
            .commit_edit(PlanEdit::SetBackground { source })
        {
            return false;
        }

        // The edit cleared the status line; the next step goes there.
        let hint = self.rust().language.text("background.hint");
        self.as_mut().say(hint);
        true
    }

    pub(super) fn remove_background(mut self: Pin<&mut Self>) -> bool {
        self.as_mut().rust_mut().get_mut().scale = ScaleStep::Idle;
        self.commit_edit(PlanEdit::RemoveBackground)
    }

    pub(super) fn calibrate(mut self: Pin<&mut Self>, distance: &QString) -> bool {
        let step = self.rust().scale;
        let edit = match tools::scale_edit(step, &distance.to_string()) {
            Some(Ok(edit)) => edit,
            typed => return self.commit_typed(typed),
        };

        self.as_mut().rust_mut().get_mut().scale = ScaleStep::Idle;
        self.commit_edit(edit)
    }

    pub(super) fn hover(mut self: Pin<&mut Self>, x: f64, y: f64) -> bool {
        if self.rust().tool != Tool::Measure {
            return false;
        }

        let Some(s) = self.as_mut().session() else {
            return false;
        };
        let at = s.snap(Point::new(x, y));
        let step = self.rust().measure;
        let live = step.hover(at);
        if live == step {
            return false;
        }

        self.as_mut().rust_mut().get_mut().measure = live;
        self.show_plan();
        true
    }

    pub(super) fn cancel_measure(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().get_mut().measure = MeasureStep::Idle;
        self.refresh();
    }

    pub(super) fn cancel_scale(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().get_mut().scale = ScaleStep::Idle;
        self.refresh();
    }

    /// A click of the measuring tool: the first or the second point, or a new start.
    pub(super) fn click_measure(mut self: Pin<&mut Self>, at: Point) -> bool {
        let step = self.rust().measure.click(at);
        self.as_mut().rust_mut().get_mut().measure = step;
        self.refresh();
        true
    }

    /// A click of the scale tool: the next point of the stretch. Without a background there
    /// is nothing to scale.
    pub(super) fn pick_scale_point(mut self: Pin<&mut Self>, at: Point) -> bool {
        if !self
            .rust()
            .session
            .as_ref()
            .is_some_and(Session::has_background)
        {
            return false;
        }

        let step = self.rust().scale.click(at);
        self.as_mut().rust_mut().get_mut().scale = step;
        self.refresh();
        true
    }
}
