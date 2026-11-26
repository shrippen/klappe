//! Locked versions of the screenplay (DB-40) and what changed since (DB-41, DB-42, DB-44).
//!
//! ```text
//! lock_version ──► save ──► pages anew ──► numbers locked ──► text + list of the version
//! revision     ──► the book now against the last version ──► Comparison (script/)
//! ```
//!
//! The versions belong to the session like the Sperrstand of the scene numbers: no undo step,
//! no part of the project's save; locking and deleting write them at once.

use klappe_model::{free_random_id, IdCase, PageStart, RevisionColor, SceneNumbering, Version};
use klappe_script::{compare, Comparison};

use super::{OnOutside, Saved, Session};
use crate::catalog::Language;
use crate::error::ServiceError;
use crate::script_pages::ScriptPrint;
use crate::versions::{check_name, header_text, next_color, text_language, today};

/// A state of the book to compare.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Draft {
    /// The locked version with this id.
    Version(String),
    /// The book as the session holds it now.
    Current,
}

/// Whether a screenplay PDF marks what changed since the last locked version (DB-41), and
/// whether it prints only the changed pages (DB-43).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Revisions {
    Off,
    On,
    /// The marks (DB-41), and only the pages that have one (DB-43), without a title page.
    ChangedPages,
}

/// What the revision adds to the pages of a PDF.
pub(crate) struct Revision {
    /// The lines (from 0) of the text that are new or changed.
    pub(crate) marked: std::collections::HashSet<usize>,
    /// The line above the page number.
    pub(crate) header: String,
    /// The numbers of the pages the last version had.
    pub(crate) old_pages: Vec<String>,
    /// The numbers of the pages whose text differs from the page of that number in the last
    /// version (DB-43).
    pub(crate) changed: Vec<String>,
}

impl Session {
    /// The locked versions in the order they were locked.
    pub fn versions(&self) -> &[Version] {
        &self.versions
    }

    /// The colour of the revision since the last version (DB-41), the one the next version
    /// gets; none before the first version.
    pub fn revision_color(&self) -> Option<RevisionColor> {
        self.versions.last()?;

        Some(next_color(&self.versions))
    }

    /// Locks the book as a version called `name` (DB-40): saves the project as Ctrl+S does
    /// (`on` as for `save`; on `Conflict` nothing is locked), lays the pages out anew (DB-34),
    /// locks the scene numbers and keeps the text and the list. The version gets the colour
    /// after the last version's. No undo step: a locked version is as paper. An empty or used
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

        // The pages as they were just laid out: they stay as they are from now on (DB-34).
        let pages = match &self.layout {
            Some(layout) => layout.page_starts(self.script_text()),
            None => Vec::new(),
        };

        let taken = self.versions.iter().map(|v| v.id.as_str());
        let version = Version {
            id: free_random_id(taken, IdCase::Ignored),
            name,
            date: today(),
            color: next_color(&self.versions),
            pages,
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
        if print.revisions == Revisions::Off {
            return Ok(None);
        }
        if self.versions.is_empty() {
            return match print.revisions {
                Revisions::ChangedPages => Err(ServiceError::NoVersions),
                _ => Ok(None),
            };
        }

        let language = text_language(&self.project.script, print.language);
        let next = next_color(&self.versions);

        let last = self.versions.last().ok_or(ServiceError::NoVersions)?;
        if print.revisions == Revisions::ChangedPages && last.pages.is_empty() {
            return Err(ServiceError::NoLockedPages);
        }

        Ok(Some(Revision {
            marked: self.revision()?.marked_lines().into_iter().collect(),
            header: header_text(language, next, &today()),
            old_pages: last.pages.iter().map(|p| p.label.clone()).collect(),
            changed: self.changed_pages(print, last)?,
        }))
    }

    /// The numbers of the pages of the layout now whose text differs from the page of that
    /// number in `last` (DB-43); only a print of the changed pages asks.
    fn changed_pages(
        &self,
        print: &ScriptPrint,
        last: &Version,
    ) -> Result<Vec<String>, ServiceError> {
        let Some(layout) = self
            .layout
            .as_ref()
            .filter(|_| print.revisions == Revisions::ChangedPages)
        else {
            return Ok(Vec::new());
        };
        let before = self.draft_text(&Draft::Version(last.id.clone()))?;
        let before = page_texts(&before, &last.pages);
        let now = page_texts(self.script_text(), &layout.page_starts(self.script_text()));

        Ok(now
            .into_iter()
            .filter(|(label, text)| before.iter().any(|(l, t)| l == label && t != text))
            .map(|(label, _)| label)
            .collect())
    }
}

/// The text of the lines `from..to`, as the comparison reads it: no blank line, no
/// surrounding space.
fn page_text(lines: &[&str], from: usize, to: usize) -> String {
    let end = to.min(lines.len());
    let from = from.min(end);

    lines[from..end]
        .iter()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

/// The text of each page of `starts` in `text`, by number: the lines up to the next page's.
fn page_texts(text: &str, starts: &[PageStart]) -> Vec<(String, String)> {
    let text = text.replace("\r\n", "\n").replace('\r', "\n");
    let lines: Vec<&str> = text.split('\n').collect();

    starts
        .iter()
        .enumerate()
        .map(|(i, page)| {
            let end = starts.get(i + 1).map_or(lines.len(), |next| next.line);

            (page.label.clone(), page_text(&lines, page.line, end))
        })
        .collect()
}
