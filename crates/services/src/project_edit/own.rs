//! Changes to the project's own library entries (LH-12).

use klappe_model::Project;

use super::name_of;
use crate::error::ServiceError;

pub(super) fn edit_item(
    project: &mut Project,
    kind: &str,
    name: &str,
    group: &str,
) -> Result<(), ServiceError> {
    let name = name_of(name)?;
    let item = project
        .library
        .iter_mut()
        .find(|i| i.kind == kind)
        .ok_or(ServiceError::Gone)?;

    item.label = name;
    item.group = group.trim().to_owned();
    Ok(())
}

pub(super) fn remove_item(project: &mut Project, kind: &str) -> Result<(), ServiceError> {
    let before = project.library.len();

    project.library.retain(|i| i.kind != kind);
    if project.library.len() == before {
        return Err(ServiceError::Gone);
    }
    Ok(())
}
