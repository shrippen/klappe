//! The breakdown's edits (DB-60, D-22): elements, the scene sheet, the elements and the
//! characters of a scene, and the element a set object stands for.

use klappe_model::{
    free_random_id, Element, ElementCategory, ElementStatus, IdCase, Project, Sheet, TextMark,
};

use crate::error::ServiceError;
use crate::field::Field;
use crate::sheet_time::{read_planned, read_prestop};

/// Edits of the breakdown: elements and scene sheets (DB-60, D-22).
#[derive(Debug, Clone, PartialEq)]
pub enum BreakdownEdit {
    /// A new element of the breakdown (DB-60, D-22). Refused: an empty name, or one the
    /// category already has.
    AddElement {
        category: ElementCategory,
        name: String,
    },
    EditElement {
        id: String,
        category: ElementCategory,
        name: String,
        description: String,
        status: ElementStatus,
    },
    /// Takes the element out of every scene sheet and every set object too.
    RemoveElement {
        id: String,
    },
    /// Sets what the scene sheet says besides its lists.
    SetSheet {
        scene: String,
        story_day: String,
        /// As typed: `m:ss` or seconds, empty for none.
        prestop: String,
        /// As typed: `h:mm` or minutes, empty for none.
        planned: String,
        continuity: String,
        note: String,
    },
    AddSceneElement {
        scene: String,
        element: String,
    },
    RemoveSceneElement {
        scene: String,
        element: String,
    },
    /// The field with suggestions: the element of that name and category, made if there is
    /// none, goes into the scene.
    AddElementToScene {
        scene: String,
        category: ElementCategory,
        name: String,
    },
    /// Turns a suggestion of the set down for this scene.
    ExcludeSuggestion {
        scene: String,
        element: String,
    },
    /// Takes a suggestion of the set into the scene.
    TakeSuggestion {
        scene: String,
        element: String,
    },
    /// A character without a line in the scene (silent, extra) is in it. Refused for one that
    /// speaks: the book says so already.
    AddSceneCharacter {
        scene: String,
        character: String,
    },
    RemoveSceneCharacter {
        scene: String,
        character: String,
    },
    /// The object `object` of the set at `location` stands for `element` (`None`: for none).
    SetObjectElement {
        location: String,
        object: String,
        element: Option<String>,
    },
    /// Words of the scene's text are marked as an element (D-23), which goes into the scene.
    /// The session checks that the words stand there; `mark.element` is not read.
    MarkText {
        scene: String,
        mark: TextMark,
        target: MarkTarget,
    },
    /// The marks are dropped from the scene; the elements stay in it.
    ForgetMarks {
        scene: String,
        marks: Vec<TextMark>,
    },
}

/// What marked words become.
#[derive(Debug, Clone, PartialEq)]
pub enum MarkTarget {
    /// The element of this id.
    Element(String),
    /// The element of this category named like the words, made if there is none.
    New(ElementCategory),
}

