//! Changes made to the project folder outside the session (the terminal beside the window, a
//! Git pull, a text editor), and what a save writes of its own.
//!
//! ```text
//!            disk (last read or written)      own (the session's encoding at that moment)
//! check:     disk_now != disk  ──► changed outside
//!            encode(now) != own ─► changed by the session
//!            outside only ──► taken over     both ──► asked (`Saved::Conflict`)
//! save:      writes the files where encode(now) != disk, nothing else
//! ```
//!
//! A file is a unit of change (`klappe_store::read_tree`). Normalisation on load (a hand
//! formatted file written the way the program writes it) is no own change: it counts against
//! `own`, which is taken after it. `orte.toml` and `nummern.toml` are read along, never own.

use std::collections::BTreeSet;

use klappe_store::{kind_of, Loaded, Tree};

use super::Session;
use crate::error::ServiceError;
use crate::shot_edit;

/// What `save` does when a file changed outside and in the session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnOutside {
    /// Writes nothing and says so (`Saved::Conflict`).
    Ask,
    /// Writes the session's version of that file.
    Overwrite,
}

/// What `save` did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Saved {
    Done,
    /// A file changed outside and in the session; nothing was written (`OnOutside::Ask`).
    Conflict,
}

/// A file of the project folder, named for the user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutsideFile {
    pub kind: klappe_store::FileKind,
    /// Below the project folder, `/` between the parts (`szenen/ab12.json`).
    pub name: String,
}

/// What `check` found.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Outside {
    /// Changed outside only: now the session's.
    pub taken: Vec<OutsideFile>,
    /// Changed outside and in the session: the session's version stays, nothing is lost yet.
    pub both: Vec<OutsideFile>,
}

/// The files that differ between two trees, by what each side holds.
struct Scan {
    disk_now: Tree,
    own_now: Tree,
    /// Changed in the session since `own`, by name.
    mine: BTreeSet<String>,
    taken: Vec<String>,
    both: Vec<String>,
}

impl Session {
    /// Compares the folder with what the session last saw of it. A file changed outside that
    /// the session did not change becomes the session's (undo is cleared, a screenplay or
    /// plan edit would not fit the new state); one both changed is reported in `both` and
    /// stays as the session has it, until `save` or `reload` settles it. An unreadable
    /// outside file is an error and changes nothing.
    pub fn check(&mut self) -> Result<Outside, ServiceError> {
        let scan = self.scan()?;
        let both = self.files(&scan.both);

        if scan.taken.is_empty() {
            self.conflicts.clone_from(&both);
            self.agree(&scan);
            self.reload_places()?;

            return Ok(Outside {
                taken: Vec::new(),
                both,
            });
        }

        self.take_outside(&scan)?;
        self.conflicts.clone_from(&both);

        Ok(Outside {
            taken: self.files(&scan.taken),
            both,
        })
    }

    /// The files `check` found changed outside and in the session, not yet settled.
    pub fn outside(&self) -> &[OutsideFile] {
        &self.conflicts
    }

    /// Drops the session's own changes and reads the folder again; undo is cleared and an
    /// unanswered backup removed, as a save does.
    pub fn reload(&mut self) -> Result<(), ServiceError> {
        let disk = klappe_store::read_tree(&self.dir)?;
        let loaded = crate::load::load_project(&self.dir)?;

        self.take(loaded)?;
        self.know_figures();
        self.disk = disk;
        self.own = self.encode()?;
        self.conflicts.clear();
        self.forget_history();
        self.dirty = false;
        self.unbacked = false;
        self.project_changed = false;
        klappe_store::drop_backup(&self.dir)?;
        self.backup = super::BackupState::None;

        Ok(())
    }

