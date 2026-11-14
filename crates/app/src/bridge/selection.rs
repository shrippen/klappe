//! Actions on the selected elements: remove, duplicate, turn, resize.

use super::qobject;
use crate::tools;
use crate::tools::{Key, Selection};
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;

impl qobject::Controller {
    pub(super) fn remove_selected(mut self: Pin<&mut Self>) -> bool {
        let edit = tools::key_edit(Key::Remove, &self.rust().selected);
        let Some(edit) = edit else {
            return false;
        };

        // The indices of what is left may shift, so the selection goes either way.
        self.as_mut().rust_mut().get_mut().selected = Selection::default();
        self.commit_edit(edit)
    }

    pub(super) fn duplicate_selected(mut self: Pin<&mut Self>) -> bool {
        let Some(edit) = tools::key_edit(Key::Duplicate, &self.rust().selected) else {
            return false;
        };
        let before = self.rust().shown_sources();

        if !self.as_mut().commit_edit(edit) {
            return false;
        }

        // The copies are what the plan shows anew; they are selected.
        let copies = tools::added(&before, &self.rust().shown_sources());
        self.as_mut().rust_mut().get_mut().selected = copies;
        self.refresh();
        true
    }

    pub(super) fn straighten_selected(self: Pin<&mut Self>) -> bool {
        let Some(edit) = tools::key_edit(Key::Straighten, &self.rust().selected) else {
            return false;
        };

        self.commit_edit(edit)
    }

    pub(super) fn turn_selected(self: Pin<&mut Self>, step_deg: f64) -> bool {
        let key = Key::Turn { by_deg: step_deg };
        let Some(edit) = tools::key_edit(key, &self.rust().selected) else {
            return false;
        };

        self.commit_edit(edit)
    }

    pub(super) fn resize_selected(self: Pin<&mut Self>, width: &QString, depth: &QString) -> bool {
        let selected = self.rust().selected.only();
        let typed = tools::resize_edit(selected, &width.to_string(), &depth.to_string());

        self.commit_typed(typed)
    }

    pub(super) fn set_object_height(self: Pin<&mut Self>, height: &QString) -> bool {
        let selected = self.rust().selected.only();
        let typed = tools::object_height_edit(selected, &height.to_string());

        self.commit_typed(typed)
    }
}
