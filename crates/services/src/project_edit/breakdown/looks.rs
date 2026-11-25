//! The looks of the breakdown: a character's named costume and make-up, and the look it wears
//! in a scene.

use klappe_model::{free_random_id, ElementCategory, IdCase, Look, Project};

use super::{bad_name, check_element, name_of, sheet_mut, BreakdownEdit};
use crate::error::ServiceError;

/// The categories whose elements a look is made of.
const LOOK_CATEGORIES: [ElementCategory; 2] = [ElementCategory::Costume, ElementCategory::Makeup];

/// What the look edits do; the other edits of the breakdown are `super::apply`'s.
pub(super) fn apply(project: &mut Project, edit: &BreakdownEdit) -> Result<(), ServiceError> {
    match edit {
        BreakdownEdit::AddLook { character, name } => add_look(project, character, name),
        BreakdownEdit::EditLook { id, name } => edit_look(project, id, name),
        BreakdownEdit::RemoveLook { id } => remove_look(project, id),
        BreakdownEdit::AddLookElement { look, element } => add_look_element(project, look, element),
        BreakdownEdit::RemoveLookElement { look, element } => {
            remove_look_element(project, look, element)
        }
        BreakdownEdit::SetSceneLook {
            scene,
            character,
            look,
        } => set_scene_look(project, scene, character, look.as_deref()),
        _ => Ok(()),
    }
}

fn look_mut<'a>(project: &'a mut Project, id: &str) -> Result<&'a mut Look, ServiceError> {
    project
        .looks
        .iter_mut()
        .find(|l| l.id == id)
        .ok_or_else(|| ServiceError::NoLook(id.to_owned()))
}

/// Whether `character` has a look called `name`, the case aside; `own` is a look being
/// renamed, which does not count.
fn name_taken(project: &Project, character: &str, name: &str, own: Option<&str>) -> bool {
    let name = name.to_lowercase();

    project
        .looks
        .iter()
        .filter(|l| Some(l.id.as_str()) != own)
        .any(|l| l.character == character && l.name.to_lowercase() == name)
}

/// A new look without elements; refused when the character has one of that name.
fn add_look(project: &mut Project, character: &str, name: &str) -> Result<(), ServiceError> {
    if project.character(character).is_none() {
        return Err(ServiceError::NoCharacter(character.to_owned()));
    }
    let name = name_of(name)?;
    if name_taken(project, character, &name, None) {
        return Err(bad_name(&name));
    }

    let id = free_random_id(project.looks.iter().map(|l| l.id.as_str()), IdCase::Exact);
    project.looks.push(Look {
        id,
        character: character.to_owned(),
        name,
        elements: Vec::new(),
    });

    Ok(())
}

fn edit_look(project: &mut Project, id: &str, name: &str) -> Result<(), ServiceError> {
    let character = look_mut(project, id)?.character.clone();
    let name = name_of(name)?;
    if name_taken(project, &character, &name, Some(id)) {
        return Err(bad_name(&name));
    }

    look_mut(project, id)?.name = name;

    Ok(())
}

/// The look goes, and every sheet that gave it to a character is without one for it.
fn remove_look(project: &mut Project, id: &str) -> Result<(), ServiceError> {
    look_mut(project, id)?;

    project.looks.retain(|l| l.id != id);
    for scene in &mut project.scenes {
        scene.sheet.looks.retain(|_, look| look != id);
    }

    Ok(())
}

/// Takes the element out of every look unless `category` is one a look is made of: a prop in a
/// look would show as a prop in the extract.
pub(super) fn fit_looks(project: &mut Project, element: &str, category: ElementCategory) {
    if LOOK_CATEGORIES.contains(&category) {
        return;
    }

    for look in &mut project.looks {
        look.elements.retain(|e| e != element);
    }
}

/// The element goes into the look, once. Refused for any category but costume and make-up.
fn add_look_element(project: &mut Project, look: &str, element: &str) -> Result<(), ServiceError> {
    look_mut(project, look)?;
    check_element(project, element)?;

    let found = project.elements.iter().find(|e| e.id == element);
    if let Some(e) = found.filter(|e| !LOOK_CATEGORIES.contains(&e.category)) {
        return Err(ServiceError::NotLookElement(e.name.clone()));
    }

    let look = look_mut(project, look)?;
    if !look.elements.iter().any(|e| e == element) {
        look.elements.push(element.to_owned());
    }

    Ok(())
}

fn remove_look_element(
    project: &mut Project,
    look: &str,
    element: &str,
) -> Result<(), ServiceError> {
    look_mut(project, look)?.elements.retain(|e| e != element);

    Ok(())
}

/// The character wears `look` in the scene, or none. Refused: a look that is not this character's.
/// That the character is in the scene is the session's check (it knows the book).
fn set_scene_look(
    project: &mut Project,
    scene: &str,
    character: &str,
    look: Option<&str>,
) -> Result<(), ServiceError> {
    if let Some(id) = look {
        // Another character's look is none to this one.
        if look_mut(project, id)?.character != character {
            return Err(ServiceError::NoLook(id.to_owned()));
        }
    }

    let looks = &mut sheet_mut(project, scene)?.looks;
    match look {
        Some(id) => looks.insert(character.to_owned(), id.to_owned()),
        None => looks.remove(character),
    };

    Ok(())
}
