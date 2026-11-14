use klappe_model::{Drawing, DrawingShape, FloorPlan, Point, Wall};
use klappe_stellprobe::{bend_point, via_through, Source};

use super::{current_index, figure_mark, normalize_deg, snap, Scope, Snapping, Target};
use crate::error::ServiceError;

pub(super) fn target<'a>(
    scope: &'a mut Scope<'_>,
    source: &Source,
) -> Result<Target<'a>, ServiceError> {
    let time_s = scope.time_s;

    match source {
        Source::Mark { character, index } => {
            let mark = scope
                .plan
                .figures
                .iter_mut()
                .find(|f| f.character == *character)
                .and_then(|f| f.marks.get_mut(*index))
                .ok_or(ServiceError::Gone)?;
            Ok(Target::Place {
                at: &mut mark.at,
                facing_deg: &mut mark.facing_deg,
            })
        }
        Source::FigureVia { character, index } => {
            let track = scope
                .plan
                .figures
                .iter_mut()
                .find(|f| f.character == *character)
                .ok_or(ServiceError::Gone)?;
            curve(&mut track.marks, *index, |m| (m.at, &mut m.via))
        }
        Source::CameraVia { camera, index } => {
            let camera = scope
                .plan
                .cameras
                .iter_mut()
                .find(|c| c.id == *camera)
                .ok_or(ServiceError::Gone)?;
            curve(&mut camera.marks, *index, |m| (m.at, &mut m.via))
        }
        Source::Figure(character) => {
            let track = scope
                .plan
                .figures
                .iter_mut()
                .find(|f| f.character == *character)
                .ok_or(ServiceError::Gone)?;
            let index = current_index(track.marks.iter().map(|m| m.at_s), time_s)
                .ok_or(ServiceError::Gone)?;
            let mark = track.marks.get_mut(index).ok_or(ServiceError::Gone)?;
            Ok(Target::Place {
                at: &mut mark.at,
                facing_deg: &mut mark.facing_deg,
            })
        }
        Source::Camera(id) => {
            let camera = scope
                .plan
                .cameras
                .iter_mut()
                .find(|c| c.id == *id)
                .ok_or(ServiceError::Gone)?;
            let index = current_index(camera.marks.iter().map(|m| m.at_s), time_s)
                .ok_or(ServiceError::Gone)?;
            let mark = camera.marks.get_mut(index).ok_or(ServiceError::Gone)?;
            Ok(Target::Place {
                at: &mut mark.at,
                facing_deg: &mut mark.facing_deg,
            })
        }
        Source::Light(id) => {
            let light = scope
                .plan
                .lights
                .iter_mut()
                .find(|l| l.id == *id)
                .ok_or(ServiceError::Gone)?;
            Ok(Target::Place {
                at: &mut light.at,
                facing_deg: &mut light.facing_deg,
            })
        }
        Source::Object(id) => {
            let object = scope
                .set
                .as_deref_mut()
                .and_then(|s| s.objects.iter_mut().find(|o| o.id == *id))
                .ok_or(ServiceError::Gone)?;
            Ok(Target::Place {
                at: &mut object.at,
                facing_deg: &mut object.rotation_deg,
            })
        }
        Source::Wall(index) => {
            let wall = scope
                .set
                .as_deref_mut()
                .and_then(|s| s.walls.get_mut(*index))
                .ok_or(ServiceError::Gone)?;
            Ok(Target::Wall(wall))
        }
        Source::Note(id) => {
            let note = scope
                .set
                .as_deref_mut()
                .and_then(|s| s.notes.iter_mut().find(|n| n.id == *id))
                .ok_or(ServiceError::Gone)?;
            Ok(Target::Pin { at: &mut note.at })
        }
        Source::Drawing(id) => scope
            .plan
            .drawings
            .iter_mut()
            .find(|d| d.id == *id)
            .map(Target::Drawing)
            .ok_or(ServiceError::Gone),
    }
}

