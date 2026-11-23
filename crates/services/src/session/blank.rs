use std::path::Path;

use klappe_model::SetPlan;

use super::{OnOutside, Session};
use crate::catalog::Language;
use crate::error::ServiceError;
use crate::project_edit::ProjectEdit;

const BLANK_EXT: &str = ".klappe";
const BLANK_SCENE_NUMBER: &str = "1";
const BLANK_SET_WIDTH_M: f64 = 8.0;
const BLANK_SET_DEPTH_M: f64 = 5.0;

impl Session {
    /// A new project in `folder/<unbenannt|untitled>-N.klappe` (first free N) with one scene
    /// and its empty plan, saved, so the folder is complete and closing asks nothing; nothing
    /// to undo.
    pub fn create_blank(folder: &Path, language: Language) -> Result<Self, ServiceError> {
        let stem = language.text("start.blankStem");
        let (n, dir) = klappe_store::free_numbered(folder, &stem, BLANK_EXT)?;
        let title = match n {
            1 => language.text("start.blankTitle"),
            _ => format!("{} {n}", language.text("start.blankTitle")),
        };

        let mut session = Self::create(&dir, &title, language)?;

        // A scene needs a location: one room of a plain size, which the user may change.
        session.edit_project(ProjectEdit::AddLocation {
            name: language.text("start.blankSet"),
            width_m: BLANK_SET_WIDTH_M,
            depth_m: BLANK_SET_DEPTH_M,
            wall_height_m: SetPlan::default().wall_height_m,
            site: None,
        })?;
        let location = session
            .project()
            .locations
            .first()
            .map(|l| l.id.clone())
            .unwrap_or_default();
        session.edit_project(ProjectEdit::AddScene {
            number: BLANK_SCENE_NUMBER.to_owned(),
            heading: language.text("start.blankHeading"),
            location: Some(location),
        })?;
        session.save(OnOutside::Overwrite)?;

        // Opened again, so undo cannot take the set and scene back into an empty project.
        Self::open(session.dir())
    }
}
