//! The other names of a character and the cues that are no character (DB-50), as the
//! character manager changes them. A name belongs to one character or to none: a name taken
//! elsewhere is refused, as when renaming.

use klappe_model::{same_character_name, Project};

use crate::error::ServiceError;
use crate::field::Field;

use super::ProjectEdit;

/// `AddNotFigure` and `RemoveNotFigure`; other edits change nothing.
pub(super) fn apply(project: &mut Project, edit: &ProjectEdit) -> Result<(), ServiceError> {
    match edit {
        ProjectEdit::AddNotFigure { name } => add_not_figure(project, name),
        ProjectEdit::RemoveNotFigure { name } => {
            remove_not_figure(project, name);
            Ok(())
        }
        _ => Ok(()),
    }
}

/// Gives the character `id` the other names `aliases`, trimmed. Refused: an empty one, the
/// character's own name, one given twice, one another character answers to, one marked as no
/// character.
pub(super) fn set_aliases(
    project: &mut Project,
    id: &str,
    aliases: &[String],
) -> Result<(), ServiceError> {
    let Some(own) = project.character(id) else {
        return Err(ServiceError::NoCharacter(id.to_owned()));
    };

    let mut kept: Vec<String> = Vec::new();
    for alias in aliases.iter().map(|a| a.trim()) {
        let taken = alias.is_empty()
            || same_character_name(alias, &own.name)
            || kept.iter().any(|k| same_character_name(k, alias))
            || is_taken(project, alias, Some(id));
        if taken {
            return Err(bad_alias(alias));
        }
        kept.push(alias.to_owned());
    }

    if let Some(character) = project.characters.iter_mut().find(|c| c.id == id) {
        character.aliases = kept;
    }
    Ok(())
}

/// Marks the cue `name` as no character. Refused: empty, or a name a character answers to or
/// already marked.
fn add_not_figure(project: &mut Project, name: &str) -> Result<(), ServiceError> {
    let name = name.trim();
    if name.is_empty() || is_taken(project, name, None) {
        return Err(ServiceError::BadValue {
            field: Field::Name,
            value: name.to_owned(),
        });
    }

    project.not_figures.push(name.to_owned());
    Ok(())
}

/// Takes the mark of the cue `name` away; a name not marked changes nothing.
fn remove_not_figure(project: &mut Project, name: &str) {
    project
        .not_figures
        .retain(|n| !same_character_name(n, name));
}

/// Whether a character other than `own` answers to `name`, or it is marked as no character.
pub(super) fn is_taken(project: &Project, name: &str, own: Option<&str>) -> bool {
    let character = project
        .characters
        .iter()
        .filter(|c| Some(c.id.as_str()) != own)
        .any(|c| c.is_called(name));

    character
        || project
            .not_figures
            .iter()
            .any(|n| same_character_name(n, name))
}

fn bad_alias(alias: &str) -> ServiceError {
    ServiceError::BadValue {
        field: Field::Alias,
        value: alias.to_owned(),
    }
}
