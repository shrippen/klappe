use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::Path;

use klappe_model::Project;
use klappe_store::Loaded;

use super::{check_empty, Autosave, BackupState, Session};
use crate::error::ServiceError;
use crate::field::Field;
use crate::plan_edit::Snapping;

impl Session {
    /// Opens a project folder; the first scene with a plan is current.
    pub fn open(dir: &Path) -> Result<Self, ServiceError> {
        let loaded = klappe_store::load(dir)?;
        let backup = check_backup(dir, &loaded);
        let script = loaded
            .script
            .as_deref()
            .map(klappe_script::parse)
            .unwrap_or_default();
        let scene = loaded
            .project
            .scenes
            .iter()
            .position(|s| !s.plans.is_empty())
            .unwrap_or(0);

        Ok(Self {
            dir: dir.to_owned(),
            project: loaded.project,
            script_text: loaded.script,
            script,
            scene,
            plan: 0,
            time_s: 0.0,
            selected_shot: None,
            hidden_layers: HashSet::new(),
            locked_layers: HashSet::new(),
            hidden_tracks: HashSet::new(),
            snapping: Snapping::On,
            undo: Vec::new(),
            redo: Vec::new(),
            dirty: false,
            image_px: RefCell::new(HashMap::new()),
            unbacked: false,
            project_changed: false,
            backup,
        })
    }

    /// Creates an empty project titled `title` in `dir` (missing or empty) and opens it.
    pub fn create(dir: &Path, title: &str) -> Result<Self, ServiceError> {
        let title = title.trim();
        if title.is_empty() {
            return Err(ServiceError::BadValue {
                field: Field::Title,
                value: String::new(),
            });
        }
        check_empty(dir)?;

        klappe_store::save(dir, &Project::new(title), None)?;
        Self::open(dir)
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn project(&self) -> &Project {
        &self.project
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Writes the project back to its folder and removes the backup. An unanswered backup
    /// (`Found`, `Unreadable`) stays, as with `autosave`: the user decides on it.
    pub fn save(&mut self) -> Result<(), ServiceError> {
        klappe_store::save(&self.dir, &self.project, self.script_text.as_deref())?;
        self.dirty = false;
        self.unbacked = false;
        self.project_changed = false;
        if self.backup != BackupState::None {
            return Ok(());
        }

        klappe_store::drop_backup(&self.dir)?;
        Ok(())
    }

    /// Writes the unsaved state into the backup, if it changed since the last save or
    /// autosave. The project files stay untouched. The surface calls this on a timer.
    pub fn autosave(&mut self) -> Result<Autosave, ServiceError> {
        if self.backup != BackupState::None {
            return Ok(Autosave::Held);
        }
        if !self.unbacked {
            return Ok(Autosave::Clean);
        }

        klappe_store::save_backup(&self.dir, &self.project, self.script_text.as_deref())?;
        self.unbacked = false;
        Ok(Autosave::Written)
    }

    /// The backup `open` found.
    pub fn backup(&self) -> &BackupState {
        &self.backup
    }

    /// Takes project and screenplay from the backup. The project change can be undone; the
    /// screenplay text cannot (undo covers the project only). The backup's screenplay replaces
    /// the one on disk at the next save, also a newer one saved after the crash.
    pub fn restore_backup(&mut self) -> Result<(), ServiceError> {
        let loaded = klappe_store::load_backup(&self.dir)?.ok_or(ServiceError::NoBackup)?;

        self.commit(loaded.project);
        if let Some(text) = loaded.script {
            self.set_script_text(&text);
        }
        self.keep_view_valid();
        self.backup = BackupState::None;
        Ok(())
    }

    /// Removes the backup.
    pub fn discard_backup(&mut self) -> Result<(), ServiceError> {
        klappe_store::drop_backup(&self.dir)?;
        self.backup = BackupState::None;
        Ok(())
    }

    /// Writes only the screenplay; every other file of the folder stays as it is on disk.
    /// The backup then no longer holds the screenplay as unsaved work: it is removed if only
    /// the screenplay was unsaved, else rewritten with the saved screenplay. An unanswered
    /// backup (`Found`, `Unreadable`) stays for the user to decide.
    pub fn save_script(&mut self) -> Result<(), ServiceError> {
        let text = self.script_text.as_deref().unwrap_or_default();
        klappe_store::save_script(&self.dir, text)?;
        if self.backup != BackupState::None {
            return Ok(());
        }

        if self.project_changed {
            klappe_store::save_backup(&self.dir, &self.project, self.script_text.as_deref())?;
            self.unbacked = false;
            return Ok(());
        }

        self.dirty = false;
        self.unbacked = false;
        klappe_store::drop_backup(&self.dir)?;
        Ok(())
    }

    /// The screenplay as Fountain text (`""` when the project has none yet).
    pub fn script_text(&self) -> &str {
        self.script_text.as_deref().unwrap_or_default()
    }

    /// Replaces the screenplay (the TUI editor, E-09); written with the next `save` or `save_script`.
    pub fn set_script_text(&mut self, text: &str) {
        if self.script_text.as_deref() == Some(text) {
            return;
        }
        self.script = klappe_script::parse(text);
        self.script_text = Some(text.to_owned());
        self.touch();
    }
}

/// Looks at the backup next to the project. One equal to the project is removed (nothing
/// would be lost); an unreadable one is reported, not fatal.
fn check_backup(dir: &Path, loaded: &Loaded) -> BackupState {
    match klappe_store::load_backup(dir) {
        Ok(None) => BackupState::None,
        Ok(Some(backup)) if backup == *loaded => {
            // A failed removal is harmless: the next open compares again.
            let _ = klappe_store::drop_backup(dir);
            BackupState::None
        }
        Ok(Some(_)) => BackupState::Found,
        Err(e) => BackupState::Unreadable(e.to_string()),
    }
}
