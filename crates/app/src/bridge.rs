//! The QML-facing controller. Thin: every call goes to `klappe_services::Session`, every result
//! comes back as a property (JSON for lists). Property names are camelCase in QML.
//!
//! cxx-qt generates C++ glue with `unsafe` blocks; the workspace lint is relaxed for this file
//! only (agent.md allows `unsafe` in the cxx-qt bridge).
#![allow(unsafe_code)]

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, texts)]
        #[qproperty(bool, loaded)]
        #[qproperty(QString, title)]
        /// Scene number, heading and plan name.
        #[qproperty(QString, plan_title, cxx_name = "planTitle")]
        /// `{ width, depth, shapes }` for KanteShapeCanvas.
        #[qproperty(QString, plan_json, cxx_name = "planJson")]
        #[qproperty(QString, tracks_json, cxx_name = "tracksJson")]
        #[qproperty(QString, shots_json, cxx_name = "shotsJson")]
        #[qproperty(QString, script_json, cxx_name = "scriptJson")]
        /// `{ set, figures, cameras, lights, notes, drawings }`: true when shown.
        #[qproperty(QString, layers_json, cxx_name = "layersJson")]
        /// `{ set, figures, cameras, lights, notes, drawings }`: true when locked.
        #[qproperty(QString, locks_json, cxx_name = "locksJson")]
        #[qproperty(f64, time)]
        #[qproperty(f64, duration)]
        #[qproperty(QString, selected_shot, cxx_name = "selectedShot")]
        #[qproperty(bool, can_undo, cxx_name = "canUndo")]
        #[qproperty(bool, can_redo, cxx_name = "canRedo")]
        #[qproperty(bool, dirty)]
        /// Key of the selected plan element (`camera:a` …), empty for none.
        #[qproperty(QString, selected_key, cxx_name = "selectedKey")]
        /// Whether points snap to the grid; change it with `setSnapping`.
        #[qproperty(bool, snapping_on, cxx_name = "snappingOn")]
        /// `{ figure: [{ id, name }], object: [kind], light: [kind], color: [name], dash: [name],
        /// fill: [name] }`: what the tool lists offer.
        #[qproperty(QString, choices_json, cxx_name = "choicesJson")]
        /// Whether the set has a background image.
        #[qproperty(bool, has_background, cxx_name = "hasBackground")]
        /// Points picked so far by the scale tool: 0, 1 or 2 (then the length is asked).
        #[qproperty(i32, scale_points, cxx_name = "scalePoints")]
        /// Points picked so far by the area tool: 0 for none, else the area is open (Enter or a
        /// click on the first point closes it, `cancelArea` drops it).
        #[qproperty(i32, area_points, cxx_name = "areaPoints")]
        /// Whether the text of a note is being asked (`commitNote` or `cancelNote` answers).
        #[qproperty(bool, asks_note, cxx_name = "asksNote")]
        /// The text the question starts with: the note's own when it is edited.
        #[qproperty(QString, note_text, cxx_name = "noteText")]
        /// Last message for the status line, already translated.
        #[qproperty(QString, status)]
        /// Backup found at open: `none`, `found` or `unreadable`; answer with `restoreBackup`
        /// or `discardBackup`.
        #[qproperty(QString, backup_state, cxx_name = "backupState")]
        /// Why an `unreadable` backup cannot be read.
        #[qproperty(QString, backup_message, cxx_name = "backupMessage")]
        /// Milliseconds between two calls of `autosave`.
        #[qproperty(i32, autosave_ms, cxx_name = "autosaveMs")]
        #[qproperty(QString, screenshot_path, cxx_name = "screenshotPath")]
        #[qproperty(i32, screenshot_after, cxx_name = "screenshotAfter")]
        type Controller = super::ControllerRust;

        /// Moves the playhead (seconds).
        #[qinvokable]
        fn seek(self: Pin<&mut Self>, time: f64);

        #[qinvokable]
        #[cxx_name = "selectShot"]
        fn select_shot(self: Pin<&mut Self>, id: &QString);

        /// Sets one field (`lens`, `size` …) of a shot; false if the value is refused.
        #[qinvokable]
        #[cxx_name = "editShot"]
        fn edit_shot(self: Pin<&mut Self>, id: &QString, field: &QString, value: &QString) -> bool;

        #[qinvokable]
        fn undo(self: Pin<&mut Self>);

        #[qinvokable]
        fn redo(self: Pin<&mut Self>);

        #[qinvokable]
        fn save(self: Pin<&mut Self>) -> bool;

        /// Writes unsaved changes to the backup; QML calls it on a timer. An error goes to the
        /// status line.
        #[qinvokable]
        fn autosave(self: Pin<&mut Self>);

        /// Takes the project from the backup found at open.
        #[qinvokable]
        #[cxx_name = "restoreBackup"]
        fn restore_backup(self: Pin<&mut Self>);

        /// Removes the backup found at open.
        #[qinvokable]
        #[cxx_name = "discardBackup"]
        fn discard_backup(self: Pin<&mut Self>);

        /// Shows or hides one figure, camera or light (`shown`: "shown" or "hidden").
        #[qinvokable]
        #[cxx_name = "setTrack"]
        fn set_track(self: Pin<&mut Self>, id: &QString, shown: &QString);

        /// `layer`: set, figures, cameras, lights; `shown`: "shown" or "hidden".
        #[qinvokable]
        #[cxx_name = "setLayer"]
        fn set_layer(self: Pin<&mut Self>, layer: &QString, shown: &QString);

        /// `layer` as for `setLayer`; `locked`: "locked" or "unlocked". A locked layer cannot
        /// be picked or changed.
        #[qinvokable]
        #[cxx_name = "setLock"]
        fn set_lock(self: Pin<&mut Self>, layer: &QString, locked: &QString);

        /// The tool in use: `select`, `wall`, `object` …; `choice` is the value picked in the
        /// tool's list (object kind, character id, lamp kind). False for an unknown tool.
        #[qinvokable]
        #[cxx_name = "setTool"]
        fn set_tool(self: Pin<&mut Self>, tool: &QString, choice: &QString) -> bool;

        /// A left drag with the tool, in metres. While `done` is false the plan shows the
        /// result only; on release it is committed. False when the drag does nothing or the
        /// edit is refused.
        #[qinvokable]
        fn drag(self: Pin<&mut Self>, from_x: f64, from_y: f64, x: f64, y: f64, done: bool)
            -> bool;

        /// A click: selects with the select tool, else sets what the tool stands for.
        #[qinvokable]
        fn place(self: Pin<&mut Self>, x: f64, y: f64) -> bool;

        /// Whether a click at (x, y) would take something: the pointer is over a thing to move.
        #[qinvokable]
        fn grabbable(self: &Self, x: f64, y: f64) -> bool;

        /// Takes the image file at `path` (a path or a `file://` URL) as the set's background
        /// and points to the scale tool in the status line.
        #[qinvokable]
        #[cxx_name = "setBackground"]
        fn set_background(self: Pin<&mut Self>, path: &QString) -> bool;

        /// Takes the background off the set.
        #[qinvokable]
        #[cxx_name = "removeBackground"]
        fn remove_background(self: Pin<&mut Self>) -> bool;

        /// Scales the background so that the two picked points are `distance` metres apart
        /// (comma or point as decimal separator). False for no two points or a bad number.
        #[qinvokable]
        fn calibrate(self: Pin<&mut Self>, distance: &QString) -> bool;

        /// Drops the points picked for a scale.
        #[qinvokable]
        #[cxx_name = "cancelScale"]
        fn cancel_scale(self: Pin<&mut Self>);

        /// What the drawing tools draw with: names as `choicesJson` lists them. False for an
        /// unknown name.
        #[qinvokable]
        #[cxx_name = "setDrawStyle"]
        fn set_draw_style(
            self: Pin<&mut Self>,
            color: &QString,
            dash: &QString,
            fill: &QString,
        ) -> bool;

        /// Closes the area being drawn; false below three points.
        #[qinvokable]
        #[cxx_name = "finishArea"]
        fn finish_area(self: Pin<&mut Self>) -> bool;

        /// Drops the area being drawn.
        #[qinvokable]
        #[cxx_name = "cancelArea"]
        fn cancel_area(self: Pin<&mut Self>);

        /// Gives the asked note its text; false for a blank text or no question.
        #[qinvokable]
        #[cxx_name = "commitNote"]
        fn commit_note(self: Pin<&mut Self>, text: &QString) -> bool;

        /// Drops the question for a note's text.
        #[qinvokable]
        #[cxx_name = "cancelNote"]
        fn cancel_note(self: Pin<&mut Self>);

        /// Asks for the text of the selected note; false when no note is selected.
        #[qinvokable]
        #[cxx_name = "editSelectedNote"]
        fn edit_selected_note(self: Pin<&mut Self>) -> bool;

        /// Removes the selected element.
        #[qinvokable]
        #[cxx_name = "removeSelected"]
        fn remove_selected(self: Pin<&mut Self>) -> bool;

        /// Turns the selected element by `step_deg`, clockwise when positive.
        #[qinvokable]
        #[cxx_name = "turnSelected"]
        fn turn_selected(self: Pin<&mut Self>, step_deg: f64) -> bool;

        #[qinvokable]
        #[cxx_name = "setSnapping"]
        fn set_snapping(self: Pin<&mut Self>, on: bool);

        /// A drag was aborted: shows the committed plan again.
        #[qinvokable]
        #[cxx_name = "cancelDrag"]
        fn cancel_drag(self: Pin<&mut Self>);

        /// Writes the plan as PDF next to the project folder; the path goes to `status`.
        /// `order` is `cut` or `shooting`: the order of the shot list.
        #[qinvokable]
        #[cxx_name = "exportPdf"]
        fn export_pdf(self: Pin<&mut Self>, order: &QString) -> bool;

        /// Writes the plan as PNG next to the project folder; the path goes to `status`.
        #[qinvokable]
        #[cxx_name = "exportPng"]
        fn export_png(self: Pin<&mut Self>) -> bool;

        /// Writes the plan as SVG next to the project folder; the path goes to `status`.
        #[qinvokable]
        #[cxx_name = "exportSvg"]
        fn export_svg(self: Pin<&mut Self>) -> bool;

        /// Writes the shot list as CSV next to the project folder; the path goes to `status`.
        /// `order` is `cut` or `shooting`.
        #[qinvokable]
        #[cxx_name = "exportCsv"]
        fn export_csv(self: Pin<&mut Self>, order: &QString) -> bool;

        /// What a tooltip says about a plan element (`camera:a` …), as JSON.
        #[qinvokable]
        fn describe(self: &Self, key: &QString) -> QString;
    }
}

