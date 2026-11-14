//! Edits that take several elements at once (LH-08). Each is one `PlanEdit`, so one undo step,
//! and applies to a copy of the project: an element that cannot take part refuses the lot.

use std::cmp::Reverse;

use klappe_model::Point;
use klappe_stellprobe::Source;

use super::{
    anchor_pair, carry_cameras, figure_mark, free_letter, move_to, points_mut, remove, rotate,
    snap, target, turn, Scope, Snapping, Target,
};
use crate::error::ServiceError;

/// Moves every element by the distance, as `shift` does for one. The distance is snapped for
/// the first element, so it lands on the grid and the others keep their place to it.
pub(super) fn shift(
    scope: &mut Scope<'_>,
    sources: &[Source],
    by_m: (f64, f64),
    snapping: Snapping,
) -> Result<(), ServiceError> {
    let members = movers(scope, sources);
    let placings = placings(scope, &members)?;
    let Some(first) = placings.first() else {
        return Ok(());
    };

    let anchor = first.anchor;
    let landed = snap(
        Point::new(anchor.x_m + by_m.0, anchor.y_m + by_m.1),
        snapping,
    );
    let (dx_m, dy_m) = (landed.x_m - anchor.x_m, landed.y_m - anchor.y_m);

    for (source, placing) in members.iter().zip(&placings) {
        let to = Point::new(placing.middle.x_m + dx_m, placing.middle.y_m + dy_m);
        carry_cameras(scope, source, |s| move_to(s, source, to))?;
    }

    Ok(())
}

/// Turns the group about the middle of its bounding box: every element swings to where the
/// turn puts it and turns by the same angle itself (what `spin` leaves unturned stays so: a
/// note, a drawing, a grip only swing).
pub(super) fn spin(
    scope: &mut Scope<'_>,
    sources: &[Source],
    by_deg: f64,
) -> Result<(), ServiceError> {
    let members = movers(scope, sources);
    let placings = placings(scope, &members)?;
    let middles: Vec<Point> = placings.iter().map(|p| p.middle).collect();
    let Some(centre) = box_middle(&middles) else {
        return Ok(());
    };

    for (source, placing) in members.iter().zip(&placings) {
        let to = rotate(placing.middle, centre, by_deg);
        carry_cameras(scope, source, |s| {
            move_to(s, source, to)?;
            let Some(facing_deg) = placing.facing_deg else {
                return Ok(());
            };

            turn(s, source, facing_deg + by_deg)
        })?;
    }

    Ok(())
}

pub(super) fn remove_all(scope: &mut Scope<'_>, sources: &[Source]) -> Result<(), ServiceError> {
    for source in removable(sources) {
        remove(scope, &source)?;
    }

    Ok(())
}

/// Whether an element had a copy to make.
enum Copied {
    Made,
    Left,
}

/// Copies what can be copied, `by_m` away from the original. The copy has an id of its own.
pub(super) fn duplicate(
    scope: &mut Scope<'_>,
    sources: &[Source],
    by_m: (f64, f64),
) -> Result<(), ServiceError> {
    let mut made = 0;

    for source in distinct(sources) {
        if matches!(copy(scope, &source, by_m)?, Copied::Made) {
            made += 1;
        }
    }
    if made > 0 {
        return Ok(());
    }

    Err(ServiceError::BadValue {
        field: "copy",
        value: String::new(),
    })
}

/// `sources` without repeats, in their order.
fn distinct(sources: &[Source]) -> Vec<Source> {
    let mut found: Vec<Source> = Vec::new();

    for source in sources {
        if !found.contains(source) {
            found.push(source.clone());
        }
    }

    found
}

/// The elements a move or turn works on. A figure is the mark it stands at, or it and that
/// mark would move twice. The order matters: a figure carries the cameras that follow it
/// (LH-37) along, so cameras come after the figures, and each element is set to a place
/// worked out beforehand, not moved by a distance, or a camera of the group would move twice.
/// Grips come last: their bend follows the marks it lies between, which have to be in place.
fn movers(scope: &Scope<'_>, sources: &[Source]) -> Vec<Source> {
    let named = |source: &Source| match figure_mark(scope, source) {
        Some((character, index)) => Source::Mark { character, index },
        None => source.clone(),
    };
    let mut members = distinct(&sources.iter().map(named).collect::<Vec<_>>());

    members.sort_by_key(|m| match m {
        Source::Camera(_) => 1,
        Source::FigureVia { .. } | Source::CameraVia { .. } => 2,
        _ => 0,
    });

    members
}

/// What `shift` and `spin` read off an element before anything moves.
struct Placing {
    /// The point that snaps.
    anchor: Point,
    /// The point that moves and turns.
    middle: Point,
    /// Where it faces, if it can turn.
    facing_deg: Option<f64>,
}

fn placings(scope: &mut Scope<'_>, sources: &[Source]) -> Result<Vec<Placing>, ServiceError> {
    sources
        .iter()
        .map(|source| {
            let held = target(scope, source)?;
            let facing_deg = match &held {
                Target::Place { facing_deg, .. } => Some(**facing_deg),
                Target::Wall(wall) => Some(wall.from.bearing_deg(wall.to)),
                _ => None,
            };
            let (anchor, middle) = anchor_pair(held)?;

            Ok(Placing {
                anchor,
                middle,
                facing_deg,
            })
        })
        .collect()
}

