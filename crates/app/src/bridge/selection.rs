//! Actions on the selected elements: remove, duplicate, turn, resize.

use super::q;
use super::qobject;
use crate::drawing;
use crate::object;
use crate::selection;
use crate::selection::{Key, Selection};
use crate::tools::DrawStyle;
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use klappe_services::{layer_of_source, PlanEdit, Session, Source};
use serde_json::json;

/// The turn a menu entry would make: any would do to ask whether turning changes something.
const MENU_TURN_DEG: f64 = 15.0;

/// What the context menu offers for the selection: the layer all of it is on (empty when it
/// is on several), and which commands would change something.
fn menu_json(s: &Session, selected: &Selection) -> serde_json::Value {
    let offers = |key| {
        selection::key_edit(key, selected)
            .as_ref()
            .is_some_and(|edit: &PlanEdit| s.changes(edit))
    };
    let mut layers: Vec<_> = selected
        .sources()
        .iter()
        .map(|source| layer_of_source(source).name())
        .collect();
    layers.dedup();
    let layer = match layers.as_slice() {
        [one] => *one,
        _ => "",
    };

    json!({
        "layer": layer,
        "remove": offers(Key::Remove),
        "duplicate": offers(Key::Duplicate),
        "turn": offers(Key::Turn { by_deg: MENU_TURN_DEG }),
        "straighten": offers(Key::Straighten),
        "note": matches!(selected.only(), Some(Source::Note(_))),
        "shot": matches!(selected.only(), Some(Source::Camera(_))),
    })
}

impl qobject::Controller {
    pub(super) fn menu_info(&self) -> QString {
        let Some(s) = self.rust().session.as_ref() else {
            return q("{}");
        };

        q(&menu_json(s, &self.rust().selected).to_string())
    }

    pub(super) fn clear_selection(mut self: Pin<&mut Self>) {
        if self.rust().selected.is_empty() {
            return;
        }

        self.as_mut().rust_mut().get_mut().selected = Selection::default();
        self.refresh();
    }

    pub(super) fn remove_selected(mut self: Pin<&mut Self>) -> bool {
        let edit = selection::key_edit(Key::Remove, &self.rust().selected);
        let Some(edit) = edit else {
            return false;
        };

        // The indices of what is left may shift, so the selection goes either way.
        self.as_mut().rust_mut().get_mut().selected = Selection::default();
        self.commit_edit(edit)
    }

    pub(super) fn duplicate_selected(mut self: Pin<&mut Self>) -> bool {
        let Some(edit) = selection::key_edit(Key::Duplicate, &self.rust().selected) else {
            return false;
        };
        let before = self.rust().shown_sources();

        if !self.as_mut().commit_edit(edit) {
            return false;
        }

        // The copies are what the plan shows anew; they are selected.
        let copies = selection::added(&before, &self.rust().shown_sources());
        self.as_mut().rust_mut().get_mut().selected = copies;
        self.refresh();
        true
    }

    pub(super) fn straighten_selected(self: Pin<&mut Self>) -> bool {
        let Some(edit) = selection::key_edit(Key::Straighten, &self.rust().selected) else {
            return false;
        };

        self.commit_edit(edit)
    }

    pub(super) fn turn_selected(self: Pin<&mut Self>, step_deg: f64) -> bool {
        let key = Key::Turn { by_deg: step_deg };
        let Some(edit) = selection::key_edit(key, &self.rust().selected) else {
            return false;
        };

        self.commit_edit(edit)
    }

    pub(super) fn resize_selected(self: Pin<&mut Self>, width: &QString, depth: &QString) -> bool {
        let selected = self.rust().selected.only();
        let typed = object::resize_edit(selected, &width.to_string(), &depth.to_string());

        self.commit_typed(typed)
    }

    pub(super) fn set_opening_width(self: Pin<&mut Self>, width: &QString) -> bool {
        let selected = self.rust().selected.only();
        let typed = object::opening_width_edit(selected, &width.to_string());

        self.commit_typed(typed)
    }

    pub(super) fn set_selected_field(
        self: Pin<&mut Self>,
        field: &QString,
        text: &QString,
    ) -> bool {
        let selected = self.rust().selected.only();

        // A door's swing (`side`, `hinge`) first; the other fields are heights.
        if let Some(edit) = object::swing_edit(selected, &field.to_string(), &text.to_string()) {
            return self.commit_edit(edit);
        }

        let Some(field) = object::HeightField::from_name(&field.to_string()) else {
            return false;
        };
        let typed = object::set_height_edit(selected, field, &text.to_string());

        self.commit_typed(typed)
    }

    pub(super) fn set_object_height(self: Pin<&mut Self>, height: &QString) -> bool {
        let selected = self.rust().selected.only();
        let typed = object::object_height_edit(selected, &height.to_string());

        self.commit_typed(typed)
    }
}

impl qobject::Controller {
    pub(super) fn set_drawing_style(
        self: Pin<&mut Self>,
        color: &QString,
        dash: &QString,
        fill: &QString,
    ) -> bool {
        let names = (color.to_string(), dash.to_string(), fill.to_string());
        let Some(style) = DrawStyle::parse(&names.0, &names.1, &names.2) else {
            return false;
        };
        let Some(edit) = drawing::style_edit(self.rust().selected.only(), style) else {
            return false;
        };

        self.commit_edit(edit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge::tests::example;
    use crate::selection::Pick;

    fn info(sources: &[Source]) -> serde_json::Value {
        let s = example();
        let selected = sources.iter().fold(Selection::default(), |all, source| {
            all.click(Some(source.clone()), Pick::Extend)
        });

        menu_json(&s, &selected)
    }

    #[test]
    fn the_menu_offers_what_the_selection_takes() {
        let note = info(&[Source::Note("n1".into())]);
        assert_eq!(note["layer"], "notes");
        assert_eq!(note["remove"], true);
        assert_eq!(note["duplicate"], true);
        assert_eq!(note["turn"], false);
        assert_eq!(note["note"], true);

        let camera = info(&[Source::Camera("a".into())]);
        assert_eq!(camera["layer"], "cameras");
        assert_eq!(camera["shot"], true);
        assert_eq!(camera["turn"], true);

        let figure = info(&[Source::Figure("elisabeth".into())]);
        assert_eq!(figure["layer"], "figures");
        assert_eq!(figure["duplicate"], false);
    }

    #[test]
    fn several_layers_or_nothing_offer_no_layer() {
        let both = info(&[Source::Note("n1".into()), Source::Object("tisch".into())]);
        assert_eq!(both["layer"], "");
        assert_eq!(both["remove"], true);

        let none = info(&[]);
        assert_eq!(none["layer"], "");
        assert_eq!(none["remove"], false);
    }
}
