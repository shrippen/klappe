//! Ways of marks: a mark before the first one, a whole way moved at once, a way's shape.
//! A way is the path to mark `index` of a figure or camera; the first mark has none.

use klappe_model::{CameraMark, Curve, Mark, Point};
use klappe_stellprobe::Source;

use super::{
    camera_parts, figure_parts, keep_bends, mark_held, normalize_deg, snap, Grid, Owner, Scope,
    Snapping,
};
use crate::error::ServiceError;
use crate::field::Field;
use crate::plan_edit::marks::WALK_SPEED_M_S;
use crate::plan_edit::PlanEdit;

/// Marks of both kinds, as a way sees them.
trait Waypoint {
    fn at_s(&mut self) -> &mut f64;
    fn place(&mut self) -> &mut Point;
    fn via(&mut self) -> &mut Vec<Point>;
    fn curve(&mut self) -> &mut Curve;
}

macro_rules! waypoint {
    ($mark:ty) => {
        impl Waypoint for $mark {
            fn at_s(&mut self) -> &mut f64 {
                &mut self.at_s
            }
            fn place(&mut self) -> &mut Point {
                &mut self.at
            }
            fn via(&mut self) -> &mut Vec<Point> {
                &mut self.via
            }
            fn curve(&mut self) -> &mut Curve {
                &mut self.curve
            }
        }
    };
}
waypoint!(Mark);
waypoint!(CameraMark);

pub(in crate::plan_edit) fn edit_way(
    scope: &mut Scope<'_>,
    edit: &PlanEdit,
    snap_to: (Snapping, Grid),
) -> Result<(), ServiceError> {
    match edit {
        PlanEdit::AddMarkBefore { source, at } => add_before(scope, source, *at),
        PlanEdit::ShiftWay { source, dx_m, dy_m } => {
            shift_way(scope, source, (*dx_m, *dy_m), snap_to)
        }
        PlanEdit::SetCurve { source, curve } => set_curve(scope, source, *curve),
        _ => Ok(()),
    }
}

/// The owner of the way `source` names and the index of the mark it leads to: a mark, a
/// figure or camera (its mark at the playhead), or a grip on the way.
fn way_to(scope: &Scope<'_>, source: &Source) -> Result<(Owner, usize), ServiceError> {
    match source {
        Source::FigureVia {
            character, index, ..
        } => Ok((Owner::Figure(character.clone()), *index)),
        Source::CameraVia { camera, index, .. } => Ok((Owner::Camera(camera.clone()), *index)),
        _ => mark_held(scope, source)
            .map(|(owner, index, _)| (owner, index))
            .ok_or_else(|| bad(Field::Source)),
    }
}

fn bad(field: Field) -> ServiceError {
    ServiceError::BadValue {
        field,
        value: String::new(),
    }
}

/// Sets the shape of the way to the mark.
fn set_curve(scope: &mut Scope<'_>, source: &Source, curve: Curve) -> Result<(), ServiceError> {
    let (owner, index) = way_to(scope, source)?;

    match owner {
        Owner::Figure(c) => {
            let track = scope.plan.figures.iter_mut().find(|f| f.character == c);
            curve_of(&mut track.ok_or(ServiceError::Gone)?.marks, index, curve)
        }
        Owner::Camera(id) => {
            let camera = scope.plan.cameras.iter_mut().find(|c| c.id == id);
            curve_of(&mut camera.ok_or(ServiceError::Gone)?.marks, index, curve)
        }
    }
}

fn curve_of<M: Waypoint>(marks: &mut [M], index: usize, curve: Curve) -> Result<(), ServiceError> {
    if index == 0 {
        return Err(bad(Field::Source));
    }

    *marks.get_mut(index).ok_or(ServiceError::Gone)?.curve() = curve;

    Ok(())
}

/// Moves the way to the mark by `by_m`: both its ends and its bends. With snapping on, the
/// end mark lands on the grid. The ways beyond the two ends keep their shape, as when a mark
/// is dragged alone.
fn shift_way(
    scope: &mut Scope<'_>,
    source: &Source,
    by_m: (f64, f64),
    snap_to: (Snapping, Grid),
) -> Result<(), ServiceError> {
    let (owner, index) = way_to(scope, source)?;

    match owner {
        Owner::Figure(c) => {
            let track = scope.plan.figures.iter_mut().find(|f| f.character == c);
            let marks = &mut track.ok_or(ServiceError::Gone)?.marks;
            shift_marks(marks, index, (by_m, snap_to), figure_parts)
        }
        Owner::Camera(id) => {
            let camera = scope.plan.cameras.iter_mut().find(|c| c.id == id);
            let marks = &mut camera.ok_or(ServiceError::Gone)?.marks;
            shift_marks(marks, index, (by_m, snap_to), camera_parts)
        }
    }
}

