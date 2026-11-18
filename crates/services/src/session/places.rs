//! The play locations against the book (DB-51): the locations of the scene headings no play
//! location is called by, and the user's answer for each.
//!
//! The answer is written to `orte.toml` at once, from the file as it is on disk: another
//! program (the TUI beside the window) may have changed it, and the answer changes only what
//! it touches. It is no undo step; undo and redo keep the places as they are (like the paper).

use super::Session;
use crate::error::ServiceError;
use crate::places::{self, PendingPlace, PlaceChoice};

/// One line of the dialog for an unknown location: a new play location, another spelling of
/// one, or the new name of one. Surfaces show the lines and hand back the position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaceAnswer {
    /// Name of the play location for `Variant` and `Rename`; empty for `Create`.
    pub name: String,
    pub choice: PlaceChoice,
}

impl Session {
    /// The locations of the headings no play location is called by (DATENMODELL principle 6):
    /// asked, never made play locations on their own.
    pub fn pending_places(&self) -> &[PendingPlace] {
        &self.pending_places
    }

    /// The answers to the question about the location `name`: new play location; for each
    /// play location the headings no longer name, its rename and its variant; for each other
    /// its variant. Empty when there is no such question.
    pub fn place_answers(&self, name: &str) -> Vec<PlaceAnswer> {
        if !self.pending_places.iter().any(|p| p.name == name) {
            return Vec::new();
        }

        let answer = |id: &str, choice: PlaceChoice| {
            let name = self
                .project
                .places
                .iter()
                .find(|p| p.id == id)?
                .name
                .clone();

            Some(PlaceAnswer { name, choice })
        };
        let create = PlaceAnswer {
            name: String::new(),
            choice: PlaceChoice::Create,
        };
        let unused = places::unused(&self.project.places, &self.script);
        let renames = unused
            .iter()
            .filter_map(|id| answer(id, PlaceChoice::Rename(id.clone())));
        let variants = unused
            .iter()
            .chain(
                self.project
                    .places
                    .iter()
                    .map(|p| &p.id)
                    .filter(|id| !unused.contains(id)),
            )
            .filter_map(|id| answer(id, PlaceChoice::Variant(id.clone())));

        std::iter::once(create)
            .chain(renames)
            .chain(variants)
            .collect()
    }

    /// Answers the question about the location `name` with line `index` of `place_answers`.
    pub fn answer_place(&mut self, name: &str, index: usize) -> Result<(), ServiceError> {
        let answers = self.place_answers(name);
        let Some(answer) = answers.into_iter().nth(index) else {
            return Err(ServiceError::NoDecision(name.to_owned()));
        };

        self.resolve_place(name, answer.choice)
    }

    /// Settles the location `name`: a new play location, a variant of one, or the new name
    /// of one. Written to `orte.toml` at once.
    pub fn resolve_place(&mut self, name: &str, choice: PlaceChoice) -> Result<(), ServiceError> {
        let none = || ServiceError::NoDecision(name.to_owned());
        if !self.pending_places.iter().any(|p| p.name == name) {
            return Err(none());
        }

        // The file as it is now, not the session's copy: it may be older.
        let mut disk = klappe_store::load_places(&self.dir)?;
        if !places::decide(&mut disk, name, &choice) {
            return Err(none());
        }
        klappe_store::save_places(&self.dir, &disk)?;

        self.project.places = disk;
        self.refresh_places();

        Ok(())
    }

    /// Takes the play locations from `orte.toml`: the file is their only source, so a save
    /// writes back what another program may have added since the project was opened.
    pub(super) fn reload_places(&mut self) -> Result<(), ServiceError> {
        self.project.places = klappe_store::load_places(&self.dir)?;
        self.refresh_places();

        Ok(())
    }

    /// Asks the headings which locations are open, changing nothing.
    pub(super) fn refresh_places(&mut self) {
        self.pending_places = places::pending(&self.project, &self.script);
    }
}
