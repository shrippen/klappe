//! Moving, turning and bending elements: which points an edit targets and how they move.

use klappe_model::{
    CameraMark, Curve, Drawing, DrawingShape, Extent, FloorPlan, Light, Mark, PlanImage, Point,
    SetObject, Wall, MAX_OBJECT_M, MIN_BEAM_DEG, MIN_OBJECT_M, NOTE_MIN_WIDTH_M,
};
use klappe_stellprobe::{
    bend_follow, bend_grips, drawing_handle_places, image_grip_place, light_grip_place, object_at,
    object_grip_place, Bend, LightGrip, ObjectGrip, Source,
};

use self::forms::{drag_form_point, insert_form_point};
use super::drawings::{drag_handle, edge_place, insert_point};
use super::openings::{drag_edge, edge_m, slide};
use super::target::{target, Target};
use super::{current_index, figure_mark, normalize_deg, snap, Grid, PlanEdit, Scope, Snapping};
use crate::error::ServiceError;
use klappe_model::FULL_TURN_DEG;

pub(super) mod forms;
mod ways;

pub(super) use ways::edit_way;

/// How far a drag pulls a new bend off its grip before it is made.
const NEW_BEND_MIN_M: f64 = 1e-6;

/// Where grip `bend` of the way from `a` to `b` through `via` sits.
fn grip_at(shape: (&[Point], Curve), (a, b): (Point, Point), bend: Bend) -> Option<Point> {
    match bend {
        Bend::At(n) => shape.0.get(n).copied(),
        Bend::New(n) => bend_grips(a, shape, b).get(n).copied(),
    }
}

/// Moves a bend to `to`, or makes a new one there. A new bend that is not pulled off its
/// grip would only add a point the way already passes.
fn pull(
    via: &mut Vec<Point>,
    curve: Curve,
    ends: (Point, Point),
    bend: Bend,
    to: Point,
) -> Result<(), ServiceError> {
    let grip = grip_at((via, curve), ends, bend).ok_or(ServiceError::Gone)?;

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

            light.beam_deg = (2.0 * off_deg.abs()).max(MIN_BEAM_DEG);
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
        Target::ImageEdge(image) => stretch_image(image, to),
        Target::ObjectEdge {
            object,
            grip: ObjectGrip::Point(n),
        } => drag_form_point(object, n, to)?,
        Target::ObjectEdge {
            object,
            grip: ObjectGrip::Side(n),
        } => insert_form_point(object, n, to)?,
        Target::ObjectEdge { object, grip } => stretch(object, grip, to),
        _ => {}
    }

    Ok(())
}

/// The extent a grip sets; none for a free form's point or side.
fn extent_of(grip: ObjectGrip) -> Option<Extent> {
    match grip {
        ObjectGrip::Width => Some(Extent::Width),
        ObjectGrip::Depth => Some(Extent::Depth),
        ObjectGrip::Point(_) | ObjectGrip::Side(_) => None,
    }
}

/// The point of grip `grip` that snaps: the middle of the edge it sizes (the right edge for the
/// width, the bottom edge for the depth, turned with the object), or the outline point itself.
fn edge_middle(object: &SetObject, grip: ObjectGrip) -> Point {
    match extent_of(grip) {
        Some(extent) => object.rect().edge_middle(extent),
        None => object_grip_place(object, grip),
    }
}

/// Sets the width or depth so the grip lands on `to` as far as its axis goes, in the object's
/// turned frame.
fn stretch(object: &mut SetObject, grip: ObjectGrip, to: Point) {
    // A point or side sets no size.
    let Some(extent) = extent_of(grip) else {
        return;
    };

    let grown = object.rect().stretched(
        extent,
        object_grip_place(object, grip),
        to,
        (MIN_OBJECT_M, MAX_OBJECT_M),
    );
    object.at = grown.at;
    object.width_m = grown.width_m;
    object.depth_m = grown.depth_m;
}

/// Sets the width so the grip lands on `to` as far as the image's turned x axis goes.
fn stretch_image(image: &mut PlanImage, to: Point) {
    let grown = image.rect().stretched(
        Extent::Width,
        image_grip_place(image),
        to,
        (MIN_OBJECT_M, MAX_OBJECT_M),
    );
    image.at = grown.at;
    image.width_m = grown.width_m;
}

