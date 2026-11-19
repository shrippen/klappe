//! The controller's state behind the QML properties: the session, what the tools hold and the
//! values the properties show. Its own file so the bridge's declarations have room.

use cxx_qt_lib::QString;
use klappe_services::{Point, Session};

use super::free;
use crate::arm::Armed;
use crate::catalog::Language;
use crate::notice::Status;
use crate::plan::Sweep;
use crate::selection::Selection;
use crate::tools::{AreaDraft, DrawStyle, Grip, MeasureStep, NoteDraft, Preview, ScaleStep, Tool};

pub struct ControllerRust {
    pub(super) session: Option<Session>,
    pub(super) language: Language,
    pub(super) notice: Status,
    pub(super) texts: QString,
    pub(super) loaded: bool,
    pub(super) title: QString,
    pub(super) plan_title: QString,
    pub(super) plan_json: QString,
    pub(super) picture_revision: i32,
    pub(super) edit_revision: i32,
    /// `cameraFrame`: the size and names of the last camera image.
    pub(super) camera_frame: String,
    pub(super) free_revision: i32,
    pub(super) view_free: bool,
    /// `freeFrame`: the size and names of the last 3D image.
    pub(super) free_frame: String,
    /// The 3D view's size and what a drag in it holds.
    pub(super) free: free::FreeState,
    pub(super) tracks_json: QString,
    pub(super) shots_json: QString,
    pub(super) shots_shooting_json: QString,
    pub(super) script_json: QString,
    pub(super) script_text: QString,
    pub(super) outline_json: QString,
    pub(super) layers_json: QString,
    pub(super) locks_json: QString,
    pub(super) fades_json: QString,
    pub(super) time: f64,
    pub(super) duration: f64,
    pub(super) selected_shot: QString,
    pub(super) can_undo: bool,
    pub(super) can_redo: bool,
    pub(super) undo_step: QString,
    pub(super) redo_step: QString,
    pub(super) dirty: bool,
    pub(super) selected_key: QString,
    pub(super) selected_count: i32,
    pub(super) extend_selection: bool,
    pub(super) lasso_selection: bool,
    pub(super) snapping_on: bool,
    pub(super) grid_name: QString,
    pub(super) choices_json: QString,
    /// What the select tool has picked; the canvas frames it.
    pub(super) selected: Selection,
    /// The box or lasso a drag on empty ground is pulling up, in metres.
    pub(super) marquee: Option<Sweep>,
    /// The points of the stroke the sketch pen is drawing, in metres (LH-120).
    pub(super) sketch: Vec<Point>,
    pub(super) tool: Tool,
    pub(super) scale: ScaleStep,
    /// The measurement of the measuring tool: a tool state of the session, not the plan's.
    pub(super) measure: MeasureStep,
    pub(super) measuring: bool,
    /// What the drawing tools draw with, and the area they are drawing.
    pub(super) draw: DrawStyle,
    pub(super) area: AreaDraft,
    /// The scene and plan on screen (`scene/plan`), to tell when undo, redo or a project
    /// edit changes them.
    pub(super) shown: String,
    pub(super) area_points: i32,
    pub(super) has_background: bool,
    pub(super) missing_heights: i32,
    pub(super) scale_points: i32,
    /// The note whose text is asked, if any.
    pub(super) note: NoteDraft,
    pub(super) asks_note: bool,
    pub(super) note_text: QString,
    /// What the next click on the plan does for the selection, and its name for QML.
    pub(super) arm: Armed,
    pub(super) armed: QString,
    /// What the plan on screen shows: a drag preview or the project.
    pub(super) preview: Preview,
    /// The last step of the drag in progress (`from_x, from_y, x, y`), shown again as time moves.
    pub(super) drag_step: Option<[f64; 4]>,
    /// What the drag in progress moves; `Free` between drags.
    pub(super) grip: Grip,
    pub(super) status: QString,
    pub(super) example_aside: QString,
    pub(super) backup_state: QString,
    pub(super) backup_message: QString,
    pub(super) autosave_ms: i32,
    pub(super) project_json: QString,
    pub(super) scene_id: QString,
    pub(super) recent_json: QString,
    pub(super) recent_store: QString,
    pub(super) screenshot_path: QString,
    pub(super) screenshot_after: i32,
    /// Stops a video that is being written when the controller goes with the window.
    pub(super) _video_stop: super::export::VideoStop,
}
