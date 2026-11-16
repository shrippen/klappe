//! The project session (planung/KONZEPT.md, section 4): the one layer GUI, TUI and CLI call.
//!
//! ```text
//! app / tui / cli
//!        │  open, select_scene, set_time, edit_shot, undo, save
//!        ▼
//!     Session ──► store (files)   stellprobe (display list, axis)   script (Fountain)
//!        │
//!        └─► views: ShotRow, Track, ScriptLine  (plain data, ready to show)
//! ```

mod camera_edit;
mod catalog;
mod edit;
mod error;
mod error_text;
mod example;
mod export_format;
mod field;
mod library;
mod light_edit;
mod measure;
mod number;
mod plan_edit;
mod project_edit;
mod script_refs;
mod session;
mod shot_edit;
mod start;
mod templates;
mod views;

pub use catalog::Language;
pub use edit::ShotField;
pub use error::ServiceError;
pub use error_text::error_text;
pub use example::{copy_example, ensure_example};
pub use export_format::ExportFormat;
pub use field::Field;
pub use light_edit::LightChange;
pub use measure::measure_angle_deg;
pub use number::{read_number, Range, Unit};
pub use plan_edit::{Grid, MarkInfo, Pace, PlanEdit, Snapping};
pub use project_edit::{PlanShift, ProjectEdit, ShotsGo};
pub use script_refs::{ChangeKind, ScriptChange};
pub use session::{
    Autosave, BackupState, ExportOptions, Grab, Layer, Lock, Looping, Playing, Session, Size, Step,
    TimeSnap, View, Visibility,
};
pub use start::{project_card, CardState, ProjectCard};
pub use views::{
    AxisChoice, CameraSpot, CastUse, CharacterRow, Chip, Hint, LibraryEntry, LinkKind, LocationRow,
    Marker, Overview, PlanRow, SceneCard, SceneRow, SceneUse, ScriptLine, Segment, SetupOption,
    ShotRow, TemplateEntry, Track, TrackKind,
};

// What surfaces draw and print, handed through so they need no lower crate.
pub use klappe_export::{DrawingTexts, LampTexts, ShotColumns, ShotTexts, Texts, Theme};
pub use klappe_model::{
    Angle, Arrival, CameraBody, CameraPresets, Dash, Departure, DoorSide, DrawingShape,
    FigureColor, Fill, FloorPlan, Hinge, LensSet, LightKind, LightPreset, Movement, Numbering,
    ObjectShape, OpeningKind, Orientation, Paper, PaperSetup, Point, Pose, Rig, Sensor, ShotSize,
    MIN_IMAGE_OPACITY, MIN_OBJECT_HEIGHT_M,
};
pub use klappe_stellprobe::{
    horizontal_fov_deg, look, scene_look, Area, Bend, Cut, DisplayList, Frame, Hue, Item,
    LightGrip, ObjectGrip, OpeningEnd, Picture, PictureLabel, Role, SceneRole, Shape, ShotOrder,
    Source, NOTE_TEXT_M,
};

// Fountain, so the TUI editor can style and list what it types without the script crate;
// `parse` for its tests.
pub use klappe_script::{parse, parse_located, Element, Located};
