//! The register against the book (DATENMODELL §4): aligns on open and save, and takes the
//! user's answer for an entry that could not be placed.

use klappe_model::{BookState, Kind};

use super::Session;
use crate::book::{self, BookReport, Choice, PendingScene};
use crate::error::ServiceError;

/// Undo step of `resolve_scene`.
const RESOLVE_STEP: &str = "ResolveScene";

impl Session {
    /// The entries of the register the book could not place without asking, with the scenes
    /// each may be (DATENMODELL principle 6). Their scenes get no entry before an answer.
    pub fn pending_scenes(&self) -> &[PendingScene] {
        &self.pending
    }

    /// What there is to say about the book and the register: for the status line.
    pub fn book_report(&self) -> BookReport {
        let not_in_book = self
            .project
            .scenes
            .iter()
            .filter(|s| s.kind == Kind::Scene && s.state == BookState::NotInBook)
            .count();

        BookReport {
            pending: self.pending.len(),
            not_in_book,
        }
    }

    /// Answers the open question about entry `old_id`: it is one of its candidates, or no
    /// scene of the book (the candidates then become new entries). One undo step. The entry
    /// takes number, heading and prints from the scene, as if it had been matched.
    pub fn resolve_scene(&mut self, old_id: &str, choice: Choice) -> Result<(), ServiceError> {
        let none = || ServiceError::NoDecision(old_id.to_owned());
        if self.script.scenes().is_empty() {
            return Err(none());
        }

        let mut plan = book::plan(&self.project, &self.script);
        if !plan.decide(&self.project, old_id, choice) {
            return Err(none());
        }

        let pending = plan.pending(&self.project);
        let mut next = self.project.clone();
        plan.apply(&mut next);

        let (scene, shown) = self.ids_on_view();
        self.commit_step(next, RESOLVE_STEP);
        self.pending = pending;
        self.follow_scene(scene, shown);

        Ok(())
    }

    /// Aligns the register with the book as it is now and writes the result into the
    /// project, without an undo step: the register follows the text, it is not an edit.
    /// Returns whether the register changed.
    pub(super) fn reconcile(&mut self) -> bool {
        if self.script.scenes().is_empty() {
            self.pending.clear();
            return false;
        }

        let plan = book::plan(&self.project, &self.script);
        let (scene, shown) = self.ids_on_view();
        self.pending = plan.pending(&self.project);
        if !plan.apply(&mut self.project) {
            return false;
        }

        self.follow_scene(scene, shown);

        true
    }

    /// Asks the alignment which entries are open, changing nothing. After undo, redo and
    /// a new screenplay text, which move the register or the book under the list.
    pub(super) fn refresh_pending(&mut self) {
        self.pending = match self.script.scenes().is_empty() {
            true => Vec::new(),
            false => book::plan(&self.project, &self.script).pending(&self.project),
        };
    }

    /// The scene and plan on view, to find them again after the entries moved.
    fn ids_on_view(&self) -> (Option<String>, Option<String>) {
        let scene = self.scene().map(|s| s.id.clone());
        let plan = self.plan().map(|p| p.id.clone());

        (scene, plan)
    }
}
