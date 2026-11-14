use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use klappe_export::{Orientation, Page, Paper, Texts, Theme};
use klappe_model::{CameraPresets, FigureColor, FloorPlan, LinkTarget, Point, Project, Scene};
use klappe_script::{Element, Script};
use klappe_stellprobe::{
    axis_jumps, handles, lens_at, ordered_shots, render, DisplayList, Role, Shape, ShotOrder,
    Source,
};
use klappe_store::ImageStamp;
use klappe_store::Loaded;

use crate::edit::{self, ShotField};
use crate::error::ServiceError;
use crate::field::Field;
use crate::library;
use crate::plan_edit::{self, MarkInfo, PlanEdit, Snapping};
use crate::project_edit::{self, ProjectEdit};
use crate::shot_edit;
use crate::views::{
    Chip, LibraryEntry, LinkKind, Marker, ScriptLine, Segment, ShotRow, Track, TrackKind,
};

/// Groups of the plan the timeline's layer switches show or hide.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Layer {
    Set,
    Figures,
    Cameras,
    Lights,
    Notes,
    Drawings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Shown,
    Hidden,
}

/// A locked layer can be seen but not picked or changed (LH-15).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lock {
    Unlocked,
    Locked,
}

/// Undo keeps this many steps (LH-06 asks for no fixed limit within a session; this only
/// guards memory on very long sessions).
const UNDO_LIMIT: usize = 500;

/// Id of the cut-order track.
const CUT_TRACK: &str = "cut";

/// Colours the cut order gives its cameras, in camera order.
const CAMERA_LABELS: [&str; 4] = ["yellow", "green", "cyan", "orange"];

/// Colour of the lights' tracks.
const LIGHT_LABEL: &str = "orange";

/// Colour of the cut track.
const CUT_LABEL: &str = "yellow";

/// What `Session::autosave` did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Autosave {
    /// The backup now holds the current state.
    Written,
    /// Nothing changed since the last save or autosave.
    Clean,
    /// An unanswered backup from an earlier run is on disk; writing would destroy it.
    /// `restore_backup` or `discard_backup` first.
    Held,
}

/// The backup of unsaved changes that `Session::open` found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackupState {
    None,
    /// Differs from the saved project: offer `restore_backup` and `discard_backup`.
    Found,
    /// A backup exists but cannot be read (the message says why): offer `discard_backup`.
    /// `open` must not fail on it, or the project could not be opened at all.
    Unreadable(String),
}

/// The pixel size of an image file and the stamp it was read at.
type StampedSize = (ImageStamp, (u32, u32));