use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use klappe_services::{
    horizontal_fov_deg, Autosave, DisplayList, Layer, Lock, Paper, PlanEdit, Point, ServiceError,
    Session, ShotField, ShotOrder, Snapping, Source, Texts, Visibility,
};
use serde_json::json;

use crate::args::AppArgs;
use crate::catalog::Language;
use crate::notice::{backup_view, error_text, Status};
use crate::plan;
use crate::tools::{
    self, AreaDraft, Backdrop, DragPhase, DragStep, DrawStyle, Key, NoteDraft, Preview, ScaleStep,
    Tool,
};

pub struct ControllerRust {
    session: Option<Session>,
    language: Language,
    notice: Status,
    texts: QString,
    loaded: bool,
    title: QString,
    plan_title: QString,
    plan_json: QString,
    tracks_json: QString,
    shots_json: QString,
    script_json: QString,
    layers_json: QString,
    locks_json: QString,
    time: f64,
    duration: f64,
    selected_shot: QString,
    can_undo: bool,
    can_redo: bool,
    dirty: bool,
    selected_key: QString,
    snapping_on: bool,
    choices_json: QString,
    /// What the select tool has picked; the canvas frames it.
    selected: Option<Source>,
    tool: Tool,
    scale: ScaleStep,
    /// What the drawing tools draw with, and the area they are drawing.
    draw: DrawStyle,
    area: AreaDraft,
    area_points: i32,
    has_background: bool,
    scale_points: i32,
    /// The note whose text is asked, if any.
    note: NoteDraft,
    asks_note: bool,
    note_text: QString,
    /// What the plan on screen shows: a drag preview or the project.
    preview: Preview,
    status: QString,
    backup_state: QString,
    backup_message: QString,
    autosave_ms: i32,
    screenshot_path: QString,
    screenshot_after: i32,
}

