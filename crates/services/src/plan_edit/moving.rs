//! Moving, turning and bending elements: which points an edit targets and how they move.

use klappe_model::{
    CameraMark, Drawing, DrawingShape, FloorPlan, Light, Mark, Point, SetObject, Wall,
    MAX_OBJECT_M, NOTE_MIN_WIDTH_M,
};
use klappe_stellprobe::{
    bend_follow, bend_grips, drawing_handle_places, light_grip_place, object_grip_place, Bend,
    LightGrip, ObjectGrip, Source,
};

use super::drawings::{drag_handle, edge_place, insert_point};
use super::openings::{drag_edge, edge_m, opening_mut, slide};
use super::FULL_TURN_DEG;
use super::{current_index, figure_mark, normalize_deg, snap, Grid, Scope, Snapping, Target};
use crate::error::ServiceError;

/// The least a drag leaves of an object's width or depth; the model only asks for more than
/// nothing.
const OBJECT_MIN_M: f64 = 0.05;

/// How far a drag pulls a new bend off its grip before it is made.
const NEW_BEND_MIN_M: f64 = 1e-6;

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
        Source::FigureVia { .. }
        | Source::CameraVia { .. }
        | Source::DrawingPoint { .. }
        | Source::DrawingEdge { .. }
        | Source::ObjectGrip { .. } => grip_target(scope, source),
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
        Source::Light(_) | Source::LightGrip { .. } => light_target(scope, source),
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
        Source::Opening { .. } | Source::OpeningEdge { .. } => opening_target(scope, source),
        Source::Note(_) | Source::NoteWidth(_) => note_target(scope, source),
        Source::Drawing(id) => scope
            .plan
            .drawings
            .iter_mut()
            .find(|d| d.id == *id)
            .map(Target::Drawing)
            .ok_or(ServiceError::Gone),
    }
}

/// A lamp, or the grip on its beam.
fn light_target<'a>(scope: &'a mut Scope<'_>, source: &Source) -> Result<Target<'a>, ServiceError> {
    let (Source::Light(id) | Source::LightGrip { light: id, .. }) = source else {
        return Err(ServiceError::Gone);
    };
    let light = scope
        .plan
        .lights
        .iter_mut()
        .find(|l| l.id == *id)
        .ok_or(ServiceError::Gone)?;

    match source {
        Source::LightGrip { grip, .. } => Ok(Target::LightEdge { light, grip: *grip }),
        _ => Ok(Target::Place {
            at: &mut light.at,
            facing_deg: &mut light.facing_deg,
        }),
    }
}

/// An opening, or the grip on one of its ends.
fn opening_target<'a>(
    scope: &'a mut Scope<'_>,
    source: &Source,
) -> Result<Target<'a>, ServiceError> {
    let (wall, index) = opening_mut(scope, source)?;

    match source {
        Source::OpeningEdge { end, .. } => Ok(Target::OpeningEdge {
            wall,
            index,
            end: *end,
        }),
        _ => Ok(Target::Slide { wall, index }),
    }
}

/// A note, or the grip on its right edge.
fn note_target<'a>(scope: &'a mut Scope<'_>, source: &Source) -> Result<Target<'a>, ServiceError> {
    let (Source::Note(id) | Source::NoteWidth(id)) = source else {
        return Err(ServiceError::Gone);
    };
    let note = scope
        .set
        .as_deref_mut()
        .and_then(|s| s.notes.iter_mut().find(|n| n.id == *id))
        .ok_or(ServiceError::Gone)?;

    match source {
        Source::NoteWidth(_) => Ok(Target::NoteEdge(note)),
        _ => Ok(Target::Pin { at: &mut note.at }),
    }
}

