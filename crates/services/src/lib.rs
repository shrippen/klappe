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
mod field;
mod library;
mod plan_edit;
mod project_edit;
mod session;
mod shot_edit;
mod views;

pub use catalog::Language;
pub use edit::ShotField;
pub use error::ServiceError;
pub use field::Field;
pub use plan_edit::{MarkInfo, Pace, PlanEdit, Snapping};
pub use project_edit::ProjectEdit;
pub use session::{Autosave, BackupState, Layer, Lock, Session, Visibility};
pub use views::{
    Chip, LibraryEntry, LinkKind, Marker, ScriptLine, Segment, ShotRow, Track, TrackKind,
};

// What surfaces draw and print, handed through so they need no lower crate.
pub use klappe_export::{Orientation, Paper, ShotColumns, ShotTexts, Texts, Theme};
pub use klappe_model::{
    Angle, Arrival, CameraBody, CameraPresets, Dash, Departure, DrawingShape, FigureColor, Fill,
    FloorPlan, LensSet, LightKind, Movement, ObjectShape, Point, Pose, Sensor, ShotSize,
};
pub use klappe_stellprobe::{
    horizontal_fov_deg, DisplayList, Item, Role, Shape, ShotOrder, Source,
};

// Fountain, so the TUI editor can style and list what it types without the script crate.
pub use klappe_script::{parse, Element};
