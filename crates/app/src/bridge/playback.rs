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
        let playing = s.advance(dt, speed, looping);
        let time = s.time_s();

        // A refresh would drop a drag preview: the plan holds still, only the time moves.
        if self.rust().preview == Preview::Shown {
            self.set_time(time);
            return playing == Playing::Running;
        }
        self.refresh();
        playing == Playing::Running
    }
}