fn q(s: &str) -> QString {
    QString::from(s)
}

const SHOWN: &str = "shown";
const LOCKED: &str = "locked";

/// Seconds between two backups of unsaved changes while a project is open.
const AUTOSAVE_S: i32 = 30;
const MS_PER_S: i32 = 1000;

impl Default for ControllerRust {
    fn default() -> Self {
        let args = AppArgs::from_env();
        let language = Language::pick(args.language.as_deref());
        let mut notice = Status::default();

        let session = args.open.as_ref().and_then(|dir| match Session::open(dir) {
            Ok(mut s) => {
                if let Some(shot) = &args.shot {
                    let _ = s.select_shot(shot);
                }
                s.set_time(args.time_s.max(s.time_s()));
                Some(s)
            }
            Err(e) => {
                notice.say(error_text(language, &e));
                None
            }
        });

        let mut this = Self {
            session,
            language,
            status: q(notice.text()),
            notice,
            texts: q(language.json()),
            loaded: false,
            title: QString::default(),
            plan_title: QString::default(),
            plan_json: QString::default(),
            tracks_json: QString::default(),
            shots_json: QString::default(),
            script_json: QString::default(),
            layers_json: QString::default(),
            locks_json: QString::default(),
            time: 0.0,
            duration: 0.0,
            selected_shot: QString::default(),
            can_undo: false,
            can_redo: false,
            dirty: false,
            selected_key: QString::default(),
            snapping_on: true,
            choices_json: QString::default(),
            selected: None,
            tool: Tool::Select,
            scale: ScaleStep::Idle,
            draw: DrawStyle::default(),
            area: AreaDraft::default(),
            area_points: 0,
            has_background: false,
            scale_points: 0,
            note: NoteDraft::Idle,
            asks_note: false,
            note_text: QString::default(),
            preview: Preview::Hidden,
            backup_state: QString::default(),
            backup_message: QString::default(),
            autosave_ms: AUTOSAVE_S * MS_PER_S,
            screenshot_path: args
                .screenshot
                .map(|p| q(&p.to_string_lossy()))
                .unwrap_or_default(),
            screenshot_after: args.screenshot_after_ms,
        };
        this.fill();
        this
    }
}

impl ControllerRust {
    /// The points picked for a scale or an area: a preview on top of the plan.
    fn picked(&self) -> Vec<Point> {
        let mut points = self.scale.points();
        points.extend_from_slice(self.area.points());

        points
    }

    /// Every property from the session, at start (no signals needed yet).
    fn fill(&mut self) {
        let Some(s) = &self.session else {
            return;
        };
        self.loaded = true;
        self.title = q(&s.project().title);
        self.plan_title = q(&s.plan_title());
        self.plan_json = q(&plan_json(s, &s.display_list(), None, &[]));
        self.has_background = has_background(s);
        self.tracks_json = q(&to_json(&s.tracks()));
        self.shots_json = q(&to_json(&s.shots()));
        self.script_json = q(&to_json(&s.script_lines()));
        self.layers_json = q(&layers_json(s));
        self.locks_json = q(&locks_json(s));
        self.time = s.time_s();
        self.duration = s.duration_s();
        self.selected_shot = q(s.selected_shot().unwrap_or_default());
        self.snapping_on = s.snapping() == Snapping::On;
        self.choices_json = q(&choices_json(s));

        let (state, message) = backup_view(s.backup());
        self.backup_state = q(state);
        self.backup_message = q(&message);
    }
}

fn to_json<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string(value).unwrap_or_default()
}

/// Whether the current scene's set has a background image.
fn has_background(s: &Session) -> bool {
    s.scene()
        .and_then(|sc| s.project().location(&sc.location))
        .is_some_and(|l| l.set.background.is_some())
}

