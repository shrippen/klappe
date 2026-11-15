//! Klappe's data model, shared by every module (planung/KONZEPT.md, section 4).
//!
//! ```text
//! Project
//! ├── characters   Character        (project-wide: Elisabeth, Burgvogt …)
//! ├── locations    Location → Set   (walls, objects, background image)
//! ├── library      LibraryItem      (the project's own objects, LH-12)
//! ├── templates    Template         (the project's own set-ups, LH-07)
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

mod camera_preset;
mod geometry;
mod library;
mod light_preset;
mod limits;
mod plan;
mod project;
mod shot;
mod template;

pub use camera_preset::{CameraBody, CameraPresets, LensSet};
pub use geometry::Point;
pub use library::{
    is_object_height_m, is_object_size_m, LibraryItem, ObjectShape, MAX_OBJECT_HEIGHT_M,
    MAX_OBJECT_M,
};
pub use light_preset::{LightPreset, MAX_BEAM_DEG, MAX_POWER_W, MAX_TEMP_K, MIN_TEMP_K};
pub use limits::{
    MAX_ASPECT, MAX_CAMERA_HEIGHT_M, MAX_LENS_MM, MAX_SENSOR_WIDTH_MM, MAX_SET_M, MAX_TILT_DEG,
    MIN_LENS_MM,
};
pub use plan::{
    Arrival, AxisPair, Camera, CameraMark, Dash, Departure, Drawing, DrawingShape, FigureTrack,
    Fill, FloorPlan, Light, LightKind, LinkTarget, Mark, Pose, Rig, ScriptLink, Sensor,
};
pub use project::{
    is_figure_size_m, Background, Character, FigureColor, Location, Note, Opening, OpeningKind,
    Project, Scene, SetObject, SetPlan, Wall, MAX_FIGURE_M, MAX_GROUP, MIN_FIGURE_M, MIN_GROUP,
};
pub use shot::{Angle, Movement, Shot, ShotSize};
pub use template::{Template, TemplateFigure};

/// The version of the project format this model reads and writes (FORMAT.md).
pub const FORMAT_VERSION: u32 = 1;
