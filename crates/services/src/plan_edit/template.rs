//! Placing a template on the plan (LH-07).

use klappe_model::{Camera, CameraMark, FigureTrack, Mark, Point, Project};

use super::free_letter;
use crate::error::ServiceError;
use crate::field::Field;
use crate::project_edit::templates::new_character;
use crate::templates;

/// Puts the figures and cameras of the template `id` on the plan `plan` of the scene `scene`,
/// `at` being the template's insertion point. A placeholder takes the first character the
/// plan has no figure of yet; when none is left, a character of the placeholder's name is
/// made. Cameras get free letters and follow the figure of their placeholder.
pub(crate) fn place(
    project: &mut Project,
    (scene, plan): (&str, &str),
    id: &str,
    at: Point,
) -> Result<(), ServiceError> {
    if !at.x_m.is_finite() || !at.y_m.is_finite() {
        return Err(ServiceError::BadValue {
            field: Field::Point,
            value: format!("{} {}", at.x_m, at.y_m),
        });
    }

    let template = templates::find(&project.templates, id)
        .cloned()
        .ok_or_else(|| ServiceError::BadValue {
            field: Field::Kind,
            value: id.to_owned(),
        })?;
    let (scene, plan) = indices(project, scene, plan)?;
    let mut taken = on_plan(project, scene, plan)?
        .figures
        .iter()
        .map(|f| f.character.clone())
        .collect::<Vec<_>>();
    let mut bound: Vec<(&str, String)> = Vec::new();

    for figure in &template.figures {
        let free = project.characters.iter().find(|c| !taken.contains(&c.id));
        let character = match free {
            Some(character) => character.id.clone(),
            None => new_character(project, &figure.slot)?,
        };

        taken.push(character.clone());
        bound.push((figure.slot.as_str(), character));
    }

    let target = project
        .scenes
        .get_mut(scene)
        .and_then(|s| s.plans.get_mut(plan))
        .ok_or(ServiceError::Gone)?;

    for (figure, (_, character)) in template.figures.iter().zip(&bound) {
        target.figures.push(FigureTrack {
            character: character.clone(),
            marks: figure.marks.iter().map(|m| moved_mark(m, at)).collect(),
        });
    }

    for camera in &template.cameras {
        let letter = free_letter(&target.cameras)?;
        let follows = camera.follows.as_ref().and_then(|slot| {
            let found = bound.iter().find(|(name, _)| *name == slot.as_str());
            found.map(|(_, character)| character.clone())
        });

        target.cameras.push(Camera {
            id: letter.to_ascii_lowercase().to_string(),
            label: letter.to_string(),
            marks: camera.marks.iter().map(|m| moved_camera(m, at)).collect(),
            follows,
            ..camera.clone()
        });
    }

    Ok(())
}

/// Where the plan `plan` of the scene `scene` is in the project.
fn indices(project: &Project, scene: &str, plan: &str) -> Result<(usize, usize), ServiceError> {
    let at = project
        .scenes
        .iter()
        .position(|s| s.id == scene)
        .ok_or_else(|| ServiceError::NoScene(scene.to_owned()))?;
    let found = project.scenes[at].plans.iter().position(|p| p.id == plan);

    Ok((
        at,
        found.ok_or_else(|| ServiceError::NoPlan(plan.to_owned()))?,
    ))
}

fn on_plan(
    project: &Project,
    scene: usize,
    plan: usize,
) -> Result<&klappe_model::FloorPlan, ServiceError> {
    project
        .scenes
        .get(scene)
        .and_then(|s| s.plans.get(plan))
        .ok_or(ServiceError::Gone)
}

fn moved(point: Point, by: Point) -> Point {
    Point::new(point.x_m + by.x_m, point.y_m + by.y_m)
}

fn moved_mark(mark: &Mark, by: Point) -> Mark {
    Mark {
        at: moved(mark.at, by),
        via: mark.via.map(|via| moved(via, by)),
        ..mark.clone()
    }
}

fn moved_camera(mark: &CameraMark, by: Point) -> CameraMark {
    CameraMark {
        at: moved(mark.at, by),
        via: mark.via.map(|via| moved(via, by)),
        ..mark.clone()
    }
}