/// `list` (the session's, or a preview of an edit) with the selected shot's camera and the
/// selected element marked; `picked` are the points of a scale in progress.
fn plan_json(
    s: &Session,
    list: &DisplayList,
    selected: Option<&Source>,
    picked: &[Point],
) -> String {
    let camera = s
        .selected_shot()
        .and_then(|id| s.plan()?.shots.iter().find(|shot| shot.id == id))
        .map(|shot| shot.camera.clone());
    let marked = |source: &Source| {
        Some(source) == selected
            || matches!(source, Source::Camera(id) if Some(id) == camera.as_ref())
    };

    plan::shapes_json(list, s.dir(), &marked, picked)
}

/// What the tool lists of the tool bar offer: the project's characters, object and lamp kinds.
fn choices_json(s: &Session) -> String {
    let figures: Vec<_> = s
        .project()
        .characters
        .iter()
        .map(|c| json!({ "id": c.id, "name": c.name }))
        .collect();
    let lights: Vec<_> = tools::LIGHT_KINDS.iter().map(|(name, _)| name).collect();

    let colors: Vec<_> = tools::DRAW_COLORS.iter().map(|(n, _)| n).collect();
    let dashes: Vec<_> = tools::DRAW_DASHES.iter().map(|(n, _)| n).collect();
    let fills: Vec<_> = tools::DRAW_FILLS.iter().map(|(n, _)| n).collect();

    json!({
        "figure": figures,
        "object": tools::OBJECT_KINDS,
        "light": lights,
        "color": colors,
        "dash": dashes,
        "fill": fills,
    })
    .to_string()
}

fn layers_json(s: &Session) -> String {
    let shown = |l| s.layer(l) == Visibility::Shown;

    json!({
        "set": shown(Layer::Set),
        "figures": shown(Layer::Figures),
        "cameras": shown(Layer::Cameras),
        "lights": shown(Layer::Lights),
        "notes": shown(Layer::Notes),
        "drawings": shown(Layer::Drawings),
    })
    .to_string()
}

fn locks_json(s: &Session) -> String {
    let locked = |l| s.lock(l) == Lock::Locked;

    json!({
        "set": locked(Layer::Set),
        "figures": locked(Layer::Figures),
        "cameras": locked(Layer::Cameras),
        "lights": locked(Layer::Lights),
        "notes": locked(Layer::Notes),
        "drawings": locked(Layer::Drawings),
    })
    .to_string()
}

fn visibility(text: &QString) -> Visibility {
    if text.to_string() == SHOWN {
        return Visibility::Shown;
    }
    Visibility::Hidden
}

/// `shooting` is the shooting order; anything else is the cut order.
fn shot_order(name: &str) -> ShotOrder {
    match name {
        "shooting" => ShotOrder::Shooting,
        _ => ShotOrder::Cut,
    }
}

fn layer_of(name: &str) -> Option<Layer> {
    match name {
        "set" => Some(Layer::Set),
        "figures" => Some(Layer::Figures),
        "cameras" => Some(Layer::Cameras),
        "lights" => Some(Layer::Lights),
        "notes" => Some(Layer::Notes),
        "drawings" => Some(Layer::Drawings),
        _ => None,
    }
}

impl qobject::Controller {
    fn session(self: Pin<&mut Self>) -> Option<&mut Session> {
        self.rust_mut().get_mut().session.as_mut()
    }

    /// Writes the plan next to the project folder with `write`; says where it went (`saved`
    /// is the catalogue key of the message) or what failed.
    fn export_plan(
        mut self: Pin<&mut Self>,
        extension: &str,
        saved: &str,
        write: impl FnOnce(&Session, &std::path::Path, Texts) -> Result<(), ServiceError>,
    ) -> bool {
        let language = self.rust().language;
        let texts = language.print_texts();
        let Some(s) = self.as_mut().session() else {
            return false;
        };

        let path = plan_file(s, extension);
        let message = match write(s, &path, texts) {
            Ok(()) => language
                .text(saved)
                .replace("{path}", &path.to_string_lossy()),
            Err(e) => error_text(language, &e),
        };
        let ok = path.exists();
        self.as_mut().say(message);
        ok
    }

    /// Forgets the selection when its element is gone (undo, a layer switched off).
    fn drop_stale(mut self: Pin<&mut Self>) {
        let stale = {
            let this = self.as_ref();
            match (&this.rust().session, &this.rust().selected) {
                (Some(s), Some(selected)) => !s
                    .display_list()
                    .items
                    .iter()
                    .any(|i| i.source.as_ref() == Some(selected)),
                _ => false,
            }
        };
        if stale {
            self.as_mut().rust_mut().get_mut().selected = None;
        }
    }

