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
        /// How many plan elements are selected.
        #[qproperty(i32, selected_count, cxx_name = "selectedCount")]
        /// Shift is held: a click or a box adds to the selection instead of replacing it.
        #[qproperty(bool, extend_selection, cxx_name = "extendSelection")]
        /// Whether points snap to the grid; change it with `setSnapping`.
        #[qproperty(bool, snapping_on, cxx_name = "snappingOn")]
        /// `{ figure: [{ id, name }], object: [{ kind, label, width_m, depth_m, shape, custom }],
        /// light: [kind], color: [name], dash: [name], fill: [name] }`: what the tool lists
        /// offer. The objects are the library: built in, then the project's own.
        #[qproperty(QString, choices_json, cxx_name = "choicesJson")]
        /// Whether the set has a background image.
        #[qproperty(bool, has_background, cxx_name = "hasBackground")]
        /// Points picked so far by the scale tool: 0, 1 or 2 (then the length is asked).
        #[qproperty(i32, scale_points, cxx_name = "scalePoints")]
        /// Whether the measuring tool has a point set (`cancelMeasure` drops it).
        #[qproperty(bool, measuring)]
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
        /// `{ scenes, locations, characters }` of the project, for the project view.
        #[qproperty(QString, project_json, cxx_name = "projectJson")]
        /// Id of the scene on view, empty for none.
        #[qproperty(QString, scene_id, cxx_name = "sceneId")]
        /// `[{ path, name }]`: the projects opened last, those still on disk.
        #[qproperty(QString, recent_json, cxx_name = "recentJson")]
        /// The same list as the app's settings keep it; QML stores it when it changes.
        #[qproperty(QString, recent_store, cxx_name = "recentStore")]
        #[qproperty(QString, screenshot_path, cxx_name = "screenshotPath")]
        #[qproperty(i32, screenshot_after, cxx_name = "screenshotAfter")]
        type Controller = super::ControllerRust;

        /// Moves the playhead (seconds).
        #[qinvokable]
        fn seek(self: Pin<&mut Self>, time: f64);

        /// Moves the playhead to the previous (`direction` < 0) or next mark: a mark of a
        /// figure or camera or the edge of a shot.
        #[qinvokable]
        #[cxx_name = "stepMark"]
        fn step_mark(self: Pin<&mut Self>, direction: i32);

        /// Plays `dt` seconds of real time at `speed` from the playhead; `looping` is "shot" to
        /// repeat the selected shot, else "off". False once the end is reached.
        #[qinvokable]
        fn advance(self: Pin<&mut Self>, dt: f64, speed: f64, looping: &QString) -> bool;

        #[qinvokable]
        #[cxx_name = "selectShot"]
        fn select_shot(self: Pin<&mut Self>, id: &QString);

        /// Sets one field (`lens`, `size` …) of a shot; false if the value is refused.
        #[qinvokable]
        #[cxx_name = "editShot"]
        fn edit_shot(self: Pin<&mut Self>, id: &QString, field: &QString, value: &QString) -> bool;

        /// Adds a shot at the playhead for the camera selected in the plan, else the selected
        /// shot's or the first one, and selects it; false if refused.
        #[qinvokable]
        #[cxx_name = "addShot"]
        fn add_shot(self: Pin<&mut Self>) -> bool;

        /// Copies a shot under a new number and selects the copy; false if refused.
        #[qinvokable]
        #[cxx_name = "copyShot"]
        fn copy_shot(self: Pin<&mut Self>, id: &QString) -> bool;

        /// Deletes a shot; false if refused.
        #[qinvokable]
        #[cxx_name = "deleteShot"]
        fn delete_shot(self: Pin<&mut Self>, id: &QString) -> bool;

        /// Couples the camera to a figure (character id), or lets it go with an empty
        /// `figure`; false if refused.
        #[qinvokable]
        #[cxx_name = "setFollow"]
        fn set_follow(self: Pin<&mut Self>, camera: &QString, figure: &QString) -> bool;

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

        /// The measuring tool: the pointer is at (`x`, `y`) metres; an open measurement follows
        /// it. False when nothing changed.
        #[qinvokable]
        fn hover(self: Pin<&mut Self>, x: f64, y: f64) -> bool;

        /// Drops the measurement.
        #[qinvokable]
        #[cxx_name = "cancelMeasure"]
        fn cancel_measure(self: Pin<&mut Self>);

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

        /// Removes the selected elements.
        #[qinvokable]
        #[cxx_name = "removeSelected"]
        fn remove_selected(self: Pin<&mut Self>) -> bool;

        /// Copies the selected elements, set apart from the originals, and selects the copies.
        #[qinvokable]
        #[cxx_name = "duplicateSelected"]
        fn duplicate_selected(self: Pin<&mut Self>) -> bool;

        /// Makes the selected way straight again; false when nothing is selected.
        #[qinvokable]
        #[cxx_name = "straightenSelected"]
        fn straighten_selected(self: Pin<&mut Self>) -> bool;

        /// Turns the selected elements by `step_deg`, clockwise when positive; several turn
        /// about the middle of them all.
        #[qinvokable]
        #[cxx_name = "turnSelected"]
        fn turn_selected(self: Pin<&mut Self>, step_deg: f64) -> bool;

        /// Gives the selected object a new width and depth (comma or point as decimal
        /// separator). False when no object is selected or a side is not a length; a refusal
        /// by the session goes to `status`.
        #[qinvokable]
        #[cxx_name = "resizeSelected"]
        fn resize_selected(self: Pin<&mut Self>, width: &QString, depth: &QString) -> bool;

        /// Sets the height in metres of the selected object (LH-17); an empty text takes it
        /// off. False when no object is selected or the text is no length; a refusal by the
        /// session goes to `status`.
        #[qinvokable]
        #[cxx_name = "setObjectHeight"]
        fn set_object_height(self: Pin<&mut Self>, height: &QString) -> bool;

        /// Sets one field of the selected camera (LH-32): `field` is `body` (a body id of
        /// `choicesJson`), `sensor` (mm), `lens` (mm) or `aspect` (`2,39` or `16:9`). False
        /// when no camera is selected or the value is not usable; a refusal by the session
        /// goes to `status`.
        #[qinvokable]
        #[cxx_name = "setCameraField"]
        fn set_camera_field(self: Pin<&mut Self>, field: &QString, value: &QString) -> bool;
        /// The fields of the selected mark (a figure's, or a camera's) as JSON: `kind`
        /// (`figure` or `camera`), `pose`, `arrival` (`walk`, `appear`) and `departure` (`stay`,
        /// `leave`), each empty for a camera, `hold` in seconds, `hasWay`,
        /// `pace` in m/s (null without time on the way), `height` in m and `tilt` in degrees
        /// (null for a figure). `{}` for any other selection.
        #[qinvokable]
        #[cxx_name = "markInfo"]
        fn mark_info(self: &Self) -> QString;

        /// Sets lens height (m) and tilt (degrees, positive looks up) of the selected camera
        /// mark (LH-34), comma or point as decimal separator. False for another selection or
        /// a text that is no number; a refusal by the session (limits) goes to `status`.
        #[qinvokable]
        #[cxx_name = "setCameraAim"]
        fn set_camera_aim(self: Pin<&mut Self>, height: &QString, tilt: &QString) -> bool;

        /// Sets the pose (`standing`, `sitting`, `lying`) of the selected figure mark. False for
        /// another selection or name; a refusal by the session goes to `status`.
        #[qinvokable]
        #[cxx_name = "setMarkPose"]
        fn set_mark_pose(self: Pin<&mut Self>, pose: &QString) -> bool;

        /// Sets the pause in seconds (comma or point as decimal separator, zero or more) at the
        /// selected mark. False for another selection or a text that is no time.
        #[qinvokable]
        #[cxx_name = "setMarkHold"]
        fn set_mark_hold(self: Pin<&mut Self>, seconds: &QString) -> bool;

        /// Sets how the figure comes to the selected mark: `walk` or `appear` (LH-26). False for
        /// another selection or name; a refusal by the session goes to `status`.
        #[qinvokable]
        #[cxx_name = "setMarkArrival"]
        fn set_mark_arrival(self: Pin<&mut Self>, arrival: &QString) -> bool;

        /// Sets what the figure does after the selected mark: `stay` or `leave` (LH-26). False
        /// for another selection or name; a refusal by the session goes to `status`.
        #[qinvokable]
        #[cxx_name = "setMarkDeparture"]
        fn set_mark_departure(self: Pin<&mut Self>, departure: &QString) -> bool;

        /// Times the way to the selected figure mark for `walk` or `run`. False for another
        /// selection or name; the first mark is refused by the session.
        #[qinvokable]
        #[cxx_name = "setMarkPace"]
        fn set_mark_pace(self: Pin<&mut Self>, pace: &QString) -> bool;

        /// Files the selected object, with its size and shape, in the project's library under
        /// `name`. False for no object or a blank name.
        #[qinvokable]
        #[cxx_name = "saveToLibrary"]
        fn save_to_library(self: Pin<&mut Self>, name: &QString) -> bool;

        /// Takes the objects of the library file `path` (a path or a `file://` URL) into the
        /// project; the result goes to `status`.
        #[qinvokable]
        #[cxx_name = "importLibrary"]
        fn import_library(self: Pin<&mut Self>, path: &QString) -> bool;

        /// Writes the project's own objects to the library file `path`; the result goes to
        /// `status`.
        #[qinvokable]
        #[cxx_name = "exportLibrary"]
        fn export_library(self: Pin<&mut Self>, path: &QString) -> bool;

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

        /// Takes the list stored in the settings (`recentStore`) and shows it in `recentJson`.
        #[qinvokable]
        #[cxx_name = "loadRecent"]
        fn load_recent(self: Pin<&mut Self>, stored: &QString);

        /// Makes an empty project in the folder `path` (a path or a `file://` URL) and opens
        /// it; an error goes to `status`.
        #[qinvokable]
        #[cxx_name = "createProject"]
        fn create_project(self: Pin<&mut Self>, path: &QString, title: &QString) -> bool;

        /// Opens the project folder `path`; the open one is backed up first.
        #[qinvokable]
        #[cxx_name = "openProject"]
        fn open_project(self: Pin<&mut Self>, path: &QString) -> bool;

        /// Shows the scene `id`.
        #[qinvokable]
        #[cxx_name = "selectScene"]
        fn select_scene(self: Pin<&mut Self>, id: &QString) -> bool;

        /// Empties the status line (a form opens without the last error).
        #[qinvokable]
        #[cxx_name = "clearStatus"]
        fn clear_status(self: Pin<&mut Self>);

        /// The project view's changes. False when refused; the reason goes to `status`.
        #[qinvokable]
        #[cxx_name = "addScene"]
        fn add_scene(
            self: Pin<&mut Self>,
            number: &QString,
            heading: &QString,
            location: &QString,
        ) -> bool;

        #[qinvokable]
        #[cxx_name = "editScene"]
        fn edit_scene(
            self: Pin<&mut Self>,
            id: &QString,
            number: &QString,
            heading: &QString,
            location: &QString,
        ) -> bool;

        #[qinvokable]
        #[cxx_name = "removeScene"]
        fn remove_scene(self: Pin<&mut Self>, id: &QString) -> bool;

        /// Sizes are text in metres (`8,5`).
        #[qinvokable]
        #[cxx_name = "addLocation"]
        fn add_location(
            self: Pin<&mut Self>,
            name: &QString,
            width: &QString,
            depth: &QString,
        ) -> bool;

        #[qinvokable]
        #[cxx_name = "editLocation"]
        fn edit_location(
            self: Pin<&mut Self>,
            id: &QString,
            name: &QString,
            width: &QString,
            depth: &QString,
        ) -> bool;

        #[qinvokable]
        #[cxx_name = "removeLocation"]
        fn remove_location(self: Pin<&mut Self>, id: &QString) -> bool;

        /// `color` is a role name (`purple`, `blue` …).
        #[qinvokable]
        #[cxx_name = "addCharacter"]
        fn add_character(
            self: Pin<&mut Self>,
            name: &QString,
            label: &QString,
            color: &QString,
        ) -> bool;

        #[qinvokable]
        #[cxx_name = "editCharacter"]
        fn edit_character(
            self: Pin<&mut Self>,
            id: &QString,
            name: &QString,
            label: &QString,
            color: &QString,
        ) -> bool;

        #[qinvokable]
        #[cxx_name = "removeCharacter"]
        fn remove_character(self: Pin<&mut Self>, id: &QString) -> bool;

        /// Variants of the scene's plan (LH-04). Shows the plan `id` of the current scene.
        #[qinvokable]
        #[cxx_name = "selectPlan"]
        fn select_plan(self: Pin<&mut Self>, id: &QString) -> bool;

        /// Adds an empty plan and shows it; an empty `name` is named by the session.
        #[qinvokable]
        #[cxx_name = "addPlan"]
        fn add_plan(self: Pin<&mut Self>, name: &QString) -> bool;

        /// A snapshot: copies the plan on view and shows the copy.
        #[qinvokable]
        #[cxx_name = "copyPlan"]
        fn copy_plan(self: Pin<&mut Self>, name: &QString) -> bool;

        #[qinvokable]
        #[cxx_name = "renamePlan"]
        fn rename_plan(self: Pin<&mut Self>, id: &QString, name: &QString) -> bool;

        #[qinvokable]
        #[cxx_name = "removePlan"]
        fn remove_plan(self: Pin<&mut Self>, id: &QString) -> bool;

        #[qinvokable]
        #[cxx_name = "removePlanWithShots"]
        fn remove_plan_with_shots(self: Pin<&mut Self>, id: &QString) -> bool;
    }
}

