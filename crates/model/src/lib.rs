//! Klappe's data model, shared by every module (planung/KONZEPT.md, section 4).
//!
//! ```text
//! Project
//! ├── characters   Character        (project-wide: Elisabeth, Burgvogt …)
//! ├── locations    Location → Set   (walls, objects, background)
//! └── scenes       Scene → FloorPlan
//!                           ├── figures  FigureTrack → Mark (time, place, facing)
//!                           ├── cameras  Camera      → CameraMark
//!                           ├── lights   Light
//!                           ├── shots    Shot (cut order = list order)
//!                           └── links    ScriptLink  (script element ↔ mark or shot)
//! ```
//!
//! Units are in the field names: metres (`_m`), degrees (`_deg`), seconds (`_s`), millimetres
//! (`_mm`), kelvin (`_k`), watts (`_w`). The plan's origin is its top left corner, x to the right,
//! y downwards (as drawn); 0° faces right (+x), angles grow clockwise on screen.

mod geometry;
mod plan;
mod project;
mod shot;

pub use geometry::Point;
pub use plan::{
    AxisPair, Camera, CameraMark, FigureTrack, FloorPlan, Light, LightKind, LinkTarget, Mark, Pose,
    ScriptLink, Sensor,
};
pub use project::{Character, FigureColor, Location, Project, Scene, SetObject, SetPlan, Wall};
pub use shot::{Movement, Shot, ShotSize};

/// The version of the project format this model reads and writes (FORMAT.md once it exists).
pub const FORMAT_VERSION: u32 = 1;
