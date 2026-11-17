//! Placing a template on the plan, filing a plan as a template, and making the character a
//! placeholder stands for (LH-07).

use klappe_model::{
    Camera, CameraMark, Character, FigureColor, FigureTrack, FloorPlan, Mark, Point, Project,
    Template, TemplateFigure,
};
use klappe_store::{builtin_templates, check_template};

use super::{add_character, name_of, slug, unique_id, ProjectEdit};
use crate::error::ServiceError;
use crate::field::Field;
use crate::plan_edit::free_letter;
use crate::templates;

const TEMPLATE_ID_FALLBACK: &str = "vorlage";

/// Places are filed to the centimetre; more would only make the file noisy.
const FILED_STEP_M: f64 = 0.01;

/// Adds a character named `name`, in a colour no character has yet if there is one; returns
/// its id.
fn new_character(project: &mut Project, name: &str) -> Result<String, ServiceError> {
    let initial: String = name
        .trim()
        .chars()
        .take(1)
        .flat_map(char::to_uppercase)
        .collect();
    let color = FigureColor::ALL
        .into_iter()
        .find(|c| !project.characters.iter().any(|o| o.color == *c))
        .unwrap_or(FigureColor::ALL[0]);

    add_character(project, name, &initial, color, (None, None))?;

    project
        .characters
        .last()
        .map(|c| c.id.clone())
        .ok_or(ServiceError::Gone)
}

/// Puts the figures and cameras of the template `id` on the plan `plan` of the scene `scene`,
/// `at` being the template's insertion point. A placeholder takes the first character the
/// plan has no figure of yet; when none is left, a character of the placeholder's name is
/// made (`A`, or `A 2` when a character has that name). Cameras get free letters and follow the figure of their placeholder.
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
            None => {
                let name = free_name(&project.characters, &figure.slot);
                new_character(project, &name)?
            }
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

fn on_plan(project: &Project, scene: usize, plan: usize) -> Result<&FloorPlan, ServiceError> {
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
        via: mark.via.iter().map(|via| moved(*via, by)).collect(),
        ..mark.clone()
    }
}

fn moved_camera(mark: &CameraMark, by: Point) -> CameraMark {
    CameraMark {
        at: moved(mark.at, by),
        via: mark.via.iter().map(|via| moved(*via, by)).collect(),
        ..mark.clone()
    }
}

/// `base`, or `base 2`, `base 3` … when a character has that name; compared in capitals, as
/// the cues are.
pub(super) fn free_name(characters: &[Character], base: &str) -> String {
    let taken = |name: &str| {
        characters
            .iter()
            .any(|c| c.name.to_uppercase() == name.to_uppercase())
    };

    let mut name = base.to_owned();
    let mut n = 1;
    while taken(&name) {
        n += 1;
        name = format!("{base} {n}");
    }

    name
}

/// Files the figures and cameras of the plan `plan` of the scene `scene` under `name` in the
/// project's templates. The insertion point is the middle of the box that holds all their
/// marks. A camera that followed a figure of the plan follows its placeholder.
fn save(project: &mut Project, scene: &str, plan: &str, name: &str) -> Result<(), ServiceError> {
    let name = name_of(name)?;
    let source = project
        .scene(scene)
        .ok_or_else(|| ServiceError::NoScene(scene.to_owned()))?
        .plans
        .iter()
        .find(|p| p.id == plan)
        .ok_or_else(|| ServiceError::NoPlan(plan.to_owned()))?;

    let Some(origin) = box_middle(&plan_points(source)) else {
        return Err(ServiceError::EmptyTemplate);
    };
    let back = Point::new(-origin.x_m, -origin.y_m);

    let mut slots: Vec<(String, String)> = Vec::new();
    let mut figures = Vec::new();
    for track in &source.figures {
        let base = project
            .character(&track.character)
            .map_or(track.character.as_str(), |c| c.name.as_str());
        let slot = free_slot(base, slots.iter().map(|(_, s)| s.as_str()));

        slots.push((track.character.clone(), slot.clone()));
        figures.push(TemplateFigure {
            slot,
            marks: track.marks.iter().map(|m| filed_mark(m, back)).collect(),
        });
    }

    let cameras = source
        .cameras
        .iter()
        .map(|camera| Camera {
            marks: camera
                .marks
                .iter()
                .map(|m| filed_camera_mark(m, back))
                .collect(),
            follows: camera.follows.as_ref().and_then(|character| {
                let found = slots.iter().find(|(id, _)| id == character);
                found.map(|(_, slot)| slot.clone())
            }),
            ..camera.clone()
        })
        .collect();

    let taken = project
        .templates
        .iter()
        .chain(builtin_templates())
        .map(|t| t.id.as_str());
    let id = unique_id(&slug(&name, TEMPLATE_ID_FALLBACK), taken);

    let template = Template {
        id,
        label: name,
        figures,
        cameras,
    };

    // The project file would refuse it: far places, a rounding that overflows.
    check_template(&template).map_err(|value| ServiceError::BadValue {
        field: Field::Point,
        value,
    })?;
    project.templates.push(template);

    Ok(())
}