use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use klappe_services::{
    horizontal_fov_deg, Autosave, DisplayList, Field, FigureColor, Layer, Lock, Looping, Paper,
    PlanEdit, Playing, Point, ProjectEdit, ServiceError, Session, ShotField, ShotOrder, ShotsGo,
    Snapping, Source, Step, Texts, Visibility,
};
use serde_json::json;

use crate::args::AppArgs;
use crate::camera::{self, CameraField};
use crate::catalog::Language;
use crate::manage;
use crate::notice::{backup_view, error_text, Status};
use crate::plan::{self, Measure};
use crate::tools::{
    self, AreaDraft, Backdrop, DragPhase, DragStep, DrawStyle, Grip, Key, MeasureStep, NoteDraft,
    Pick, Preview, ScaleStep, Selection, Tool,
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
    selected_count: i32,
    extend_selection: bool,
    snapping_on: bool,
    choices_json: QString,
    /// What the select tool has picked; the canvas frames it.
    selected: Selection,
    /// The box a drag on empty ground is pulling up, in metres.
    marquee: Option<(Point, Point)>,
    tool: Tool,
    scale: ScaleStep,
    /// The measurement of the measuring tool: a tool state of the session, not the plan's.
    measure: MeasureStep,
    measuring: bool,
    /// What the drawing tools draw with, and the area they are drawing.
    draw: DrawStyle,
    area: AreaDraft,
    /// The scene and plan on screen (`scene/plan`), to tell when undo, redo or a project
    /// edit changes them.
    shown: String,
    area_points: i32,
    has_background: bool,
    scale_points: i32,
    /// The note whose text is asked, if any.
    note: NoteDraft,
    asks_note: bool,
    note_text: QString,
    /// What the plan on screen shows: a drag preview or the project.
    preview: Preview,
    /// What the drag in progress moves; `Free` between drags.
    grip: Grip,
    status: QString,
    backup_state: QString,
    backup_message: QString,
    autosave_ms: i32,
    project_json: QString,
    scene_id: QString,
    recent_json: QString,
    recent_store: QString,
    screenshot_path: QString,
    screenshot_after: i32,
}