/// The bend of the way to mark `index` of `marks`; `place` gives a mark's place and its `via`.
/// The first mark has no way to it.
pub(super) fn curve<'a, M>(
    marks: &'a mut [M],
    index: usize,
    place: impl Fn(&'a mut M) -> (Point, &'a mut Option<Point>),
) -> Result<Target<'a>, ServiceError> {
    let before = index.checked_sub(1).ok_or(ServiceError::Gone)?;
    let (head, tail) = marks.split_at_mut(index);
    let a = head.get_mut(before).ok_or(ServiceError::Gone)?;
    let this = tail.first_mut().ok_or(ServiceError::Gone)?;

    let (a, _) = place(a);
    let (b, via) = place(this);

    Ok(Target::Curve { via, a, b })
}

pub(super) fn middle(wall: &Wall) -> Point {
    wall.from.lerp(wall.to, 0.5)
}

/// Every point of a drawing, to read or to move.
pub(super) fn points_mut(shape: &mut DrawingShape) -> Vec<&mut Point> {
    match shape {
        DrawingShape::Line { from, to }
        | DrawingShape::Arrow { from, to }
        | DrawingShape::Rect { from, to }
        | DrawingShape::Ellipse { from, to } => vec![from, to],
        DrawingShape::Area { points } => points.iter_mut().collect(),
    }
}

/// The first point of a drawing and the middle of its bounding box (the middle of a line,
/// the centre of a rectangle or ellipse); `None` for a shape without points.
fn anchors(drawing: &mut Drawing) -> Option<(Point, Point)> {
    let points = points_mut(&mut drawing.shape);
    let first = **points.first()?;
    let (mut min, mut max) = (first, first);

    for p in &points {
        min = Point::new(min.x_m.min(p.x_m), min.y_m.min(p.y_m));
        max = Point::new(max.x_m.max(p.x_m), max.y_m.max(p.y_m));
    }

    Some((first, min.lerp(max, 0.5)))
}

pub(super) fn move_to(
    scope: &mut Scope<'_>,
    source: &Source,
    to: Point,
) -> Result<(), ServiceError> {
    match target(scope, source)? {
        Target::Place { at, .. } | Target::Pin { at } => *at = to,
        // `to` is where the way passes halfway, which is what the grip sits on.
        Target::Curve { via, a, b } => *via = Some(via_through(a, to, b)),
        Target::Drawing(drawing) => {
            // The whole drawing keeps its shape; the middle of its box lands on `to`.
            let Some((_, mid)) = anchors(drawing) else {
                return Ok(());
            };
            let (dx, dy) = (to.x_m - mid.x_m, to.y_m - mid.y_m);

            for p in points_mut(&mut drawing.shape) {
                *p = Point::new(p.x_m + dx, p.y_m + dy);
            }
        }
        Target::Wall(wall) => {
            // The whole wall keeps its shape; its middle lands on `to`.
            let mid = middle(wall);
            let (dx, dy) = (to.x_m - mid.x_m, to.y_m - mid.y_m);
            wall.from = Point::new(wall.from.x_m + dx, wall.from.y_m + dy);
            wall.to = Point::new(wall.to.x_m + dx, wall.to.y_m + dy);
        }
    }

    Ok(())
}

/// What `shift` reads off an element: the point that snaps and the point that moves.
pub(super) fn anchor_pair(target: Target<'_>) -> Result<(Point, Point), ServiceError> {
    match target {
        Target::Place { at, .. } | Target::Pin { at } => Ok((*at, *at)),
        Target::Curve { via, a, b } => {
            let grip = bend_point(a, *via, b);
            Ok((grip, grip))
        }
        Target::Wall(wall) => Ok((wall.from, middle(wall))),
        Target::Drawing(drawing) => anchors(drawing).ok_or(ServiceError::Gone),
    }
}

/// Moves by `by_m` (dx, dy) from where the element is now; the new place is snapped. A wall
/// snaps its start, not its middle: a wall 3.25 m long keeps both ends where they meet others.
/// A drawing does the same with its first point.
pub(super) fn shift(
    scope: &mut Scope<'_>,
    source: &Source,
    by_m: (f64, f64),
    snapping: Snapping,
) -> Result<(), ServiceError> {
    let (anchor, middle_now) = anchor_pair(target(scope, source)?)?;
    let landed = snap(
        Point::new(anchor.x_m + by_m.0, anchor.y_m + by_m.1),
        snapping,
    );
    let (dx, dy) = (landed.x_m - anchor.x_m, landed.y_m - anchor.y_m);

    move_to(
        scope,
        source,
        Point::new(middle_now.x_m + dx, middle_now.y_m + dy),
    )
}

