//! A scene's floor plans as project edits (LH-04): add, copy, rename, move, remove.

use klappe_model::{FloorPlan, Project, Scene};

use super::{slug, unique_id};
use crate::error::ServiceError;
use crate::field::Field;
use crate::plan_edit::unbind_plan;

const PLAN_ID_FALLBACK: &str = "plan";
const PLAN_NAME_PREFIX: &str = "Plan";
const PLAN_LETTERS: std::ops::RangeInclusive<char> = 'A'..='Z';

/// A copy of "Plan A" is named "Plan A (2)", the next "Plan A (3)".
const COPY_FIRST_NUMBER: usize = 2;

/// An empty plan in the scene at `scene`, named `name` (empty: the next free `Plan B` …);
/// the index of the new plan.
pub(crate) fn add_plan(
    project: &mut Project,
    scene: usize,
    name: &str,
) -> Result<usize, ServiceError> {
    let scene = scene_at(project, scene)?;
    let name = match name.trim() {
        "" => free_letter_name(scene),
        given => given.to_owned(),
    };
    check_plan_name(scene, &name, None)?;

    let plan = FloorPlan::new(&plan_id(scene, &name), &name);
    scene.plans.push(plan);
    Ok(scene.plans.len() - 1)
}

/// A snapshot (LH-04): a copy of the plan at `plan`, with an id of its own, right after it;
/// named `name` (empty: `Plan A (2)` …). The index of the copy.
pub(crate) fn copy_plan(
    project: &mut Project,
    scene: usize,
    plan: usize,
    name: &str,
) -> Result<usize, ServiceError> {
    let scene = scene_at(project, scene)?;
    let source = scene.plans.get(plan).ok_or(ServiceError::Gone)?;
    let name = match name.trim() {
        "" => copy_name(scene, &source.name),
        given => given.to_owned(),
    };
    check_plan_name(scene, &name, None)?;

    let mut copy = source.clone();
    copy.id = plan_id(scene, &name);
    copy.name = name;
    scene.plans.insert(plan + 1, copy);
    Ok(plan + 1)
}

/// The id stays; only the name the user sees changes.
pub(crate) fn rename_plan(
    project: &mut Project,
    scene: usize,
    id: &str,
    name: &str,
) -> Result<(), ServiceError> {
    let scene = scene_at(project, scene)?;
    let name = name.trim();
    if name.is_empty() {
        return Err(ServiceError::BadValue {
            field: Field::Name,
            value: String::new(),
        });
    }
    check_plan_name(scene, name, Some(id))?;

    let plan = plan_of(scene, id)?;
    plan.name = name.to_owned();
    Ok(())
}

/// Moves the plan `id` one place; at the first or last place nothing changes.
pub(crate) fn move_plan(
    project: &mut Project,
    scene: usize,
    id: &str,
    shift: PlanShift,
) -> Result<(), ServiceError> {
    let scene = scene_at(project, scene)?;
    let from = scene
        .plans
        .iter()
        .position(|p| p.id == id)
        .ok_or_else(|| ServiceError::NoPlan(id.to_owned()))?;
    let to = match shift {
        PlanShift::Earlier => from.checked_sub(1),
        PlanShift::Later => Some(from + 1).filter(|&to| to < scene.plans.len()),
    };
    if let Some(to) = to {
        scene.plans.swap(from, to);
    }
    Ok(())
}

/// Which way a plan moves in its scene's list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlanShift {
    Earlier,
    Later,
}

/// What deleting a plan does with the shots it holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShotsGo {
    /// The plan stays: deleting it is refused.
    Refuse,
    /// The shots go with the plan.
    Delete,
}

/// Refused for the scene's last plan and, with `ShotsGo::Refuse`, for a plan with shots.
pub(crate) fn remove_plan(
    project: &mut Project,
    index: usize,
    id: &str,
    shots_go: ShotsGo,
) -> Result<(), ServiceError> {
    let scene = scene_at(project, index)?;
    let shots = plan_of(scene, id)?.shots.len();
    if scene.plans.len() == 1 {
        return Err(ServiceError::LastPlan(id.to_owned()));
    }
    if shots > 0 && shots_go == ShotsGo::Refuse {
        return Err(ServiceError::PlanHasShots {
            id: id.to_owned(),
            shots,
        });
    }

    scene.plans.retain(|p| p.id != id);
    unbind_plan(project, index, Some(id));
    Ok(())
}

fn scene_at(project: &mut Project, scene: usize) -> Result<&mut Scene, ServiceError> {
    project.scenes.get_mut(scene).ok_or(ServiceError::Gone)
}

fn plan_of<'a>(scene: &'a mut Scene, id: &str) -> Result<&'a mut FloorPlan, ServiceError> {
    scene
        .plans
        .iter_mut()
        .find(|p| p.id == id)
        .ok_or_else(|| ServiceError::NoPlan(id.to_owned()))
}

fn plan_id(scene: &Scene, name: &str) -> String {
    unique_id(
        &slug(name, PLAN_ID_FALLBACK),
        scene.plans.iter().map(|p| p.id.as_str()),
    )
}

/// Names differ without regard to case: `Plan a` next to `Plan A` is no variant. `own` is the
/// plan being renamed, which may keep its name.
fn check_plan_name(scene: &Scene, name: &str, own: Option<&str>) -> Result<(), ServiceError> {
    let taken = scene
        .plans
        .iter()
        .filter(|p| Some(p.id.as_str()) != own)
        .any(|p| p.name.to_lowercase() == name.to_lowercase());
    if taken {
        return Err(ServiceError::PlanNameTaken(name.to_owned()));
    }
    Ok(())
}

fn is_name_free(scene: &Scene, name: &str) -> bool {
    check_plan_name(scene, name, None).is_ok()
}

/// `Plan A`, else `Plan B` … the first letter no plan of the scene has; `Plan 28` after Z.
fn free_letter_name(scene: &Scene) -> String {
    let named = |letter: char| format!("{PLAN_NAME_PREFIX} {letter}");

    PLAN_LETTERS
        .map(named)
        .find(|name| is_name_free(scene, name))
        .unwrap_or_else(|| format!("{PLAN_NAME_PREFIX} {}", scene.plans.len() + 1))
}

/// `Plan A (2)` for a copy of `Plan A` or of `Plan A (2)`: the first number not taken.
fn copy_name(scene: &Scene, source: &str) -> String {
    let base = strip_copy_number(source);

    (COPY_FIRST_NUMBER..)
        .map(|n| format!("{base} ({n})"))
        .find(|name| is_name_free(scene, name))
        .unwrap_or_default()
}

/// `Plan A (2)` → `Plan A`; names without a trailing ` (n)` stay.
fn strip_copy_number(name: &str) -> &str {
    let Some(head) = name.strip_suffix(')') else {
        return name;
    };
    let Some((base, number)) = head.rsplit_once(" (") else {
        return name;
    };
    match !number.is_empty() && number.chars().all(|c| c.is_ascii_digit()) {
        true => base,
        false => name,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copy_numbers_are_stripped() {
        assert_eq!(strip_copy_number("Plan A (2)"), "Plan A");
        assert_eq!(strip_copy_number("Plan A"), "Plan A");
        assert_eq!(strip_copy_number("Plan (alt)"), "Plan (alt)");
        assert_eq!(strip_copy_number("Plan ()"), "Plan ()");
    }
}
