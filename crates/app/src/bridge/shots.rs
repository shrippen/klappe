//! Shots: selecting, editing, adding and removing them, and who a camera follows.

use super::view::to_json;
use super::{q, qobject};
use crate::fileurl;
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use klappe_services::error_text;
use klappe_services::{
    Field, Numbering, PlanEdit, ProjectEdit, ServiceError, Session, ShotField, Source,
};

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

    pub(super) fn setup_choices(&self, id: &QString) -> QString {
        let options = self
            .rust()
            .session
            .as_ref()
            .map(|s| s.setup_options(&id.to_string()))
            .unwrap_or_default();

        q(&to_json(&options))
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

    pub(super) fn name_shot(
        self: Pin<&mut Self>,
        id: &QString,
        short_name: &QString,
        description: &QString,
    ) -> bool {
        let (id, short_name, description) = (
            id.to_string(),
            short_name.to_string(),
            description.to_string(),
        );

        self.edit_shots(|s| s.name_shot(&id, &short_name, &description))
    }

    pub(super) fn set_shot_image(self: Pin<&mut Self>, id: &QString, path: &QString) -> bool {
        let id = id.to_string();
        let path = path.to_string();
        let file = (!path.is_empty()).then(|| fileurl::to_path(&path));

        self.edit_shots(|s| s.set_shot_image(&id, file.as_deref()))
    }

    pub(super) fn shot_still(self: Pin<&mut Self>, id: &QString) -> bool {
        let id = id.to_string();

        self.edit_shots(|s| s.shot_still(&id))
    }

    pub(super) fn project_file_url(&self, file: &QString) -> QString {
        let Some(s) = self.rust().session.as_ref() else {
            return QString::default();
        };
        let Some(joined) = s.image_file(&file.to_string()) else {
            return QString::default();
        };
        let absolute = std::path::absolute(&joined).unwrap_or(joined);

        q(&fileurl::from_path(&absolute))
    }

    pub(super) fn set_numbering(self: Pin<&mut Self>, name: &QString) -> bool {
        let name = name.to_string();
        let edit = Numbering::from_name(&name)
            .map(|numbering| ProjectEdit::SetNumbering { numbering })
            .ok_or(ServiceError::BadValue {
                field: Field::Kind,
                value: name,
            });

        self.commit_project(edit)
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

    pub(super) fn version_shot(self: Pin<&mut Self>, id: &QString) -> bool {
        self.edit_shots(|s| s.version_shot(&id.to_string()).map(drop))
    }

    pub(super) fn renumber_shots(self: Pin<&mut Self>) -> bool {
        self.edit_shots(Session::renumber_shots)
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

    pub(super) fn set_axis(self: Pin<&mut Self>, a: &QString, b: &QString) -> bool {
        let (a, b) = (a.to_string(), b.to_string());

        self.edit_shots(|s| s.set_axis((!a.is_empty()).then_some((a.as_str(), b.as_str()))))
    }

    pub(super) fn set_follow(self: Pin<&mut Self>, camera: &QString, figure: &QString) -> bool {
        let figure = figure.to_string();

        self.commit_edit(PlanEdit::Follow {
            camera: camera.to_string(),
            figure: (!figure.is_empty()).then_some(figure),
        })
    }
}