    /// Pushes what may have changed after a call to the properties (and their signals).
    fn refresh(mut self: Pin<&mut Self>) {
        self.as_mut().drop_stale();

        // Undo and redo can take the image away; its scale points go with it.
        let background = self.rust().session.as_ref().is_some_and(has_background);
        let backdrop = if background {
            Backdrop::Present
        } else {
            Backdrop::Missing
        };
        let scale = self.rust().scale.under(backdrop);
        self.as_mut().rust_mut().get_mut().scale = scale;

        let values = {
            let this = self.as_ref();
            let Some(s) = this.rust().session.as_ref() else {
                return;
            };
            (
                plan_json(
                    s,
                    &s.display_list(),
                    this.rust().selected.as_ref(),
                    &this.rust().picked(),
                ),
                to_json(&s.tracks()),
                to_json(&s.shots()),
                to_json(&s.script_lines()),
                layers_json(s),
                locks_json(s),
                s.time_s(),
                s.selected_shot().unwrap_or_default().to_owned(),
                s.can_undo(),
                s.can_redo(),
                s.is_dirty(),
                this.rust().selected.as_ref().map(plan::key),
            )
        };
        let (plan, tracks, shots, script, layers, locks, time, shot, undo, redo, dirty, selected) =
            values;
        let scale_points = self.rust().scale.points().len();
        let area_points = self.rust().area.points().len();
        let asks_note = self.rust().note != NoteDraft::Idle;

        // Restoring a backup or undoing it can change what only the start used to set.
        let header = {
            let this = self.as_ref();
            this.rust().session.as_ref().map(|s| {
                (
                    s.project().title.clone(),
                    s.plan_title(),
                    s.duration_s(),
                    choices_json(s),
                )
            })
        };
        if let Some((title, plan_title, duration, choices)) = header {
            self.as_mut().set_title(q(&title));
            self.as_mut().set_plan_title(q(&plan_title));
            self.as_mut().set_duration(duration);
            self.as_mut().set_choices_json(q(&choices));
        }

        // The plan set here is the project's: a preview is gone.
        self.as_mut().rust_mut().get_mut().preview = Preview::Hidden;
        self.as_mut().set_plan_json(q(&plan));
        self.as_mut().set_tracks_json(q(&tracks));
        self.as_mut().set_shots_json(q(&shots));
        self.as_mut().set_script_json(q(&script));
        self.as_mut().set_layers_json(q(&layers));
        self.as_mut().set_locks_json(q(&locks));
        self.as_mut().set_time(time);
        self.as_mut().set_selected_shot(q(&shot));
        self.as_mut()
            .set_selected_key(q(&selected.unwrap_or_default()));
        self.as_mut().set_can_undo(undo);
        self.as_mut().set_can_redo(redo);
        self.as_mut().set_dirty(dirty);
        self.as_mut().set_has_background(background);
        self.as_mut()
            .set_scale_points(i32::try_from(scale_points).unwrap_or_default());
        self.as_mut()
            .set_area_points(i32::try_from(area_points).unwrap_or_default());
        self.as_mut().set_asks_note(asks_note);
    }

    /// Shows `text` in the header until the project changes.
    fn say(mut self: Pin<&mut Self>, text: String) {
        self.as_mut().rust_mut().get_mut().notice.say(text.clone());
        self.set_status(q(&text));
    }

    /// The project changed: a "saved" or error message no longer fits.
    fn changed(mut self: Pin<&mut Self>) {
        if self.as_mut().rust_mut().get_mut().notice.project_changed() {
            self.set_status(QString::default());
        }
    }

    fn seek(mut self: Pin<&mut Self>, time: f64) {
        let Some(s) = self.as_mut().session() else {
            return;
        };
        s.set_time(time);
        self.refresh();
    }

    fn select_shot(mut self: Pin<&mut Self>, id: &QString) {
        let Some(s) = self.as_mut().session() else {
            return;
        };
        if s.select_shot(&id.to_string()).is_err() {
            return;
        }
        self.refresh();
    }

    fn edit_shot(mut self: Pin<&mut Self>, id: &QString, field: &QString, value: &QString) -> bool {
        let language = self.rust().language;
        let Some(field) = ShotField::from_name(&field.to_string()) else {
            return false;
        };
        let Some(s) = self.as_mut().session() else {
            return false;
        };
        if let Err(e) = s.edit_shot(&id.to_string(), field, &value.to_string()) {
            self.say(error_text(language, &e));
            return false;
        }
        self.as_mut().changed();
        self.refresh();
        true
    }

    fn undo(mut self: Pin<&mut Self>) {
        if let Some(s) = self.as_mut().session() {
            s.undo();
        }
        self.as_mut().changed();
        self.refresh();
    }

    fn redo(mut self: Pin<&mut Self>) {
        if let Some(s) = self.as_mut().session() {
            s.redo();
        }
        self.as_mut().changed();
        self.refresh();
    }

    fn save(mut self: Pin<&mut Self>) -> bool {
        let language = self.rust().language;
        let result = match self.as_mut().session() {
            Some(s) => s.save().map_err(|e| error_text(language, &e)),
            None => return false,
        };

        let message = match &result {
            Ok(()) => language.text("menu.saved"),
            Err(e) => e.clone(),
        };
        self.as_mut().say(message);
        self.refresh();
        result.is_ok()
    }

    fn autosave(mut self: Pin<&mut Self>) {
        let language = self.rust().language;
        let Some(s) = self.as_mut().session() else {
            return;
        };

        // Written: nothing to show. Clean and Held: nothing to write.
        match s.autosave() {
            Ok(Autosave::Written | Autosave::Clean | Autosave::Held) => {}
            Err(e) => self.say(error_text(language, &e)),
        }
    }

    fn restore_backup(mut self: Pin<&mut Self>) {
        let language = self.rust().language;
        let Some(s) = self.as_mut().session() else {
            return;
        };

        if let Err(e) = s.restore_backup() {
            self.say(error_text(language, &e));
            return;
        }
        self.as_mut().changed();
        self.as_mut().refresh();
        self.show_backup();
    }

    fn discard_backup(mut self: Pin<&mut Self>) {
        let language = self.rust().language;
        let Some(s) = self.as_mut().session() else {
            return;
        };

        if let Err(e) = s.discard_backup() {
            self.say(error_text(language, &e));
            return;
        }
        self.show_backup();
    }