/// The grip a `FigureVia` or `CameraVia` names, on its way.
fn grip_target<'a>(scope: &'a mut Scope<'_>, source: &Source) -> Result<Target<'a>, ServiceError> {
    match source {
        Source::FigureVia {
            character,
            index,
            bend,
        } => {
            let track = scope
                .plan
                .figures
                .iter_mut()
                .find(|f| f.character == *character)
                .ok_or(ServiceError::Gone)?;
            curve(&mut track.marks, (*index, *bend), |m| (m.at, &mut m.via))
        }
        Source::CameraVia {
            camera,
            index,
            bend,
        } => {
            let camera = scope
                .plan
                .cameras
                .iter_mut()
                .find(|c| c.id == *camera)
                .ok_or(ServiceError::Gone)?;
            curve(&mut camera.marks, (*index, *bend), |m| (m.at, &mut m.via))
        }
        Source::DrawingPoint { drawing, index } => {
            let drawing = drawing_mut(scope, drawing)?;

            Ok(Target::Corner {
                drawing,
                index: *index,
            })
        }
        Source::DrawingEdge { drawing, index } => {
            let drawing = drawing_mut(scope, drawing)?;

            Ok(Target::Edge {
                drawing,
                index: *index,
            })
        }
        Source::ObjectGrip { object, grip } => {
            let object = scope
                .set
                .as_deref_mut()
                .and_then(|s| s.objects.iter_mut().find(|o| o.id == *object))
                .ok_or(ServiceError::Gone)?;

            Ok(Target::ObjectEdge {
                object,
                grip: *grip,
            })
        }
        _ => Err(ServiceError::Gone),
    }
}

fn drawing_mut<'a>(scope: &'a mut Scope<'_>, id: &str) -> Result<&'a mut Drawing, ServiceError> {
    scope
        .plan
        .drawings
        .iter_mut()
        .find(|d| d.id == id)
        .ok_or(ServiceError::Gone)
}

/// The grip `bend` on the way to mark `index` of `marks`; `place` gives a mark's place and its
/// `via`. The first mark has no way to it.
fn curve<'a, M>(
    marks: &'a mut [M],
    (index, bend): (usize, Bend),
    place: impl Fn(&'a mut M) -> (Point, &'a mut Vec<Point>),
) -> Result<Target<'a>, ServiceError> {
    let before = index.checked_sub(1).ok_or(ServiceError::Gone)?;
    let (head, tail) = marks.split_at_mut(index);
    let a = head.get_mut(before).ok_or(ServiceError::Gone)?;
    let this = tail.first_mut().ok_or(ServiceError::Gone)?;

    let (a, _) = place(a);
    let (b, via) = place(this);

    Ok(Target::Curve { via, a, b, bend })
}

/// Where grip `bend` of the way from `a` to `b` through `via` sits.
fn grip_at(via: &[Point], (a, b): (Point, Point), bend: Bend) -> Option<Point> {
    match bend {
        Bend::At(n) => via.get(n).copied(),
        Bend::New(n) => bend_grips(a, via, b).get(n).copied(),
    }
}

/// Moves a bend to `to`, or makes a new one there. A new bend that is not pulled off its
/// grip would only add a point the way already passes.
fn pull(
    via: &mut Vec<Point>,
    ends: (Point, Point),
    bend: Bend,
    to: Point,
) -> Result<(), ServiceError> {
    let grip = grip_at(via, ends, bend).ok_or(ServiceError::Gone)?;

    match bend {
        Bend::At(n) => via[n] = to,
        Bend::New(_) if grip.distance_m(to) < NEW_BEND_MIN_M => {}
        Bend::New(n) => via.insert(n, to),
    }

    Ok(())
}