pub(super) fn spin(
    scope: &mut Scope<'_>,
    source: &Source,
    by_deg: f64,
) -> Result<(), ServiceError> {
    let now_deg = match target(scope, source)? {
        Target::Place { facing_deg, .. } => *facing_deg,
        Target::Wall(wall) => wall.from.bearing_deg(wall.to),
        // Nothing to turn.
        Target::Pin { .. } | Target::Drawing(_) | Target::Curve { .. } => return Ok(()),
    };

    turn(scope, source, now_deg + by_deg)
}

pub(super) fn turn(
    scope: &mut Scope<'_>,
    source: &Source,
    facing_deg: f64,
) -> Result<(), ServiceError> {
    let facing_deg = normalize_deg(facing_deg);

    match target(scope, source)? {
        Target::Place { facing_deg: f, .. } => *f = facing_deg,
        // Nothing to turn.
        Target::Pin { .. } | Target::Drawing(_) | Target::Curve { .. } => {}
        Target::Wall(wall) => {
            // Turns about its middle, keeping its length; from -> to then points to `facing_deg`.
            let mid = middle(wall);
            let half_m = wall.from.distance_m(wall.to) / 2.0;
            let (sin, cos) = facing_deg.to_radians().sin_cos();
            wall.from = Point::new(mid.x_m - half_m * cos, mid.y_m - half_m * sin);
            wall.to = Point::new(mid.x_m + half_m * cos, mid.y_m + half_m * sin);
        }
    }

    Ok(())
}

/// Runs `change`; if it moved or turned a mark of a figure that cameras follow (LH-37), those
/// cameras' marks within the mark's time window (arrival to the end of the pause) take the
/// same move and turn about the mark, so distance and angle to the figure stay. In the same
/// edit, hence one undo step.
pub(super) fn carry_cameras(
    scope: &mut Scope<'_>,
    source: &Source,
    change: impl FnOnce(&mut Scope<'_>) -> Result<(), ServiceError>,
) -> Result<(), ServiceError> {
    let Some((character, index)) = figure_mark(scope, source) else {
        return change(scope);
    };
    let mark_of = |plan: &FloorPlan| plan.figure(&character)?.marks.get(index).cloned();
    let before = mark_of(scope.plan);

    change(scope)?;

    let (Some(before), Some(after)) = (before, mark_of(scope.plan)) else {
        return Ok(());
    };
    let (dx_m, dy_m) = (after.at.x_m - before.at.x_m, after.at.y_m - before.at.y_m);
    let turn_deg = normalize_deg(after.facing_deg - before.facing_deg);
    let window_s = before.at_s..=before.at_s + before.hold_s;

    let followers = scope
        .plan
        .cameras
        .iter_mut()
        .filter(|c| c.follows.as_deref() == Some(character.as_str()));
    for camera in followers {
        for camera_mark in camera
            .marks
            .iter_mut()
            .filter(|m| window_s.contains(&m.at_s))
        {
            // The bend of the move goes along, or the move turns into a loop (LH-21).
            camera_mark.at = carry(camera_mark.at, (dx_m, dy_m), after.at, turn_deg);
            camera_mark.via = camera_mark
                .via
                .map(|via| carry(via, (dx_m, dy_m), after.at, turn_deg));
            camera_mark.facing_deg = normalize_deg(camera_mark.facing_deg + turn_deg);
        }
    }

    Ok(())
}

/// `point` shifted by `delta_m`, then turned by `deg` about `centre`: where a following
/// camera's point goes when its figure moves and turns.
fn carry(point: Point, delta_m: (f64, f64), centre: Point, deg: f64) -> Point {
    let moved = Point::new(point.x_m + delta_m.0, point.y_m + delta_m.1);

    rotate(moved, centre, deg)
}

/// `point` turned clockwise on the plan by `deg` about `centre`.
pub(super) fn rotate(point: Point, centre: Point, deg: f64) -> Point {
    if deg == 0.0 {
        return point;
    }

    let (sin, cos) = deg.to_radians().sin_cos();
    let (x_m, y_m) = (point.x_m - centre.x_m, point.y_m - centre.y_m);

    Point::new(
        centre.x_m + x_m * cos - y_m * sin,
        centre.y_m + x_m * sin + y_m * cos,
    )
}
