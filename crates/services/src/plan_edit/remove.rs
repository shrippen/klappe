//! Removing elements: a figure takes what points at it along; a camera a shot uses stays.

use klappe_model::{FloorPlan, LinkTarget};
use klappe_stellprobe::{Bend, Source};

use super::bound::remove_note;
use super::images;
use super::moving::forms::remove_grip;
use super::openings::opening_mut;
use super::props;
use super::target::{target, Target};
use super::Scope;
use crate::error::ServiceError;

/// Takes the bend `source` names out of its way.
fn remove_bend(scope: &mut Scope<'_>, source: &Source) -> Result<(), ServiceError> {
    let Target::Curve {
        via,
        bend: Bend::At(n),
        ..
    } = target(scope, source)?
    else {
        return Ok(());
    };
    if n >= via.len() {
        return Err(ServiceError::Gone);
    }

    via.remove(n);

    Ok(())
}

pub(super) fn remove(scope: &mut Scope<'_>, source: &Source) -> Result<(), ServiceError> {
    match source {
        Source::Mark { character, index } => remove_mark(scope.plan, character, *index),
        Source::ObjectMark { object, index } => props::remove_object_mark(scope, object, *index),
        Source::Figure(character) => {
            let known = scope.plan.figure(character).is_some();
            if !known {
                return Err(ServiceError::Gone);
            }

            remove_figure(scope.plan, character);
            Ok(())
        }
        Source::Camera(id) => remove_camera(scope.plan, id),
        // A grip is no element of its own: removing a bend takes it out; a grip for a new
        // bend has nothing to remove.
        Source::FigureVia { .. } | Source::CameraVia { .. } => remove_bend(scope, source),
        Source::Light(_) | Source::Object(_) | Source::Wall(_) => remove_item(scope, source),
        Source::Opening { .. } => {
            let (wall, index) = opening_mut(scope, source)?;
            wall.openings.remove(index);
            Ok(())
        }
        Source::Note(id) => remove_note(scope, id),
        Source::Image(id) => images::remove_image(scope, id),
        Source::Drawing(id) => {
            let index = scope.plan.drawings.iter().position(|d| d.id == *id);
            let index = index.ok_or(ServiceError::Gone)?;
            scope.plan.drawings.remove(index);
            Ok(())
        }
        // The other grips are no element of their own: nothing to remove, but a point.
        grip => remove_grip(scope, grip),
    }
}

/// Takes a lamp, an object (and its way) or a wall out of the plan.
fn remove_item(scope: &mut Scope<'_>, source: &Source) -> Result<(), ServiceError> {
    match source {
        Source::Light(id) => {
            let index = scope.plan.lights.iter().position(|l| l.id == *id);
            let index = index.ok_or(ServiceError::Gone)?;
            scope.plan.lights.remove(index);
            Ok(())
        }
        Source::Object(id) => {
            let set = scope.set.as_deref_mut().ok_or(ServiceError::Gone)?;
            let index = set.objects.iter().position(|o| o.id == *id);
            let index = index.ok_or(ServiceError::Gone)?;
            set.objects.remove(index);
            Ok(())
        }
        Source::Wall(index) => {
            let set = scope.set.as_deref_mut().ok_or(ServiceError::Gone)?;
            if *index >= set.walls.len() {
                return Err(ServiceError::Gone);
            }

            set.walls.remove(*index);
            Ok(())
        }
        _ => Ok(()),
    }
}

fn remove_mark(plan: &mut FloorPlan, character: &str, index: usize) -> Result<(), ServiceError> {
    let track = plan
        .figures
        .iter_mut()
        .find(|f| f.character == character)
        .ok_or(ServiceError::Gone)?;
    if index >= track.marks.len() {
        return Err(ServiceError::Gone);
    }

    track.marks.remove(index);
    if track.marks.is_empty() {
        remove_figure(plan, character);
        return Ok(());
    }

    // The new first mark has no way to it: its bend would reappear on a later first mark.
    if index == 0 {
        track.marks[0].via.clear();
    }

    // Links to the removed mark go; links to later marks move up by one.
    plan.links.retain(|l| {
        !matches!(&l.target, LinkTarget::Mark { character: c, index: i } if c == character && *i == index)
    });
    for link in &mut plan.links {
        let LinkTarget::Mark {
            character: c,
            index: i,
        } = &mut link.target
        else {
            continue;
        };
        if c == character && *i > index {
            *i -= 1;
        }
    }

    Ok(())
}

/// The whole track, with everything that pointed at the figure.
fn remove_figure(plan: &mut FloorPlan, character: &str) {
    plan.figures.retain(|f| f.character != character);
    props::drop_carrier(plan, character);
    plan.links
        .retain(|l| !matches!(&l.target, LinkTarget::Mark { character: c, .. } if c == character));

    let names_it = |axis: &klappe_model::AxisPair| axis.a == character || axis.b == character;
    if plan.axis.as_ref().is_some_and(names_it) {
        plan.axis = None;
    }

    for camera in &mut plan.cameras {
        if camera.follows.as_deref() == Some(character) {
            camera.follows = None;
        }
    }
}

fn remove_camera(plan: &mut FloorPlan, id: &str) -> Result<(), ServiceError> {
    let index = plan
        .cameras
        .iter()
        .position(|c| c.id == id)
        .ok_or(ServiceError::Gone)?;
    if let Some(shot) = plan.shots.iter().find(|s| s.camera == id) {
        return Err(ServiceError::CameraInUse {
            camera: plan.cameras[index].label.clone(),
            shot: shot.number.clone(),
        });
    }

    plan.cameras.remove(index);
    Ok(())
}
