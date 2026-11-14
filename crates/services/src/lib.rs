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

mod catalog;
mod edit;
mod error;
mod plan_edit;
mod session;
mod views;

pub use catalog::Language;
pub use edit::ShotField;
pub use error::ServiceError;
pub use plan_edit::{PlanEdit, Snapping};
pub use session::{Autosave, BackupState, Layer, Session, Visibility};
pub use views::{Chip, LinkKind, Marker, ScriptLine, Segment, ShotRow, Track, TrackKind};

// What surfaces draw and print, handed through so they need no lower crate.
pub use klappe_export::{Orientation, Paper, ShotColumns, ShotTexts, Texts, Theme};
pub use klappe_model::{
    Angle, FigureColor, FloorPlan, LightKind, Movement, Point, Sensor, ShotSize,
};
pub use klappe_stellprobe::{
    horizontal_fov_deg, DisplayList, Item, Role, Shape, ShotOrder, Source,
};

// Fountain, so the TUI editor can style and list what it types without the script crate.
pub use klappe_script::{parse, Element};
