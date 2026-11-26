use std::collections::HashSet;
use std::path::Path;

use klappe_model::{Project, SceneNumbering};
use klappe_store::Loaded;

use super::outside::{OnOutside, Saved, REGISTER_FILE, SCRIPT_FILE};
use super::plan_view::PlanView;
use super::{check_empty, check_title, Autosave, BackupState, Session, View};
use crate::book::ImportReport;
use crate::catalog::Language;
use crate::error::ServiceError;
use crate::plan_edit::{Grid, Snapping};
use crate::shot_edit;

/// The extension of a Final Draft file, whatever its case.
const FDX_EXTENSION: &str = "fdx";

impl Session {
    /// Opens a project folder; the first scene with a plan is current.
    pub fn open(dir: &Path) -> Result<Self, ServiceError> {
        // Read first: a change between the two reads is then taken over by the next check.
        let disk = klappe_store::read_tree(dir)?;
        let loaded = crate::load::load_project(dir)?;

        let mut session = Self {
            dir: dir.to_owned(),
            project: Project::new(""),
            script_text: None,
            script: Default::default(),
            scene: 0,
            plan: 0,
            time_s: 0.0,
            selected_shot: None,
            locked_layers: HashSet::new(),
            snapping: Snapping::On,
            grid: Grid::default(),
            undo: Vec::new(),
            redo: Vec::new(),
            dirty: false,
            plan_view: PlanView::new(),
            unbacked: false,
            project_changed: false,
            edits: super::undo::next_edit(),
            backup: BackupState::None,
            load_notes: Vec::new(),
            view: View::Plan,
            orbit: None,
            cut: klappe_stellprobe::Cut::Off,
            through: None,
            layout: None,
            lengths: Default::default(),
            pending: Vec::new(),
            pending_figures: Vec::new(),
            disk_step: None,
            file_figures: Default::default(),
            book_scene: Vec::new(),
            heading_line: Vec::new(),
            places: Vec::new(),
            pending_places: Vec::new(),
            lock: None,
            twin_numbers: 0,
            disk,
            own: Default::default(),
            conflicts: Vec::new(),
        };
        session.take(loaded)?;
        session.know_figures();
        session.own = session.encode()?;
        session.scene = session
            .project
            .scenes
            .iter()
            .position(|s| !s.plans.is_empty())
            .unwrap_or(0);

        // Against the register as it follows the book: a backup that only differs by that is
        // no unsaved work.
        let reconciled = Loaded {
            project: session.project.clone(),
            script: session.script_text.clone(),
            notes: session.load_notes.clone(),
        };
        session.backup = check_backup(dir, &reconciled);

        Ok(session)
    }

