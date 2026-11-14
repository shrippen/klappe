use klappe_model::FloorPlan;

use super::{Looping, Playing, Session, Step};

/// Two times closer than this (s) are the same mark.
const MARK_EPS_S: f64 = 1e-3;

impl Session {
    pub fn time_s(&self) -> f64 {
        self.time_s
    }

    pub fn duration_s(&self) -> f64 {
        self.plan().map(FloorPlan::duration_s).unwrap_or(0.0)
    }

    /// Moves the playhead, kept within the scene. NaN is ignored.
    pub fn set_time(&mut self, t_s: f64) {
        if t_s.is_nan() {
            return;
        }

        self.time_s = t_s.clamp(0.0, self.duration_s());
    }

    /// Every mark of every figure and camera and both edges of every shot, sorted, each once.
    pub fn marks_s(&self) -> Vec<f64> {
        let Some(plan) = self.plan() else {
            return Vec::new();
        };
        let figures = plan
            .figures
            .iter()
            .flat_map(|f| f.marks.iter().map(|m| m.at_s));
        let cameras = plan
            .cameras
            .iter()
            .flat_map(|c| c.marks.iter().map(|m| m.at_s));
        let edges = plan.shots.iter().flat_map(|s| [s.from_s, s.to_s]);
        let mut marks: Vec<f64> = figures.chain(cameras).chain(edges).collect();

        marks.sort_by(f64::total_cmp);
        marks.dedup_by(|b, a| *b - *a < MARK_EPS_S);
        marks
    }

    /// Moves the playhead to the mark before or after it; the start or the end of the scene
    /// when there is none.
    pub fn step_mark(&mut self, step: Step) {
        let marks = self.marks_s();
        let now_s = self.time_s;
        let target_s = match step {
            Step::Back => marks
                .iter()
                .rev()
                .find(|m| **m < now_s - MARK_EPS_S)
                .copied()
                .unwrap_or(0.0),
            Step::On => marks
                .iter()
                .find(|m| **m > now_s + MARK_EPS_S)
                .copied()
                .unwrap_or_else(|| self.duration_s()),
        };

        self.set_time(target_s);
    }

    /// Plays `dt_s` seconds of real time at `speed` (1.0 = real time) from the playhead.
    /// Without a loop it stops at the end of the scene; with one it stays in the selected
    /// shot (the scene when none is selected), also when the playhead began outside it.
    pub fn advance(&mut self, dt_s: f64, speed: f64, looping: Looping) -> Playing {
        let step_s = dt_s * speed;
        if !step_s.is_finite() || step_s <= 0.0 {
            return Playing::Running;
        }

        let next_s = self.time_s + step_s;
        if looping == Looping::Off {
            self.set_time(next_s);
            return match self.time_s >= self.duration_s() {
                true => Playing::Ended,
                false => Playing::Running,
            };
        }

        let (from_s, to_s) = self.loop_range_s();
        let span_s = to_s - from_s;
        if span_s <= 0.0 {
            return Playing::Ended;
        }

        // Outside the range the loop starts at its beginning.
        let inside = self.time_s >= from_s && self.time_s <= to_s;
        let start_s = if inside { next_s } else { from_s };
        self.set_time(from_s + (start_s - from_s).rem_euclid(span_s));
        Playing::Running
    }

    fn loop_range_s(&self) -> (f64, f64) {
        let shot = self
            .selected_shot
            .as_deref()
            .and_then(|id| self.plan()?.shots.iter().find(|s| s.id == id));

        match shot {
            Some(shot) => (shot.from_s, shot.to_s),
            None => (0.0, self.duration_s()),
        }
    }
}
