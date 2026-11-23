//! The breakdown's edits (DB-60, D-22): elements, the scene sheet, the elements and the
//! characters of a scene, and the element a set object stands for.

use klappe_model::{random_id, Element, ElementCategory, ElementStatus, Project, Sheet};

use crate::error::ServiceError;
use crate::field::Field;

use super::ProjectEdit;

/// The breakdown variants of `ProjectEdit`; others change nothing.
pub(super) fn apply(project: &mut Project, edit: &ProjectEdit) -> Result<(), ServiceError> {
    match edit {
        ProjectEdit::AddElement { category, name } => {
            add_element(project, *category, name).map(drop)
        }
        ProjectEdit::EditElement {
            id,
            category,
            name,
            description,
            status,
        } => edit_element(project, id, (*category, name), (description, *status)),
        ProjectEdit::RemoveElement { id } => remove_element(project, id),
        ProjectEdit::SetSheet { scene, .. } => set_sheet(project, scene, edit),
        ProjectEdit::AddSceneElement { scene, element }
        | ProjectEdit::TakeSuggestion { scene, element } => take(project, scene, element),
        ProjectEdit::RemoveSceneElement { scene, element } => {
            sheet_mut(project, scene)?.elements.retain(|e| e != element);
            Ok(())
        }
        ProjectEdit::AddElementToScene {
            scene,
            category,
            name,
        } => add_to_scene(project, scene, *category, name),
        ProjectEdit::ExcludeSuggestion { scene, element } => exclude(project, scene, element),
        ProjectEdit::AddSceneCharacter { scene, character } => {
            add_character(project, scene, character)
        }
        ProjectEdit::RemoveSceneCharacter { scene, character } => {
            sheet_mut(project, scene)?
                .characters
                .retain(|c| c != character);
            Ok(())
        }
        ProjectEdit::SetObjectElement {
            location,
            object,
            element,
        } => set_object_element(project, (location, object), element.as_deref()),
        _ => Ok(()),
    }
}

fn bad_name(name: &str) -> ServiceError {
    ServiceError::BadValue {
        field: Field::Name,
        value: name.to_owned(),
    }
}

fn sheet_mut<'a>(project: &'a mut Project, scene: &str) -> Result<&'a mut Sheet, ServiceError> {
    project
        .scenes
        .iter_mut()
        .find(|s| s.id == scene)
        .map(|s| &mut s.sheet)
        .ok_or_else(|| ServiceError::NoScene(scene.to_owned()))
}

fn check_element(project: &Project, id: &str) -> Result<(), ServiceError> {
    match project.elements.iter().any(|e| e.id == id) {
        true => Ok(()),
        false => Err(ServiceError::NoElement(id.to_owned())),
    }
}

/// The id of the element of `category` called `name`, the case of the name aside; `own` is an
/// element being edited, which does not count.
fn find_element(
    project: &Project,
    (category, name): (ElementCategory, &str),
    own: Option<&str>,
) -> Option<String> {
    let name = name.to_lowercase();

    project
        .elements
        .iter()
        .filter(|e| Some(e.id.as_str()) != own)
        .find(|e| e.category == category && e.name.to_lowercase() == name)
        .map(|e| e.id.clone())
}

/// The trimmed name, not empty.
fn name_of(name: &str) -> Result<String, ServiceError> {
    match name.trim() {
        "" => Err(bad_name("")),
        name => Ok(name.to_owned()),
    }
}

/// A new element; refused when the category has one of that name. Returns its id.
fn add_element(
    project: &mut Project,
    category: ElementCategory,
    name: &str,
) -> Result<String, ServiceError> {
    let name = name_of(name)?;
    if find_element(project, (category, &name), None).is_some() {
        return Err(bad_name(&name));
    }

    let mut id = random_id();
    while project.elements.iter().any(|e| e.id == id) {
        id = random_id();
    }
    project.elements.push(Element {
        id: id.clone(),
        category,
        name,
        description: String::new(),
        status: ElementStatus::default(),
    });

    Ok(id)
}