fn q(s: &str) -> QString {
    QString::from(s)
}

const SHOWN: &str = "shown";
const LOCKED: &str = "locked";
const LOOP_SHOT: &str = "shot";

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
            selected_count: 0,
            extend_selection: false,
            snapping_on: true,
            choices_json: QString::default(),
            selected: Selection::default(),
            marquee: None,
            tool: Tool::Select,
            scale: ScaleStep::Idle,
            measure: MeasureStep::Idle,
            measuring: false,
            draw: DrawStyle::default(),
            area: AreaDraft::default(),
            shown: String::new(),
            area_points: 0,
            has_background: false,
            scale_points: 0,
            note: NoteDraft::Idle,
            asks_note: false,
            note_text: QString::default(),
            preview: Preview::Hidden,
            grip: Grip::Free,
            backup_state: QString::default(),
            backup_message: QString::default(),
            autosave_ms: AUTOSAVE_S * MS_PER_S,
            project_json: QString::default(),
            scene_id: QString::default(),
            recent_json: q("[]"),
            recent_store: q("[]"),
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
    /// Forgets what was picked and half drawn: it belongs to the plan on screen, not to the
    /// next scene or project.
    fn drop_drafts(&mut self) {
        self.selected = Selection::default();
        self.marquee = None;
        self.scale = ScaleStep::Idle;
        self.measure = MeasureStep::Idle;
        self.note = NoteDraft::Idle;
        self.area = AreaDraft::default();
    }

    /// Notes the scene and plan on screen; when they changed, the drafts made on the old
    /// one go (undo, redo and project edits can change the view, not only a click).
    fn follow_view(&mut self, shown: String) {
        if shown == self.shown {
            return;
        }

        self.drop_drafts();
        self.shown = shown;
    }

    /// What a pick does to the selection: Shift held adds to it.
    fn pick_mode(&self) -> Pick {
        if self.extend_selection {
            return Pick::Extend;
        }

        Pick::Replace
    }

    /// What the plan on screen is made of, to tell what an edit added.
    fn shown_sources(&self) -> Vec<Source> {
        let Some(s) = &self.session else {
            return Vec::new();
        };

        s.display_list()
            .items
            .into_iter()
            .filter_map(|item| item.source)
            .collect()
    }

    /// The points picked for a scale or an area: a preview on top of the plan.
    fn picked(&self) -> Vec<Point> {
        let mut points = self.scale.points();
        points.extend_from_slice(self.area.points());

        points
    }

    /// The measurement to draw: its points, and the length once there are two.
    fn measure_view(&self) -> Option<Measure> {
        let points = self.measure.points();
        if points.is_empty() {
            return None;
        }

        let caption = self.measure.length_m().map(|length_m| {
            let decimal = self.language.text("unit.decimal");
            tools::length_text(length_m, &decimal, &self.language.text("unit.m"))
        });

        Some(Measure {
            points,
            caption: caption.unwrap_or_default(),
        })
    }

    /// Every property from the session, at start (no signals needed yet).
    fn fill(&mut self) {
        let Some(s) = &self.session else {
            return;
        };
        self.loaded = true;
        self.shown = shown_view(s);
        self.title = q(&s.project().title);
        self.plan_title = q(&s.plan_title());
        self.plan_json = q(&plan_json(
            s,
            &s.display_list(),
            &Selection::default(),
            (&[], None, None),
        ));
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
        self.project_json = q(&manage::project_json(s));
        self.scene_id = q(s.scene().map(|sc| sc.id.as_str()).unwrap_or_default());

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
/// selected elements marked; `picked` are the points of a scale in progress, the box the
/// one a drag pulls up.
fn plan_json(
    s: &Session,
    list: &DisplayList,
    selected: &Selection,
    (picked, marquee, measure): (&[Point], Option<(Point, Point)>, Option<&Measure>),
) -> String {
    let camera = s
        .selected_shot()
        .and_then(|id| s.plan()?.shots.iter().find(|shot| shot.id == id))
        .map(|shot| shot.camera.clone());
    let marked = |source: &Source| {
        selected.contains(source)
            || matches!(source, Source::Camera(id) if Some(id) == camera.as_ref())
    };

    plan::shapes_json(list, s.dir(), &marked, picked, marquee, measure)
}

/// What the tool lists of the tool bar offer: the project's characters, the object library
/// and the lamp kinds.
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
        "object": s.library(),
        "camera": camera::presets_json(s.camera_presets()),
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

    /// Forgets the selected elements that are gone (undo, a layer switched off).
    fn drop_stale(mut self: Pin<&mut Self>) {
        let kept = {
            let this = self.as_ref();
            let Some(s) = &this.rust().session else {
                return;
            };

            // A grip shows only with its own way selected.
            let plain = s.display_list();
            let shown = |list: &DisplayList, source: &Source| {
                list.items.iter().any(|i| i.source.as_ref() == Some(source))
            };
            this.rust().selected.clone().keep(|source| {
                let grip = matches!(source, Source::FigureVia { .. } | Source::CameraVia { .. });

                shown(&plain, source) || (grip && shown(&s.display_list_for(Some(source)), source))
            })
        };
        self.as_mut().rust_mut().get_mut().selected = kept;
    }

    /// Pushes what may have changed after a call to the properties (and their signals).
    fn refresh(mut self: Pin<&mut Self>) {
        let shown = self
            .rust()
            .session
            .as_ref()
            .map(shown_view)
            .unwrap_or_default();
        self.as_mut().rust_mut().get_mut().follow_view(shown);
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
            let measure = this.rust().measure_view();
            (
                plan_json(
                    s,
                    &s.display_list_for(this.rust().selected.only()),
                    &this.rust().selected,
                    (&this.rust().picked(), this.rust().marquee, measure.as_ref()),
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
                this.rust().selected.only().map(plan::key),
                this.rust().selected.len(),
            )
        };
        let (
            plan,
            tracks,
            shots,
            script,
            layers,
            locks,
            time,
            shot,
            undo,
            redo,
            dirty,
            selected,
            count,
        ) = values;
        let scale_points = self.rust().scale.points().len();
        let area_points = self.rust().area.points().len();
        let asks_note = self.rust().note != NoteDraft::Idle;
        let measuring = self.rust().measure != MeasureStep::Idle;

        // Restoring a backup or undoing it can change what only the start used to set.
        let header = {
            let this = self.as_ref();
            this.rust().session.as_ref().map(|s| {
                (
                    s.project().title.clone(),
                    s.plan_title(),
                    s.duration_s(),
                    choices_json(s),
                    manage::project_json(s),
                    s.scene().map(|sc| sc.id.clone()).unwrap_or_default(),
                )
            })
        };
        if let Some((title, plan_title, duration, choices, project, scene)) = header {
            self.as_mut().set_project_json(q(&project));
            self.as_mut().set_scene_id(q(&scene));
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
        self.as_mut()
            .set_selected_count(i32::try_from(count).unwrap_or_default());
        self.as_mut().set_can_undo(undo);
        self.as_mut().set_can_redo(redo);
        self.as_mut().set_dirty(dirty);
        self.as_mut().set_has_background(background);
        self.as_mut()
            .set_scale_points(i32::try_from(scale_points).unwrap_or_default());
        self.as_mut()
            .set_area_points(i32::try_from(area_points).unwrap_or_default());
        self.as_mut().set_asks_note(asks_note);
        self.as_mut().set_measuring(measuring);
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

    fn step_mark(mut self: Pin<&mut Self>, direction: i32) {
        let Some(s) = self.as_mut().session() else {
            return;
        };
        let step = match direction < 0 {
            true => Step::Back,
            false => Step::On,
        };
        s.step_mark(step);
        self.refresh();
    }

    fn advance(mut self: Pin<&mut Self>, dt: f64, speed: f64, looping: &QString) -> bool {
        let Some(s) = self.as_mut().session() else {
            return false;
        };
        let looping = match looping.to_string() == LOOP_SHOT {
            true => Looping::Shot,
            false => Looping::Off,
        };
        let playing = s.advance(dt, speed, looping);
        self.refresh();
        playing == Playing::Running
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

    fn add_shot(self: Pin<&mut Self>) -> bool {
        let camera = match self.rust().selected.only() {
            Some(Source::Camera(id)) => Some(id.clone()),
            _ => None,
        };

        self.edit_shots(|s| s.add_shot(camera.as_deref()).map(drop))
    }

    fn copy_shot(self: Pin<&mut Self>, id: &QString) -> bool {
        self.edit_shots(|s| s.copy_shot(&id.to_string()).map(drop))
    }

    fn delete_shot(self: Pin<&mut Self>, id: &QString) -> bool {
        self.edit_shots(|s| s.delete_shot(&id.to_string()))
    }

    /// Runs a change of the shot list; an error goes to the status line.
    fn edit_shots(
        mut self: Pin<&mut Self>,
        change: impl FnOnce(&mut Session) -> Result<(), ServiceError>,
    ) -> bool {
        let language = self.rust().language;
        let Some(s) = self.as_mut().session() else {
            return false;
        };
        if let Err(e) = change(s) {
            self.say(error_text(language, &e));
            return false;
        }
        self.as_mut().changed();
        self.refresh();
        true
    }

    fn set_follow(self: Pin<&mut Self>, camera: &QString, figure: &QString) -> bool {
        let figure = figure.to_string();

        self.commit_edit(PlanEdit::Follow {
            camera: camera.to_string(),
            figure: (!figure.is_empty()).then_some(figure),
        })
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
        let leaves_measure = tool != Tool::Measure && self.rust().measure != MeasureStep::Idle;
        let leaves_note = tool != Tool::Note && self.rust().note != NoteDraft::Idle;
        let leaves_area = tool != Tool::Area && !self.rust().area.is_empty();
        self.as_mut().rust_mut().get_mut().tool = tool;
        if leaves_scale {
            self.as_mut().cancel_scale();
        }
        if leaves_area {
            self.as_mut().cancel_area();
        }
        if leaves_measure {
            self.as_mut().cancel_measure();
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
        let selected = self.rust().selected.clone();
        let how = self.rust().pick_mode();

        // The canvas' key can name a view cone or a path that lies over the thing meant;
        // the session picks what a user can grab, once per drag.
        let from = Point::new(from_x, from_y);
        let to = Point::new(x, y);
        let grip = std::mem::take(&mut self.as_mut().rust_mut().get_mut().grip);
        let Some(s) = self.as_mut().session() else {
            return false;
        };
        let (grip, grabbed) = grip.take(|| s.pick_for(from, selected.only()));

        // Select and Scale move by a distance (the session snaps where an element lands, a
        // background moves freely); the other tools draw between two points, which snap here.
        let (from, to) = match tool {
            Tool::Select | Tool::Scale => (from, to),
            _ => (s.snap(from), s.snap(to)),
        };
        let group = grabbed
            .as_ref()
            .filter(|_| tool == Tool::Select)
            .and_then(|g| tools::group_drag_edit(&selected, g, from, to));
        let mut edit = group
            .or_else(|| tools::drag_edit(&tool, grabbed.as_ref(), from, to))
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
        self.as_mut().rust_mut().get_mut().grip = grip.after(phase);

        // Empty ground with the select tool pulls up a box to take what lies in it.
        if tool == Tool::Select && grabbed.is_none() {
            return self.drag_box((from, to), phase, how);
        }

        if tool == Tool::Select && edit.is_some() {
            self.as_mut().rust_mut().get_mut().selected = selected.grab(grabbed);
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
        let selected = self.rust().selected.clone();
        let how = self.rust().pick_mode();
        let Some(s) = self.as_mut().session() else {
            return false;
        };

        if tool == Tool::Select {
            let picked = s.pick_for(Point::new(x, y), selected.only());
            let found = picked.is_some();
            self.as_mut().rust_mut().get_mut().selected = selected.click(picked, how);
            self.refresh();
            return found;
        }

        let at = tools::lands_at(&tool, Point::new(x, y), |p| s.snap(p));
        if tool == Tool::Scale {
            return self.pick_scale_point(at);
        }
        if tool == Tool::Measure {
            return self.click_measure(at);
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

    fn hover(mut self: Pin<&mut Self>, x: f64, y: f64) -> bool {
        if self.rust().tool != Tool::Measure {
            return false;
        }

        let Some(s) = self.as_mut().session() else {
            return false;
        };
        let at = s.snap(Point::new(x, y));
        let step = self.rust().measure;
        let live = step.hover(at);
        if live == step {
            return false;
        }

        self.as_mut().rust_mut().get_mut().measure = live;
        self.show_plan();
        true
    }

    fn cancel_measure(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().get_mut().measure = MeasureStep::Idle;
        self.refresh();
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
        let Some(Source::Note(id)) = self.rust().selected.only().cloned() else {
            return false;
        };

        self.ask_note(NoteDraft::Edit(id))
    }

    fn remove_selected(mut self: Pin<&mut Self>) -> bool {
        let edit = tools::key_edit(Key::Remove, &self.rust().selected);
        let Some(edit) = edit else {
            return false;
        };

        // The indices of what is left may shift, so the selection goes either way.
        self.as_mut().rust_mut().get_mut().selected = Selection::default();
        self.commit_edit(edit)
    }

    fn duplicate_selected(mut self: Pin<&mut Self>) -> bool {
        let Some(edit) = tools::key_edit(Key::Duplicate, &self.rust().selected) else {
            return false;
        };
        let before = self.rust().shown_sources();

        if !self.as_mut().commit_edit(edit) {
            return false;
        }

        // The copies are what the plan shows anew; they are selected.
        let copies = tools::added(&before, &self.rust().shown_sources());
        self.as_mut().rust_mut().get_mut().selected = copies;
        self.refresh();
        true
    }

    fn straighten_selected(self: Pin<&mut Self>) -> bool {
        let Some(edit) = tools::key_edit(Key::Straighten, &self.rust().selected) else {
            return false;
        };

        self.commit_edit(edit)
    }

    fn turn_selected(self: Pin<&mut Self>, step_deg: f64) -> bool {
        let key = Key::Turn { by_deg: step_deg };
        let Some(edit) = tools::key_edit(key, &self.rust().selected) else {
            return false;
        };

        self.commit_edit(edit)
    }

    fn resize_selected(self: Pin<&mut Self>, width: &QString, depth: &QString) -> bool {
        let selected = self.rust().selected.only();
        let Some(edit) = tools::resize_edit(selected, &width.to_string(), &depth.to_string())
        else {
            return false;
        };

        self.commit_edit(edit)
    }

    fn set_object_height(self: Pin<&mut Self>, height: &QString) -> bool {
        let selected = self.rust().selected.only();
        let Some(edit) = tools::object_height_edit(selected, &height.to_string()) else {
            return false;
        };

        self.commit_edit(edit)
    }

    fn set_camera_aim(self: Pin<&mut Self>, height: &QString, tilt: &QString) -> bool {
        let selected = self.rust().selected.only();
        let Some(edit) = tools::aim_edit(selected, &height.to_string(), &tilt.to_string()) else {
            return false;
        };

        self.commit_edit(edit)
    }

    fn set_camera_field(self: Pin<&mut Self>, field: &QString, value: &QString) -> bool {
        let Some(field) = CameraField::named(&field.to_string()) else {
            return false;
        };
        let Some(s) = self.rust().session.as_ref() else {
            return false;
        };
        let selected = self.rust().selected.only();
        let edit = camera::camera_edit(selected, s.camera_presets(), field, &value.to_string());
        let Some(edit) = edit else {
            return false;
        };

        self.commit_edit(edit)
    }

    fn mark_info(&self) -> QString {
        let Some(s) = self.rust().session.as_ref() else {
            return q("{}");
        };

        q(&mark_json(s, self.rust().selected.only()).to_string())
    }

    fn set_mark_pose(self: Pin<&mut Self>, pose: &QString) -> bool {
        let selected = self.rust().selected.only();
        let Some(edit) = tools::pose_edit(selected, &pose.to_string()) else {
            return false;
        };

        self.commit_edit(edit)
    }

    fn set_mark_arrival(self: Pin<&mut Self>, arrival: &QString) -> bool {
        let selected = self.rust().selected.only();
        let Some(edit) = tools::arrival_edit(selected, &arrival.to_string()) else {
            return false;
        };

        self.commit_edit(edit)
    }

    fn set_mark_departure(self: Pin<&mut Self>, departure: &QString) -> bool {
        let selected = self.rust().selected.only();
        let Some(edit) = tools::departure_edit(selected, &departure.to_string()) else {
            return false;
        };

        self.commit_edit(edit)
    }

    fn set_mark_hold(self: Pin<&mut Self>, seconds: &QString) -> bool {
        let selected = self.rust().selected.only();
        let Some(edit) = tools::hold_edit(selected, &seconds.to_string()) else {
            return false;
        };

        self.commit_edit(edit)
    }

    fn set_mark_pace(self: Pin<&mut Self>, pace: &QString) -> bool {
        let selected = self.rust().selected.only();
        let Some(edit) = tools::pace_edit(selected, &pace.to_string()) else {
            return false;
        };

        self.commit_edit(edit)
    }

    fn save_to_library(mut self: Pin<&mut Self>, name: &QString) -> bool {
        let language = self.rust().language;
        let name = name.to_string();
        let location = {
            let this = self.as_ref();
            let scene = this.rust().session.as_ref().and_then(|s| s.scene());
            scene.map(|scene| scene.location.clone())
        };
        let selected = self.rust().selected.only();
        let Some(edit) = tools::library_edit(selected, location.as_deref(), &name) else {
            return false;
        };

        if !self.as_mut().commit_project(Ok(edit)) {
            return false;
        }

        // The edit cleared the status line; the result goes there.
        let saved = language
            .text("library.saved")
            .replace("{name}", name.trim());
        self.as_mut().say(saved);
        true
    }

    fn import_library(mut self: Pin<&mut Self>, path: &QString) -> bool {
        let language = self.rust().language;
        let path = tools::local_path(&path.to_string());
        let Some(s) = self.as_mut().session() else {
            return false;
        };

        let (message, ok) = match s.import_library(&path) {
            Ok(0) => (language.text("library.importedNone"), true),
            Ok(count) => {
                let text = language.text("library.imported");
                (text.replace("{count}", &count.to_string()), true)
            }
            Err(e) => (error_text(language, &e), false),
        };
        self.as_mut().changed();
        self.as_mut().refresh();
        self.as_mut().say(message);
        ok
    }

    fn export_library(mut self: Pin<&mut Self>, path: &QString) -> bool {
        let language = self.rust().language;
        let path = tools::local_path(&path.to_string());
        let Some(s) = self.as_mut().session() else {
            return false;
        };

        let (message, ok) = match s.export_library(&path) {
            Ok(count) => {
                let text = language.text("library.exported");
                let text = text.replace("{count}", &count.to_string());
                (text.replace("{path}", &path.to_string_lossy()), true)
            }
            Err(e) => (error_text(language, &e), false),
        };
        self.as_mut().say(message);
        ok
    }

    fn set_snapping(mut self: Pin<&mut Self>, on: bool) {
        let snapping = if on { Snapping::On } else { Snapping::Off };
        if let Some(s) = self.as_mut().session() {
            s.set_snapping(snapping);
        }
        self.set_snapping_on(on);
    }

    /// A drag on empty ground: the box on screen while the button is down, the elements wholly
    /// in it selected on release.
    fn drag_box(
        mut self: Pin<&mut Self>,
        corners: (Point, Point),
        phase: DragPhase,
        how: Pick,
    ) -> bool {
        if phase == DragPhase::Moving {
            self.as_mut().rust_mut().get_mut().marquee = Some(corners);
            let json = {
                let this = self.as_ref();
                this.rust().session.as_ref().map(|s| {
                    let selected = &this.rust().selected;
                    let list = s.display_list_for(selected.only());

                    plan_json(
                        s,
                        &list,
                        selected,
                        (&this.rust().picked(), Some(corners), None),
                    )
                })
            };
            if let Some(json) = json {
                self.set_plan_json(q(&json));
            }
            return false;
        }

        let found = {
            let this = self.as_ref();
            this.rust()
                .session
                .as_ref()
                .map(|s| s.pick_in(corners.0, corners.1))
                .unwrap_or_default()
        };
        let any = !found.is_empty();
        let selected = self.rust().selected.clone().boxed(found, how);
        self.as_mut().rust_mut().get_mut().marquee = None;
        self.as_mut().rust_mut().get_mut().selected = selected;
        self.refresh();
        any
    }

    fn cancel_drag(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().get_mut().grip = Grip::Free;
        self.as_mut().rust_mut().get_mut().marquee = None;
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

    /// A click of the measuring tool: the first or the second point, or a new start.
    fn click_measure(mut self: Pin<&mut Self>, at: Point) -> bool {
        let step = self.rust().measure.click(at);
        self.as_mut().rust_mut().get_mut().measure = step;
        self.refresh();
        true
    }

    /// Sets the plan on screen again, nothing else: for what changes with the pointer.
    fn show_plan(self: Pin<&mut Self>) {
        let json = {
            let this = self.as_ref();
            this.rust().session.as_ref().map(|s| {
                let measure = this.rust().measure_view();
                plan_json(
                    s,
                    &s.display_list_for(this.rust().selected.only()),
                    &this.rust().selected,
                    (&this.rust().picked(), this.rust().marquee, measure.as_ref()),
                )
            })
        };

        let Some(json) = json else {
            return;
        };
        self.set_plan_json(q(&json));
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
                let list = s.preview_for(edit, this.rust().selected.only()).ok()?;
                let picked = this.rust().picked();
                Some(plan_json(
                    s,
                    &list,
                    &this.rust().selected,
                    (&picked, None, None),
                ))
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

    fn load_recent(mut self: Pin<&mut Self>, stored: &QString) {
        let mut list = manage::parse_recent(&stored.to_string());
        // A project opened from the command line counts as opened last.
        if let Some(s) = self.rust().session.as_ref() {
            list = manage::push_recent(&list, &s.dir().to_string_lossy());
        }

        self.as_mut()
            .set_recent_json(q(&manage::recent_view(&list)));
        self.set_recent_store(q(&manage::recent_store(&list)));
    }

    fn create_project(self: Pin<&mut Self>, path: &QString, title: &QString) -> bool {
        let dir = tools::local_path(&path.to_string());
        let created = Session::create(&dir, &title.to_string());

        self.switch_to(created)
    }

    fn open_project(self: Pin<&mut Self>, path: &QString) -> bool {
        let dir = tools::local_path(&path.to_string());
        let opened = Session::open(&dir);

        self.switch_to(opened)
    }

    fn select_scene(mut self: Pin<&mut Self>, id: &QString) -> bool {
        let Some(s) = self.as_mut().session() else {
            return false;
        };
        if s.select_scene(&id.to_string()).is_err() {
            return false;
        }

        self.as_mut().rust_mut().get_mut().drop_drafts();
        self.refresh();
        true
    }

    fn clear_status(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().get_mut().notice.say(String::new());
        self.set_status(QString::default());
    }

    fn add_scene(
        self: Pin<&mut Self>,
        number: &QString,
        heading: &QString,
        location: &QString,
    ) -> bool {
        self.commit_project(Ok(ProjectEdit::AddScene {
            number: number.to_string(),
            heading: heading.to_string(),
            location: location.to_string(),
        }))
    }

    fn edit_scene(
        self: Pin<&mut Self>,
        id: &QString,
        number: &QString,
        heading: &QString,
        location: &QString,
    ) -> bool {
        self.commit_project(Ok(ProjectEdit::EditScene {
            id: id.to_string(),
            number: number.to_string(),
            heading: heading.to_string(),
            location: location.to_string(),
        }))
    }

    fn remove_scene(self: Pin<&mut Self>, id: &QString) -> bool {
        self.commit_project(Ok(ProjectEdit::RemoveScene { id: id.to_string() }))
    }

    fn add_location(
        self: Pin<&mut Self>,
        name: &QString,
        width: &QString,
        depth: &QString,
    ) -> bool {
        let edit = sizes(width, depth).map(|(width_m, depth_m)| ProjectEdit::AddLocation {
            name: name.to_string(),
            width_m,
            depth_m,
        });
        self.commit_project(edit)
    }

    fn edit_location(
        self: Pin<&mut Self>,
        id: &QString,
        name: &QString,
        width: &QString,
        depth: &QString,
    ) -> bool {
        let edit = sizes(width, depth).map(|(width_m, depth_m)| ProjectEdit::EditLocation {
            id: id.to_string(),
            name: name.to_string(),
            width_m,
            depth_m,
        });
        self.commit_project(edit)
    }

    fn remove_location(self: Pin<&mut Self>, id: &QString) -> bool {
        self.commit_project(Ok(ProjectEdit::RemoveLocation { id: id.to_string() }))
    }

    fn add_character(
        self: Pin<&mut Self>,
        name: &QString,
        label: &QString,
        color: &QString,
    ) -> bool {
        let edit = color_edit(color).map(|color| ProjectEdit::AddCharacter {
            name: name.to_string(),
            short: label.to_string(),
            color,
        });
        self.commit_project(edit)
    }

    fn edit_character(
        self: Pin<&mut Self>,
        id: &QString,
        name: &QString,
        label: &QString,
        color: &QString,
    ) -> bool {
        let edit = color_edit(color).map(|color| ProjectEdit::EditCharacter {
            id: id.to_string(),
            name: name.to_string(),
            short: label.to_string(),
            color,
        });
        self.commit_project(edit)
    }

    fn remove_character(self: Pin<&mut Self>, id: &QString) -> bool {
        self.commit_project(Ok(ProjectEdit::RemoveCharacter { id: id.to_string() }))
    }

    fn select_plan(mut self: Pin<&mut Self>, id: &QString) -> bool {
        let Some(s) = self.as_mut().session() else {
            return false;
        };
        if s.select_plan(&id.to_string()).is_err() {
            return false;
        }

        self.as_mut().rust_mut().get_mut().drop_drafts();
        self.refresh();
        true
    }

    fn add_plan(mut self: Pin<&mut Self>, name: &QString) -> bool {
        let result = self
            .as_mut()
            .session()
            .map(|s| s.add_plan(&name.to_string()).map(|_| ()));
        self.report(result)
    }

    fn copy_plan(mut self: Pin<&mut Self>, name: &QString) -> bool {
        let result = self
            .as_mut()
            .session()
            .map(|s| s.copy_plan(&name.to_string()).map(|_| ()));
        self.report(result)
    }

    fn rename_plan(mut self: Pin<&mut Self>, id: &QString, name: &QString) -> bool {
        let result = self
            .as_mut()
            .session()
            .map(|s| s.rename_plan(&id.to_string(), &name.to_string()));
        self.report(result)
    }

    fn remove_plan(self: Pin<&mut Self>, id: &QString) -> bool {
        self.drop_plan(id, ShotsGo::Refuse)
    }

    fn remove_plan_with_shots(self: Pin<&mut Self>, id: &QString) -> bool {
        self.drop_plan(id, ShotsGo::Delete)
    }

    fn drop_plan(mut self: Pin<&mut Self>, id: &QString, shots_go: ShotsGo) -> bool {
        let result = self
            .as_mut()
            .session()
            .map(|s| s.remove_plan(&id.to_string(), shots_go));
        self.report(result)
    }

    /// Applies `edit` (or says why its input was refused) to the project; an error goes to
    /// the status line.
    fn commit_project(mut self: Pin<&mut Self>, edit: Result<ProjectEdit, ServiceError>) -> bool {
        let result = match (edit, self.as_mut().session()) {
            (Ok(edit), Some(s)) => s.edit_project(edit),
            (Err(e), _) => Err(e),
            (_, None) => return false,
        };

        self.report(Some(result))
    }

    /// Tells what became of a change (`None`: no project open): an error goes to the status
    /// line, a change makes an old message stale; the view is redrawn either way.
    fn report(mut self: Pin<&mut Self>, result: Option<Result<(), ServiceError>>) -> bool {
        let Some(result) = result else {
            return false;
        };
        let language = self.rust().language;

        let ok = result.is_ok();
        match result {
            Ok(()) => self.as_mut().changed(),
            Err(e) => self.as_mut().say(error_text(language, &e)),
        }
        self.refresh();
        ok
    }

    /// Makes `next` the open project: the one on screen goes to its backup first, so a
    /// switch loses nothing. A refusal leaves the open project as it is.
    fn switch_to(mut self: Pin<&mut Self>, next: Result<Session, ServiceError>) -> bool {
        let language = self.rust().language;
        let session = match next {
            Ok(session) => session,
            Err(e) => {
                self.say(error_text(language, &e));
                return false;
            }
        };

        if let Some(old) = self.as_mut().session() {
            if let Err(e) = old.autosave() {
                self.say(error_text(language, &e));
                return false;
            }
        }

        let dir = session.dir().to_string_lossy().into_owned();
        {
            let this = self.as_mut().rust_mut().get_mut();
            this.session = Some(session);
            this.drop_drafts();
        }
        self.as_mut().set_loaded(true);
        self.as_mut().changed();
        self.as_mut().refresh();
        self.as_mut().show_backup();
        self.remember(&dir);
        true
    }

    /// Puts the folder first in the list of recent projects.
    fn remember(mut self: Pin<&mut Self>, dir: &str) {
        let list = manage::parse_recent(&self.rust().recent_store.to_string());
        let list = manage::push_recent(&list, dir);

        self.as_mut()
            .set_recent_json(q(&manage::recent_view(&list)));
        self.set_recent_store(q(&manage::recent_store(&list)));
    }
}

/// Width and depth typed in the location form.
fn sizes(width: &QString, depth: &QString) -> Result<(f64, f64), ServiceError> {
    let width_m = manage::metres(Field::Width, &width.to_string())?;
    let depth_m = manage::metres(Field::Depth, &depth.to_string())?;

    Ok((width_m, depth_m))
}

/// The colour a form sends by its role name.
fn color_edit(name: &QString) -> Result<FigureColor, ServiceError> {
    let name = name.to_string();

    manage::color_of(&name).ok_or(ServiceError::BadValue {
        field: Field::Color,
        value: name,
    })
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

/// The scene and plan on screen as `scene/plan`, empty parts when there is none.
fn shown_view(s: &Session) -> String {
    let scene = s.scene().map(|sc| sc.id.as_str()).unwrap_or_default();
    let plan = s.plan().map(|p| p.id.as_str()).unwrap_or_default();

    format!("{scene}/{plan}")
}

/// The text of the current set's note `id`.
fn note_text(s: &Session, id: &str) -> Option<String> {
    let location = s.project().location(&s.scene()?.location)?;

    Some(location.set.notes.iter().find(|n| n.id == id)?.text.clone())
}

/// The fields of the selected mark for QML; `{}` when the selection is no mark.
fn mark_json(s: &Session, selected: Option<&Source>) -> serde_json::Value {
    let Some(selected) = selected else {
        return json!({});
    };
    let Some(info) = s.mark_info(selected) else {
        return json!({});
    };
    let kind = match selected {
        Source::Camera(_) => "camera",
        _ => "figure",
    };

    json!({
        "kind": kind,
        "pose": info.pose.map(tools::pose_name).unwrap_or_default(),
        "arrival": info.arrival.map(tools::arrival_name).unwrap_or_default(),
        "departure": info.departure.map(tools::departure_name).unwrap_or_default(),
        "hold": info.hold_s,
        "hasWay": info.has_way,
        "pace": info.pace_m_s,
        "height": info.height_m,
        "tilt": info.tilt_deg,
    })
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
            let follows = c
                .follows
                .as_deref()
                .and_then(|character| s.project().character(character))
                .map(|character| character.name.clone())
                .unwrap_or_default();
            let lens_mm = s.lens_now_mm(id).unwrap_or(c.lens_mm);
            json!({
                "kind": "camera",
                "label": c.label,
                "lens": lens_mm,
                "ownLens": c.lens_mm,
                "sensor": c.sensor.width_mm(),
                "aspect": c.aspect,
                "body": camera::body_id(s.camera_presets(), c.sensor.width_mm()),
                "fov": (horizontal_fov_deg(c.sensor, lens_mm) * 10.0).round() / 10.0,
                "height": c.marks.first().map(|m| m.height_m).unwrap_or(0.0),
                "shots": shots.join(", "),
                "follows": follows,
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
        "figurevia" | "cameravia" => {
            let index: Option<usize> = parts.next().and_then(|i| i.parse().ok());
            json!({ "kind": "bend", "index": index.map(|i| i + 1) })
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
            let Some(o) = set.and_then(|set| set.objects.iter().find(|o| o.id == id)) else {
                return json!({});
            };
            json!({
                "kind": "object",
                "label": o.label,
                "library": o.kind,
                "width": o.width_m,
                "depth": o.depth_m,
                "height": o.height_m,
            })
        }
        "note" => json!({ "kind": "note", "text": note_text(s, id).unwrap_or_default() }),
        _ => json!({}),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example() -> Session {
        let dir =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe");
        Session::open(&dir).unwrap()
    }

    /// The mark fields read the selection: a figure's mark has a pose and a pace, a camera
    /// has neither a pose nor a first way; anything else has no fields.
    #[test]
    fn the_mark_fields_follow_the_selection() {
        let s = example();
        let mark = |index| Source::Mark {
            character: "elisabeth".into(),
            index,
        };

        let second = mark_json(&s, Some(&mark(1)));
        assert_eq!(second["kind"], "figure");
        assert_eq!(second["pose"], "standing");
        assert_eq!(second["arrival"], "walk");
        assert_eq!(second["departure"], "stay");
        assert_eq!(second["hold"], 4.0);
        assert_eq!(second["hasWay"], true);
        assert_eq!(second["pace"], 1.75);

        assert_eq!(mark_json(&s, Some(&mark(0)))["hasWay"], false);

        let camera = mark_json(&s, Some(&Source::Camera("a".into())));
        assert_eq!(camera["kind"], "camera");
        assert_eq!(camera["pose"], "");
        assert_eq!(camera["arrival"], "");
        assert_eq!(camera["departure"], "");
        assert_eq!(camera["hold"], 6.0);
        assert_eq!(camera["height"], 1.6);
        assert_eq!(camera["tilt"], 0.0);
        assert_eq!(second["height"], serde_json::Value::Null);
        assert_eq!(second["tilt"], serde_json::Value::Null);

        assert_eq!(mark_json(&s, Some(&Source::Wall(0))), json!({}));
        assert_eq!(mark_json(&s, None), json!({}));
    }

    /// The object tool lists the library: sizes in metres, built in first.
    #[test]
    fn the_object_list_is_the_library() {
        let s = example();

        let choices: serde_json::Value = serde_json::from_str(&choices_json(&s)).unwrap();
        let objects = choices["object"].as_array().unwrap();

        assert_eq!(objects[0]["kind"], "table");
        assert_eq!(objects[0]["width_m"], 1.6);
        assert_eq!(objects[0]["depth_m"], 0.9);
        assert_eq!(objects[0]["shape"], "rect");
        assert_eq!(objects[0]["custom"], false);
        assert_eq!(objects[0]["height_m"], 0.75);
        assert!(objects
            .iter()
            .any(|o| o["kind"] == "horse" && o["shape"] == "ellipse"));
        assert_eq!(objects.len(), s.library().len());
    }

    /// The size fields show what the selected object measures now.
    #[test]
    fn describe_gives_the_size_of_an_object() {
        let s = example();

        let table = describe(&s, "object:tisch");

        assert_eq!(table["kind"], "object");
        assert_eq!(table["library"], "table");
        assert_eq!(
            (table["width"].as_f64(), table["depth"].as_f64()),
            (Some(6.0), Some(1.2))
        );
        // The example's table has no height: `null`, the field stays empty.
        assert_eq!(table["height"], serde_json::Value::Null);
        assert_eq!(describe(&s, "object:o99"), json!({}));
    }

    /// The camera card starts from what the selected camera has now, and offers the presets.
    #[test]
    fn describe_gives_the_cameras_body_and_frame() {
        let s = example();

        let a = describe(&s, "camera:a");

        assert_eq!(a["body"], "super35");
        assert_eq!(a["sensor"], 24.89);
        assert_eq!(a["aspect"], 2.39);
        assert_eq!(a["ownLens"], 35.0);

        let choices: serde_json::Value = serde_json::from_str(&choices_json(&s)).unwrap();
        assert_eq!(choices["camera"]["bodies"][0]["id"], "super35");
        assert!(!choices["camera"]["lens_sets"]
            .as_array()
            .unwrap()
            .is_empty());
    }

    #[test]
    fn shift_makes_a_pick_extend_the_selection() {
        let mut c = ControllerRust::default();
        assert_eq!(c.pick_mode(), Pick::Replace);

        c.extend_selection = true;

        assert_eq!(c.pick_mode(), Pick::Extend);
    }

    /// Review of PR 23: area, scale and note drafts outlived a change of scene or project;
    /// Enter then made an area from the old plan's points in the new one.
    #[test]
    fn drafts_go_with_the_plan() {
        let mut c = ControllerRust {
            area: AreaDraft::default().click(Point::new(1.0, 1.0)),
            scale: ScaleStep::Idle.click(Point::new(2.0, 2.0)),
            measure: MeasureStep::Idle.click(Point::new(4.0, 4.0)),
            note: NoteDraft::New(Point::new(3.0, 3.0)),
            selected: Selection::of(Some(Source::Wall(0))),
            ..ControllerRust::default()
        };

        c.drop_drafts();

        assert_eq!(c.measure, MeasureStep::Idle);
        assert!(c.area.is_empty());
        assert_eq!(c.scale, ScaleStep::Idle);
        assert_eq!(c.note, NoteDraft::Idle);
        assert!(c.selected.is_empty());
    }

    /// Review of PR 23, round 4: after a start with `--open` the view on screen was not
    /// noted, so the first refresh dropped the selection or draft it was to show.
    #[test]
    fn the_first_refresh_keeps_its_draft() {
        let dir =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe");
        let mut c = ControllerRust {
            session: Some(Session::open(&dir).unwrap()),
            ..ControllerRust::default()
        };
        c.fill();
        c.area = AreaDraft::default().click(Point::new(1.0, 1.0));

        let shown = shown_view(c.session.as_ref().unwrap());
        c.follow_view(shown);

        assert!(!c.area.is_empty());
    }

    /// LH-04: choosing, adding or copying a plan changes the plan on screen; a draft made on
    /// the old plan must not be finished on the new one.
    #[test]
    fn drafts_go_when_another_plan_is_shown() {
        let dir =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe");
        let mut session = Session::open(&dir).unwrap();
        let mut c = ControllerRust {
            shown: shown_view(&session),
            area: AreaDraft::default().click(Point::new(1.0, 1.0)),
            ..ControllerRust::default()
        };
        c.follow_view(shown_view(&session));
        assert!(!c.area.is_empty());

        session.copy_plan("Stand 1").unwrap();
        c.follow_view(shown_view(&session));

        assert!(c.area.is_empty());
        assert_eq!(c.shown, "szene-012/stand-1");
    }

    /// Review of PR 23, round 3: undo, redo or deleting the scene on view changed the scene
    /// without a click; the drafts stayed and Enter drew in the other scene. Here only the
    /// comparison; `refresh` calls it with `shown_view`.
    #[test]
    fn drafts_go_when_the_view_changes() {
        let mut c = ControllerRust {
            shown: "szene-012/a".into(),
            area: AreaDraft::default().click(Point::new(1.0, 1.0)),
            ..ControllerRust::default()
        };

        c.follow_view("szene-012/a".into());
        assert!(!c.area.is_empty());

        c.follow_view("szene-013/a".into());
        assert!(c.area.is_empty());
        assert_eq!(c.shown, "szene-013/a");
    }
}
