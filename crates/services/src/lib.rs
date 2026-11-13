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
