//! Playback: the time on the timeline and the play head.

use super::qobject;
use crate::tools::Preview;
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use klappe_services::{Looping, Playing, Step};

const LOOP_SHOT: &str = "shot";

impl qobject::Controller {
    pub(super) fn seek(mut self: Pin<&mut Self>, time: f64) {
        let Some(s) = self.as_mut().session() else {
            return;
        };
        s.set_time(time);
        self.refresh();
    }

    pub(super) fn step_mark(mut self: Pin<&mut Self>, direction: i32) {
        let Some(s) = self.as_mut().session() else {
            return;
        };
        let step = match direction < 0 {
            true => Step::Back,
            false => Step::On,
        };
        s.step_mark(step);
        self.refresh();
    }

    pub(super) fn advance(
        mut self: Pin<&mut Self>,
        dt: f64,
        speed: f64,
        looping: &QString,
    ) -> bool {
        let Some(s) = self.as_mut().session() else {
            return false;
        };
        let looping = match looping.to_string() == LOOP_SHOT {
            true => Looping::Shot,
            false => Looping::Off,
        };
        let tick = s.advance(dt, speed, looping);
        let playing = tick.playing;
        let time = s.time_s();

        if let Some(to_s) = tick.looped_to_s {
            self.as_mut().looped(to_s);
        }

        // A refresh would drop a drag preview: show the drag again at the new time, as the
        // release will store it.
        let drag = self
            .rust()
            .drag_step
            .filter(|_| self.rust().preview == Preview::Shown);
        if let Some([from_x, from_y, x, y]) = drag {
            self.as_mut().set_time(time);
            self.drag(from_x, from_y, x, y, false);
            return playing == Playing::Running;
        }
        self.refresh();
        playing == Playing::Running
    }

    pub(super) fn speech_json(&self) -> QString {
        let rust = self.rust();
        let speech = rust.session.as_ref().map(|s| s.speech(rust.language));

        QString::from(super::view::to_json(&speech).as_str())
    }
}
