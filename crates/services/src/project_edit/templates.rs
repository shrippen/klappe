//! Filing a plan as a template, and making the character a placeholder stands for (LH-07).

use klappe_model::{
    Camera, CameraMark, FigureColor, FloorPlan, Mark, Point, Project, Template, TemplateFigure,
};
use klappe_store::{builtin_templates, check_template};

use super::{add_character, name_of, slug, unique_id, ProjectEdit};
use crate::error::ServiceError;
use crate::field::Field;
use crate::plan_edit::template::place;

const TEMPLATE_ID_FALLBACK: &str = "vorlage";

/// Places are filed to the centimetre; more would only make the file noisy.
const FILED_STEP_M: f64 = 0.01;

/// Adds a character named `name`, in a colour no character has yet if there is one; returns
/// its id.
pub(crate) fn new_character(project: &mut Project, name: &str) -> Result<String, ServiceError> {
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
        .map(|m| (m.at, m.via))
        .chain(cameras.map(|m| (m.at, m.via)));

    marks
        .flat_map(|(at, via)| std::iter::once(at).chain(via))
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
        via: mark.via.map(|via| filed(via, back)),
        ..mark.clone()
    }
}

fn filed_camera_mark(mark: &CameraMark, back: Point) -> CameraMark {
    CameraMark {
        at: filed(mark.at, back),
        via: mark.via.map(|via| filed(via, back)),
        ..mark.clone()
    }
}

/// What `ProjectEdit::PlaceTemplate` and `ProjectEdit::SaveTemplate` do.
pub(super) fn apply(project: &mut Project, edit: &ProjectEdit) -> Result<(), ServiceError> {
    match edit {
        ProjectEdit::PlaceTemplate {
            scene,
            plan,
            template,
            at,
        } => place(project, (scene, plan), template, *at),
        ProjectEdit::SaveTemplate { scene, plan, name } => save(project, scene, plan, name),
        _ => Ok(()),
    }
}
