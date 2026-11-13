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

mod edit;
mod error;
mod session;
mod views;

pub use edit::ShotField;
pub use error::ServiceError;
pub use session::{Layer, Session, Visibility};
pub use views::{Chip, LinkKind, Marker, ScriptLine, Segment, ShotRow, Track, TrackKind};

// What surfaces draw and print, handed through so they need no lower crate.
pub use klappe_export::{Orientation, Paper, Texts, Theme};
pub use klappe_model::{FigureColor, Point, Sensor};
pub use klappe_stellprobe::{horizontal_fov_deg, DisplayList, Item, Role, Shape, Source};

// Fountain, so the TUI editor can style and list what it types without the script crate.
pub use klappe_script::{parse, Element};
