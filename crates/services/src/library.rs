//! The object library as the session sees it: the built-in objects and the project's own
//! (LH-11, LH-12).

use klappe_model::{LibraryItem, SetPlan};
use klappe_stellprobe::Forms;
use klappe_store::{builtin_forms, builtin_library};

use crate::views::LibraryEntry;

/// The entry for `kind`: the project's own first, then the built-in ones.
pub(crate) fn find<'a>(own: &'a [LibraryItem], kind: &str) -> Option<&'a LibraryItem> {
    own.iter().chain(builtin_library()).find(|i| i.kind == kind)
}

/// Every entry in the order a list shows them: built-in, then own. A built-in entry an own
/// one has the kind of is left out, as `find` would never reach it.
pub(crate) fn entries(own: &[LibraryItem]) -> Vec<LibraryEntry> {
    let built_in = builtin_library()
        .iter()
        .filter(|b| !own.iter().any(|o| o.kind == b.kind))
        .map(|item| LibraryEntry::of(item, false));
    let custom = own.iter().map(|item| LibraryEntry::of(item, true));

    built_in.chain(custom).collect()
}

/// The body forms of the objects of `set` (LH-97): each kind's library entry names a form,
/// the project's own entry before the built-in one.
pub(crate) fn forms_of(own: &[LibraryItem], set: &SetPlan) -> Forms {
    let mut forms = Forms::default();

    for object in &set.objects {
        let name = find(own, &object.kind).and_then(|item| item.form.as_deref());
        let Some(form) = name.and_then(|n| builtin_forms().iter().find(|f| f.name == n)) else {
            continue;
        };

        forms.insert(&object.kind, &form.boxes);
    }

    forms
}

#[cfg(test)]
mod tests {
    use klappe_model::{ObjectShape, Point, SetObject};

    use super::*;

    fn object(kind: &str, shape: ObjectShape) -> SetObject {
        SetObject {
            id: kind.into(),
            kind: kind.into(),
            label: String::new(),
            at: Point::new(1.0, 1.0),
            width_m: 1.0,
            depth_m: 1.0,
            rotation_deg: 0.0,
            shape,
            height_m: Some(1.0),
            base_m: None,
        }
    }

    /// What `forms_of` gives for objects of `kind` that take the built-in form `name`.
    fn with(pairs: &[(&str, &str)]) -> Forms {
        let mut forms = Forms::default();
        for (kind, name) in pairs {
            let form = builtin_forms().iter().find(|f| f.name == *name).unwrap();
            forms.insert(kind, &form.boxes);
        }

        forms
    }

    fn set_of(objects: Vec<SetObject>) -> SetPlan {
        SetPlan {
            objects,
            ..SetPlan::default()
        }
    }

    /// LH-97: a placed library table gets the form its entry names; a kind without an entry
    /// (a free form is `form`) and an entry without a form get none.
    #[test]
    fn a_placed_table_gets_its_form_and_a_free_form_none() {
        let set = set_of(vec![
            object("table", ObjectShape::Rect),
            object("platform", ObjectShape::Rect),
            object("form", ObjectShape::Rect),
        ]);
        let forms = forms_of(&[], &set);

        assert_eq!(forms, with(&[("table", "table")]));
    }

    /// An own entry names a built-in form too, and its kind is looked up before the built-in
    /// ones.
    #[test]
    fn an_own_entry_names_a_form() {
        let item = |kind: &str, form: Option<&str>| LibraryItem {
            kind: kind.into(),
            label: kind.into(),
            group: String::new(),
            width_m: 1.0,
            depth_m: 1.0,
            shape: ObjectShape::Rect,
            height_m: None,
            form: form.map(str::to_owned),
        };
        let own = [item("bank", Some("sofa")), item("table", None)];
        let set = set_of(vec![
            object("bank", ObjectShape::Rect),
            object("table", ObjectShape::Rect),
        ]);
        let forms = forms_of(&own, &set);

        assert_eq!(forms, with(&[("bank", "sofa")]));
    }
}