fn place(scope: &mut Scope<'_>, source: &Source, to: Point) -> Result<(), ServiceError> {
    match target(scope, source)? {
        Target::Place { at, .. } | Target::Pin { at } => *at = to,
        // `to` is where the grip sits: a bend moves there, or a new one is made there.
        Target::Curve {
            via,
            curve,
            a,
            b,
            bend,
        } => pull(via, curve, (a, b), bend, to)?,
        Target::Drawing(drawing) => place_drawing(drawing, to),
        Target::Corner { drawing, index } => drag_handle(drawing, index, to)?,
        Target::Edge { drawing, index } => insert_point(drawing, index, to)?,
        Target::Slide { wall, index } => slide(wall, index, to)?,
        grip @ (Target::OpeningEdge { .. }
        | Target::LightEdge { .. }
        | Target::NoteEdge(_)
        | Target::ImageEdge(_)
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
        Target::Curve {
            via,
            curve,
            a,
            b,
            bend,
        } => {
            let grip = grip_at((via, curve), (a, b), bend).ok_or(ServiceError::Gone)?;
            Ok((grip, grip))
        }
        // Only the x of the grip counts; the card's height is Stellprobe's to know.
        Target::NoteEdge(note) => {
            let edge = Point::new(note.at.x_m + note.width_m, note.at.y_m);

            Ok((edge, edge))
        }
        // As an object's: the edge snaps, the grip outside it follows.
        Target::ImageEdge(image) => Ok((
            image.rect().edge_middle(Extent::Width),
            image_grip_place(image),
        )),
        // The edge snaps, the grip outside it follows: an unturned object gets round sizes.
        Target::ObjectEdge { object, grip } => {
            Ok((edge_middle(object, grip), object_grip_place(object, grip)))
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
            | Source::ObjectGrip {
                grip: ObjectGrip::Side(_),
                ..
            }
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
        // Nothing to turn: a note, an opening and every grip.
        _ => return Ok(()),
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
        Target::Wall(wall) => {
            // Turns about its middle, keeping its length; from -> to then points to `facing_deg`.
            let mid = middle(wall);
            let half_m = wall.from.distance_m(wall.to) / 2.0;
            let (sin, cos) = facing_deg.to_radians().sin_cos();
            wall.from = Point::new(mid.x_m - half_m * cos, mid.y_m - half_m * sin);
            wall.to = Point::new(mid.x_m + half_m * cos, mid.y_m + half_m * sin);
        }
        // Nothing to turn: a note, an opening and every grip.
        _ => {}
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
        return placed_grip(scope, source, change);
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

fn object_in<'a>(scope: &'a mut Scope<'_>, id: &str) -> Option<&'a mut SetObject> {
    scope
        .set
        .as_deref_mut()
        .and_then(|s| s.objects.iter_mut().find(|o| o.id == id))
}

/// A grip of a moving object sits where the object is at the playhead, so the change works
/// on the object put there; afterwards the set keeps its own place and the way takes the
/// difference (resizing from the left edge moves the middle), each mark in its own frame.
pub(super) fn placed_grip(
    scope: &mut Scope<'_>,
    source: &Source,
    change: impl FnOnce(&mut Scope<'_>) -> Result<(), ServiceError>,
) -> Result<(), ServiceError> {
    let Source::ObjectGrip { object, .. } = source else {
        return change(scope);
    };
    let track = scope.plan.object_track(object);
    let Some(place) = track.and_then(|t| object_at(scope.plan, t, scope.time_s)) else {
        return change(scope);
    };
    let Some(own) = object_in(scope, object) else {
        return change(scope);
    };
    let home = (own.at, own.rotation_deg);
    (own.at, own.rotation_deg) = (place.at, place.facing_deg);

    let done = change(scope);

    let Some(own) = object_in(scope, object) else {
        return done;
    };
    let delta = Point::new(own.at.x_m - place.at.x_m, own.at.y_m - place.at.y_m);
    (own.at, own.rotation_deg) = home;

    // The shift is the object's own: along its axes at the playhead, so a mark turned
    // otherwise takes it along its own axes.
    let origin = Point::new(0.0, 0.0);
    let local = delta.turned(origin, -place.facing_deg);
    for mark in scope
        .plan
        .object_tracks
        .iter_mut()
        .filter(|t| t.object == *object)
        .flat_map(|t| t.marks.iter_mut())
    {
        let by = local.turned(origin, mark.rotation_deg);
        mark.at = Point::new(mark.at.x_m + by.x_m, mark.at.y_m + by.y_m);
    }

    done
}

/// Moves, shifts, spins or turns what the edit names; a figure's mark takes its cameras along.
pub(super) fn place_edit(
    scope: &mut Scope<'_>,
    edit: &PlanEdit,
    snap_to: (Snapping, Grid),
) -> Result<(), ServiceError> {
    match edit {
        PlanEdit::Move { source, to } => carry_cameras(scope, source, |s| move_to(s, source, *to)),
        PlanEdit::Shift { source, dx_m, dy_m } => {
            carry_cameras(scope, source, |s| shift(s, source, (*dx_m, *dy_m), snap_to))
        }
        PlanEdit::Spin { source, by_deg } => {
            carry_cameras(scope, source, |s| spin(s, source, *by_deg))
        }
        PlanEdit::Turn { source, facing_deg } => {
            carry_cameras(scope, source, |s| turn(s, source, *facing_deg))
        }
        _ => Ok(()),
    }
}

/// `point` shifted by `delta_m`, then turned by `deg` about `centre`: where a following
/// camera's point goes when its figure moves and turns.
fn carry(point: Point, delta_m: (f64, f64), centre: Point, deg: f64) -> Point {
    let moved = Point::new(point.x_m + delta_m.0, point.y_m + delta_m.1);

    moved.turned(centre, deg)
}
