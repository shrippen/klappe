//! Shots: selecting, editing, adding and removing them, and who a camera follows.

use super::qobject;
use crate::notice::error_text;
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use klappe_services::{PlanEdit, ServiceError, Session, ShotField, Source};

impl qobject::Controller {
    pub(super) fn select_shot(mut self: Pin<&mut Self>, id: &QString) {
        let Some(s) = self.as_mut().session() else {
            return;
        };
        if s.select_shot(&id.to_string()).is_err() {
            return;
        }
        self.refresh();
    }

    pub(super) fn edit_shot(
        mut self: Pin<&mut Self>,
        id: &QString,
        field: &QString,
        value: &QString,
    ) -> bool {
        let language = self.rust().language;
        let Some(field) = ShotField::from_name(&field.to_string()) else {
            return false;
        };
        let Some(s) = self.as_mut().session() else {
            return false;
        };
        if let Err(e) = s.edit_shot(&id.to_string(), field, &value.to_string()) {
            self.say(error_text(language, &e));
            return false;
        }
        self.as_mut().changed();
        self.refresh();
        true
    }

    pub(super) fn add_shot(self: Pin<&mut Self>) -> bool {
        let camera = match self.rust().selected.only() {
            Some(Source::Camera(id)) => Some(id.clone()),
            _ => None,
        };

        self.edit_shots(|s| s.add_shot(camera.as_deref()).map(drop))
    }

    pub(super) fn copy_shot(self: Pin<&mut Self>, id: &QString) -> bool {
        self.edit_shots(|s| s.copy_shot(&id.to_string()).map(drop))
    }

    pub(super) fn delete_shot(self: Pin<&mut Self>, id: &QString) -> bool {
        self.edit_shots(|s| s.delete_shot(&id.to_string()))
    }

    /// Runs a change of the shot list; an error goes to the status line.
    fn edit_shots(
        mut self: Pin<&mut Self>,
        change: impl FnOnce(&mut Session) -> Result<(), ServiceError>,
    ) -> bool {
        let language = self.rust().language;
        let Some(s) = self.as_mut().session() else {
            return false;
        };
        if let Err(e) = change(s) {
            self.say(error_text(language, &e));
            return false;
        }
        self.as_mut().changed();
        self.refresh();
        true
    }

    pub(super) fn set_follow(self: Pin<&mut Self>, camera: &QString, figure: &QString) -> bool {
        let figure = figure.to_string();

        self.commit_edit(PlanEdit::Follow {
            camera: camera.to_string(),
            figure: (!figure.is_empty()).then_some(figure),
        })
    }
}
