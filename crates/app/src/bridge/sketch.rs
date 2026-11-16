//! The sketch pen (LH-120): a drag collects the pointer's places, its release commits them as
//! one stroke.

use super::qobject;
use crate::plan::Sweep;
use core::pin::Pin;
use cxx_qt::CxxQtType;
use klappe_services::Point;

impl qobject::Controller {
    pub(super) fn sketch_drag(
        mut self: Pin<&mut Self>,
        from_x: f64,
        from_y: f64,
        x: f64,
        y: f64,
        done: bool,
    ) -> bool {
        let state = self.as_mut().rust_mut().get_mut();
        if state.sketch.is_empty() {
            state.sketch.push(Point::new(from_x, from_y));
        }
        state.sketch.push(Point::new(x, y));

        // The pen draws as it goes: the stroke so far shows as the lasso does.
        if !done {
            let so_far = Sweep::Lasso(state.sketch.clone());
            self.as_mut().show_sweep(&so_far);
            return false;
        }

        let state = self.as_mut().rust_mut().get_mut();
        let stroke = std::mem::take(&mut state.sketch);
        state.marquee = None;
        let result = self.as_mut().session().map(|s| s.add_sketch(&stroke));
        self.report(result)
    }

    pub(super) fn clear_sketches(mut self: Pin<&mut Self>) -> bool {
        let result = self.as_mut().session().map(|s| s.clear_sketches());
        self.report(result)
    }
}