fn middle(wall: &Wall) -> Point {
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

/// The first point of a drawing as it lies on the plan and the middle of its bounding box
/// (the middle of a line, the centre of a rectangle or ellipse), which a turn keeps; `None`
/// for a shape without points.
fn anchors(drawing: &Drawing) -> Option<(Point, Point)> {
    let first = *drawing.points().first()?;

    Some((drawing.placed(first), drawing.middle()?))
}

/// A mark of a figure or camera: whose, and which.
enum Owner {
    Figure(String),
    Camera(String),
}

/// The mark `source` names or stands at, as its owner and index, with the place it is at.
fn mark_held(scope: &Scope<'_>, source: &Source) -> Option<(Owner, usize, Point)> {
    let time_s = scope.time_s;

    match source {
        Source::Mark { character, index } => {
            let at = scope.plan.figure(character)?.marks.get(*index)?.at;
            Some((Owner::Figure(character.clone()), *index, at))
        }
        Source::Figure(character) => {
            let track = scope.plan.figure(character)?;
            let index = current_index(track.marks.iter().map(|m| m.at_s), time_s)?;
            Some((
                Owner::Figure(character.clone()),
                index,
                track.marks[index].at,
            ))
        }
        Source::Camera(id) => {
            let camera = scope.plan.camera(id)?;
            let index = current_index(camera.marks.iter().map(|m| m.at_s), time_s)?;
            Some((Owner::Camera(id.clone()), index, camera.marks[index].at))
        }
        _ => None,
    }
}

fn figure_parts(m: &mut Mark) -> (&mut Point, &mut Vec<Point>) {
    (&mut m.at, &mut m.via)
}

fn camera_parts(m: &mut CameraMark) -> (&mut Point, &mut Vec<Point>) {
    (&mut m.at, &mut m.via)
}

/// After mark `index` moved from `old`: the bends of the way to it and of the way from it keep
/// their shape, so a move does not leave a bend behind that now cuts across the plan (LH-21).
fn keep_bends<M>(
    marks: &mut [M],
    index: usize,
    old: Point,
    parts: for<'a> fn(&'a mut M) -> (&'a mut Point, &'a mut Vec<Point>),
) {
    let Some(this) = marks.get_mut(index) else {
        return;
    };
    let new = *parts(this).0;

    if let Some(before) = index.checked_sub(1) {
        let a = *parts(&mut marks[before]).0;
        let (_, via) = parts(&mut marks[index]);
        via.iter_mut()
            .for_each(|v| *v = bend_follow(*v, (a, old), (a, new)));
    }

    let Some(next) = marks.get_mut(index + 1) else {
        return;
    };
    let (b, via) = parts(next);
    let b = *b;
    via.iter_mut()
        .for_each(|v| *v = bend_follow(*v, (old, b), (new, b)));
}

pub(super) fn move_to(
    scope: &mut Scope<'_>,
    source: &Source,
    to: Point,
) -> Result<(), ServiceError> {
    let held = mark_held(scope, source);

    place(scope, source, to)?;

    match held {
        Some((Owner::Figure(character), index, old)) => {
            if let Some(track) = scope
                .plan
                .figures
                .iter_mut()
                .find(|f| f.character == character)
            {
                keep_bends(&mut track.marks, index, old, figure_parts);
            }
        }
        Some((Owner::Camera(id), index, old)) => {
            if let Some(camera) = scope.plan.cameras.iter_mut().find(|c| c.id == id) {
                keep_bends(&mut camera.marks, index, old, camera_parts);
            }
        }
        None => {}
    }

    Ok(())
}

/// The whole drawing keeps its shape; the middle of its box lands on `to`.
fn place_drawing(drawing: &mut Drawing, to: Point) {
    let Some((_, mid)) = anchors(drawing) else {
        return;
    };
    let (dx, dy) = (to.x_m - mid.x_m, to.y_m - mid.y_m);

    for p in points_mut(&mut drawing.shape) {
        *p = Point::new(p.x_m + dx, p.y_m + dy);
    }
}

/// The narrowest beam a drag sets; `is_beam_deg` wants more than 0°. A drag gives 360° at most.
const LIGHT_MIN_BEAM_DEG: f64 = 1.0;