/// The middle of the box that holds all `points`; `None` for no points.
fn box_middle(points: &[Point]) -> Option<Point> {
    let first = *points.first()?;
    let (mut min, mut max) = (first, first);

    for p in points {
        min = Point::new(min.x_m.min(p.x_m), min.y_m.min(p.y_m));
        max = Point::new(max.x_m.max(p.x_m), max.y_m.max(p.y_m));
    }

    Some(min.lerp(max, 0.5))
}

/// What a removal takes, in the order it takes it. A grip goes first (it only straightens a
/// way by an index the removals would shift); marks and walls go from the back, so the
/// indices of those still to go stay right. What a whole figure or camera takes with it is
/// not removed a second time.
fn removable(sources: &[Source]) -> Vec<Source> {
    let members = distinct(sources);
    let gone_figure = |character: &str| members.contains(&Source::Figure(character.to_owned()));
    let gone_camera = |id: &str| members.contains(&Source::Camera(id.to_owned()));

    let mut kept: Vec<Source> = members
        .iter()
        .filter(|m| match m {
            Source::Mark { character, .. } | Source::FigureVia { character, .. } => {
                !gone_figure(character)
            }
            Source::CameraVia { camera, .. } => !gone_camera(camera),
            _ => true,
        })
        .cloned()
        .collect();

    kept.sort_by_key(|m| match m {
        Source::FigureVia { index, .. } | Source::CameraVia { index, .. } => (0, Reverse(*index)),
        Source::Mark { index, .. } | Source::Wall(index) => (1, Reverse(*index)),
        _ => (1, Reverse(0)),
    });

    kept
}

/// The first id `<prefix><n>` that `taken` does not have.
fn free_id<'a>(prefix: &str, taken: impl Iterator<Item = &'a str> + Clone) -> String {
    let n = (1..)
        .find(|n| !taken.clone().any(|id| id == format!("{prefix}{n}")))
        .unwrap_or(1);

    format!("{prefix}{n}")
}

fn shifted(p: Point, by_m: (f64, f64)) -> Point {
    Point::new(p.x_m + by_m.0, p.y_m + by_m.1)
}

fn copy(scope: &mut Scope<'_>, source: &Source, by_m: (f64, f64)) -> Result<Copied, ServiceError> {
    match source {
        Source::Object(id) => {
            let set = scope.set.as_deref_mut().ok_or(ServiceError::Gone)?;
            let mut object = set
                .objects
                .iter()
                .find(|o| o.id == *id)
                .cloned()
                .ok_or(ServiceError::Gone)?;

            object.id = free_id("o", set.objects.iter().map(|o| o.id.as_str()));
            object.at = shifted(object.at, by_m);
            set.objects.push(object);
        }
        Source::Light(id) => {
            let lights = &mut scope.plan.lights;
            let mut light = lights
                .iter()
                .find(|l| l.id == *id)
                .cloned()
                .ok_or(ServiceError::Gone)?;

            light.id = free_id("l", lights.iter().map(|l| l.id.as_str()));
            light.label = light.id.to_uppercase();
            light.at = shifted(light.at, by_m);
            lights.push(light);
        }
        Source::Note(id) => {
            let set = scope.set.as_deref_mut().ok_or(ServiceError::Gone)?;
            let mut note = set
                .notes
                .iter()
                .find(|n| n.id == *id)
                .cloned()
                .ok_or(ServiceError::Gone)?;

            note.id = free_id("n", set.notes.iter().map(|n| n.id.as_str()));
            note.at = shifted(note.at, by_m);
            set.notes.push(note);
        }
        Source::Drawing(id) => {
            let drawings = &mut scope.plan.drawings;
            let mut drawing = drawings
                .iter()
                .find(|d| d.id == *id)
                .cloned()
                .ok_or(ServiceError::Gone)?;

            drawing.id = free_id("d", drawings.iter().map(|d| d.id.as_str()));
            for p in points_mut(&mut drawing.shape) {
                *p = shifted(*p, by_m);
            }
            drawings.push(drawing);
        }
        Source::Wall(index) => {
            let set = scope.set.as_deref_mut().ok_or(ServiceError::Gone)?;
            let mut wall = set.walls.get(*index).cloned().ok_or(ServiceError::Gone)?;

            wall.from = shifted(wall.from, by_m);
            wall.to = shifted(wall.to, by_m);
            set.walls.push(wall);
        }
        Source::Camera(id) => {
            let mut camera = scope.plan.camera(id).cloned().ok_or(ServiceError::Gone)?;
            let letter = free_letter(&scope.plan.cameras)?;

            camera.id = letter.to_ascii_lowercase().to_string();
            camera.label = letter.to_string();
            for mark in &mut camera.marks {
                mark.at = shifted(mark.at, by_m);
                mark.via = mark.via.map(|via| shifted(via, by_m));
            }
            scope.plan.cameras.push(camera);
        }
        // A character has one track; a grip is no element.
        Source::Figure(_)
        | Source::Mark { .. }
        | Source::FigureVia { .. }
        | Source::CameraVia { .. } => return Ok(Copied::Left),
    }

    Ok(Copied::Made)
}
