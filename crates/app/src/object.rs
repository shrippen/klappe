//! The fields of a placed object typed or picked on its card: size, height and the library
//! it is filed in, each as an edit. Like the tools (`tools.rs`), no geometry here.

use klappe_services::{
    read_number, Field, PlanEdit, ProjectEdit, Range, ServiceError, Source, Unit,
};

use crate::tools::{length_m, Typed};

/// The heights of the set's elements a card can set (LH-90), each with an empty text for
/// "the default again".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeightField {
    /// Of the selected wall.
    Wall,
    /// Sill of the selected door or window.
    Sill,
    /// Head of the selected door or window.
    Head,
    /// Underside of the selected object.
    Base,
}

impl HeightField {
    /// The name QML sends: `wall`, `sill`, `head` or `base`.
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "wall" => Some(Self::Wall),
            "sill" => Some(Self::Sill),
            "head" => Some(Self::Head),
            "base" => Some(Self::Base),
            _ => None,
        }
    }
}

/// The height typed for the selected wall, opening or object, as an edit (LH-90): an empty
/// text takes the height off. A top (wall, head) is more than nothing, a start (sill,
/// underside) may be the floor; the session sets the upper limit and the sill-head order.
pub fn set_height_edit(selected: Option<&Source>, field: HeightField, text: &str) -> Typed {
    let source = selected?.clone();
    let (name, range) = match (field, &source) {
        (HeightField::Wall, Source::Wall(_)) => (Field::Height, Range::Positive),
        (HeightField::Head, Source::Opening { .. }) => (Field::Head, Range::Positive),
        (HeightField::Sill, Source::Opening { .. }) => (Field::Sill, Range::NotNegative),
        (HeightField::Base, Source::Object(_)) => (Field::Base, Range::NotNegative),
        _ => return None,
    };
    let typed: Result<Option<f64>, ServiceError> = match text.trim().is_empty() {
        true => Ok(None),
        false => read_number(name, text, Unit::Metre, range).map(Some),
    };

    Some(typed.map(|value| match field {
        HeightField::Wall => PlanEdit::SetWallHeight {
            source,
            height_m: value,
        },
        HeightField::Sill => PlanEdit::SetOpeningSill {
            source,
            sill_m: value,
        },
        HeightField::Head => PlanEdit::SetOpeningHead {
            source,
            head_m: value,
        },
        HeightField::Base => PlanEdit::SetObjectBase {
            source,
            base_m: value,
        },
    }))
}

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

/// The width typed for the selected door or window, as an edit (LH-10).
pub fn opening_width_edit(selected: Option<&Source>, width: &str) -> Typed {
    let Some(source @ Source::Opening { .. }) = selected else {
        return None;
    };

    Some(
        length_m(Field::Width, width).map(|width_m| PlanEdit::SetOpeningWidth {
            source: source.clone(),
            width_m,
        }),
    )
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
    fn a_width_typed_for_the_selected_opening_is_a_width_edit() {
        let opening = Source::Opening { wall: 2, index: 0 };

        assert_eq!(
            edit_of(opening_width_edit(Some(&opening), " 1,1 ")),
            Some(PlanEdit::SetOpeningWidth {
                source: opening.clone(),
                width_m: 1.1
            })
        );
        assert_eq!(edit_of(opening_width_edit(Some(&opening), "0")), None);
        assert_eq!(edit_of(opening_width_edit(Some(&opening), "breit")), None);
        assert!(opening_width_edit(Some(&Source::Wall(0)), "1").is_none());
        assert!(opening_width_edit(Some(&Source::Object("o1".into())), "1").is_none());
        assert!(opening_width_edit(None, "1").is_none());
    }

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

    #[test]
    fn a_height_typed_for_the_selection_is_a_height_edit() {
        let wall = Source::Wall(1);
        let door = Source::Opening { wall: 2, index: 0 };
        let object = Source::Object("o1".into());

        assert_eq!(
            edit_of(set_height_edit(Some(&wall), HeightField::Wall, " 3,1 ")),
            Some(PlanEdit::SetWallHeight {
                source: wall.clone(),
                height_m: Some(3.1)
            })
        );
        assert_eq!(
            edit_of(set_height_edit(Some(&wall), HeightField::Wall, "")),
            Some(PlanEdit::SetWallHeight {
                source: wall.clone(),
                height_m: None
            })
        );
        assert_eq!(
            edit_of(set_height_edit(Some(&door), HeightField::Sill, "0")),
            Some(PlanEdit::SetOpeningSill {
                source: door.clone(),
                sill_m: Some(0.0)
            })
        );
        assert_eq!(
            edit_of(set_height_edit(Some(&door), HeightField::Head, "2.2 m")),
            Some(PlanEdit::SetOpeningHead {
                source: door.clone(),
                head_m: Some(2.2)
            })
        );
        assert_eq!(
            edit_of(set_height_edit(Some(&object), HeightField::Base, " ")),
            Some(PlanEdit::SetObjectBase {
                source: object.clone(),
                base_m: None
            })
        );

        // A top is more than nothing; a refused text is an error, a wrong selection is none.
        assert_eq!(
            edit_of(set_height_edit(Some(&wall), HeightField::Wall, "0")),
            None
        );
        assert_eq!(
            edit_of(set_height_edit(Some(&door), HeightField::Head, "-1")),
            None
        );
        assert_eq!(
            edit_of(set_height_edit(Some(&door), HeightField::Sill, "-1")),
            None
        );
        assert!(matches!(
            set_height_edit(Some(&object), HeightField::Base, "hoch"),
            Some(Err(ServiceError::BadValue {
                field: Field::Base,
                ..
            }))
        ));
        assert!(set_height_edit(Some(&wall), HeightField::Sill, "1").is_none());
        assert!(set_height_edit(Some(&door), HeightField::Wall, "1").is_none());
        assert!(set_height_edit(None, HeightField::Wall, "1").is_none());
        assert_eq!(HeightField::from_name("head"), Some(HeightField::Head));
        assert_eq!(HeightField::from_name("width"), None);
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