/// What the breakdown edits do.
pub(super) fn apply(project: &mut Project, edit: &BreakdownEdit) -> Result<(), ServiceError> {
    match edit {
        BreakdownEdit::AddElement { category, name } => {
            add_element(project, *category, name).map(drop)
        }
        BreakdownEdit::EditElement {
            id,
            category,
            name,
            description,
            status,
        } => edit_element(project, id, (*category, name), (description, *status)),
        BreakdownEdit::RemoveElement { id } => remove_element(project, id),
        BreakdownEdit::SetSheet {
            scene,
            story_day,
            prestop,
            planned,
            continuity,
            note,
        } => set_sheet(
            project,
            scene,
            SheetText {
                story_day,
                prestop,
                planned,
                continuity,
                note,
            },
        ),
        BreakdownEdit::AddSceneElement { scene, element }
        | BreakdownEdit::TakeSuggestion { scene, element } => take(project, scene, element),
        BreakdownEdit::RemoveSceneElement { scene, element } => leave(project, scene, element),
        BreakdownEdit::AddElementToScene {
            scene,
            category,
            name,
        } => add_to_scene(project, scene, *category, name),
        BreakdownEdit::ExcludeSuggestion { scene, element } => exclude(project, scene, element),
        BreakdownEdit::AddSceneCharacter { scene, character } => {
            add_character(project, scene, character)
        }
        BreakdownEdit::RemoveSceneCharacter { scene, character } => {
            sheet_mut(project, scene)?
                .characters
                .retain(|c| c != character);
            Ok(())
        }
        BreakdownEdit::SetObjectElement {
            location,
            object,
            element,
        } => set_object_element(project, (location, object), element.as_deref()),
        BreakdownEdit::MarkText {
            scene,
            mark,
            target,
        } => mark_text(project, scene, mark, target),
        BreakdownEdit::ForgetMarks { scene, marks } => forget(project, scene, marks),
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

    let id = free_random_id(
        project.elements.iter().map(|e| e.id.as_str()),
        IdCase::Exact,
    );
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
        scene.sheet.marks.retain(|m| m.element != id);
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

/// What `SetSheet` says; the text is trimmed when it is stored.
struct SheetText<'a> {
    story_day: &'a str,
    prestop: &'a str,
    planned: &'a str,
    continuity: &'a str,
    note: &'a str,
}

fn set_sheet(project: &mut Project, scene: &str, text: SheetText) -> Result<(), ServiceError> {
    let prestop_s = read_prestop(text.prestop)?;
    let planned_min = read_planned(text.planned)?;
    let sheet = sheet_mut(project, scene)?;
    sheet.story_day = text.story_day.trim().to_owned();
    sheet.prestop_s = prestop_s;
    sheet.planned_min = planned_min;
    sheet.continuity = text.continuity.trim().to_owned();
    sheet.note = text.note.trim().to_owned();

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
    let id = found_or_new(project, category, name)?;

    take(project, scene, &id)
}

/// The id of the element of that name and category, made if there is none.
fn found_or_new(
    project: &mut Project,
    category: ElementCategory,
    name: &str,
) -> Result<String, ServiceError> {
    let name = name_of(name)?;

    match find_element(project, (category, &name), None) {
        Some(id) => Ok(id),
        None => add_element(project, category, &name),
    }
}

/// The element leaves the scene's list, its marks there with it.
fn leave(project: &mut Project, scene: &str, element: &str) -> Result<(), ServiceError> {
    let sheet = sheet_mut(project, scene)?;
    sheet.elements.retain(|e| e != element);
    sheet.marks.retain(|m| m.element != element);

    Ok(())
}

fn forget(project: &mut Project, scene: &str, marks: &[TextMark]) -> Result<(), ServiceError> {
    sheet_mut(project, scene)?
        .marks
        .retain(|m| !marks.contains(m));

    Ok(())
}

/// The words become the target element, which goes into the scene; the same mark twice is
/// kept once.
fn mark_text(
    project: &mut Project,
    scene: &str,
    mark: &TextMark,
    target: &MarkTarget,
) -> Result<(), ServiceError> {
    let element = match target {
        MarkTarget::Element(id) => id.clone(),
        MarkTarget::New(category) => found_or_new(project, *category, &mark.text)?,
    };
    take(project, scene, &element)?;

    let mark = TextMark {
        element,
        ..mark.clone()
    };
    let sheet = sheet_mut(project, scene)?;
    if !sheet.marks.contains(&mark) {
        sheet.marks.push(mark);
    }

    Ok(())
}

/// A suggestion is turned down: the scene does not list it and the set does not offer it again.
fn exclude(project: &mut Project, scene: &str, element: &str) -> Result<(), ServiceError> {
    check_element(project, element)?;
    let sheet = sheet_mut(project, scene)?;

    sheet.elements.retain(|e| e != element);
    sheet.marks.retain(|m| m.element != element);
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
