//! Scene numbers in the text (DB-33): set when the book is saved in Klappe, never on open.

use klappe_model::{SceneNumbering, ScriptLanguage};

use crate::error::ServiceError;
use klappe_script::{number_scenes, NumberMode, NumberRules, OMITTED_DE, OMITTED_EN};

use super::outside::PROJECT_FILE;
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
        let before = self.before(PROJECT_FILE)?;
        let setup = &mut self.project.script;
        if (setup.scene_numbering, setup.first_scene) == (numbering, first) {
            return Ok(());
        }

        setup.scene_numbering = numbering;
        setup.first_scene = first;
        klappe_store::save_scene_numbering(&self.dir, numbering, first)?;
        self.rebase(PROJECT_FILE, before)?;

        Ok(())
    }

    /// Numbers the screenplay by the project's setting: sequential from the first number
    /// while automatic (struck headings go); while locked, by the Sperrstand: no number
    /// changes, a new scene is numbered between its neighbours (12A), a number the book lost
    /// stays as an "OMITTED" heading. Without a Sperrstand (a book from before it) nothing
    /// changes at all. Saving does this first, so the numbers in the file are the ones the
    /// register holds afterwards; opening never writes them (a change on open would be a
    /// change in Git).
    pub(super) fn number_script(&mut self) {
        let Some(text) = self.script_text.clone() else {
            return;
        };
        if self.script.scenes().is_empty() {
            return;
        }

        let setup = self.project.script;
        let (mode, locked, holders) = match (setup.scene_numbering, &self.lock) {
            (SceneNumbering::Automatic, _) => (NumberMode::Automatic, Vec::new(), Vec::new()),
            // A Sperrstand without a number (a book never numbered) is none.
            (SceneNumbering::Locked, None) => return,
            (SceneNumbering::Locked, Some(lock)) if lock.is_empty() => return,
            (SceneNumbering::Locked, Some(lock)) => {
                let plan = book::plan(&self.project, &self.script);
                let holding = plan.holding(&self.project, lock);
                let numbers = lock.iter().map(|l| l.number.clone()).collect();

                (NumberMode::Locked, numbers, holding.holders)
            }
        };
        let omitted = match setup.language {
            Some(ScriptLanguage::En) => OMITTED_EN,
            _ => OMITTED_DE,
        };
        let rules = NumberRules {
            mode,
            first: setup.first_scene,
            omitted,
            locked: &locked,
            holders: &holders,
        };

        let numbered = number_scenes(&text, &rules);
        if numbered != text {
            self.set_script_text(&numbered);
        }
    }

    /// Makes the Sperrstand of the book as saved (numbers and the entries that hold them),
    /// in memory and in `nummern.toml`; the last write of a save, after the register was
    /// aligned. Only saving gets here: no other way changes the Sperrstand.
    pub(super) fn write_lock(&mut self) -> Result<(), ServiceError> {
        if self.script_text.is_none() {
            return Ok(());
        }

        let numbers = match self.script.scenes().is_empty() {
            true => Vec::new(),
            false => book::plan(&self.project, &self.script).numbers(&self.project),
        };
        klappe_store::save_lock(&self.dir, &numbers)?;
        self.lock = Some(numbers);

        Ok(())
    }

    /// Counts the numbers two scenes carry that the Sperrstand does not settle, for the
    /// status line; none while automatic or without a Sperrstand.
    pub(super) fn count_twin_numbers(&mut self, plan: &book::Plan) {
        self.twin_numbers = match (&self.lock, self.project.script.scene_numbering) {
            (Some(lock), SceneNumbering::Locked) => plan.holding(&self.project, lock).twins.len(),
            _ => 0,
        };
    }
}