/// `base`, or `base 2`, `base 3` … when `taken` has it.
fn free_slot<'a>(base: &str, taken: impl Iterator<Item = &'a str> + Clone) -> String {
    let mut slot = base.to_owned();
    let mut n = 1;

    while taken.clone().any(|t| t == slot) {
        n += 1;
        slot = format!("{base} {n}");
    }

    slot
}

/// Every place a mark or its bend has.
fn plan_points(plan: &FloorPlan) -> Vec<Point> {
    let figures = plan.figures.iter().flat_map(|f| &f.marks);
    let cameras = plan.cameras.iter().flat_map(|c| &c.marks);
    let marks = figures
        .map(|m| (m.at, &m.via))
        .chain(cameras.map(|m| (m.at, &m.via)));

    marks
        .flat_map(|(at, via)| std::iter::once(at).chain(via.iter().copied()))
        .collect()
}

fn box_middle(points: &[Point]) -> Option<Point> {
    let mut found: Option<(Point, Point)> = None;

    for &p in points {
        let (min, max) = found.unwrap_or((p, p));
        found = Some((
            Point::new(min.x_m.min(p.x_m), min.y_m.min(p.y_m)),
            Point::new(max.x_m.max(p.x_m), max.y_m.max(p.y_m)),
        ));
    }

    found.map(|(min, max)| min.lerp(max, 0.5))
}

fn filed(point: Point, back: Point) -> Point {
    let step = |v: f64| (v / FILED_STEP_M).round() * FILED_STEP_M;
    Point::new(step(point.x_m + back.x_m), step(point.y_m + back.y_m))
}

fn filed_mark(mark: &Mark, back: Point) -> Mark {
    Mark {
        at: filed(mark.at, back),
        via: mark.via.iter().map(|via| filed(*via, back)).collect(),
        ..mark.clone()
    }
}

fn filed_camera_mark(mark: &CameraMark, back: Point) -> CameraMark {
    CameraMark {
        at: filed(mark.at, back),
        via: mark.via.iter().map(|via| filed(*via, back)).collect(),
        ..mark.clone()
    }
}

fn rename(project: &mut Project, id: &str, name: &str) -> Result<(), ServiceError> {
    let name = name_of(name)?;
    let template = project
        .templates
        .iter_mut()
        .find(|t| t.id == id)
        .ok_or(ServiceError::Gone)?;

    template.label = name;
    Ok(())
}

fn remove(project: &mut Project, id: &str) -> Result<(), ServiceError> {
    let before = project.templates.len();

    project.templates.retain(|t| t.id != id);
    if project.templates.len() == before {
        return Err(ServiceError::Gone);
    }
    Ok(())
}

/// Whether the templates are the same but for their ids.
fn same_content(a: &Template, b: &Template) -> bool {
    a.label == b.label && a.figures == b.figures && a.cameras == b.cameras
}

/// Takes the templates in; one that is there already is skipped, so a file passed on twice
/// adds nothing. Checked as a whole first: one unusable template changes nothing.
fn import(project: &mut Project, items: &[Template]) -> Result<(), ServiceError> {
    for item in items {
        check_template(item).map_err(|value| ServiceError::BadValue {
            field: Field::Point,
            value,
        })?;
    }

    for item in items {
        if project.templates.iter().any(|t| same_content(t, item)) {
            continue;
        }

        let taken = project
            .templates
            .iter()
            .chain(builtin_templates())
            .map(|t| t.id.as_str());
        let id = unique_id(&slug(&item.id, TEMPLATE_ID_FALLBACK), taken);

        project.templates.push(Template { id, ..item.clone() });
    }
    Ok(())
}

/// What the template edits of `ProjectEdit` do.
pub(super) fn apply(project: &mut Project, edit: &ProjectEdit) -> Result<(), ServiceError> {
    match edit {
        ProjectEdit::PlaceTemplate {
            scene,
            plan,
            template,
            at,
        } => place(project, (scene, plan), template, *at),
        ProjectEdit::SaveTemplate { scene, plan, name } => save(project, scene, plan, name),
        ProjectEdit::EditTemplate { id, name } => rename(project, id, name),
        ProjectEdit::RemoveTemplate { id } => remove(project, id),
        ProjectEdit::ImportTemplates { items } => import(project, items),
        _ => Ok(()),
    }
}
