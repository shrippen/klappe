//! Locked versions of the screenplay (DB-40) and what changed since (DB-41, DB-42, DB-44).
//!
//! ```text
//! lock_version ──► save ──► pages anew ──► numbers locked ──► text + list of the version
//! revision     ──► the book now against the last version ──► Comparison (script/)
//! ```
//!
//! The versions belong to the session like the Sperrstand of the scene numbers: no undo step,
//! no part of the project's save; locking and deleting write them at once.

use klappe_model::{free_random_id, IdCase, RevisionColor, SceneNumbering, Version};
use klappe_script::{compare, Comparison};

use super::{OnOutside, Saved, Session};
use crate::catalog::Language;
use crate::error::ServiceError;
use crate::script_pages::ScriptPrint;
use crate::versions::{check_name, header_text, text_language, today};

/// A state of the book to compare.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Draft {
    /// The locked version with this id.
    Version(String),
    /// The book as the session holds it now.
    Current,
}

/// Whether a screenplay PDF marks what changed since the last locked version (DB-41).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Revisions {
    Off,
    On,
}

/// What the revision adds to the pages of a PDF.
pub(crate) struct Revision {
    /// The lines (from 0) of the text that are new or changed.
    pub(crate) marked: std::collections::HashSet<usize>,
    /// The line above the page number.
    pub(crate) header: String,
}

impl Session {
    /// The locked versions in the order they were locked.
    pub fn versions(&self) -> &[Version] {
        &self.versions
    }

    /// Locks the book as a version called `name` (DB-40): saves the project as Ctrl+S does
    /// (`on` as for `save`; on `Conflict` nothing is locked), lays the pages out anew (DB-34),
    /// locks the scene numbers and keeps the text and the list. The version gets the colour of
    /// its place in the order. No undo step: a locked version is as paper. An empty or used
    /// name is refused before anything is written.
    ///
    /// `language` is the surface's, for the pages' words while the project has none.
    pub fn lock_version(
        &mut self,
        name: &str,
        language: Language,
        on: OnOutside,
    ) -> Result<Saved, ServiceError> {
        let name = check_name(name, &self.versions)?.to_owned();
        if self.script_text().trim().is_empty() {
            return Err(ServiceError::NoScript);
        }

        // Numbers first (a save numbers by the old setting), then the lock on what was saved.
        if self.save(on)? == Saved::Conflict {
            return Ok(Saved::Conflict);
        }
        self.paginate(language);
        let first = self.project.script.first_scene;
        self.set_scene_numbering(SceneNumbering::Locked, first)?;

        let taken = self.versions.iter().map(|v| v.id.as_str());
        let version = Version {
            id: free_random_id(taken, IdCase::Ignored),
            name,
            date: today(),
            color: RevisionColor::nth(self.versions.len()),
        };

        // The text first, so the list never names a text that is not there.
        klappe_store::save_version_text(&self.dir, &version.id, self.script_text())?;
        let mut versions = self.versions.clone();
        versions.push(version);
        klappe_store::save_versions(&self.dir, &versions)?;
        self.versions = versions;

        Ok(Saved::Done)
    }

    /// Deletes version `id` with its text; no undo step. The colours of the others stay.
    pub fn remove_version(&mut self, id: &str) -> Result<(), ServiceError> {
        let mut versions = self.versions.clone();
        let at = versions
            .iter()
            .position(|v| v.id == id)
            .ok_or_else(|| ServiceError::NoVersion(id.to_owned()))?;
        versions.remove(at);

        // The list first: a text left behind is harmless, a name without text is not.
        klappe_store::save_versions(&self.dir, &versions)?;
        self.versions = versions;
        klappe_store::remove_version_text(&self.dir, id)?;

        Ok(())
    }

    /// What differs between two states of the book: scene by scene, moved scenes found,
    /// lines of a changed scene told apart (DB-42, DB-44).
    pub fn compare_drafts(&self, from: &Draft, to: &Draft) -> Result<Comparison, ServiceError> {
        let (old, new) = (self.draft_text(from)?, self.draft_text(to)?);

        Ok(compare(&old, &new))
    }

    /// The revision: the book now against the last locked version (DB-41).
    pub fn revision(&self) -> Result<Comparison, ServiceError> {
        let last = self.versions.last().ok_or(ServiceError::NoVersions)?;

        self.compare_drafts(&Draft::Version(last.id.clone()), &Draft::Current)
    }

    fn draft_text(&self, draft: &Draft) -> Result<String, ServiceError> {
        let Draft::Version(id) = draft else {
            return Ok(self.script_text().to_owned());
        };
        if !self.versions.iter().any(|v| v.id == *id) {
            return Err(ServiceError::NoVersion(id.clone()));
        }

        Ok(klappe_store::load_version_text(&self.dir, id)?)
    }

    /// What a PDF adds for the revision, if asked for and there is a version to compare with:
    /// the lines to mark and the header of the next version.
    pub(super) fn revision_of(
        &self,
        print: &ScriptPrint,
    ) -> Result<Option<Revision>, ServiceError> {
        if print.revisions == Revisions::Off || self.versions.is_empty() {
            return Ok(None);
        }

        let language = text_language(&self.project.script, print.language);
        let next = RevisionColor::nth(self.versions.len());

        Ok(Some(Revision {
            marked: self.revision()?.marked_lines().into_iter().collect(),
            header: header_text(language, next, &today()),
        }))
    }
}
