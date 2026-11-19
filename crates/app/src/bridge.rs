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
        include!("cxx-qt-lib/qimage.h");
        type QImage = cxx_qt_lib::QImage;
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
        /// Counts refreshes: the camera's picture is made again when it changes.
        #[qproperty(i32, picture_revision, cxx_name = "pictureRevision")]
        /// Counts what turns, shifts or zooms the 3D view: the 3D picture is made again.
        #[qproperty(i32, free_revision, cxx_name = "freeRevision")]
        /// Whether the plan area shows the free 3D view (J) instead of the plan.
        #[qproperty(bool, view_free, cxx_name = "viewFree")]
        #[qproperty(QString, tracks_json, cxx_name = "tracksJson")]
        #[qproperty(QString, shots_json, cxx_name = "shotsJson")]
        /// The shot list in shooting order: setups grouped (LH-72); `shotsJson` is the cut order.
        #[qproperty(QString, shots_shooting_json, cxx_name = "shotsShootingJson")]
        #[qproperty(QString, script_json, cxx_name = "scriptJson")]
        /// The whole screenplay as Fountain text, for the editor of the screenplay view; change
        /// it with `editScript`.
        #[qproperty(QString, script_text, cxx_name = "scriptText")]
        /// `[{ kind: "section" | "scene", depth, line, number, text }]`: the sections and scenes
        /// of the screenplay in order; `line` counts from 0.
        #[qproperty(QString, outline_json, cxx_name = "outlineJson")]
        /// `{ set, figures, cameras, lights, notes, drawings }`: true when shown.
        #[qproperty(QString, layers_json, cxx_name = "layersJson")]
        /// `{ set, figures, cameras, lights, notes, drawings }`: true when locked.
        #[qproperty(QString, locks_json, cxx_name = "locksJson")]
        /// `{ set, figures, … }`: how strongly each layer is drawn, as `choicesJson.fade` names it.
        #[qproperty(QString, fades_json, cxx_name = "fadesJson")]
        #[qproperty(f64, time)]
        #[qproperty(f64, duration)]
        #[qproperty(QString, selected_shot, cxx_name = "selectedShot")]
        #[qproperty(bool, can_undo, cxx_name = "canUndo")]
        #[qproperty(bool, can_redo, cxx_name = "canRedo")]
        /// What undo and redo do, as the name of the step (`AddObject`); empty for none or an
        /// unnamed one.
        #[qproperty(QString, undo_step, cxx_name = "undoStep")]
        #[qproperty(QString, redo_step, cxx_name = "redoStep")]
        #[qproperty(bool, dirty)]
        /// Key of the selected plan element (`camera:a` …), empty for none.
        #[qproperty(QString, selected_key, cxx_name = "selectedKey")]
        /// How many plan elements are selected.
        #[qproperty(i32, selected_count, cxx_name = "selectedCount")]
        /// Shift is held: a click or a box adds to the selection instead of replacing it.
        #[qproperty(bool, extend_selection, cxx_name = "extendSelection")]
        /// Ctrl is held: a drag on empty ground draws a lasso instead of a box.
        #[qproperty(bool, lasso_selection, cxx_name = "lassoSelection")]
        /// Whether points snap to the grid; change it with `setSnapping`.
        #[qproperty(bool, snapping_on, cxx_name = "snappingOn")]
        /// The grid spacing, a name of `choicesJson.grid`; change it with `setGrid`.
        #[qproperty(QString, grid_name, cxx_name = "gridName")]
        /// `{ figure: [{ id, name }], object: [{ kind, label, width_m, depth_m, shape, custom }],
        /// light: [kind], color: [name], dash: [name], fill: [name], rig: [name], movement: [name],
        /// grid: [{ id, step_m }], template: [{ id, label, figures, cameras, custom }] }`: what the
        /// tool lists offer. Objects and templates: built in first, then the project's own.
        #[qproperty(QString, choices_json, cxx_name = "choicesJson")]
        /// Whether the set has a background image.
        #[qproperty(bool, has_background, cxx_name = "hasBackground")]
        #[qproperty(i32, missing_heights, cxx_name = "missingHeights")]
        /// Points picked so far by the scale tool: 0, 1 or 2 (then the length is asked).
        #[qproperty(i32, scale_points, cxx_name = "scalePoints")]
        /// Whether the measuring tool has a point set (`cancelMeasure` drops it).
        #[qproperty(bool, measuring)]
        /// Points picked so far by the area tool: 0 for none, else the area is open (Enter or a
        /// click on the first point closes it, `cancelArea` drops it).
        #[qproperty(i32, area_points, cxx_name = "areaPoints")]
        /// Whether the text of a note is being asked (`commitNote` or `cancelNote` answers).
        #[qproperty(bool, asks_note, cxx_name = "asksNote")]
        /// What the next click on the plan does for the selection: `nextMark`, `markBefore`,
        /// `axisTo` (`arm`), empty for nothing; `cancelArm` ends it.
        #[qproperty(QString, armed)]
        /// The text the question starts with: the note's own when it is edited.
        #[qproperty(QString, note_text, cxx_name = "noteText")]
        /// Last message for the status line, already translated.
        #[qproperty(QString, status)]
        /// Where the example's old copy went, already translated; empty for none. Kept until
        /// the user closes it or another project opens (`AsideNotice`).
        #[qproperty(QString, example_aside, cxx_name = "exampleAside")]
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

        /// What the selected shot's camera sees, `width_px` wide, coloured with `palette_json`
        /// (`{ name: [r, g, b] }`); a null image without a camera or marks.
        #[qinvokable]
        #[cxx_name = "cameraImage"]
        fn camera_image(self: Pin<&mut Self>, width_px: i32, palette_json: &QString) -> QImage;

        /// What camera `id` sees at the playhead, `width_px` wide, for the small picture beside
        /// its body in the plan (LH-113); a null image for a camera without marks.
        #[qinvokable]
        #[cxx_name = "cameraImageOf"]
        fn camera_image_of(
            self: &Self,
            id: &QString,
            width_px: i32,
            palette_json: &QString,
        ) -> QImage;

        /// The cameras whose body shows on the plan and where it stands at the playhead, as a
        /// JSON array of `{id, x_m, y_m}` (LH-113).
        #[qinvokable]
        #[cxx_name = "cameraSpots"]
        fn camera_spots(self: &Self) -> QString;

        /// `{ width, height, labels: [{ text, x, y }] }` of the last image; empty if null.
        #[qinvokable]
        #[cxx_name = "cameraFrame"]
        fn camera_frame(self: &Self) -> QString;

        /// Switches between plan and 3D (key J).
        #[qinvokable]
        #[cxx_name = "toggleView"]
        fn toggle_view(self: Pin<&mut Self>);

        /// The size of the 3D view in device pixels; the calls below take pointers in them.
        #[qinvokable]
        #[cxx_name = "freeSize"]
        fn free_size(self: Pin<&mut Self>, width_px: f64, height_px: f64);

        /// The 3D view at `scale` times its size (less than 1 while a drag runs), coloured as
        /// `cameraImage`; a null image where there is nothing to see.
        #[qinvokable]
        #[cxx_name = "freeImage"]
        fn free_image(self: Pin<&mut Self>, scale: f64, palette_json: &QString) -> QImage;

        /// `{ width, height, labels: [{ text, x, y }] }` of the last 3D image; empty if null.
        #[qinvokable]
        #[cxx_name = "freeFrame"]
        fn free_frame(self: &Self) -> QString;

        /// Runs `"cut"` or `"camera"` in 3D (other words only ask); returns what is on.
        #[qinvokable]
        #[cxx_name = "freeLook"]
        fn free_look(self: Pin<&mut Self>, command: &QString) -> QString;

        /// Turns the 3D view by a drag of `dx`, `dy` pixels.
        #[qinvokable]
        #[cxx_name = "turnView"]
        fn turn_view(self: Pin<&mut Self>, dx: f64, dy: f64);

        /// Shifts the 3D view by a drag of `dx`, `dy` pixels.
        #[qinvokable]
        #[cxx_name = "panView"]
        fn pan_view(self: Pin<&mut Self>, dx: f64, dy: f64);

        /// Wheel `steps` (positive comes closer) towards the floor under the pointer.
        #[qinvokable]
        #[cxx_name = "zoomView"]
        fn zoom_view(self: Pin<&mut Self>, steps: f64, x: f64, y: f64);

        /// A key's step of the 3D view: `turnLeft`, `panUp`, `zoomIn` … (`ViewStep`).
        #[qinvokable]
        #[cxx_name = "stepView"]
        fn step_view(self: Pin<&mut Self>, step: &QString);

        /// A click in 3D: the select tool takes what is seen there, the other tools get the
        /// plan point of the floor there. False where nothing happened.
        #[qinvokable]
        #[cxx_name = "freePlace"]
        fn free_place(self: Pin<&mut Self>, x: f64, y: f64) -> bool;

        /// A drag in 3D from one pointer position to another, as `drag` on the floor points.
        #[qinvokable]
        #[cxx_name = "freeDrag"]
        fn free_drag(
            self: Pin<&mut Self>,
            from_x: f64,
            from_y: f64,
            x: f64,
            y: f64,
            done: bool,
        ) -> bool;

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

        /// Shows what a script line's chip is linked to (DB-63): `kind` `mark` selects mark
        /// `index` of the character `target` and goes to its arrival, `shot` selects the shot
        /// `target` and no mark. False for a link the plan on view does not have.
        #[qinvokable]
        #[cxx_name = "showLink"]
        fn show_link(self: Pin<&mut Self>, kind: &QString, target: &QString, index: i32) -> bool;

        /// The line of the scene's script the selected mark (else the selected shot) is linked
        /// to, counted as `scriptJson` lists them; -1 when no line names it (DB-63).
        #[qinvokable]
        #[cxx_name = "scriptLine"]
        fn script_line(self: &Self) -> i32;

        /// Sets one field (`lens`, `size` …) of a shot; false if the value is refused.
        #[qinvokable]
        #[cxx_name = "editShot"]
        fn edit_shot(self: Pin<&mut Self>, id: &QString, field: &QString, value: &QString) -> bool;

        /// Sets short name and description of shot `id` as one change (LH-110); false if refused.
        #[qinvokable]
        #[cxx_name = "nameShot"]
        fn name_shot(
            self: Pin<&mut Self>,
            id: &QString,
            short_name: &QString,
            description: &QString,
        ) -> bool;

        /// Gives shot `id` the storyboard picture `path` (a path or `file://` URL), or takes it
        /// away with an empty path (LH-74); false if refused.
        #[qinvokable]
        #[cxx_name = "setShotImage"]
        fn set_shot_image(self: Pin<&mut Self>, id: &QString, path: &QString) -> bool;

        /// Takes what the camera of shot `id` sees, at the playhead or else the shot's start, as
        /// its storyboard picture (LH-75); false if refused.
        #[qinvokable]
        #[cxx_name = "shotStill"]
        fn shot_still(self: Pin<&mut Self>, id: &QString) -> bool;

        /// The `file://` URL of `file`, a file of the project folder (a shot's picture).
        #[qinvokable]
        #[cxx_name = "projectFileUrl"]
        fn project_file_url(self: &Self, file: &QString) -> QString;

        /// Chooses how the project numbers its shots (LH-76), by the name `Overview` lists.
        #[qinvokable]
        #[cxx_name = "setNumbering"]
        fn set_numbering(self: Pin<&mut Self>, name: &QString) -> bool;

        /// The setups shot `id` can be assigned to without itself (LH-77), as a JSON array of
        /// `{setup, first_id, first_number, camera_label, lens_mm}`.
        #[qinvokable]
        #[cxx_name = "setupChoices"]
        fn setup_choices(self: &Self, id: &QString) -> QString;

        /// Adds a shot at the playhead for the camera selected in the plan, else the selected
        /// shot's or the first one, and selects it; false if refused.
        #[qinvokable]
        #[cxx_name = "addShot"]
        fn add_shot(self: Pin<&mut Self>) -> bool;

        /// Copies a shot under a new number and selects the copy; false if refused.
        #[qinvokable]
        #[cxx_name = "copyShot"]
        fn copy_shot(self: Pin<&mut Self>, id: &QString) -> bool;

        /// Copies a shot as its next version (12.3A) and selects it; false if refused.
        #[qinvokable]
        #[cxx_name = "versionShot"]
        fn version_shot(self: Pin<&mut Self>, id: &QString) -> bool;

        /// Renumbers the shots in cut order, keeping version letters; false if refused.
        #[qinvokable]
        #[cxx_name = "renumberShots"]
        fn renumber_shots(self: Pin<&mut Self>) -> bool;

        /// Deletes a shot; false if refused.
        #[qinvokable]
        #[cxx_name = "deleteShot"]
        fn delete_shot(self: Pin<&mut Self>, id: &QString) -> bool;

        /// Couples the camera to a figure (character id), or lets it go with an empty
        /// `figure`; false if refused.
        #[qinvokable]
        #[cxx_name = "setFollow"]
        fn set_follow(self: Pin<&mut Self>, camera: &QString, figure: &QString) -> bool;

        /// Sets the characters (ids) the 180° axis runs between, or lets the plan take the
        /// first two figures with an empty `a`; false if refused.
        #[qinvokable]
        #[cxx_name = "setAxis"]
        fn set_axis(self: Pin<&mut Self>, a: &QString, b: &QString) -> bool;

        #[qinvokable]
        fn undo(self: Pin<&mut Self>);

        #[qinvokable]
        fn redo(self: Pin<&mut Self>);

        #[qinvokable]
        fn save(self: Pin<&mut Self>) -> bool;

        /// Replaces the screenplay with `text` (the editor's typing, debounced); unsaved until
        /// `save`. The scene list and the script panel follow.
        #[qinvokable]
        #[cxx_name = "editScript"]
        fn edit_script(self: Pin<&mut Self>, text: &QString);

        /// Replaces the screenplay with the Fountain file `path` (a path or a `file://` URL);
        /// unsaved until `save`. The result goes to `status`; false when the file is refused.
        #[qinvokable]
        #[cxx_name = "importScript"]
        fn import_script(self: Pin<&mut Self>, path: &QString) -> bool;

        /// Writes unsaved changes to the backup; QML calls it on a timer. An error goes to the
        /// status line.
        #[qinvokable]
        fn autosave(self: Pin<&mut Self>);

        /// Answers the open question about the register entry `id` (`projectJson.pending`): the
        /// answer at position `index` of its `answers` (the scenes of the book, then "not in
        /// the book" last). False when the question is gone or the answer is refused.
        #[qinvokable]
        #[cxx_name = "resolveScene"]
        fn resolve_scene(self: Pin<&mut Self>, id: &QString, index: i32) -> bool;

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

        /// `layer` as for `setLayer`: takes it one step paler in the plan and in 3D (LH-15),
        /// after the faintest step back to full; `fadesJson` says where it is.
        #[qinvokable]
        #[cxx_name = "cycleFade"]
        fn cycle_fade(self: Pin<&mut Self>, layer: &QString);

        /// Saves what the plan shows (layers, cameras, figures) as a view called `name`
        /// (LH-119); false when the name is refused.
        #[qinvokable]
        #[cxx_name = "saveView"]
        fn save_view(self: Pin<&mut Self>, name: &QString) -> bool;

        /// Shows what the saved view `id` names.
        #[qinvokable]
        #[cxx_name = "applyView"]
        fn apply_view(self: Pin<&mut Self>, id: &QString) -> bool;

        /// Deletes the saved view `id`.
        #[qinvokable]
        #[cxx_name = "removeView"]
        fn remove_view(self: Pin<&mut Self>, id: &QString) -> bool;

        /// A drag with the sketch pen (LH-120), in metres: every call adds the pointer's place
        /// to the stroke, `done` ends it and commits it as one undo step.
        #[qinvokable]
        #[cxx_name = "sketchDrag"]
        fn sketch_drag(
            self: Pin<&mut Self>,
            from_x: f64,
            from_y: f64,
            x: f64,
            y: f64,
            done: bool,
        ) -> bool;

        /// Takes every sketch off the plan (LH-120).
        #[qinvokable]
        #[cxx_name = "clearSketches"]
        fn clear_sketches(self: Pin<&mut Self>) -> bool;

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

        /// A right click at (`x`, `y`) with any tool: selects what lies there as a click of the
        /// select tool does, but a member of the selection keeps it and empty ground clears it.
        /// False for empty ground.
        #[qinvokable]
        #[cxx_name = "selectAt"]
        fn select_at(self: Pin<&mut Self>, x: f64, y: f64) -> bool;

        /// Takes the image file at `path` (a path or a `file://` URL) as the set's background
        /// and points to the scale tool in the status line.
        #[qinvokable]
        #[cxx_name = "setBackground"]
        fn set_background(self: Pin<&mut Self>, path: &QString) -> bool;

        /// Places the image file at `path` (a path or a `file://` URL), centred at (`x`, `y`),
        /// on the notes layer (LH-18).
        #[qinvokable]
        #[cxx_name = "addImage"]
        fn add_image(self: Pin<&mut Self>, path: &QString, x: f64, y: f64) -> bool;

        /// Gives the selected image the opacity typed in percent (10 to 100) (LH-18).
        #[qinvokable]
        #[cxx_name = "setImageOpacity"]
        fn set_image_opacity(self: Pin<&mut Self>, percent: &QString) -> bool;

        /// Swaps the file of the selected image for the image file at `path` (a path or a
        /// `file://` URL); place, width, turn and opacity stay (LH-18).
        #[qinvokable]
        #[cxx_name = "replaceImage"]
        fn replace_image(self: Pin<&mut Self>, path: &QString) -> bool;

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

        /// Selects nothing: Escape, a change of mode.
        #[qinvokable]
        #[cxx_name = "clearSelection"]
        fn clear_selection(self: Pin<&mut Self>);

        /// Removes the selected elements.
        #[qinvokable]
        #[cxx_name = "removeSelected"]
        fn remove_selected(self: Pin<&mut Self>) -> bool;

        /// Copies the selected elements, set apart from the originals, and selects the copies.
        #[qinvokable]
        #[cxx_name = "duplicateSelected"]
        fn duplicate_selected(self: Pin<&mut Self>) -> bool;

        /// Arms the next click on the plan (`nextMark`, `markBefore`, `axisTo`) for the selected
        /// figure or camera, the hint line shows what it does; false when `canArm` is not.
        #[qinvokable]
        fn arm(self: Pin<&mut Self>, what: &QString) -> bool;

        /// Whether the selection has what `arm` needs (a figure or camera; the axis a figure).
        #[qinvokable]
        #[cxx_name = "canArm"]
        fn can_arm(self: &Self, what: &QString) -> bool;

        /// Drops what `arm` armed.
        #[qinvokable]
        #[cxx_name = "cancelArm"]
        fn cancel_arm(self: Pin<&mut Self>);

        /// Moves the selected way, when a grip of it is selected, by one grid step to the right
        /// and down (each -1, 0 or 1; LH-107). False for another selection.
        #[qinvokable]
        #[cxx_name = "nudgeWay"]
        fn nudge_way(self: Pin<&mut Self>, right: i32, down: i32) -> bool;

        /// The shape of the selected way, `smooth` or `straight`; empty when the selection has
        /// no way (LH-108).
        #[qinvokable]
        #[cxx_name = "wayCurve"]
        fn way_curve(self: &Self) -> QString;

        /// Makes the selected way `smooth` or `straight` (LH-108); false when it has none.
        #[qinvokable]
        #[cxx_name = "setWayCurve"]
        fn set_way_curve(self: Pin<&mut Self>, name: &QString) -> bool;

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

        /// Gives the selected door or window a new width in metres (LH-10). False when none is
        /// selected or the text is no length; a refusal by the session goes to `status`.
        #[qinvokable]
        #[cxx_name = "setOpeningWidth"]
        fn set_opening_width(self: Pin<&mut Self>, width: &QString) -> bool;

        /// Sets `field` of the selection: `wall`, `sill`, `head`, `base`; a door's `side`, `hinge`.
        #[qinvokable]
        #[cxx_name = "setSelectedField"]
        fn set_selected_field(self: Pin<&mut Self>, field: &QString, text: &QString) -> bool;

        /// Sets the height in metres of the selected object (LH-17); an empty text takes it
        /// off. False when no object is selected or the text is no length; a refusal by the
        /// session goes to `status`.
        #[qinvokable]
        #[cxx_name = "setObjectHeight"]
        fn set_object_height(self: Pin<&mut Self>, height: &QString) -> bool;

        /// Changes the selection (LH-114 to LH-116, LH-118): `what` is `kind` (with the library
        /// kind in `arg`), `mirror`, `resetSize` for an object, `unbind` for a bound note.
        /// False when it does not fit the selection; a refusal by the session goes to `status`.
        #[qinvokable]
        #[cxx_name = "changeSelected"]
        fn change_selected(self: Pin<&mut Self>, what: &QString, arg: &QString) -> bool;

        /// Asks for the text of a note bound to the selected object, figure or mark (LH-118);
        /// `commitNote` sets it. False when the selection cannot be labelled or the notes layer
        /// is locked.
        #[qinvokable]
        #[cxx_name = "labelSelected"]
        fn label_selected(self: Pin<&mut Self>) -> bool;

        /// A typed measure (LH-95): the length of the wall being dragged, else the selection's height.
        #[qinvokable]
        #[cxx_name = "typeMeasure"]
        fn type_measure(self: Pin<&mut Self>, text: &QString) -> bool;

        /// Selects the next object without a height (LH-93), in 3D the view follows; false if none.
        #[qinvokable]
        #[cxx_name = "nextMissingHeight"]
        fn next_missing_height(self: Pin<&mut Self>) -> bool;

        /// Gives the selected drawing a colour, line and fill (LH-19), named as `choicesJson`
        /// lists them. False when no drawing is selected or a name is unknown.
        #[qinvokable]
        #[cxx_name = "setDrawingStyle"]
        fn set_drawing_style(
            self: Pin<&mut Self>,
            color: &QString,
            dash: &QString,
            fill: &QString,
        ) -> bool;

        /// Sets one field of the selected camera (LH-32): `field` is `body` (a body id of
        /// `choicesJson`), `sensor` (mm), `lens` (mm) or `aspect` (`2,39` or `16:9`). False
        /// when no camera is selected or the value is not usable; a refusal by the session
        /// goes to `status`.
        #[qinvokable]
        #[cxx_name = "setCameraField"]
        fn set_camera_field(self: Pin<&mut Self>, field: &QString, value: &QString) -> bool;
        /// Sets one field of the selected lamp (LH-50, LH-51): `field` is `kind` (a kind's
        /// file name), `preset` (a lamp id of `choicesJson.light`), `facing`, `beam` (degrees),
        /// `power` (W) or `temp` (K). False when no lamp is selected or the value is not
        /// usable; a refusal by the session goes to `status`.
        #[qinvokable]
        #[cxx_name = "setLightField"]
        fn set_light_field(self: Pin<&mut Self>, field: &QString, value: &QString) -> bool;
        /// The fields of the selected mark (a figure's, or a camera's) as JSON: `kind`
        /// (`figure` or `camera`), `pose`, `arrival` (`walk`, `appear`) and `departure` (`stay`,
        /// `leave`), each empty for a camera, `hold` in seconds, `hasWay`,
        /// `pace` in m/s (null without time on the way), `height` in m and `tilt` in degrees
        /// (null for a figure), `rig` (empty for a figure) and `lens` in mm (null for a figure
        /// and a camera mark that states none). `{}` for any other selection.
        #[qinvokable]
        #[cxx_name = "markInfo"]
        fn mark_info(self: &Self) -> QString;

        /// What the context menu offers for the selection, as JSON: `layer` (name, empty for
        /// none or several), and `remove`, `duplicate`, `turn`, `straighten` (the command
        /// would change something), `note` and `shot` (one note or one camera is selected),
        /// `nextMark`, `markBefore` and `axisTo` (what `arm` can arm for the selection) and
        /// `way` (`smooth` or `straight` for a selected way, else empty).
        #[qinvokable]
        #[cxx_name = "menuInfo"]
        fn menu_info(self: &Self) -> QString;

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

        /// Sets the rig (`tripod`, `dolly`, `track`, `crane`, `handheld`, `gimbal`) of the
        /// selected camera mark (LH-33). False for another selection or name; a refusal by the
        /// session (a tripod that would have to move) goes to `status`.
        #[qinvokable]
        #[cxx_name = "setMarkRig"]
        fn set_mark_rig(self: Pin<&mut Self>, rig: &QString) -> bool;

        /// Sets the lens in mm (comma or point as decimal separator) at the selected camera
        /// mark; an empty text states none (LH-33). False for another selection or a text that
        /// is no number; a refusal by the session (limits) goes to `status`.
        #[qinvokable]
        #[cxx_name = "setMarkLens"]
        fn set_mark_lens(self: Pin<&mut Self>, lens: &QString) -> bool;

        /// Lets the figure with id `figure` (empty: none) carry the selected object from its mark
        /// on (LH-24). False for another selection or a figure the plan does not have.
        #[qinvokable]
        #[cxx_name = "setMarkCarrier"]
        fn set_mark_carrier(self: Pin<&mut Self>, figure: &QString) -> bool;

        /// Times the way to the selected figure mark for `walk`, `run` or a typed speed in m/s.
        /// False for another selection or a refused text; the first mark is refused by the session.
        #[qinvokable]
        #[cxx_name = "setMarkPace"]
        fn set_mark_pace(self: Pin<&mut Self>, pace: &QString) -> bool;

        /// Files the selected object, with its size and shape, in the project's library under
        /// `name`. False for no object or a blank name.
        #[qinvokable]
        #[cxx_name = "saveToLibrary"]
        fn save_to_library(self: Pin<&mut Self>, name: &QString) -> bool;

        /// Renames and regroups the project's own library entry `kind`. False for a blank
        /// name or an entry that is not the project's.
        #[qinvokable]
        #[cxx_name = "editLibraryItem"]
        fn edit_library_item(
            self: Pin<&mut Self>,
            kind: &QString,
            name: &QString,
            group: &QString,
        ) -> bool;

        /// Deletes the project's own library entry `kind`; placed objects stay.
        #[qinvokable]
        #[cxx_name = "removeLibraryItem"]
        fn remove_library_item(self: Pin<&mut Self>, kind: &QString) -> bool;

        /// Takes the objects (`kind` "objects") or templates ("templates") of the file `path` (a
        /// path or a `file://` URL) into the project; the result goes to `status`.
        #[qinvokable]
        #[cxx_name = "importLibrary"]
        fn import_library(self: Pin<&mut Self>, path: &QString, kind: &QString) -> bool;

        /// Writes the project's own objects or templates (`kind` as for `importLibrary`) to the
        /// file `path`; the result goes to `status`.
        #[qinvokable]
        #[cxx_name = "exportLibrary"]
        fn export_library(self: Pin<&mut Self>, path: &QString, kind: &QString) -> bool;

        #[qinvokable]
        #[cxx_name = "setSnapping"]
        fn set_snapping(self: Pin<&mut Self>, on: bool);

        /// The time a timeline drag of `grab` (`kind:index:id`, see `retime.rs`) ends at, snapped
        /// (`snap`: `tenths` or `free`) and kept between the neighbours.
        #[qinvokable]
        #[cxx_name = "dragTime"]
        fn drag_time(self: &Self, grab: &QString, seconds: f64, snap: &QString) -> f64;

        /// Ends such a drag: moves the mark or shot edge, one undo step; false if refused.
        #[qinvokable]
        #[cxx_name = "dropTime"]
        fn drop_time(self: Pin<&mut Self>, grab: &QString, seconds: f64, snap: &QString) -> bool;

        /// Moves what the selection holds on the timeline by `delta_s` (`what`: `mark` of the
        /// selected figure mark, `shot-from` or `shot-to` of the selected shot), one undo
        /// step; false if nothing of that kind is selected or the move is refused.
        #[qinvokable]
        #[cxx_name = "nudgeTime"]
        fn nudge_time(self: Pin<&mut Self>, what: &QString, delta_s: f64) -> bool;

        #[qinvokable]
        #[cxx_name = "setGrid"]
        fn set_grid(self: Pin<&mut Self>, name: &QString);

        /// A drag was aborted: shows the committed plan again.
        #[qinvokable]
        #[cxx_name = "cancelDrag"]
        fn cancel_drag(self: Pin<&mut Self>);

        /// Writes an export to `path` (a `file://` URL); the result goes to `status`. `kind` is
        /// `pdf`, `png`, `svg`, `csv`, `view3d` or `camera` (PNG of that view); `order` orders the
        /// shot list of PDF and CSV; `paper` (`a4-portrait` …) is the sheet of PDF, PNG, SVG, frames.
        #[qinvokable]
        #[cxx_name = "exportTo"]
        fn export_to(
            self: Pin<&mut Self>,
            kind: &QString,
            order: &QString,
            paper: &QString,
            path: &QString,
        ) -> bool;

        /// The file URL an export of `kind` is offered under: next to the project
        /// folder, named by the plan; empty without a project.
        #[qinvokable]
        #[cxx_name = "exportFile"]
        fn export_file(self: Pin<&mut Self>, kind: &QString) -> QString;

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

        /// Gives the recent project in `path` a new title; an error goes to `status`.
        #[qinvokable]
        #[cxx_name = "renameRecent"]
        fn rename_recent(self: Pin<&mut Self>, path: &QString, title: &QString) -> bool;

        /// Packs the recent project in `path` into a ZIP beside it, trashes the folder and
        /// takes it off the list; an error goes to `status`.
        #[qinvokable]
        #[cxx_name = "archiveRecent"]
        fn archive_recent(self: Pin<&mut Self>, path: &QString) -> bool;

        /// Moves the recent project in `path` to the trash and takes it off the list; an
        /// error goes to `status`.
        #[qinvokable]
        #[cxx_name = "trashRecent"]
        fn trash_recent(self: Pin<&mut Self>, path: &QString) -> bool;

        /// Makes an empty project in the folder `path` (a path or a `file://` URL) and opens
        /// it; an error goes to `status`.
        #[qinvokable]
        #[cxx_name = "createProject"]
        fn create_project(self: Pin<&mut Self>, path: &QString, title: &QString) -> bool;

        /// Opens the project folder `path`; the open one is backed up first.
        #[qinvokable]
        #[cxx_name = "openProject"]
        fn open_project(self: Pin<&mut Self>, path: &QString) -> bool;

        /// Closes the project (unsaved changes go to the backup); false if that fails.
        #[qinvokable]
        #[cxx_name = "closeProject"]
        fn close_project(self: Pin<&mut Self>) -> bool;

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

        /// Where a scene (`kind` `scene`: time of day, plans, shots, cast) or a character
        /// (`character`: the scenes it plays in) occurs, as JSON (`null` for an unknown scene).
        #[qinvokable]
        #[cxx_name = "usesOf"]
        fn uses_of(self: &Self, kind: &QString, id: &QString) -> QString;

        /// What the new number and heading of scene `id` would change in the script text,
        /// as JSON `[{ kind, from, to, count }]`; empty when the text stays.
        #[qinvokable]
        #[cxx_name = "sceneChanges"]
        fn scene_changes(self: &Self, id: &QString, number: &QString, heading: &QString)
            -> QString;

        /// The same for the new name of character `id`.
        #[qinvokable]
        #[cxx_name = "characterChanges"]
        fn character_changes(self: &Self, id: &QString, name: &QString) -> QString;

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

        /// Sizes and the room height are text in metres (`8,5`).
        #[qinvokable]
        #[cxx_name = "addLocation"]
        fn add_location(
            self: Pin<&mut Self>,
            name: &QString,
            width: &QString,
            depth: &QString,
            height: &QString,
        ) -> bool;

        #[qinvokable]
        #[cxx_name = "editLocation"]
        fn edit_location(
            self: Pin<&mut Self>,
            id: &QString,
            name: &QString,
            width: &QString,
            depth: &QString,
            height: &QString,
        ) -> bool;

        #[qinvokable]
        #[cxx_name = "removeLocation"]
        fn remove_location(self: Pin<&mut Self>, id: &QString) -> bool;

        /// `color` is a role name (`purple`, `blue` …); `group` is the head count of a group
        /// of extras, 0 for a single person; `size` the diameter as typed (LH-25), blank: standard.
        #[qinvokable]
        #[cxx_name = "addCharacter"]
        fn add_character(
            self: Pin<&mut Self>,
            name: &QString,
            label: &QString,
            color: &QString,
            group: u32,
            size: &QString,
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
            size: &QString,
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

        /// Moves the plan `id` one place; `shift` is "earlier" or "later".
        #[qinvokable]
        #[cxx_name = "movePlan"]
        fn move_plan(self: Pin<&mut Self>, id: &QString, shift: &QString) -> bool;

        #[qinvokable]
        #[cxx_name = "removePlan"]
        fn remove_plan(self: Pin<&mut Self>, id: &QString) -> bool;

        #[qinvokable]
        #[cxx_name = "removePlanWithShots"]
        fn remove_plan_with_shots(self: Pin<&mut Self>, id: &QString) -> bool;

        /// Puts the template `id` in the middle of the plan on view; false when refused.
        #[qinvokable]
        #[cxx_name = "applyTemplate"]
        fn apply_template(self: Pin<&mut Self>, id: &QString) -> bool;

        /// Files the figures and cameras of the plan on view as the template `name`.
        #[qinvokable]
        #[cxx_name = "saveTemplate"]
        fn save_template(self: Pin<&mut Self>, name: &QString) -> bool;

        /// Renames the project's own template `id`; false for a blank name.
        #[qinvokable]
        #[cxx_name = "renameTemplate"]
        fn rename_template(self: Pin<&mut Self>, id: &QString, name: &QString) -> bool;

        /// Deletes the project's own template `id`.
        #[qinvokable]
        #[cxx_name = "removeTemplate"]
        fn remove_template(self: Pin<&mut Self>, id: &QString) -> bool;
    }

    // The start page's cards are read on a thread of their own and handed back through it.
    impl cxx_qt::Threading for Controller {}
}

use cxx_qt_lib::QString;

pub use state::ControllerRust;

mod arm;
mod book;
mod draw;
mod drehbuch;
mod export;
mod free;
mod hints;
mod history;
mod jump;
mod layers;
mod library;
mod marks;
mod measure;
mod notes;
mod picture;
mod plans;
mod playback;
mod project;
#[cfg(test)]
mod qml_tests;
mod retime;
mod saved;
mod script;
mod selection;
mod shots;
mod sketch;
mod start;
mod state;
mod templates;
mod view;

fn q(s: &str) -> QString {
    QString::from(s)
}

#[cfg(test)]
mod tests;
