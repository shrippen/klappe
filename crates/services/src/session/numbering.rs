//! Scene numbers in the text (DB-33): set when the book is saved in Klappe, never on open.

use klappe_model::{SceneNumbering, ScriptLanguage};

use crate::error::ServiceError;
use klappe_script::{number_scenes, NumberMode, NumberRules, OMITTED_DE, OMITTED_EN};

use super::Session;
use crate::book;

impl Session {
    /// Sets who numbers the scenes and the first number (DB-33) and writes `projekt.toml` at
    /// once, nothing else: where the book is edited without the plans (TUI) the setting is
    /// kept as an answer about the book is. The numbers in the text change with the next
    /// save. Locking a version (DB-40) will lock them together with the pages.
    pub fn set_scene_numbering(
        &mut self,
        numbering: SceneNumbering,
        first: u32,
    ) -> Result<(), ServiceError> {
        let setup = &mut self.project.script;
        if (setup.scene_numbering, setup.first_scene) == (numbering, first) {
            return Ok(());
        }

        setup.scene_numbering = numbering;
        setup.first_scene = first;
        klappe_store::save_header(&self.dir, &self.project)?;

        Ok(())
    }

    /// Numbers the screenplay by the project's setting: sequential from the first number
    /// while automatic; while locked, no number changes, a new scene is numbered between its
    /// neighbours (12A) and a scene the book lost stays as an "OMITTED" heading. Saving does
    /// this first, so the numbers in the file are the ones the register holds afterwards;
    /// opening never writes them (a change on open would be a change in Git).
    pub(super) fn number_script(&mut self) {
        let Some(text) = self.script_text.clone() else {
            return;
        };
        let setup = self.project.script;
        let (mode, struck) = match setup.scene_numbering {
            SceneNumbering::Automatic => (NumberMode::Automatic, Vec::new()),
            SceneNumbering::Locked => (
                NumberMode::Locked,
                book::plan(&self.project, &self.script).struck(&self.project),
            ),
        };
        let omitted = match setup.language {
            Some(ScriptLanguage::En) => OMITTED_EN,
            _ => OMITTED_DE,
        };
        let rules = NumberRules {
            mode,
            first: setup.first_scene,
            omitted,
            struck: &struck,
        };

        let numbered = number_scenes(&text, &rules);
        if numbered != text {
            self.set_script_text(&numbered);
        }
    }
}
