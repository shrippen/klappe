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
        /// `[{ path, name, state, scenes, modifiedS, preview }]`: the projects opened last, with
        /// the preview of their first plan as it is made (`state` `loading` until then).
        #[qproperty(QString, recent_json, cxx_name = "recentJson")]
        /// The same list as the app's settings keep it; QML stores it when it changes.
        #[qproperty(QString, recent_store, cxx_name = "recentStore")]
        #[qproperty(QString, screenshot_path, cxx_name = "screenshotPath")]
        #[qproperty(i32, screenshot_after, cxx_name = "screenshotAfter")]
        type Controller = super::ControllerRust;

        /// Another project, scene or plan is shown (chosen, opened, or by undo, redo or removing
        /// the one on view; also a project with the same title and scene id): play stops,
        /// nobody pressed it there.
        #[qsignal]
        fn switched(self: Pin<&mut Self>);

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

        /// Takes a project out of the list of recent ones.
        #[qinvokable]
        #[cxx_name = "forgetRecent"]
        fn forget_recent(self: Pin<&mut Self>, path: &QString);

        /// Makes an empty project in the folder `path` (a path or a `file://` URL) and opens
        /// it; an error goes to `status`.
        #[qinvokable]
        #[cxx_name = "createProject"]
        fn create_project(self: Pin<&mut Self>, path: &QString, title: &QString) -> bool;

        /// Opens the project folder `path`; the open one is backed up first.
        #[qinvokable]
        #[cxx_name = "openProject"]
        fn open_project(self: Pin<&mut Self>, path: &QString) -> bool;

        /// Opens the user's copy of the example in the language of the window, made first if it
        /// is not there; an error goes to `status`.
        #[qinvokable]
        #[cxx_name = "openExample"]
        fn open_example(self: Pin<&mut Self>) -> bool;

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

        /// `color` is a role name (`purple`, `blue` …); `group` is the head count of a group
        /// of extras, 0 for a single person.
        #[qinvokable]
        #[cxx_name = "addCharacter"]
        fn add_character(
            self: Pin<&mut Self>,
            name: &QString,
            label: &QString,
            color: &QString,
            group: u32,
        ) -> bool;

        #[qinvokable]
        #[cxx_name = "editCharacter"]
        fn edit_character(
            self: Pin<&mut Self>,
            id: &QString,
            name: &QString,
            label: &QString,
            color: &QString,
            group: u32,
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

    // The start page's cards are read on a thread of their own and handed back through it.
    impl cxx_qt::Threading for Controller {}
}

use cxx_qt_lib::QString;
use klappe_services::{Point, Session};

use crate::args::AppArgs;
use crate::catalog::Language;
use crate::notice::{error_text, Status};
use crate::tools::{
    AreaDraft, DrawStyle, Grip, MeasureStep, NoteDraft, Preview, ScaleStep, Selection, Tool,
};

mod draw;
mod export;
mod hints;
mod history;
mod layers;
mod library;
mod marks;
mod measure;
mod notes;
mod plans;
mod playback;
mod project;
mod script;
mod selection;
mod shots;
mod view;

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

#[cfg(test)]
mod tests {
    use super::*;

    pub(super) fn example() -> Session {
        let dir =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe");
        Session::open(&dir).unwrap()
    }
}