    /// Writes the project back to its folder and removes the backup: only the files that
    /// differ from what is on disk. A file changed outside and in the session is `Conflict`
    /// with `Ask` and written over with `Overwrite`; one changed outside only was taken over
    /// first. An unanswered backup (`Found`, `Unreadable`) stays: the user decides on it.
    pub fn save(&mut self, on: OnOutside) -> Result<Saved, ServiceError> {
        // What the terminal answered since the file was read goes into the project first.
        self.adopt_figures()?;
        match (self.check(), on) {
            (Ok(outside), OnOutside::Ask) if !outside.both.is_empty() => {
                return Ok(Saved::Conflict);
            }
            (Err(error), OnOutside::Ask) => return Err(error),
            _ => {}
        }
        if on == OnOutside::Overwrite {
            self.disk = klappe_store::read_tree(&self.dir)?;
            self.conflicts.clear();
        }

        self.number_script();
        // So the register holds the book as saved: the next outside change aligns against it.
        self.reconcile();
        let now = self.encode()?;
        klappe_store::write_changes(&self.dir, &changes(&self.disk, &now))?;
        self.disk.clone_from(&now);
        self.own = now;
        self.know_figures();
        self.write_lock()?;
        self.dirty = false;
        self.unbacked = false;
        self.project_changed = false;
        if self.backup != super::BackupState::None {
            return Ok(Saved::Done);
        }

        klappe_store::drop_backup(&self.dir)?;

        Ok(Saved::Done)
    }

    /// Makes `loaded` the project and the screenplay, and brings what follows from them in
    /// line: the numbers of the lock file, the places, the register against the book, the
    /// view (it stays on the scene and plan it showed), a look through a camera that is gone.
    /// Used by `open`, `reload` and a take-over. Fails before anything changes.
    pub(super) fn take(&mut self, mut loaded: Loaded) -> Result<(), ServiceError> {
        let lock = klappe_store::load_lock(&self.dir)?;
        let places = klappe_store::load_places(&self.dir)?;
        let (scene, plan) = self.ids_on_view();

        shot_edit::clear_orphans(&mut loaded.project);
        self.script = loaded
            .script
            .as_deref()
            .map(klappe_script::parse)
            .unwrap_or_default();
        self.project = loaded.project;
        self.script_text = loaded.script;
        self.load_notes = loaded.notes;
        self.lock = lock;
        self.places = places;
        self.follow_scene(scene, plan);
        self.reconcile();
        self.end_stale_look();
        self.edits = super::undo::next_edit();

        Ok(())
    }

    /// The tree of what the session holds now.
    pub(super) fn encode(&self) -> Result<Tree, ServiceError> {
        Ok(klappe_store::encode_tree(
            &self.project,
            self.script_text.as_deref(),
        )?)
    }

    /// Tells the files apart: changed outside, changed by the session, both.
    fn scan(&self) -> Result<Scan, ServiceError> {
        let disk_now = klappe_store::read_tree(&self.dir)?;
        let own_now = self.encode()?;
        let mut scan = Scan {
            disk_now,
            own_now,
            mine: BTreeSet::new(),
            taken: Vec::new(),
            both: Vec::new(),
        };

        let names: BTreeSet<&String> = [&scan.disk_now, &scan.own_now, &self.disk, &self.own]
            .into_iter()
            .flat_map(|tree| tree.keys())
            .collect();
        for name in names {
            let outside = scan.disk_now.get(name) != self.disk.get(name);
            let mine = scan.own_now.get(name) != self.own.get(name);
            if mine {
                scan.mine.insert(name.clone());
            }
            if !outside {
                continue;
            }

            // The same change on both sides needs no question.
            let same = scan.disk_now.get(name) == scan.own_now.get(name);
            match (mine, same) {
                (false, _) => scan.taken.push(name.clone()),
                (true, false) => scan.both.push(name.clone()),
                (true, true) => {}
            }
        }

        Ok(scan)
    }

    /// Settles the files both sides changed to the same bytes: nothing to ask or write.
    fn agree(&mut self, scan: &Scan) {
        for name in &scan.mine {
            let same = scan.disk_now.get(name) == scan.own_now.get(name);
            if !same || scan.both.contains(name) {
                continue;
            }
            copy_entry(&mut self.disk, &scan.disk_now, name);
            copy_entry(&mut self.own, &scan.own_now, name);
        }
    }