    /// Pushes the session's backup state to QML.
    fn show_backup(mut self: Pin<&mut Self>) {
        let view = match self.as_mut().session() {
            Some(s) => backup_view(s.backup()),
            None => return,
        };
        self.as_mut().set_backup_state(q(view.0));
        self.set_backup_message(q(&view.1));
    }

    fn set_tool(mut self: Pin<&mut Self>, tool: &QString, choice: &QString) -> bool {
        let Some(tool) = Tool::parse(&tool.to_string(), &choice.to_string()) else {
            return false;
        };

        // Points picked for a scale do not outlast the tool.
        let leaves_scale = tool != Tool::Scale && self.rust().scale != ScaleStep::Idle;
        let leaves_note = tool != Tool::Note && self.rust().note != NoteDraft::Idle;
        let leaves_area = tool != Tool::Area && !self.rust().area.is_empty();
        self.as_mut().rust_mut().get_mut().tool = tool;
        if leaves_scale {
            self.as_mut().cancel_scale();
        }
        if leaves_area {
            self.as_mut().cancel_area();
        }
        if leaves_note {
            self.cancel_note();
        }
        true
    }

    fn drag(
        mut self: Pin<&mut Self>,
        from_x: f64,
        from_y: f64,
        x: f64,
        y: f64,
        done: bool,
    ) -> bool {
        let tool = self.rust().tool.clone();
        let style = self.rust().draw;
        let Some(s) = self.as_mut().session() else {
            return false;
        };

        // The canvas' key can name a view cone or a path that lies over the thing meant;
        // the session picks what a user can grab.
        let from = Point::new(from_x, from_y);
        let to = Point::new(x, y);
        let grabbed = s.pick(from);

        // Select and Scale move by a distance (the session snaps where an element lands, a
        // background moves freely); the other tools draw between two points, which snap here.
        let (from, to) = match tool {
            Tool::Select | Tool::Scale => (from, to),
            _ => (s.snap(from), s.snap(to)),
        };
        let mut edit = tools::drag_edit(&tool, grabbed.as_ref(), from, to)
            .or_else(|| tools::draw_edit(&tool, style, from, to));
        if tool == Tool::Scale && !has_background(s) {
            edit = None;
        }
        let preview = self.rust().preview;
        let phase = if done {
            DragPhase::Done
        } else {
            DragPhase::Moving
        };

        if tool == Tool::Select && edit.is_some() {
            self.as_mut().rust_mut().get_mut().selected = grabbed;
        }
        match tools::drag_step(edit, phase, preview) {
            DragStep::Preview(edit) => self.preview_edit(&edit),
            DragStep::Commit(edit) => self.commit_edit(edit),
            DragStep::Restore => {
                self.refresh();
                false
            }
            DragStep::Nothing => false,
        }
    }

    fn place(mut self: Pin<&mut Self>, x: f64, y: f64) -> bool {
        let tool = self.rust().tool.clone();
        let Some(s) = self.as_mut().session() else {
            return false;
        };

        if tool == Tool::Select {
            let picked = s.pick(Point::new(x, y));
            let found = picked.is_some();
            self.as_mut().rust_mut().get_mut().selected = picked;
            self.refresh();
            return found;
        }

        let at = tools::lands_at(&tool, Point::new(x, y), |p| s.snap(p));
        if tool == Tool::Scale {
            return self.pick_scale_point(at);
        }
        if tool == Tool::Note {
            let picked = s.pick(Point::new(x, y));
            return self.ask_note(NoteDraft::click(at, picked.as_ref()));
        }
        if tool == Tool::Area {
            return self.click_area(at);
        }

        let Some(edit) = tools::click_edit(&tool, at) else {
            return false;
        };

        self.commit_edit(edit)
    }

    fn set_background(mut self: Pin<&mut Self>, path: &QString) -> bool {
        let source = tools::local_path(&path.to_string());
        if !self
            .as_mut()
            .commit_edit(PlanEdit::SetBackground { source })
        {
            return false;
        }

        // The edit cleared the status line; the next step goes there.
        let hint = self.rust().language.text("background.hint");
        self.as_mut().say(hint);
        true
    }

    fn remove_background(mut self: Pin<&mut Self>) -> bool {
        self.as_mut().rust_mut().get_mut().scale = ScaleStep::Idle;
        self.commit_edit(PlanEdit::RemoveBackground)
    }

    fn calibrate(mut self: Pin<&mut Self>, distance: &QString) -> bool {
        let step = self.rust().scale;
        let Some(edit) = tools::scale_edit(step, &distance.to_string()) else {
            return false;
        };

        self.as_mut().rust_mut().get_mut().scale = ScaleStep::Idle;
        self.commit_edit(edit)
    }

    fn cancel_scale(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().get_mut().scale = ScaleStep::Idle;
        self.refresh();
    }

    fn set_draw_style(
        mut self: Pin<&mut Self>,
        color: &QString,
        dash: &QString,
        fill: &QString,
    ) -> bool {
        let names = (color.to_string(), dash.to_string(), fill.to_string());
        let Some(style) = DrawStyle::parse(&names.0, &names.1, &names.2) else {
            return false;
        };

        self.as_mut().rust_mut().get_mut().draw = style;
        true
    }