/// The pointer at `to` turns the lamp towards it (direction grip) or sets the beam so that
/// its edge runs through it (beam grip). The lamp stays; a pointer on the lamp changes nothing.
fn drag_light(light: &mut Light, grip: LightGrip, to: Point) {
    if light.at.distance_m(to) == 0.0 {
        return;
    }
    let to_deg = light.at.bearing_deg(to);

    match grip {
        LightGrip::Direction => light.facing_deg = normalize_deg(to_deg),
        LightGrip::Beam => {
            // Half the beam is the angle between the facing and the pointer, 0 to 180°.
            let half_turn_deg = FULL_TURN_DEG / 2.0;
            let off_deg = normalize_deg(to_deg - light.facing_deg + half_turn_deg) - half_turn_deg;

            light.beam_deg = (2.0 * off_deg.abs()).max(LIGHT_MIN_BEAM_DEG);
        }
    }
}

/// The grips that set a size or an angle: `to` is where the pointer drags the grip.
fn place_grip(grip: Target<'_>, to: Point) -> Result<(), ServiceError> {
    match grip {
        Target::OpeningEdge { wall, index, end } => drag_edge(wall, index, end, to)?,
        Target::LightEdge { light, grip } => drag_light(light, grip, to),
        // The card keeps its top left; its right edge lands on `to`.
        Target::NoteEdge(note) => {
            note.width_m = (to.x_m - note.at.x_m).max(NOTE_MIN_WIDTH_M);
        }
        Target::ObjectEdge { object, grip } => stretch(object, grip, to),
        _ => {}
    }

    Ok(())
}

/// Sets the width or depth so the grip lands on `to` as far as its axis goes, in the object's
/// turned frame; the opposite edge (left or top) stays, so the centre moves half the change.
fn stretch(object: &mut SetObject, grip: ObjectGrip, to: Point) {
    let (sin, cos) = object.rotation_deg.to_radians().sin_cos();
    let (axis, size_m) = match grip {
        ObjectGrip::Width => ((cos, sin), object.width_m),
        ObjectGrip::Depth => ((-sin, cos), object.depth_m),
    };

    // The grip sits a gap outside the edge: the size changes by how far `to` lies beyond it.
    let grip_at = object_grip_place(object, grip);
    let beyond_m = (to.x_m - grip_at.x_m) * axis.0 + (to.y_m - grip_at.y_m) * axis.1;
    let new_m = (size_m + beyond_m).clamp(OBJECT_MIN_M, MAX_OBJECT_M);
    let moved_m = (new_m - size_m) / 2.0;

    object.at = Point::new(
        object.at.x_m + moved_m * axis.0,
        object.at.y_m + moved_m * axis.1,
    );
    match grip {
        ObjectGrip::Width => object.width_m = new_m,
        ObjectGrip::Depth => object.depth_m = new_m,
    }
}

