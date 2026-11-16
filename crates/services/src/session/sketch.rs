//! The sketch pen (LH-120): freehand strokes over the plan, on a layer of their own. They
//! are part of the plan, saved and printed with it; one command clears them all.

use klappe_model::{Point, Sketch};

use super::{Layer, Lock, Session};
use crate::error::ServiceError;
use crate::field::Field;

/// A stroke keeps a point only when it is this far from the one kept before: a slow hand
/// would otherwise write hundreds of points for a short line.
const MIN_STEP_M: f64 = 0.03;
/// A stroke needs a start and somewhere it went.
const MIN_POINTS: usize = 2;

impl Session {
    /// Adds one stroke of the pen to the plan on view; undoable. Refused on a locked layer,
    /// and for a stroke that went nowhere or has a point that is no number.
    pub fn add_sketch(&mut self, points: &[Point]) -> Result<(), ServiceError> {
        self.check_sketch_lock()?;

        let points = thinned(points);
        let finite = points
            .iter()
            .all(|p| p.x_m.is_finite() && p.y_m.is_finite());
        if points.len() < MIN_POINTS || !finite {
            return Err(ServiceError::BadValue {
                field: Field::Drawing,
                value: "empty".to_owned(),
            });
        }

        let mut next = self.project.clone();
        self.plan_in(&mut next)?.sketches.push(Sketch { points });
        self.commit_step(next, "AddSketch");

        Ok(())
    }

    /// Takes every stroke off the plan on view; one undo step, none when there are none.
    pub fn clear_sketches(&mut self) -> Result<(), ServiceError> {
        self.check_sketch_lock()?;

        let mut next = self.project.clone();
        self.plan_in(&mut next)?.sketches.clear();
        if next == self.project {
            return Ok(());
        }

        self.commit_step(next, "ClearSketches");

        Ok(())
    }

    fn check_sketch_lock(&self) -> Result<(), ServiceError> {
        match self.lock(Layer::Sketches) {
            Lock::Locked => Err(ServiceError::Locked(Layer::Sketches)),
            Lock::Unlocked => Ok(()),
        }
    }
}

/// `points` without those closer than `MIN_STEP_M` to the one before; the last one stays, so
/// the stroke ends where the pen did.
fn thinned(points: &[Point]) -> Vec<Point> {
    let mut kept: Vec<Point> = Vec::new();

    for (i, point) in points.iter().enumerate() {
        let last = i + 1 == points.len();
        let far = kept
            .last()
            .is_none_or(|k| k.distance_m(*point) >= MIN_STEP_M);
        if far {
            kept.push(*point);
            continue;
        }
        if last && kept.len() > 1 {
            // Only ever the tail of a slow stroke: it replaces the point kept just before.
            kept.pop();
            kept.push(*point);
        }
    }

    kept
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn close_points_are_dropped_and_the_end_is_kept() {
        let line: Vec<Point> = (0..=100)
            .map(|i| Point::new(f64::from(i) * 0.001, 0.0))
            .collect();

        let kept = thinned(&line);

        assert_eq!(kept.first(), line.first());
        assert_eq!(kept.last(), line.last());
        assert!(kept.len() < 10, "{}", kept.len());
    }
}