    /// Takes over what changed outside and the session did not change; the files it changed
    /// itself stay as they are.
    fn take_outside(&mut self, scan: &Scan) -> Result<(), ServiceError> {
        let loaded = match scan.mine.is_empty() {
            true => crate::load::load_project(&self.dir)?,
            false => crate::load::load_overlay(&self.dir, &self.overlay(scan))?,
        };
        let taken_figures = scan
            .taken
            .iter()
            .any(|n| kind_of(n) == Some(klappe_store::FileKind::Figures));

        self.take(loaded)?;
        if taken_figures {
            self.know_figures();
        }

        // What the folder holds now; the files both changed keep the answer they were asked.
        let mut disk = scan.disk_now.clone();
        for name in &scan.both {
            copy_entry(&mut disk, &self.disk, name);
        }
        self.disk = disk;

        // Own is what the session encodes now, except in the files it changed itself before:
        // a change that follows from the take-over (the register against a new screenplay) is
        // none of the session's.
        let now = self.encode()?;
        let names: BTreeSet<String> = now.keys().chain(self.own.keys()).cloned().collect();
        for name in names.iter().filter(|n| !scan.mine.contains(*n)) {
            copy_entry(&mut self.own, &now, name);
        }

        self.forget_history();
        let own_changes: Vec<&String> = names
            .iter()
            .filter(|n| now.get(*n) != self.own.get(*n))
            .collect();
        self.dirty = !own_changes.is_empty();
        self.unbacked = self.dirty;
        self.project_changed = own_changes
            .iter()
            .any(|n| kind_of(n) != Some(klappe_store::FileKind::Script));

        Ok(())
    }

    /// The folder as it is, with the files the session changed in their own version.
    fn overlay(&self, scan: &Scan) -> Tree {
        let mut tree = scan.disk_now.clone();
        for name in &scan.mine {
            copy_entry(&mut tree, &scan.own_now, name);
        }

        tree
    }

    /// Clears undo and redo: their steps hold the state from before.
    fn forget_history(&mut self) {
        self.undo.clear();
        self.redo.clear();
        self.disk_step = None;
    }

    /// `names` as files for the surface.
    fn files(&self, names: &[String]) -> Vec<OutsideFile> {
        names
            .iter()
            .filter_map(|name| {
                let kind = kind_of(name)?;

                Some(OutsideFile {
                    kind,
                    name: name.clone(),
                })
            })
            .collect()
    }

    /// The files `names` were just written whole from the session's state (`save_script`):
    /// the folder and the session agree on them.
    pub(super) fn wrote(&mut self, names: &[&str]) -> Result<(), ServiceError> {
        let now = self.encode()?;
        for name in names {
            copy_entry(&mut self.disk, &now, name);
            copy_entry(&mut self.own, &now, name);
        }

        Ok(())
    }
}

/// Makes the entry `name` of `to` the one of `from`, absent if it is absent there.
fn copy_entry(to: &mut Tree, from: &Tree, name: &str) {
    match from.get(name) {
        Some(bytes) => to.insert(name.to_owned(), bytes.clone()),
        None => to.remove(name),
    };
}

/// What to write to turn `disk` into `now`: the files that differ, and the files that are gone.
/// The screenplay is never removed: the session not holding one is no wish to delete it.
fn changes(disk: &Tree, now: &Tree) -> Vec<(String, Option<Vec<u8>>)> {
    let written = now
        .iter()
        .filter(|(name, bytes)| disk.get(*name) != Some(bytes))
        .map(|(name, bytes)| (name.clone(), Some(bytes.clone())));
    let removed = disk
        .keys()
        .filter(|name| !now.contains_key(*name))
        .filter(|name| kind_of(name) != Some(klappe_store::FileKind::Script))
        .map(|name| (name.clone(), None));

    written.chain(removed).collect()
}