/// One open project and what the user looks at.
#[derive(Debug)]
pub struct Session {
    dir: PathBuf,
    project: Project,
    script_text: Option<String>,
    script: Script,
    scene: usize,
    plan: usize,
    time_s: f64,
    selected_shot: Option<String>,
    hidden_layers: HashSet<Layer>,
    locked_layers: HashSet<Layer>,
    hidden_tracks: HashSet<String>,
    snapping: Snapping,
    undo: Vec<Project>,
    redo: Vec<Project>,
    dirty: bool,
    /// Pixel size of the image files read so far: every redraw asks, the file header is read
    /// once. `bilder/` is the user's, so a file may change under its name: an entry counts
    /// only while the file's stamp (length, modification time) is the one it was read at.
    /// Unreadable files are not kept and are tried again.
    image_px: RefCell<HashMap<String, StampedSize>>,
    /// Changed since the last save or autosave: what `autosave` writes.
    unbacked: bool,
    /// The project (not only the screenplay) changed since the last save.
    project_changed: bool,
    backup: BackupState,
}

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

    pub fn scene(&self) -> Option<&Scene> {
        self.project.scenes.get(self.scene)
    }

    /// The plan on view in the current scene.
    pub fn plan(&self) -> Option<&FloorPlan> {
        self.scene()?.plans.get(self.plan)
    }

    pub fn select_scene(&mut self, id: &str) -> Result<(), ServiceError> {
        let index = self
            .project
            .scenes
            .iter()
            .position(|s| s.id == id)
            .ok_or_else(|| ServiceError::NoScene(id.to_owned()))?;

        self.scene = index;
        self.plan = 0;
        self.time_s = 0.0;
        self.selected_shot = None;
        Ok(())
    }

    /// Shows the plan with this id of the current scene.
    pub fn select_plan(&mut self, id: &str) -> Result<(), ServiceError> {
        let index = self
            .scene()
            .and_then(|s| s.plans.iter().position(|p| p.id == id))
            .ok_or_else(|| ServiceError::NoPlan(id.to_owned()))?;

        self.plan = index;
        self.time_s = 0.0;
        self.selected_shot = None;
        Ok(())
    }

    pub fn time_s(&self) -> f64 {
        self.time_s
    }

    pub fn duration_s(&self) -> f64 {
        self.plan().map(FloorPlan::duration_s).unwrap_or(0.0)
    }

    /// Moves the playhead, kept within the scene. NaN is ignored.
    pub fn set_time(&mut self, t_s: f64) {
        if t_s.is_nan() {
            return;
        }

        self.time_s = t_s.clamp(0.0, self.duration_s());
    }

    pub fn selected_shot(&self) -> Option<&str> {
        self.selected_shot.as_deref()
    }

    /// Selects a shot and moves the playhead to its start (LH-73).
    pub fn select_shot(&mut self, id: &str) -> Result<(), ServiceError> {
        let shot = self
            .plan()
            .and_then(|p| p.shots.iter().find(|s| s.id == id))
            .ok_or_else(|| ServiceError::NoShot(id.to_owned()))?;

        self.time_s = shot.from_s;
        self.selected_shot = Some(id.to_owned());
        Ok(())
    }

    pub fn set_layer(&mut self, layer: Layer, visibility: Visibility) {
        match visibility {
            Visibility::Shown => self.hidden_layers.remove(&layer),
            Visibility::Hidden => self.hidden_layers.insert(layer),
        };
    }

    pub fn layer(&self, layer: Layer) -> Visibility {
        if self.hidden_layers.contains(&layer) {
            return Visibility::Hidden;
        }
        Visibility::Shown
    }

    /// Locks or unlocks a layer for this session: its elements cannot be picked, and edits
    /// that change or add something on it are refused with `ServiceError::Locked`. The lock
    /// is a view setting like `set_layer`, not part of the project.
    pub fn set_lock(&mut self, layer: Layer, lock: Lock) {
        match lock {
            Lock::Unlocked => self.locked_layers.remove(&layer),
            Lock::Locked => self.locked_layers.insert(layer),
        };
    }

    pub fn lock(&self, layer: Layer) -> Lock {
        if self.locked_layers.contains(&layer) {
            return Lock::Locked;
        }
        Lock::Unlocked
    }

    /// Refuses `edit` when it changes a locked layer.
    fn check_lock(&self, edit: &PlanEdit) -> Result<(), ServiceError> {
        match edit
            .layers()
            .into_iter()
            .find(|layer| self.lock(*layer) == Lock::Locked)
        {
            Some(layer) => Err(ServiceError::Locked(layer)),
            None => Ok(()),
        }
    }

    /// Refuses a result `next` in which a locked camera layer has moved: a figure drags its
    /// coupled cameras along (LH-37), and a camera left behind would break distance and angle.
    fn check_cameras(&self, next: &Project) -> Result<(), ServiceError> {
        if self.lock(Layer::Cameras) == Lock::Unlocked {
            return Ok(());
        }

        let marks = |project: &Project| {
            let plan = project.scenes.get(self.scene)?.plans.get(self.plan)?;
            Some(
                plan.cameras
                    .iter()
                    .map(|c| c.marks.clone())
                    .collect::<Vec<_>>(),
            )
        };
        if marks(next) == marks(&self.project) {
            return Ok(());
        }

        Err(ServiceError::Locked(Layer::Cameras))
    }

    /// Shows or hides one figure, camera or light in plan and timeline.
    pub fn set_track(&mut self, id: &str, visibility: Visibility) {
        match visibility {
            Visibility::Shown => self.hidden_tracks.remove(id),
            Visibility::Hidden => self.hidden_tracks.insert(id.to_owned()),
        };
    }

    /// The plan at the playhead, without hidden layers and tracks.
    pub fn display_list(&self) -> DisplayList {
        self.list_of(&self.project, None)
    }

    /// `display_list` with the grips on the bends of the way `selected` belongs to (LH-21).
    pub fn display_list_for(&self, selected: Option<&Source>) -> DisplayList {
        self.list_of(&self.project, selected)
    }

    /// `display_list` of `project`, a changed copy of this session's.
    fn list_of(&self, project: &Project, selected: Option<&Source>) -> DisplayList {
        let scene = project.scenes.get(self.scene);
        let Some((scene, plan)) = scene.and_then(|s| Some((s, s.plans.get(self.plan)?))) else {
            return DisplayList::default();
        };
        let Some(location) = project.location(&scene.location) else {
            return DisplayList::default();
        };

        let background_px = location
            .set
            .background
            .as_ref()
            .and_then(|b| self.image_size(&b.file));
        let mut list = render(project, &location.set, plan, self.time_s, background_px);
        if let Some(selected) = selected {
            list.items.extend(handles(project, plan, selected));
        }
        list.items
            .retain(|item| self.shows(item.role, item.source.as_ref()));
        list
    }

    /// Pixel size of an image of the project folder; remembered per file.
    fn image_size(&self, file: &str) -> Option<(u32, u32)> {
        let stamp = klappe_store::image_stamp(&self.dir, file).ok()?;
        let mut known = self.image_px.borrow_mut();
        if let Some((_, px)) = known.get(file).filter(|(seen, _)| *seen == stamp) {
            return Some(*px);
        }

        let px = klappe_store::image_size(&self.dir, file).ok()?;
        known.insert(file.to_owned(), (stamp, px));

        Some(px)
    }

    /// The lens a camera has at the playhead: its shot's own, else its own.
    pub fn lens_now_mm(&self, camera: &str) -> Option<f64> {
        let plan = self.plan()?;
        let camera = plan.camera(camera)?;
        Some(lens_at(camera, &plan.shots, self.time_s))
    }

    /// What the plan says about the mark `source` names; `None` for anything but a figure's
    /// mark, a figure or a camera (those at the playhead).
    pub fn mark_info(&self, source: &Source) -> Option<MarkInfo> {
        plan_edit::mark_info(self.plan()?, self.time_s, source)
    }

    fn shows(&self, role: Role, source: Option<&Source>) -> bool {
        if self.hidden_layers.contains(&layer_of(role)) {
            return false;
        }

        let id = match source {
            Some(Source::Camera(id) | Source::Light(id) | Source::Figure(id)) => id,
            Some(Source::Mark { character, .. } | Source::FigureVia { character, .. }) => character,
            Some(Source::CameraVia { camera, .. }) => camera,
            _ => return true,
        };
        !self.hidden_tracks.contains(id)
    }

    /// Prints the current plan at the playhead as PDF (LH-80), followed by its shot list;
    /// `title` names scene and plan.
    pub fn export_pdf(
        &self,
        path: &Path,
        paper: Paper,
        texts: Texts,
        order: ShotOrder,
    ) -> Result<(), ServiceError> {
        let (list, page) = self.printout(paper, texts)?;
        let bytes = match self.plan() {
            Some(plan) => klappe_export::pdf_with_shots(&list, &page, Theme::Light, plan, order)?,
            None => klappe_export::pdf(&list, &page, Theme::Light)?,
        };

        write_file(path, bytes)
    }

    /// Writes the current plan at the playhead as PNG (LH-81): RGB, 150 dpi, no shot list.
    pub fn export_png(&self, path: &Path, paper: Paper, texts: Texts) -> Result<(), ServiceError> {
        let (list, page) = self.printout(paper, texts)?;
        let bytes = klappe_export::png(&list, &page, Theme::Light)?;

        write_file(path, bytes)
    }

    /// Writes the current plan at the playhead as SVG (LH-81), without the shot list.
    pub fn export_svg(&self, path: &Path, paper: Paper, texts: Texts) -> Result<(), ServiceError> {
        let (list, page) = self.printout(paper, texts)?;
        let svg = klappe_export::svg(&list, &page, Theme::Light)?;

        write_file(path, svg.into_bytes())
    }

    /// The display list at the playhead and the sheet around it, with the images the list
    /// names read from the project; what every plan export starts from.
    fn printout(&self, paper: Paper, texts: Texts) -> Result<(DisplayList, Page), ServiceError> {
        let mut page = Page::new(paper, Orientation::Landscape, self.plan_title(), texts);
        let list = self.display_list();
        for item in &list.items {
            let Shape::Image { file, .. } = &item.shape else {
                continue;
            };
            page.images
                .insert(file.clone(), klappe_store::read_image(&self.dir, file)?);
        }

        Ok((list, page))
    }

    /// Writes the plan's shot list as CSV (LH-82); `texts` carries the column heads and the
    /// names of sizes and movements.
    pub fn export_csv(
        &self,
        path: &Path,
        texts: &Texts,
        order: ShotOrder,
    ) -> Result<(), ServiceError> {
        let empty = FloorPlan::new("", "");
        let csv = klappe_export::shots_csv(self.plan().unwrap_or(&empty), &texts.shots, order);

        write_file(path, csv.into_bytes())
    }

    /// "Szene 12 · INT. SÄNGERSAAL – TAG · Plan A" without the word "Szene": the surface adds it.
    pub fn plan_title(&self) -> String {
        let (Some(scene), Some(plan)) = (self.scene(), self.plan()) else {
            return String::new();
        };
        format!("{} · {} · {}", scene.number, scene.heading, plan.name)
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

    /// The shot list of the current plan, in cut order.
    pub fn shots(&self) -> Vec<ShotRow> {
        self.shots_in(ShotOrder::Cut)
    }

    /// The shot list of the current plan in `order`; `setup` counts the same in both.
    pub fn shots_in(&self, order: ShotOrder) -> Vec<ShotRow> {
        let Some(plan) = self.plan() else {
            return Vec::new();
        };
        let jumps = axis_jumps(plan);

        ordered_shots(plan, order)
            .into_iter()
            .map(|(setup, shot)| {
                let camera = plan.camera(&shot.camera);
                ShotRow {
                    id: shot.id.clone(),
                    number: shot.number.clone(),
                    camera: shot.camera.clone(),
                    camera_label: camera.map(|c| c.label.clone()).unwrap_or_default(),
                    follows: camera.and_then(|c| c.follows.clone()).unwrap_or_default(),
                    size: shot.size.name().to_owned(),
                    angle: shot.angle.name().to_owned(),
                    movement: shot.movement.name().to_owned(),
                    lens_mm: shot.lens_mm.or(camera.map(|c| c.lens_mm)).unwrap_or(0.0),
                    own_lens_mm: shot.lens_mm,
                    from_s: shot.from_s,
                    to_s: shot.to_s,
                    description: shot.description.clone(),
                    props: shot.props.clone(),
                    gear: shot.gear.clone(),
                    crew: shot.crew.clone(),
                    note: shot.note.clone(),
                    axis_jump: jumps.contains(&shot.id),
                    setup,
                }
            })
            .collect()
    }

    /// Changes one field of a shot; undoable (LH-06).
    pub fn edit_shot(
        &mut self,
        id: &str,
        field: ShotField,
        value: &str,
    ) -> Result<(), ServiceError> {
        let mut next = self.project.clone();
        let plan = next
            .scenes
            .get_mut(self.scene)
            .and_then(|s| s.plans.get_mut(self.plan))
            .ok_or_else(|| ServiceError::NoShot(id.to_owned()))?;
        let shot = plan
            .shots
            .iter_mut()
            .find(|s| s.id == id)
            .ok_or_else(|| ServiceError::NoShot(id.to_owned()))?;

        edit::apply(shot, &plan.cameras, field, value)?;
        if next == self.project {
            return Ok(());
        }

        self.commit(next);
        Ok(())
    }

    /// Adds a shot at the playhead and selects it; undoable. Its camera is `camera`, else the
    /// selected shot's, else the first (LH-70). Refused while the camera layer is locked: a
    /// shot belongs to its camera.
    pub fn add_shot(&mut self, camera: Option<&str>) -> Result<String, ServiceError> {
        self.check_shot_lock()?;

        let camera = self.camera_for_shot(camera)?;
        let scene = self.scene_number();
        let at_s = self.time_s;
        let mut next = self.project.clone();
        let plan = self.plan_in(&mut next)?;

        let id = shot_edit::add(plan, &scene, &camera, at_s);
        self.commit(next);
        self.selected_shot = Some(id.clone());
        Ok(id)
    }

    /// Copies a shot with a new number and selects the copy; undoable. A copy is how one
    /// setup yields several shots (LH-70).
    pub fn copy_shot(&mut self, id: &str) -> Result<String, ServiceError> {
        self.check_shot_lock()?;

        let scene = self.scene_number();
        let mut next = self.project.clone();
        let plan = self.plan_in(&mut next)?;

        let copy = shot_edit::copy(plan, &scene, id)?;
        self.commit(next);
        self.selected_shot = Some(copy.clone());
        Ok(copy)
    }

    /// Deletes a shot and the script links to it; undoable. If it was selected, the next
    /// shot in the list is, else the previous.
    pub fn delete_shot(&mut self, id: &str) -> Result<(), ServiceError> {
        self.check_shot_lock()?;

        let mut next = self.project.clone();
        let plan = self.plan_in(&mut next)?;

        let neighbour = shot_edit::remove(plan, id)?;
        self.commit(next);
        if self.selected_shot.as_deref() == Some(id) {
            self.selected_shot = neighbour;
        }
        Ok(())
    }

    /// Shots belong to cameras: no new, copied or deleted shot while they are locked.
    fn check_shot_lock(&self) -> Result<(), ServiceError> {
        if self.lock(Layer::Cameras) == Lock::Locked {
            return Err(ServiceError::Locked(Layer::Cameras));
        }

        Ok(())
    }

    /// The camera id a new shot gets: `camera` if asked for, else the selected shot's, else
    /// the first of the plan.
    fn camera_for_shot(&self, camera: Option<&str>) -> Result<String, ServiceError> {
        let plan = self.plan().ok_or(ServiceError::NoCamera)?;
        let wanted = camera.map(str::to_owned).or_else(|| {
            let id = self.selected_shot.as_deref()?;
            plan.shots
                .iter()
                .find(|s| s.id == id)
                .map(|s| s.camera.clone())
        });
        let Some(wanted) = wanted else {
            return plan
                .cameras
                .first()
                .map(|c| c.id.clone())
                .ok_or(ServiceError::NoCamera);
        };

        match plan.camera(&wanted) {
            Some(c) => Ok(c.id.clone()),
            None => Err(ServiceError::BadValue {
                field: Field::Camera,
                value: wanted,
            }),
        }
    }

    fn scene_number(&self) -> String {
        self.scene().map(|s| s.number.clone()).unwrap_or_default()
    }

    /// The plan on view in `project`, a changed copy of this session's.
    fn plan_in<'a>(&self, project: &'a mut Project) -> Result<&'a mut FloorPlan, ServiceError> {
        project
            .scenes
            .get_mut(self.scene)
            .and_then(|s| s.plans.get_mut(self.plan))
            .ok_or(ServiceError::Gone)
    }

    /// Changes the current plan; undoable. Points are taken as given: surfaces call `snap`.
    pub fn edit_plan(&mut self, edit: PlanEdit) -> Result<(), ServiceError> {
        self.check_lock(&edit)?;

        let mut next = self.project.clone();
        let at = (self.scene, self.plan);
        plan_edit::apply(
            &self.dir,
            &mut next,
            at.0,
            at.1,
            self.time_s,
            self.snapping,
            &edit,
        )?;
        self.check_cameras(&next)?;
        if next == self.project {
            return Ok(());
        }

        self.commit(next);
        Ok(())
    }

    /// Changes scenes, locations or characters; undoable. The scene on view stays, also when
    /// others come or go before it.
    pub fn edit_project(&mut self, edit: ProjectEdit) -> Result<(), ServiceError> {
        let mut next = self.project.clone();
        project_edit::apply(&mut next, &edit)?;
        if next == self.project {
            return Ok(());
        }

        let on_view = self.scene().map(|s| s.id.clone());
        let plan_on_view = self.plan().map(|p| p.id.clone());
        self.commit(next);
        self.follow_scene(on_view, plan_on_view);
        Ok(())
    }

    /// Adds an empty plan to the current scene and shows it; `name` empty: `Plan B` …
    /// Undoable. Returns the new plan's id.
    pub fn add_plan(&mut self, name: &str) -> Result<String, ServiceError> {
        let mut next = self.project.clone();
        let at = project_edit::add_plan(&mut next, self.scene, name)?;

        Ok(self.commit_new_plan(next, at))
    }

    /// Snapshot (LH-04): copies the plan on view under a new name and shows the copy; `name`
    /// empty: `Plan A (2)` …. Going back is choosing the earlier plan again. Undoable.
    /// Returns the copy's id.
    pub fn copy_plan(&mut self, name: &str) -> Result<String, ServiceError> {
        let mut next = self.project.clone();
        let at = project_edit::copy_plan(&mut next, self.scene, self.plan, name)?;

        Ok(self.commit_new_plan(next, at))
    }

    /// Changes the name of the plan `id` of the current scene; undoable.
    pub fn rename_plan(&mut self, id: &str, name: &str) -> Result<(), ServiceError> {
        let mut next = self.project.clone();
        project_edit::rename_plan(&mut next, self.scene, id, name)?;
        if next == self.project {
            return Ok(());
        }

        self.commit(next);
        Ok(())
    }

    /// Deletes the plan `id` of the current scene; undoable. Refused for the last plan of a
    /// scene and for one with shots. When it was on view, the plan before it is.
    pub fn remove_plan(&mut self, id: &str) -> Result<(), ServiceError> {
        let mut next = self.project.clone();
        project_edit::remove_plan(&mut next, self.scene, id)?;

        let on_view = self.scene().map(|s| s.id.clone());
        let plan_on_view = self.plan().map(|p| p.id.clone());
        let was_shown = plan_on_view.as_deref() == Some(id);
        let before = self.plan.saturating_sub(1);
        self.commit(next);
        self.follow_scene(on_view, plan_on_view);
        if was_shown {
            self.plan = before;
        }
        Ok(())
    }

    /// Commits `next`, which has a new plan at `at` in the current scene, and shows it.
    fn commit_new_plan(&mut self, next: Project, at: usize) -> String {
        self.commit(next);
        self.plan = at;
        self.time_s = 0.0;
        self.selected_shot = None;

        self.plan().map(|p| p.id.clone()).unwrap_or_default()
    }

    /// What the object tool offers: the built-in library, then the project's own objects.
    pub fn library(&self) -> Vec<LibraryEntry> {
        library::entries(&self.project.library)
    }

    /// What the camera card offers: sensor formats and lens sets (LH-32).
    pub fn camera_presets(&self) -> &'static CameraPresets {
        klappe_store::builtin_camera_presets()
    }

    /// Takes the entries of the library file `path` into the project; undoable. Returns how
    /// many it added: entries the project has already are not added again. A file with an
    /// unusable entry is refused as a whole and changes nothing.
    pub fn import_library(&mut self, path: &Path) -> Result<usize, ServiceError> {
        let items = klappe_store::load_library(path)?;
        let before = self.project.library.len();

        self.edit_project(ProjectEdit::ImportLibrary { items })?;

        Ok(self.project.library.len() - before)
    }

    /// Writes the project's own objects to the library file `path`, to pass on. Returns how
    /// many it wrote; none to write is an error.
    pub fn export_library(&self, path: &Path) -> Result<usize, ServiceError> {
        if self.project.library.is_empty() {
            return Err(ServiceError::EmptyLibrary);
        }

        klappe_store::save_library(path, &self.project.library)?;

        Ok(self.project.library.len())
    }

    /// The display list as it would be after `edit`, without committing it or taking an undo
    /// step: what a drag shows while the button is down.
    pub fn preview(&self, edit: &PlanEdit) -> Result<DisplayList, ServiceError> {
        self.preview_for(edit, None)
    }

    /// `preview` with the grips of the way `selected` belongs to, as after the edit.
    pub fn preview_for(
        &self,
        edit: &PlanEdit,
        selected: Option<&Source>,
    ) -> Result<DisplayList, ServiceError> {
        self.check_lock(edit)?;

        let mut next = self.project.clone();
        let at = (self.scene, self.plan);
        // Copying a file is no preview: the list stays as it is until the edit is made.
        if matches!(edit, PlanEdit::SetBackground { .. }) {
            return Ok(self.display_list_for(selected));
        }
        plan_edit::apply(
            &self.dir,
            &mut next,
            at.0,
            at.1,
            self.time_s,
            self.snapping,
            edit,
        )?;
        self.check_cameras(&next)?;

        Ok(self.list_of(&next, selected))
    }

    /// Topmost shown item at `at` (front of the display list first), within the pick radius.
    /// Items of locked layers are not taken.
    pub fn pick(&self, at: Point) -> Option<Source> {
        self.pick_for(at, None)
    }

    /// `pick` with the grips of the way `selected` belongs to in reach: they lie in front.
    pub fn pick_for(&self, at: Point, selected: Option<&Source>) -> Option<Source> {
        let locked = |role| self.lock(layer_of(role)) == Lock::Locked;

        plan_edit::pick(&self.display_list_for(selected).items, at, locked)
    }

    /// Every element whose shown items lie wholly in the box with corners `a` and `b` (LH-08).
    /// Items of locked layers are not taken; no grips (they show for one selected way only).
    pub fn pick_in(&self, a: Point, b: Point) -> Vec<Source> {
        let locked = |role| self.lock(layer_of(role)) == Lock::Locked;

        plan_edit::pick_in(&self.display_list().items, (a, b), locked)
    }

    pub fn set_snapping(&mut self, snapping: Snapping) {
        self.snapping = snapping;
    }

    pub fn snapping(&self) -> Snapping {
        self.snapping
    }

    /// `at` on the grid when snapping is on.
    pub fn snap(&self, at: Point) -> Point {
        plan_edit::snap(at, self.snapping)
    }

    fn commit(&mut self, next: Project) {
        let previous = std::mem::replace(&mut self.project, next);
        self.undo.push(previous);
        if self.undo.len() > UNDO_LIMIT {
            self.undo.remove(0);
        }
        self.redo.clear();
        self.project_changed = true;
        self.touch();
    }

    fn touch(&mut self) {
        self.dirty = true;
        self.unbacked = true;
    }

    /// The view stays with the scene `id` if the (new) project has it, else it starts over.
    fn follow_scene(&mut self, id: Option<String>, plan: Option<String>) {
        let index = id.and_then(|id| self.project.scenes.iter().position(|s| s.id == id));
        let Some(index) = index else {
            self.scene = self.scene.min(self.project.scenes.len().saturating_sub(1));
            self.plan = 0;
            self.time_s = 0.0;
            self.selected_shot = None;
            return;
        };

        if index != self.scene {
            self.scene = index;
            self.plan = 0;
        }

        // Plans come and go too (add, snapshot, delete, undo): stay with the one on view.
        let plan_index = plan.and_then(|id| {
            self.scene()
                .and_then(|s| s.plans.iter().position(|p| p.id == id))
        });
        match plan_index {
            Some(index) => self.plan = index,
            None => {
                self.plan = 0;
                self.time_s = 0.0;
                self.selected_shot = None;
            }
        }
        self.keep_view_valid();
    }

    /// Scene and plan indices of the view fit the (new) project again.
    fn keep_view_valid(&mut self) {
        if self.scene >= self.project.scenes.len() {
            self.scene = 0;
        }
        if self.scene().is_none_or(|s| self.plan >= s.plans.len()) {
            self.plan = 0;
        }

        // Undo and redo can take the selected shot away.
        let selected_is_gone = self.selected_shot.as_deref().is_some_and(|id| {
            self.plan()
                .is_none_or(|p| !p.shots.iter().any(|s| s.id == id))
        });
        if selected_is_gone {
            self.selected_shot = None;
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub fn undo(&mut self) {
        let Some(previous) = self.undo.pop() else {
            return;
        };
        let on_view = self.scene().map(|s| s.id.clone());
        let plan_on_view = self.plan().map(|p| p.id.clone());
        self.redo
            .push(std::mem::replace(&mut self.project, previous));
        self.project_changed = true;
        self.touch();
        self.follow_scene(on_view, plan_on_view);
    }

    pub fn redo(&mut self) {
        let Some(next) = self.redo.pop() else {
            return;
        };
        let on_view = self.scene().map(|s| s.id.clone());
        let plan_on_view = self.plan().map(|p| p.id.clone());
        self.undo.push(std::mem::replace(&mut self.project, next));
        self.project_changed = true;
        self.touch();
        self.follow_scene(on_view, plan_on_view);
    }

    /// The timeline: cut order first, then figures, cameras and lights.
    pub fn tracks(&self) -> Vec<Track> {
        let Some(plan) = self.plan() else {
            return Vec::new();
        };
        let mut tracks = vec![self.cut_track(plan)];

        for figure in &plan.figures {
            let Some(character) = self.project.character(&figure.character) else {
                continue;
            };
            tracks.push(Track {
                kind: TrackKind::Figure,
                id: figure.character.clone(),
                name: character.title(),
                label: label_of(character.color).to_owned(),
                shown: self.track_shown(&figure.character, Layer::Figures),
                segments: walks(figure.marks.iter().map(|m| (m.at_s, m.hold_s))),
                markers: figure
                    .marks
                    .iter()
                    .enumerate()
                    .map(|(i, m)| Marker {
                        at_s: m.at_s,
                        text: format!("{} {}", character.short, i + 1),
                    })
                    .collect(),
            });
        }

        for (i, camera) in plan.cameras.iter().enumerate() {
            tracks.push(Track {
                kind: TrackKind::Camera,
                id: camera.id.clone(),
                name: camera.label.clone(),
                label: CAMERA_LABELS[i % CAMERA_LABELS.len()].to_owned(),
                shown: self.track_shown(&camera.id, Layer::Cameras),
                segments: walks(camera.marks.iter().map(|m| (m.at_s, m.hold_s))),
                markers: camera
                    .marks
                    .iter()
                    .enumerate()
                    .map(|(n, m)| Marker {
                        at_s: m.at_s,
                        text: format!("{} {}", camera.label, n + 1),
                    })
                    .collect(),
            });
        }

        for light in &plan.lights {
            tracks.push(Track {
                kind: TrackKind::Light,
                id: light.id.clone(),
                name: light.label.clone(),
                label: LIGHT_LABEL.to_owned(),
                shown: self.track_shown(&light.id, Layer::Lights),
                segments: vec![Segment {
                    from_s: 0.0,
                    to_s: self.duration_s(),
                    text: format!("{:.0} K", light.color_temp_k),
                    key: String::new(),
                    selected: false,
                }],
                markers: Vec::new(),
            });
        }

        tracks
    }

    fn track_shown(&self, id: &str, layer: Layer) -> bool {
        !self.hidden_tracks.contains(id) && !self.hidden_layers.contains(&layer)
    }

    fn cut_track(&self, plan: &FloorPlan) -> Track {
        let segments = plan
            .shots
            .iter()
            .map(|shot| {
                let camera = plan
                    .camera(&shot.camera)
                    .map(|c| c.label.as_str())
                    .unwrap_or("?");
                Segment {
                    from_s: shot.from_s,
                    to_s: shot.to_s,
                    text: format!("{} · {}", shot.number, camera),
                    key: shot.id.clone(),
                    selected: self.selected_shot.as_deref() == Some(&shot.id),
                }
            })
            .collect();

        Track {
            kind: TrackKind::Cut,
            id: CUT_TRACK.to_owned(),
            name: String::new(),
            label: CUT_LABEL.to_owned(),
            shown: true,
            segments,
            markers: Vec::new(),
        }
    }

    /// The current scene's script with its links to marks and shots (design B).
    pub fn script_lines(&self) -> Vec<ScriptLine> {
        let (Some(scene), Some(plan)) = (self.scene(), self.plan()) else {
            return Vec::new();
        };
        let Some(view) = self.script.scene(&scene.number) else {
            return Vec::new();
        };

        let mut lines: Vec<ScriptLine> = view
            .elements
            .iter()
            .enumerate()
            .map(|(index, element)| {
                let (kind, text) = line_of(element);
                ScriptLine {
                    index,
                    kind,
                    text,
                    chips: self.chips(plan, index),
                    current: false,
                }
            })
            .collect();

        mark_current(&mut lines, self.time_s);
        lines
    }

    fn chips(&self, plan: &FloorPlan, index: usize) -> Vec<Chip> {
        plan.links
            .iter()
            .filter(|l| l.element == index)
            .filter_map(|l| match &l.target {
                LinkTarget::Mark { character, index } => {
                    let c = self.project.character(character)?;
                    let mark = plan.figure(character)?.marks.get(*index)?;
                    Some(Chip {
                        kind: LinkKind::Mark,
                        text: format!("{} · M{}", c.short, index + 1),
                        target: character.clone(),
                        label: label_of(c.color).to_owned(),
                        at_s: mark.at_s,
                    })
                }
                LinkTarget::Shot { id } => {
                    let shot = plan.shots.iter().find(|s| &s.id == id)?;
                    Some(Chip {
                        kind: LinkKind::Shot,
                        text: shot.number.clone(),
                        target: id.clone(),
                        label: String::new(),
                        at_s: shot.from_s,
                    })
                }
            })
            .collect()
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

/// Which layer switch an item of the display list belongs to.
fn layer_of(role: Role) -> Layer {
    match role {
        Role::Background | Role::Wall | Role::Object => Layer::Set,
        Role::Light | Role::LightBeam => Layer::Lights,
        Role::Camera | Role::Frustum | Role::CameraPath | Role::CameraHandle => Layer::Cameras,
        Role::Note => Layer::Notes,
        Role::Drawing { .. } | Role::ArrowHead(_) => Layer::Drawings,
        Role::Axis
        | Role::Figure(_)
        | Role::GroupDot
        | Role::FigurePath(_)
        | Role::FigureHandle(_)
        | Role::Mark(_) => Layer::Figures,
    }
}

/// Kante's label name for a figure colour.
fn label_of(color: FigureColor) -> &'static str {
    match color {
        FigureColor::Purple => "purple",
        FigureColor::Blue => "blue",
        FigureColor::Green => "green",
        FigureColor::Yellow => "yellow",
        FigureColor::Orange => "orange",
        FigureColor::Red => "red",
        FigureColor::Cyan => "cyan",
    }
}

/// Moving stretches between marks: leave after the hold, arrive at the next mark.
fn walks(marks: impl Iterator<Item = (f64, f64)>) -> Vec<Segment> {
    let marks: Vec<(f64, f64)> = marks.collect();

    marks
        .windows(2)
        .filter_map(|pair| {
            let ((at, hold), (next, _)) = (pair[0], pair[1]);
            let leave = (at + hold).min(next);
            (next > leave).then(|| Segment {
                from_s: leave,
                to_s: next,
                text: String::new(),
                key: String::new(),
                selected: false,
            })
        })
        .collect()
}

fn line_of(element: &Element) -> (&'static str, String) {
    match element {
        Element::Action(t) => ("action", t.clone()),
        Element::Character {
            name, extension, ..
        } => match extension {
            Some(ext) => ("character", format!("{name} ({ext})")),
            None => ("character", name.clone()),
        },
        Element::Parenthetical(t) => ("parenthetical", t.clone()),
        Element::Dialogue(t) => ("dialogue", t.clone()),
        Element::Transition(t) => ("transition", t.clone()),
        Element::SceneHeading { text, .. } => ("other", text.clone()),
        Element::Centered(t) | Element::Synopsis(t) | Element::Note(t) | Element::Lyrics(t) => {
            ("other", t.clone())
        }
        Element::Section { text, .. } => ("other", text.clone()),
        Element::PageBreak => ("other", String::new()),
    }
}

/// The last line with a link whose time has come is current.
fn mark_current(lines: &mut [ScriptLine], t_s: f64) {
    let current = lines
        .iter()
        .enumerate()
        .filter_map(|(i, l)| {
            let start = l.chips.iter().map(|c| c.at_s).fold(f64::INFINITY, f64::min);
            (start <= t_s).then_some((i, start))
        })
        .max_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)))
        .map(|(i, _)| i);

    if let Some(i) = current {
        lines[i].current = true;
    }
}

/// `dir` may be missing or an empty folder; anything else would mix the project with it.
fn check_empty(dir: &Path) -> Result<(), ServiceError> {
    let entries = match std::fs::read_dir(dir) {
        Ok(mut entries) => entries.next().is_some(),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => false,
        Err(source) => {
            return Err(ServiceError::Write {
                path: dir.to_owned(),
                source,
            })
        }
    };

    if entries {
        return Err(ServiceError::NotEmpty(dir.to_owned()));
    }
    Ok(())
}

fn write_file(path: &Path, bytes: Vec<u8>) -> Result<(), ServiceError> {
    std::fs::write(path, bytes).map_err(|source| ServiceError::Write {
        path: path.to_owned(),
        source,
    })
}