    fn finish_area(mut self: Pin<&mut Self>) -> bool {
        let style = self.rust().draw;
        let Some(edit) = tools::area_edit(&self.rust().area, style) else {
            return false;
        };

        // The points stay when the edit is refused, so the work is not lost.
        if !self.as_mut().commit_edit(edit) {
            return false;
        }

        self.as_mut().rust_mut().get_mut().area = AreaDraft::default();
        self.refresh();
        true
    }

    fn cancel_area(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().get_mut().area = AreaDraft::default();
        self.refresh();
    }

    fn commit_note(mut self: Pin<&mut Self>, text: &QString) -> bool {
        let draft = self.rust().note.clone();
        let Some(edit) = tools::note_edit(&draft, &text.to_string()) else {
            return false;
        };

        // The draft stays open when the edit is refused, so the typed text is not lost.
        if !self.as_mut().commit_edit(edit) {
            return false;
        }

        self.as_mut().rust_mut().get_mut().note = NoteDraft::Idle;
        self.refresh();
        true
    }

    fn cancel_note(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().get_mut().note = NoteDraft::Idle;
        self.refresh();
    }

    fn edit_selected_note(self: Pin<&mut Self>) -> bool {
        let Some(Source::Note(id)) = self.rust().selected.clone() else {
            return false;
        };

        self.ask_note(NoteDraft::Edit(id))
    }

    fn grabbable(&self, x: f64, y: f64) -> bool {
        let Some(s) = self.rust().session.as_ref() else {
            return false;
        };
        s.pick(Point::new(x, y)).is_some()
    }

    fn remove_selected(mut self: Pin<&mut Self>) -> bool {
        let edit = tools::key_edit(Key::Remove, self.rust().selected.as_ref());
        let Some(edit) = edit else {
            return false;
        };

        // The indices of what is left may shift, so the selection goes either way.
        self.as_mut().rust_mut().get_mut().selected = None;
        self.commit_edit(edit)
    }

    fn turn_selected(self: Pin<&mut Self>, step_deg: f64) -> bool {
        let key = Key::Turn { by_deg: step_deg };
        let Some(edit) = tools::key_edit(key, self.rust().selected.as_ref()) else {
            return false;
        };

        self.commit_edit(edit)
    }

    fn set_snapping(mut self: Pin<&mut Self>, on: bool) {
        let snapping = if on { Snapping::On } else { Snapping::Off };
        if let Some(s) = self.as_mut().session() {
            s.set_snapping(snapping);
        }
        self.set_snapping_on(on);
    }

    fn cancel_drag(self: Pin<&mut Self>) {
        self.refresh();
    }

    /// Starts asking for a note's text: empty for a new note, the note's own for an edit.
    /// False when the note to edit is gone.
    fn ask_note(mut self: Pin<&mut Self>, draft: NoteDraft) -> bool {
        let language = self.rust().language;
        let lock = self.rust().session.as_ref().map(|s| s.lock(Layer::Notes));
        if let Some(Err(e)) = lock.map(tools::note_open) {
            self.as_mut().say(error_text(language, &e));
            return false;
        }

        let text = match &draft {
            NoteDraft::Edit(id) => {
                let text = self.rust().session.as_ref().and_then(|s| note_text(s, id));
                let Some(text) = text else {
                    return false;
                };
                text
            }
            _ => String::new(),
        };

        self.as_mut().rust_mut().get_mut().note = draft;
        self.as_mut().set_note_text(q(&text));
        self.refresh();
        true
    }

    /// A click of the area tool: the next point, or the closing click on the first one.
    fn click_area(mut self: Pin<&mut Self>, at: Point) -> bool {
        let language = self.rust().language;
        let lock = self
            .rust()
            .session
            .as_ref()
            .map(|s| s.lock(Layer::Drawings));
        if let Some(Err(e)) = lock.map(tools::drawing_open) {
            self.as_mut().say(error_text(language, &e));
            return false;
        }

        let draft = self.rust().area.clone();
        if draft.closes(at) {
            return self.finish_area();
        }

        self.as_mut().rust_mut().get_mut().area = draft.click(at);
        self.refresh();
        true
    }

    /// A click of the scale tool: the next point of the stretch. Without a background there
    /// is nothing to scale.
    fn pick_scale_point(mut self: Pin<&mut Self>, at: Point) -> bool {
        if !self.rust().session.as_ref().is_some_and(has_background) {
            return false;
        }

        let step = self.rust().scale.click(at);
        self.as_mut().rust_mut().get_mut().scale = step;
        self.refresh();
        true
    }

    /// Shows what `edit` would do, as the plan; the project stays as it is. A failed preview
    /// shows the project, not the last preview.
    fn preview_edit(mut self: Pin<&mut Self>, edit: &PlanEdit) -> bool {
        let json = {
            let this = self.as_ref();
            this.rust().session.as_ref().and_then(|s| {
                let list = s.preview(edit).ok()?;
                let picked = this.rust().picked();
                Some(plan_json(s, &list, this.rust().selected.as_ref(), &picked))
            })
        };

        let Some(json) = json else {
            self.refresh();
            return false;
        };
        self.as_mut().rust_mut().get_mut().preview = Preview::Shown;
        self.set_plan_json(q(&json));
        true
    }

    /// Applies `edit`; an error goes to the status line.
    fn commit_edit(mut self: Pin<&mut Self>, edit: PlanEdit) -> bool {
        let language = self.rust().language;
        let Some(s) = self.as_mut().session() else {
            return false;
        };

        let result = s.edit_plan(edit);
        let ok = result.is_ok();
        match result {
            Ok(()) => self.as_mut().changed(),
            Err(e) => self.as_mut().say(error_text(language, &e)),
        }
        self.refresh();
        ok
    }