fn place(scope: &mut Scope<'_>, source: &Source, to: Point) -> Result<(), ServiceError> {
    match target(scope, source)? {
        Target::Place { at, .. } | Target::Pin { at } => *at = to,
        // `to` is where the grip sits: a bend moves there, or a new one is made there.
        Target::Curve { via, a, b, bend } => pull(via, (a, b), bend, to)?,
        Target::Drawing(drawing) => place_drawing(drawing, to),
        Target::Corner { drawing, index } => drag_handle(drawing, index, to)?,
        Target::Edge { drawing, index } => insert_point(drawing, index, to)?,
        Target::Slide { wall, index } => slide(wall, index, to)?,
        grip @ (Target::OpeningEdge { .. }
        | Target::LightEdge { .. }
        | Target::NoteEdge(_)
        | Target::ObjectEdge { .. }) => place_grip(grip, to)?,
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
        Target::Curve { via, a, b, bend } => {
            let grip = grip_at(via, (a, b), bend).ok_or(ServiceError::Gone)?;
            Ok((grip, grip))
        }
        // Only the x of the grip counts; the card's height is Stellprobe's to know.
        Target::NoteEdge(note) => {
            let edge = Point::new(note.at.x_m + note.width_m, note.at.y_m);

            Ok((edge, edge))
        }
        Target::ObjectEdge { object, grip } => {
            let place = object_grip_place(object, grip);

            Ok((place, place))
        }
        Target::Wall(wall) => Ok((wall.from, middle(wall))),
        Target::LightEdge { light, grip } => {
            let place = light_grip_place(light, grip);

            Ok((place, place))
        }
        Target::OpeningEdge { wall, index, end } => {
            let grip = wall.point_at(edge_m(&wall.openings[index], end));

            Ok((grip, grip))
        }
        Target::Slide { wall, index } => {
            let centre = wall.point_at(wall.openings[index].at_m);
            Ok((centre, centre))
        }
        Target::Drawing(drawing) => anchors(drawing).ok_or(ServiceError::Gone),
        Target::Corner { drawing, index } => {
            let place = drawing_handle_places(drawing).get(index).copied();
            let place = place.ok_or(ServiceError::Gone)?;

            Ok((place, place))
        }
        Target::Edge { drawing, index } => {
            let place = edge_place(drawing, index).ok_or(ServiceError::Gone)?;

            Ok((place, place))
        }
    }
}

/// Moves by `by_m` (dx, dy) from where the element is now; the new place is snapped. A wall
/// snaps its start, not its middle: a wall 3.25 m long keeps both ends where they meet others.
/// A drawing does the same with its first point.
pub(super) fn shift(
    scope: &mut Scope<'_>,
    source: &Source,
    by_m: (f64, f64),
    snap_to: (Snapping, Grid),
) -> Result<(), ServiceError> {
    // A grip for a new bend not pulled at all makes none; snapping would move it off its
    // spot and the bend would be made there.
    let pulled = by_m != (0.0, 0.0);
    if matches!(
        source,
        Source::FigureVia {
            bend: Bend::New(_),
            ..
        } | Source::CameraVia {
            bend: Bend::New(_),
            ..
        } | Source::DrawingEdge { .. }
    ) && !pulled
    {
        return Ok(());
    }

    let (anchor, middle_now) = anchor_pair(target(scope, source)?)?;
    let landed = snap(
        Point::new(anchor.x_m + by_m.0, anchor.y_m + by_m.1),
        snap_to.0,
        snap_to.1,
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
        Target::Drawing(drawing) => drawing.rotation_deg,
        // Nothing to turn.
        Target::Pin { .. }
        | Target::Corner { .. }
        | Target::Edge { .. }
        | Target::NoteEdge(_)
        | Target::ObjectEdge { .. }
        | Target::Curve { .. }
        | Target::OpeningEdge { .. }
        | Target::LightEdge { .. }
        | Target::Slide { .. } => return Ok(()),
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
        Target::Drawing(drawing) => drawing.rotation_deg = facing_deg,
        // Nothing to turn.
        Target::Pin { .. }
        | Target::Corner { .. }
        | Target::Edge { .. }
        | Target::NoteEdge(_)
        | Target::ObjectEdge { .. }
        | Target::Curve { .. }
        | Target::OpeningEdge { .. }
        | Target::LightEdge { .. }
        | Target::Slide { .. } => {}
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
        for i in 0..camera.marks.len() {
            let camera_mark = &mut camera.marks[i];
            if !window_s.contains(&camera_mark.at_s) {
                continue;
            }

            // Moved like a mark moved by hand, so the ways to and from it keep their shape
            // (LH-21), also to a mark outside the window.
            let old = camera_mark.at;
            camera_mark.at = carry(old, (dx_m, dy_m), after.at, turn_deg);
            camera_mark.facing_deg = normalize_deg(camera_mark.facing_deg + turn_deg);
            keep_bends(&mut camera.marks, i, old, camera_parts);
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
    point.turned(centre, deg)
}
