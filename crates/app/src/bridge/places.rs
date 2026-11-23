//! The list of scripted locations (D-21): change one, take one out, shoot the scene on view at
//! another location. A refusal goes to the status line like every other edit.

use super::qobject;
use super::script::{aliases_of, own_location};
use core::pin::Pin;
use cxx_qt_lib::QString;
use klappe_services::PlaceEdit;

impl qobject::Controller {
    pub(super) fn edit_place(
        mut self: Pin<&mut Self>,
        id: &QString,
        name: &QString,
        variants: &QString,
        location: &QString,
    ) -> bool {
        let result = aliases_of(variants).and_then(|variants| {
            let edit = PlaceEdit {
                name: name.to_string(),
                variants,
                location: own_location(location),
            };

            self.as_mut()
                .session()
                .map_or(Ok(()), |s| s.edit_place(&id.to_string(), &edit))
        });

        self.report(Some(result))
    }

    pub(super) fn remove_place(mut self: Pin<&mut Self>, id: &QString) -> bool {
        let result = self
            .as_mut()
            .session()
            .map_or(Ok(()), |s| s.remove_place(&id.to_string()));

        self.report(Some(result))
    }

    pub(super) fn shoot_scene_at(mut self: Pin<&mut Self>, location: &QString) -> bool {
        let result = self.as_mut().session().map_or(Ok(()), |s| {
            s.shoot_scene_at(own_location(location).as_deref())
        });

        self.report(Some(result))
    }
}
