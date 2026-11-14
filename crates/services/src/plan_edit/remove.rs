//! Removing elements: a figure takes what points at it along; a camera a shot uses stays.

use klappe_model::{FloorPlan, LinkTarget};
use klappe_stellprobe::Source;

use super::{straighten, Scope};
use crate::error::ServiceError;

pub(super) fn remove(scope: &mut Scope<'_>, source: &Source) -> Result<(), ServiceError> {
    match source {
        Source::Mark { character, index } => remove_mark(scope.plan, character, *index),
        Source::Figure(character) => {
            let known = scope.plan.figure(character).is_some();
            if !known {
                return Err(ServiceError::Gone);
            }

            remove_figure(scope.plan, character);
            Ok(())
        }
        Source::Camera(id) => remove_camera(scope.plan, id),
        // A grip is no element of its own: removing it takes the bend out.
        Source::FigureVia { .. } | Source::CameraVia { .. } => straighten(scope, source),
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
        Source::Note(id) => {
            let set = scope.set.as_deref_mut().ok_or(ServiceError::Gone)?;
            let index = set.notes.iter().position(|n| n.id == *id);
            let index = index.ok_or(ServiceError::Gone)?;
            set.notes.remove(index);
            Ok(())
        }
        Source::Drawing(id) => {
            let index = scope.plan.drawings.iter().position(|d| d.id == *id);
            let index = index.ok_or(ServiceError::Gone)?;
            scope.plan.drawings.remove(index);
            Ok(())
        }
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
        track.marks[0].via = None;
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