fn edit_element(
    project: &mut Project,
    id: &str,
    (category, name): (ElementCategory, &str),
    (description, status): (&str, ElementStatus),
) -> Result<(), ServiceError> {
    check_element(project, id)?;
    let name = name_of(name)?;
    if find_element(project, (category, &name), Some(id)).is_some() {
        return Err(bad_name(&name));
    }

    let Some(element) = project.elements.iter_mut().find(|e| e.id == id) else {
        return Ok(());
    };
    element.category = category;
    element.name = name;
    element.description = description.trim().to_owned();
    element.status = status;

    Ok(())
}

/// Takes the element out of the project, every sheet and every set object that stands for it.
fn remove_element(project: &mut Project, id: &str) -> Result<(), ServiceError> {
    check_element(project, id)?;

    project.elements.retain(|e| e.id != id);
    for scene in &mut project.scenes {
        scene.sheet.elements.retain(|e| e != id);
        scene.sheet.excluded.retain(|e| e != id);
    }
    let objects = project
        .locations
        .iter_mut()
        .flat_map(|l| &mut l.set.objects);
    for object in objects.filter(|o| o.element.as_deref() == Some(id)) {
        object.element = None;
    }

    Ok(())
}

fn set_sheet(project: &mut Project, scene: &str, edit: &ProjectEdit) -> Result<(), ServiceError> {
    let ProjectEdit::SetSheet {
        story_day,
        prestop_s,
        planned_min,
        continuity,
        note,
        ..
    } = edit
    else {
        return Ok(());
    };

    let sheet = sheet_mut(project, scene)?;
    sheet.story_day = story_day.trim().to_owned();
    sheet.prestop_s = *prestop_s;
    sheet.planned_min = *planned_min;
    sheet.continuity = continuity.trim().to_owned();
    sheet.note = note.trim().to_owned();

    Ok(())
}

/// The element goes into the scene's list, once, and is no longer turned down.
fn take(project: &mut Project, scene: &str, element: &str) -> Result<(), ServiceError> {
    check_element(project, element)?;
    let sheet = sheet_mut(project, scene)?;

    sheet.excluded.retain(|e| e != element);
    if !sheet.elements.iter().any(|e| e == element) {
        sheet.elements.push(element.to_owned());
    }

    Ok(())
}

/// The element of that name and category, made if there is none, goes into the scene.
fn add_to_scene(
    project: &mut Project,
    scene: &str,
    category: ElementCategory,
    name: &str,
) -> Result<(), ServiceError> {
    sheet_mut(project, scene)?;
    let name = name_of(name)?;

    let id = match find_element(project, (category, &name), None) {
        Some(id) => id,
        None => add_element(project, category, &name)?,
    };

    take(project, scene, &id)
}

/// A suggestion is turned down: the scene does not list it and the set does not offer it again.
fn exclude(project: &mut Project, scene: &str, element: &str) -> Result<(), ServiceError> {
    check_element(project, element)?;
    let sheet = sheet_mut(project, scene)?;

    sheet.elements.retain(|e| e != element);
    if !sheet.excluded.iter().any(|e| e == element) {
        sheet.excluded.push(element.to_owned());
    }

    Ok(())
}

fn add_character(project: &mut Project, scene: &str, character: &str) -> Result<(), ServiceError> {
    if project.character(character).is_none() {
        return Err(ServiceError::NoCharacter(character.to_owned()));
    }
    let sheet = sheet_mut(project, scene)?;

    if !sheet.characters.iter().any(|c| c == character) {
        sheet.characters.push(character.to_owned());
    }

    Ok(())
}

/// The object of the location's set stands for `element`, or for none.
fn set_object_element(
    project: &mut Project,
    (location, object): (&str, &str),
    element: Option<&str>,
) -> Result<(), ServiceError> {
    if let Some(id) = element {
        check_element(project, id)?;
    }

    let found = project
        .locations
        .iter_mut()
        .find(|l| l.id == location)
        .ok_or_else(|| ServiceError::NoLocation(location.to_owned()))?
        .set
        .objects
        .iter_mut()
        .find(|o| o.id == object)
        .ok_or(ServiceError::Gone)?;
    found.element = element.map(str::to_owned);

    Ok(())
}