fn shift_marks<M>(
    marks: &mut [M],
    index: usize,
    (by_m, (snapping, grid)): ((f64, f64), (Snapping, Grid)),
    parts: for<'a> fn(&'a mut M) -> (&'a mut Point, &'a mut Vec<Point>),
) -> Result<(), ServiceError> {
    let before = index.checked_sub(1).ok_or_else(|| bad(Field::Source))?;
    if before >= marks.len() || index >= marks.len() {
        return Err(ServiceError::Gone);
    }

    let end = *parts(&mut marks[index]).0;
    let landed = snap(
        Point::new(end.x_m + by_m.0, end.y_m + by_m.1),
        snapping,
        grid,
    );
    let (dx, dy) = (landed.x_m - end.x_m, landed.y_m - end.y_m);
    if (dx, dy) == (0.0, 0.0) {
        return Ok(());
    }

    let moved = |p: Point| Point::new(p.x_m + dx, p.y_m + dy);
    let (old_start, old_end) = (*parts(&mut marks[before]).0, end);

    // The way's own bends and ends go along; its neighbours' bends follow the moved end.
    for i in [before, index] {
        let (at, via) = parts(&mut marks[i]);
        *at = moved(*at);
        if i == index {
            via.iter_mut().for_each(|v| *v = moved(*v));
        }
    }

    // Undo what `keep_bends` would do to the way itself: only the outer ways follow.
    let own = std::mem::take(parts(&mut marks[index]).1);
    keep_bends(marks, before, old_start, parts);
    keep_bends(marks, index, old_end, parts);
    *parts(&mut marks[index]).1 = own;

    Ok(())
}

/// Sets a new mark at `at` before the first mark of the figure or camera: where it comes
/// from. The old marks move later by the way from the new mark to the old first one, so the
/// old first mark keeps its pause and every later way keeps its duration; the new mark takes
/// the old first mark's time.
fn add_before(scope: &mut Scope<'_>, source: &Source, at: Point) -> Result<(), ServiceError> {
    let (owner, _) = way_to(scope, source)?;

    match owner {
        Owner::Figure(c) => {
            let track = scope.plan.figures.iter_mut().find(|f| f.character == c);
            let marks = &mut track.ok_or(ServiceError::Gone)?.marks;
            let first = marks.first_mut().ok_or(ServiceError::Gone)?;
            // A figure that appeared at the old first mark now appears at the new one.
            let arrival = std::mem::take(&mut first.arrival);
            let new = Mark {
                arrival,
                departure: Default::default(),
                at,
                facing_deg: normalize_deg(at.bearing_deg(first.at)),
                hold_s: 0.0,
                via: Vec::new(),
                curve: Curve::Smooth,
                ..first.clone()
            };
            insert_first(marks, new);
        }
        Owner::Camera(id) => {
            let camera = scope.plan.cameras.iter_mut().find(|c| c.id == id);
            let marks = &mut camera.ok_or(ServiceError::Gone)?.marks;
            let first = marks.first_mut().ok_or(ServiceError::Gone)?;
            // The rig did not count on the first mark; now it is the way to it, by dolly.
            first.rig = Default::default();
            let new = CameraMark {
                at,
                hold_s: 0.0,
                via: Vec::new(),
                curve: Curve::Smooth,
                rig: Default::default(),
                ..first.clone()
            };
            insert_first(marks, new);
        }
    }

    Ok(())
}

fn insert_first<M: Waypoint + Clone>(marks: &mut Vec<M>, mut new: M) {
    let walk_s = marks[0].place().distance_m(*new.place()) / WALK_SPEED_M_S;
    *new.at_s() = *marks[0].at_s();

    // What was ignored on the first mark counts now, and must not surprise.
    marks[0].via().clear();
    *marks[0].curve() = Curve::Smooth;
    for mark in marks.iter_mut() {
        *mark.at_s() += walk_s;
    }

    marks.insert(0, new);
}