    /// Creates an empty project titled `title` in `dir` (missing or empty) and opens it. The
    /// project's language is `language`, the surface's (DN-05).
    pub fn create(dir: &Path, title: &str, language: Language) -> Result<Self, ServiceError> {
        let title = check_title(title)?;
        check_empty(dir)?;

        let mut project = Project::new(title);
        project.script.scene_numbering = SceneNumbering::Automatic;
        project.script.language = Some(language.into());
        klappe_store::save(dir, &project, None)?;
        klappe_store::save_lock(dir, &[])?;
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

    /// What `open` found odd in the project; shown once after opening.
    pub fn load_notes(&self) -> &[klappe_store::LoadNote] {
        &self.load_notes
    }

    /// The backup `open` found.
    pub fn backup(&self) -> &BackupState {
        &self.backup
    }

    /// Takes project and screenplay from the backup. The project change can be undone; the
    /// screenplay text cannot (undo covers the project only). The backup's screenplay replaces
    /// the one on disk at the next save, also a newer one saved after the crash.
    pub fn restore_backup(&mut self) -> Result<(), ServiceError> {
        let mut loaded = klappe_store::load_backup(&self.dir)?.ok_or(ServiceError::NoBackup)?;

        shot_edit::clear_orphans(&mut loaded.project);
        // The figures are the backup's alone: it cannot tell a character the file gained
        // since from one it deleted, so an answer given meanwhile is asked again.
        self.commit(loaded.project);
        if let Some(text) = loaded.script {
            self.set_script_text(&text);
        }
        // Another program may have answered meanwhile: the places come from the disk.
        self.reload_places()?;
        self.refresh_figures();
        self.reconcile();
        self.keep_view_valid();
        self.end_stale_look();
        self.backup = BackupState::None;
        Ok(())
    }

    /// Removes the backup.
    pub fn discard_backup(&mut self) -> Result<(), ServiceError> {
        klappe_store::drop_backup(&self.dir)?;
        self.backup = BackupState::None;
        Ok(())
    }

    /// Numbers the scenes (DB-33), aligns the register with the book, then writes the screenplay and the register file
    /// `szenen.toml`, not the scene files with the plans; every other file of the folder stays
    /// as it is on disk. The backup then no longer holds the screenplay as unsaved work: it
    /// is removed if only the screenplay was unsaved, else rewritten with the saved screenplay.
    /// An unanswered backup (`Found`, `Unreadable`) stays for the user to decide.
    /// A screenplay or register changed outside and in the session is `Conflict` with `Ask`,
    /// written over with `Overwrite`; the other files are not its business.
    pub fn save_script(&mut self, on: OnOutside) -> Result<Saved, ServiceError> {
        if self.script_asks(on)? {
            return Ok(Saved::Conflict);
        }

        self.number_script();
        self.reconcile();
        let text = self.script_text.as_deref().unwrap_or_default();
        klappe_store::save_script(&self.dir, text)?;
        klappe_store::save_register(&self.dir, &self.project.scenes)?;
        self.wrote(&[SCRIPT_FILE, REGISTER_FILE])?;
        self.conflicts
            .retain(|f| f.name != SCRIPT_FILE && f.name != REGISTER_FILE);
        self.write_lock()?;
        if self.backup != BackupState::None {
            return Ok(Saved::Done);
        }

        if self.project_changed {
            klappe_store::save_backup(&self.dir, &self.project, self.script_text.as_deref())?;
            self.unbacked = false;
            return Ok(Saved::Done);
        }

        self.dirty = false;
        self.unbacked = false;
        klappe_store::drop_backup(&self.dir)?;
        Ok(Saved::Done)
    }

    /// Whether `save_script` has to ask: the screenplay or register changed outside and in the
    /// session, and `on` is `Ask`.
    fn script_asks(&mut self, on: OnOutside) -> Result<bool, ServiceError> {
        // A file this save does not write that cannot be read is no reason to refuse.
        let both = match (self.check(), on) {
            (Ok(outside), _) => outside.both,
            (Err(error), OnOutside::Ask) if self.moved(&[SCRIPT_FILE, REGISTER_FILE])? => {
                return Err(error);
            }
            (Err(_), _) => Vec::new(),
        };
        let asks = both
            .iter()
            .any(|f| f.name == SCRIPT_FILE || f.name == REGISTER_FILE);

        Ok(on == OnOutside::Ask && asks)
    }

    /// Writes only the register file `szenen.toml`, as the session holds it: an answer about
    /// the book is a decision, kept at once where the book is edited without the plans (TUI).
    pub fn save_register(&mut self) -> Result<(), ServiceError> {
        let before = self.before(REGISTER_FILE)?;
        klappe_store::save_register(&self.dir, &self.project.scenes)?;
        self.rebase(REGISTER_FILE, before)?;

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
        self.refresh_pending();
    }

    /// Replaces the screenplay with the Fountain or FDX (`.fdx`, DB-72) file `path` (UTF-8) and aligns the register
    /// with it at once (D-16); unsaved until `save`, one undo step. A file that cannot be
    /// read changes nothing.
    pub fn import_script(&mut self, path: &Path) -> Result<ImportReport, ServiceError> {
        let text = klappe_store::load_script_file(path)?;
        let is_fdx = path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case(FDX_EXTENSION));
        if !is_fdx {
            return Ok(self.import_book(&text));
        }

        // The book is kept as Fountain, whatever it was read from.
        let script =
            klappe_script::read_fdx(&text).map_err(|e| ServiceError::BadFdx(e.to_string()))?;

        Ok(self.import_book(&klappe_script::write(&script)))
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