    fn set_track(mut self: Pin<&mut Self>, id: &QString, shown: &QString) {
        if let Some(s) = self.as_mut().session() {
            s.set_track(&id.to_string(), visibility(shown));
        }
        self.refresh();
    }

    fn set_layer(mut self: Pin<&mut Self>, layer: &QString, shown: &QString) {
        let Some(layer) = layer_of(&layer.to_string()) else {
            return;
        };
        if let Some(s) = self.as_mut().session() {
            s.set_layer(layer, visibility(shown));
        }
        self.refresh();
    }

    fn set_lock(mut self: Pin<&mut Self>, layer: &QString, locked: &QString) {
        let Some(layer) = layer_of(&layer.to_string()) else {
            return;
        };
        let lock = match locked.to_string() == LOCKED {
            true => Lock::Locked,
            false => Lock::Unlocked,
        };
        if let Some(s) = self.as_mut().session() {
            s.set_lock(layer, lock);
        }
        self.refresh();
    }

    fn export_pdf(self: Pin<&mut Self>, order: &QString) -> bool {
        let order = shot_order(&order.to_string());

        self.export_plan("pdf", "menu.pdfSaved", |s, path, texts| {
            s.export_pdf(path, Paper::A4, texts, order)
        })
    }

    fn export_png(self: Pin<&mut Self>) -> bool {
        self.export_plan("png", "menu.pngSaved", |s, path, texts| {
            s.export_png(path, Paper::A4, texts)
        })
    }

    fn export_svg(self: Pin<&mut Self>) -> bool {
        self.export_plan("svg", "menu.svgSaved", |s, path, texts| {
            s.export_svg(path, Paper::A4, texts)
        })
    }

    fn export_csv(mut self: Pin<&mut Self>, order: &QString) -> bool {
        let language = self.rust().language;
        let texts = language.print_texts();
        let Some(s) = self.as_mut().session() else {
            return false;
        };

        let path = plan_file(s, "csv");
        let message = match s.export_csv(&path, &texts, shot_order(&order.to_string())) {
            Ok(()) => language
                .text("menu.csvSaved")
                .replace("{path}", &path.to_string_lossy()),
            Err(e) => error_text(language, &e),
        };
        let ok = path.exists();
        self.as_mut().say(message);
        ok
    }

    fn describe(&self, key: &QString) -> QString {
        let Some(s) = self.rust().session.as_ref() else {
            return QString::default();
        };
        q(&describe(s, &key.to_string()).to_string())
    }
}

/// Where an export of the current plan goes: next to the project folder, named by the plan.
fn plan_file(s: &Session, extension: &str) -> std::path::PathBuf {
    let name = format!(
        "{}.{extension}",
        s.plan_title().replace([' ', '·', '/', '–'], "-")
    );

    s.dir()
        .parent()
        .map(|p| p.join(&name))
        .unwrap_or_else(|| name.into())
}

/// The text of the current set's note `id`.
fn note_text(s: &Session, id: &str) -> Option<String> {
    let location = s.project().location(&s.scene()?.location)?;

    Some(location.set.notes.iter().find(|n| n.id == id)?.text.clone())
}

/// Data for a tooltip; QML puts it into words with the catalogue.
fn describe(s: &Session, key: &str) -> serde_json::Value {
    let mut parts = key.split(':');
    let (Some(kind), Some(id)) = (parts.next(), parts.next()) else {
        return json!({});
    };
    let Some(plan) = s.plan() else {
        return json!({});
    };

    match kind {
        "camera" => {
            let Some(c) = plan.camera(id) else {
                return json!({});
            };
            let shots: Vec<&str> = plan
                .shots
                .iter()
                .filter(|shot| shot.camera == id)
                .map(|shot| shot.number.as_str())
                .collect();
            let lens_mm = s.lens_now_mm(id).unwrap_or(c.lens_mm);
            json!({
                "kind": "camera",
                "label": c.label,
                "lens": lens_mm,
                "fov": (horizontal_fov_deg(c.sensor, lens_mm) * 10.0).round() / 10.0,
                "height": c.marks.first().map(|m| m.height_m).unwrap_or(0.0),
                "shots": shots.join(", "),
            })
        }
        "figure" | "mark" => {
            let name = s
                .project()
                .character(id)
                .map(|c| c.name.clone())
                .unwrap_or_default();
            let index: Option<usize> = parts.next().and_then(|i| i.parse().ok());
            let at = index
                .and_then(|i| plan.figure(id)?.marks.get(i))
                .map(|m| m.at_s);
            json!({ "kind": kind, "name": name, "index": index.map(|i| i + 1), "time": at })
        }
        "light" => {
            let Some(l) = plan.lights.iter().find(|l| l.id == id) else {
                return json!({});
            };
            json!({ "kind": "light", "label": l.label, "temp": l.color_temp_k, "power": l.power_w })
        }
        "object" => {
            let set = s
                .scene()
                .and_then(|sc| s.project().location(&sc.location))
                .map(|l| &l.set);
            let label = set
                .and_then(|set| set.objects.iter().find(|o| o.id == id))
                .map(|o| o.label.clone())
                .unwrap_or_default();
            json!({ "kind": "object", "label": label })
        }
        "note" => json!({ "kind": "note", "text": note_text(s, id).unwrap_or_default() }),
        _ => json!({}),
    }
}
