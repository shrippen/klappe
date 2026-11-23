//! The scripted locations against the book (DB-51): the place names of the scene headings no
//! scripted location is called by, and the user's answer for each.
//!
//! The answer is written to `orte.toml` at once, from the file as it is on disk: another
//! program (the TUI beside the window) may have changed it, and the answer changes only what
//! it touches. It is one undo step that carries its entry, not the places: undo and redo
//! change only that entry in the file as it is (`save_answer_step`, `place_file.rs`).
//! The list of scripted locations (D-21) writes it the same way: a change of one scripted
//! location is one undo step that carries its entry. Nothing else writes the file: a save
//! leaves it alone.

use klappe_model::Place;

use super::questions::Answered;
use super::undo::AnswerDelta;
use super::Session;
use crate::answer_way::Way;
use crate::error::ServiceError;
use crate::field::Field;
use crate::place_file::PlaceDelta;
use crate::places::{self, PendingPlace, PlaceChoice, PlaceEdit};

/// Undo step of `resolve_place`.
pub(super) const RESOLVE_STEP: &str = "ResolvePlace";
/// Undo steps of `edit_place` and `remove_place`.
const EDIT_STEP: &str = "EditPlace";
const REMOVE_STEP: &str = "RemovePlace";

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
    pub fn answer_place(&mut self, name: &str, index: usize) -> Result<Answered, ServiceError> {
        let answers = self.place_answers(name);
        if answers.is_empty() {
            return self.reload_places().map(|()| Answered::Gone);
        }

        let Some(answer) = answers.into_iter().nth(index) else {
            return Err(ServiceError::NoPlaceAnswer(index));
        };

        self.resolve_place(name, answer.choice)
    }

    /// Settles the place name `name`: a new scripted location, a variant of one, or the new name
    /// of one. One undo step, written to `orte.toml` at once. A question that is gone (another program
    /// answered it, or a backup came back) is no error: the disk is taken, the questions are
    /// refreshed and the question disappears.
    pub fn resolve_place(
        &mut self,
        name: &str,
        choice: PlaceChoice,
    ) -> Result<Answered, ServiceError> {
        if !self.pending_places.iter().any(|p| p.name == name) {
            return self.reload_places().map(|()| Answered::Gone);
        }

        let delta = self.rewrite_places(|disk| {
            let before = disk.clone();
            match places::decide(disk, name, &choice) {
                true => PlaceDelta::of(&before, &choice, disk),
                false => None,
            }
        })?;
        let Some(delta) = delta else {
            return self.reload_places().map(|()| Answered::Gone);
        };

        let (scene, shown) = self.ids_on_view();
        self.commit_answer(
            self.project.clone(),
            RESOLVE_STEP,
            AnswerDelta::Place(delta),
        );
        self.follow_scene(scene, shown);

        Ok(Answered::Step)
    }

    /// Changes scripted location `id` as the list of scripted locations says (D-21): its name,
    /// other spellings and shooting location. A name the headings use and the edit drops stays
    /// as a spelling. One undo step, written to `orte.toml` at once like an answer.
    pub fn edit_place(&mut self, id: &str, edit: &PlaceEdit) -> Result<(), ServiceError> {
        if let Some(location) = &edit.location {
            if self.project.location(location).is_none() {
                return Err(ServiceError::NoLocation(location.clone()));
            }
        }

        // The file as it is now, not the session's copy: it may be older.
        let disk = klappe_store::load_places(&self.dir)?;
        let Some(before) = disk.iter().find(|p| p.id == id).cloned() else {
            self.reload_places()?;
            return Err(ServiceError::Gone);
        };
        let after = places::edited(&disk, &self.script, id, edit).map_err(|value| {
            ServiceError::BadValue {
                field: Field::Name,
                value,
            }
        })?;
        if after == before {
            return Ok(());
        }

        self.write_place(PlaceDelta::Edited { before, after }, EDIT_STEP)
    }

    /// Takes scripted location `id` out of the list; refused while a heading names it. One undo
    /// step, written to `orte.toml` at once.
    pub fn remove_place(&mut self, id: &str) -> Result<(), ServiceError> {
        let disk = klappe_store::load_places(&self.dir)?;
        let Some(place) = disk.iter().find(|p| p.id == id).cloned() else {
            self.reload_places()?;
            return Err(ServiceError::Gone);
        };
        let headings = places::headings_of(&place, &self.script);
        if headings > 0 {
            return Err(ServiceError::PlaceInUse { headings });
        }

        self.write_place(PlaceDelta::Removed(place), REMOVE_STEP)
    }

    /// Writes `delta` to `orte.toml` as it is on disk and keeps it as undo step `step`.
    fn write_place(&mut self, delta: PlaceDelta, step: &str) -> Result<(), ServiceError> {
        self.save_place_delta(&delta, Way::Apply)?;

        let (scene, shown) = self.ids_on_view();
        self.commit_answer(self.project.clone(), step, AnswerDelta::Place(delta));
        self.follow_scene(scene, shown);

        Ok(())
    }

    /// Undo or redo of a place answer: its entry in `orte.toml`, in the file as it is on disk.
    pub(super) fn save_place_delta(
        &mut self,
        delta: &PlaceDelta,
        way: Way,
    ) -> Result<(), ServiceError> {
        self.rewrite_places(|disk| {
            delta.run(disk, way);
            Some(())
        })
        .map(|_| ())
    }

    /// The one way `orte.toml` is written: reads it from disk, lets `change` alter it, writes it,
    /// takes it as the session's places and recomputes the open questions. `change` returning
    /// `None` leaves the file alone.
    fn rewrite_places<T>(
        &mut self,
        change: impl FnOnce(&mut Vec<Place>) -> Option<T>,
    ) -> Result<Option<T>, ServiceError> {
        // The file as it is now, not the session's copy: it may be older.
        let mut disk = klappe_store::load_places(&self.dir)?;
        let Some(done) = change(&mut disk) else {
            return Ok(None);
        };
        klappe_store::save_places(&self.dir, &disk)?;

        self.places = disk;
        self.refresh_places();

        Ok(Some(done))
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
