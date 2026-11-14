//! Klappe's data model, shared by every module (planung/KONZEPT.md, section 4).
//!
//! ```text
//! Project
//! ├── characters   Character        (project-wide: Elisabeth, Burgvogt …)
//! ├── locations    Location → Set   (walls, objects, background image)
//! ├── library      LibraryItem      (the project's own objects, LH-12)
//! └── scenes       Scene → FloorPlan
//!                           ├── figures  FigureTrack → Mark (time, place, facing)
//!                           ├── cameras  Camera      → CameraMark
//!                           ├── lights   Light
//!                           ├── drawings Drawing  (lines, arrows, shapes)
//!                           ├── shots    Shot (cut order = list order)
//!                           └── links    ScriptLink  (script element ↔ mark or shot)
//! ```
//!
//! Units are in the field names: metres (`_m`), degrees (`_deg`), seconds (`_s`), millimetres
//! (`_mm`), kelvin (`_k`), watts (`_w`). The plan's origin is its top left corner, x to the right,
//! y downwards (as drawn); 0° faces right (+x), angles grow clockwise on screen.

mod geometry;
mod library;
mod plan;
mod project;
mod shot;

pub use geometry::Point;
pub use library::{is_object_size_m, LibraryItem, ObjectShape, MAX_OBJECT_M};
pub use plan::{
    AxisPair, Camera, CameraMark, Dash, Drawing, DrawingShape, FigureTrack, Fill, FloorPlan, Light,
    LightKind, LinkTarget, Mark, Pose, ScriptLink, Sensor,
};
pub use project::{
    Background, Character, FigureColor, Location, Note, Project, Scene, SetObject, SetPlan, Wall,
};
pub use shot::{Angle, Movement, Shot, ShotSize};

/// The version of the project format this model reads and writes (FORMAT.md).
pub const FORMAT_VERSION: u32 = 1;
