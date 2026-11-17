//! The controller's start: open the project named on the command line, set every property empty.

use cxx_qt_lib::QString;
use klappe_services::Session;

use super::{q, ControllerRust};
use crate::args::AppArgs;
use crate::arm::Armed;
use crate::catalog::Language;
use crate::notice::Status;
use crate::selection::Selection;
use crate::tools::{AreaDraft, DrawStyle, Grip, MeasureStep, NoteDraft, Preview, ScaleStep, Tool};
use klappe_services::{error_text, load_notes_text};

/// Seconds between two backups of unsaved changes while a project is open.
const AUTOSAVE_S: i32 = 30;
const MS_PER_S: i32 = 1000;

/// The project named on the command line, at its shot and time; a failure goes to `notice`.
fn open_at_start(args: &AppArgs, language: Language, notice: &mut Status) -> Option<Session> {
    let dir = args.open.as_ref()?;
    match Session::open(dir) {
        Ok(mut s) => {
            if let Some(shot) = &args.shot {
                let _ = s.select_shot(shot);
            }
            s.set_time(args.time_s.max(s.time_s()));
            notice.say(load_notes_text(language, s.load_notes()));
            Some(s)
        }
        Err(e) => {
            notice.say(error_text(language, &e));
            None
        }
    }
}

/// The screenshot file the start asked for, as the property holds it; empty for none.
fn screenshot_url(path: Option<std::path::PathBuf>) -> QString {
    path.map(|p| q(&p.to_string_lossy())).unwrap_or_default()
}

impl Default for ControllerRust {
    fn default() -> Self {
        let args = AppArgs::from_env();
        let language = Language::pick(args.language.as_deref());
        let mut notice = Status::default();
        let session = open_at_start(&args, language, &mut notice);

        let mut this = Self {
            session,
            language,
            status: q(notice.text()),
            example_aside: QString::default(),
            notice,
            texts: q(language.json()),
            loaded: false,
            title: QString::default(),
            plan_title: QString::default(),
            plan_json: QString::default(),
            picture_revision: 0,
            camera_frame: String::new(),
            free_revision: 0,
            view_free: false,
            free_frame: String::new(),
            free: Default::default(),
            tracks_json: QString::default(),
            shots_json: QString::default(),
            shots_shooting_json: QString::default(),
            script_json: QString::default(),
            layers_json: QString::default(),
            locks_json: QString::default(),
            fades_json: QString::default(),
            time: 0.0,
            duration: 0.0,
            selected_shot: QString::default(),
            can_undo: false,
            can_redo: false,
            undo_step: QString::default(),
            redo_step: QString::default(),
            dirty: false,
            selected_key: QString::default(),
            selected_count: 0,
            extend_selection: false,
            lasso_selection: false,
            snapping_on: true,
            grid_name: QString::default(),
            choices_json: QString::default(),
            selected: Selection::default(),
            marquee: None,
            sketch: Vec::new(),
            tool: Tool::Select,
            scale: ScaleStep::Idle,
            measure: MeasureStep::Idle,
            measuring: false,
            draw: DrawStyle::default(),
            area: AreaDraft::default(),
            shown: String::new(),
            area_points: 0,
            has_background: false,
            missing_heights: 0,
            scale_points: 0,
            note: NoteDraft::Idle,
            asks_note: false,
            note_text: QString::default(),
            arm: Armed::Idle,
            armed: QString::default(),
            preview: Preview::Hidden,
            drag_step: None,
            grip: Grip::Free,
            backup_state: QString::default(),
            backup_message: QString::default(),
            autosave_ms: AUTOSAVE_S * MS_PER_S,
            project_json: QString::default(),
            scene_id: QString::default(),
            recent_json: q("[]"),
            recent_store: q("[]"),
            screenshot_path: screenshot_url(args.screenshot),
            screenshot_after: args.screenshot_after_ms,
        };
        this.fill();
        this
    }
}
