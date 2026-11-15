//! The fields of a placed object typed or picked on its card: size, height and the library
//! it is filed in, each as an edit. Like the tools (`tools.rs`), no geometry here.

use klappe_services::{Field, PlanEdit, ProjectEdit, Source};

use crate::tools::{length_m, Typed};

/// The size typed for the selected object, as an edit.
pub fn resize_edit(selected: Option<&Source>, width: &str, depth: &str) -> Typed {
    let Some(Source::Object(id)) = selected else {
        return None;
    };
    let resized = length_m(Field::Width, width).and_then(|width_m| {
        Ok(PlanEdit::Resize {
            source: Source::Object(id.clone()),
            width_m,
            depth_m: length_m(Field::Depth, depth)?,
        })
    });

    Some(resized)
}

/// The height typed for the selected object, as an edit (LH-17): an empty text takes the
/// height off.
pub fn object_height_edit(selected: Option<&Source>, text: &str) -> Typed {
    let Some(Source::Object(id)) = selected else {
        return None;
    };
    let height_m = match text.trim().is_empty() {
        true => Ok(None),
        false => length_m(Field::Height, text).map(Some),
    };

    Some(height_m.map(|height_m| PlanEdit::SetObjectHeight {
        source: Source::Object(id.clone()),
        height_m,
    }))
}

/// Files the selected object in the library of the project under `name`. `None` when
/// something other than an object is selected, there is no scene to take the set from, or the
/// name is blank.
pub fn library_edit(
    selected: Option<&Source>,
    location: Option<&str>,
    name: &str,
) -> Option<ProjectEdit> {
    let Some(Source::Object(id)) = selected else {
        return None;
    };
    let name = name.trim();
    if name.is_empty() {
        return None;
    }

    Some(ProjectEdit::LibraryFromObject {
        location: location?.to_owned(),
        object: id.clone(),
        name: name.to_owned(),
    })
}

/// Renames and regroups the own library entry `kind` (LH-12). `None` for a blank name.
pub fn library_rename_edit(kind: &str, name: &str, group: &str) -> Option<ProjectEdit> {
    let name = name.trim();
    if name.is_empty() {
        return None;
    }

    Some(ProjectEdit::EditLibraryItem {
        kind: kind.to_owned(),
        name: name.to_owned(),
        group: group.trim().to_owned(),
    })
}

/// Deletes the own library entry `kind` (LH-12).
pub fn library_remove_edit(kind: &str) -> ProjectEdit {
    ProjectEdit::RemoveLibraryItem {
        kind: kind.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::edit_of;
    use klappe_services::{Field, ServiceError};

    #[test]
    fn a_size_typed_for_the_selected_object_is_a_resize() {
        let object = Source::Object("o1".into());

        assert_eq!(
            edit_of(resize_edit(Some(&object), "2,5", " 0.8 ")),
            Some(PlanEdit::Resize {
                source: object.clone(),
                width_m: 2.5,
                depth_m: 0.8
            })
        );
        assert_eq!(edit_of(resize_edit(Some(&object), "2,5", "0")), None);
        assert_eq!(edit_of(resize_edit(Some(&object), "", "1")), None);
        assert_eq!(edit_of(resize_edit(Some(&object), "1", "breit")), None);
        assert_eq!(edit_of(resize_edit(Some(&Source::Wall(0)), "1", "1")), None);
        assert_eq!(edit_of(resize_edit(None, "1", "1")), None);
    }

    #[test]
    fn the_selected_object_is_filed_under_a_trimmed_name() {
        let object = Source::Object("o1".into());

        assert_eq!(
            library_edit(Some(&object), Some("saal"), "  Thron "),
            Some(ProjectEdit::LibraryFromObject {
                location: "saal".into(),
                object: "o1".into(),
                name: "Thron".into()
            })
        );
        assert_eq!(library_edit(Some(&object), Some("saal"), "  "), None);
        assert_eq!(library_edit(Some(&object), None, "Thron"), None);
        assert_eq!(
            library_edit(Some(&Source::Wall(0)), Some("saal"), "T"),
            None
        );
        assert_eq!(library_edit(None, Some("saal"), "T"), None);
    }

    #[test]
    fn an_own_entry_is_renamed_under_a_trimmed_name_and_group() {
        assert_eq!(
            library_rename_edit("thron", "  Sessel ", " Hof "),
            Some(ProjectEdit::EditLibraryItem {
                kind: "thron".into(),
                name: "Sessel".into(),
                group: "Hof".into()
            })
        );
        assert_eq!(library_rename_edit("thron", "  ", "Hof"), None);
        assert_eq!(
            library_remove_edit("thron"),
            ProjectEdit::RemoveLibraryItem {
                kind: "thron".into()
            }
        );
    }

    #[test]
    fn a_height_typed_for_an_object_is_a_set_object_height() {
        let object = Source::Object("o1".into());
        let edit = |height_m| {
            Some(PlanEdit::SetObjectHeight {
                source: object.clone(),
                height_m,
            })
        };

        assert_eq!(
            edit_of(object_height_edit(Some(&object), " 1,2 ")),
            edit(Some(1.2))
        );
        // Empty takes the height off.
        assert_eq!(edit_of(object_height_edit(Some(&object), "  ")), edit(None));
        assert_eq!(edit_of(object_height_edit(Some(&object), "0")), None);
        assert_eq!(edit_of(object_height_edit(Some(&object), "-1")), None);
        assert_eq!(edit_of(object_height_edit(Some(&object), "hoch")), None);
        assert_eq!(
            edit_of(object_height_edit(Some(&Source::Wall(0)), "1")),
            None
        );
        assert_eq!(edit_of(object_height_edit(None, "1")), None);
    }

    /// Audit, Fund 2: a refused text is an error for the status line, a selection that does
    /// not fit is no error.
    #[test]
    fn a_refused_text_names_its_field() {
        let object = Source::Object("o".into());
        let refused = resize_edit(Some(&object), "2", "breit");

        assert!(matches!(
            refused,
            Some(Err(ServiceError::BadValue {
                field: Field::Depth,
                ..
            }))
        ));
        assert!(resize_edit(Some(&Source::Wall(0)), "x", "x").is_none());
    }
}
