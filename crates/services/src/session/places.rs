//! The scripted locations against the book (DB-51): the place names of the scene headings no
//! scripted location is called by, and the user's answer for each.
//!
//! The answer is written to `orte.toml` at once, from the file as it is on disk: another
//! program (the TUI beside the window) may have changed it, and the answer changes only what
//! it touches. It is no undo step; undo and redo keep the places as they are (like the paper).
//! Nothing else writes the file: a save leaves it alone.

use klappe_model::Place;

use super::Session;
use crate::error::ServiceError;
use crate::places::{self, PendingPlace, PlaceChoice};

/// One line of the dialog for an unknown place name: a new scripted location, another spelling of
/// one, or the new name of one. Surfaces show the lines and hand back the position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaceAnswer {
    /// Name of the scripted location for `Variant` and `Rename`; empty for `Create`.
    pub name: String,
    pub choice: PlaceChoice,
}

impl Session {
    /// The place names of the headings no scripted location is called by (DATENMODELL principle 6):
    /// asked, never made scripted locations on their own.
    pub fn pending_places(&self) -> &[PendingPlace] {
        &self.pending_places
    }

    /// The answers to the question about the place name `name`: new scripted location; for each
    /// scripted location the headings no longer name, its rename and its variant; for each other
    /// its variant. Empty when there is no such question.
    pub fn place_answers(&self, name: &str) -> Vec<PlaceAnswer> {
        if !self.pending_places.iter().any(|p| p.name == name) {
            return Vec::new();
        }

        let answer = |id: &str, choice: PlaceChoice| {
            let name = self.places.iter().find(|p| p.id == id)?.name.clone();

            Some(PlaceAnswer { name, choice })
        };
        let create = PlaceAnswer {
            name: String::new(),
            choice: PlaceChoice::Create,
        };
        let unused = places::unused(&self.places, &self.script);
        let renames = unused
            .iter()
            .filter_map(|id| answer(id, PlaceChoice::Rename(id.clone())));
        let variants = unused
            .iter()
            .chain(
                self.places
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

    /// Answers the question about the place name `name` with line `index` of `place_answers`.
    /// A question that is gone is no error: the disk is taken and the questions refreshed.
    pub fn answer_place(&mut self, name: &str, index: usize) -> Result<(), ServiceError> {
        let answers = self.place_answers(name);
        if answers.is_empty() {
            return self.reload_places();
        }

        let Some(answer) = answers.into_iter().nth(index) else {
            return Err(ServiceError::NoPlaceAnswer(index));
        };

        self.resolve_place(name, answer.choice)
    }

    /// Settles the place name `name`: a new scripted location, a variant of one, or the new name
    /// of one. Written to `orte.toml` at once. A question that is gone (another program
    /// answered it, or a backup came back) is no error: the disk is taken, the questions are
    /// refreshed and the question disappears.
    pub fn resolve_place(&mut self, name: &str, choice: PlaceChoice) -> Result<(), ServiceError> {
        if !self.pending_places.iter().any(|p| p.name == name) {
            return self.reload_places();
        }

        // The file as it is now, not the session's copy: it may be older.
        let mut disk = klappe_store::load_places(&self.dir)?;
        if !places::decide(&mut disk, name, &choice) {
            return self.reload_places();
        }
        klappe_store::save_places(&self.dir, &disk)?;

        self.places = disk;
        self.refresh_places();

        Ok(())
    }

    /// The scripted locations as `orte.toml` held them when last read or written.
    pub fn places(&self) -> &[Place] {
        &self.places
    }

    /// Takes the scripted locations from `orte.toml`: the file is their only source.
    pub(super) fn reload_places(&mut self) -> Result<(), ServiceError> {
        self.places = klappe_store::load_places(&self.dir)?;
        self.refresh_places();

        Ok(())
    }

    /// Asks the headings which place names are open, changing nothing.
    pub(super) fn refresh_places(&mut self) {
        self.pending_places = places::pending(&self.places, &self.script);
    }
}
