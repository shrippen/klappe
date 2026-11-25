//! Klappe's data model, shared by every module (planung/KONZEPT.md, section 4).
//!
//! ```text
//! Project
//! ├── characters   Character        (project-wide: Elisabeth, Burgvogt …)
//! ├── locations    Location → Set   (walls, objects, notes, images, background image)
//! ├── library      LibraryItem      (the project's own objects, LH-12)
//! ├── templates    Template         (the project's own set-ups, LH-07)
//! ├── views        SavedView        (named choices of visible layers, cameras, figures, LH-119)
//! ├── elements     Element          (the breakdown's things: costumes, props …, DB-60)
//! ├── team         Team             (persons, casting of the characters, contacts)
//! └── scenes       Scene → FloorPlan   (the register of what is shot: a scene or a content item,
//!                           by `kind`; its order is the project's order)
//!                           ├── sheet    Sheet       (the scene sheet: story day, elements, extras)
//!                           ├── figures  FigureTrack → Mark (time, place, facing)
//!                           ├── object_tracks ObjectTrack → ObjectMark (a set object's way, LH-24)
//!                           ├── cameras  Camera      → CameraMark
//!                           ├── lights   Light
//!                           ├── drawings Drawing  (lines, arrows, shapes)
//!                           ├── sketches Sketch   (freehand strokes of the pen, LH-120)
//!                           ├── shots    Shot (cut order = list order)
//!                           └── links    ScriptLink  (script element ↔ mark or shot)
//! ```
//!
//! Units are in the field names: metres (`_m`), degrees (`_deg`), seconds (`_s`), millimetres
//! (`_mm`), kelvin (`_k`), watts (`_w`). The plan's origin is its top left corner, x to the right,
//! y downwards (as drawn); 0° faces right (+x), angles grow clockwise on screen.

mod binding;
mod breakdown;
mod camera_color;
mod camera_preset;
mod curve;
mod drawing_style;
mod geometry;
mod library;
mod light_preset;
mod limits;
mod object_track;
mod paper;
mod place;
mod plan;
mod project;
mod register;
mod saved_view;
mod script_setup;
mod shot;
mod site;
mod sketch;
mod team;
mod template;

pub use binding::Binding;
pub use breakdown::{Element, ElementCategory, ElementStatus, Sheet, TextMark};
pub use camera_preset::{CameraBody, CameraPresets, LensSet};
pub use curve::Curve;
pub use drawing_style::{Dash, Fill};
pub use geometry::{normalize_deg, Extent, Point, TurnedRect, FULL_TURN_DEG};
pub use library::{
    is_floor_offset_m, is_object_height_m, is_object_size_m, is_outline, outline_area_m2,
    outline_box, outline_label_point, BodyForm, FormBox, LibraryItem, ObjectShape, FREE_FORM_KIND,
    MAX_OBJECT_HEIGHT_M, MAX_OBJECT_M, MIN_OBJECT_HEIGHT_M, MIN_OBJECT_M,
};
pub use light_preset::{is_beam_deg, is_power_w, is_temp_k, LightPreset, MIN_BEAM_DEG};
pub use limits::{
    is_aspect, is_camera_height_m, is_camera_lens_mm, is_hold_s, is_sensor_width_mm, is_tilt_deg,
    MAX_ASPECT, MAX_CAMERA_HEIGHT_M, MAX_CAMERA_LENS_MM, MAX_SENSOR_WIDTH_MM, MAX_SET_M,
    MAX_TILT_DEG, MIN_CAMERA_LENS_MM,
};
pub use object_track::{ObjectMark, ObjectTrack};
pub use paper::{Orientation, Paper, PaperSetup};
pub use place::{same_place_name, Place};
pub use plan::{
    Arrival, AxisPair, Camera, CameraMark, Departure, Drawing, DrawingShape, FigureTrack,
    FloorPlan, Light, LightKind, LinkTarget, Mark, Pose, Rig, ScriptLink, Sensor,
};
pub use project::{
    is_figure_size_m, is_image_opacity, is_opening_width_m, same_character_name, Background,
    Character, DoorSide, FigureColor, Hinge, Location, Note, Opening, OpeningKind, PlanImage,
    Project, Scene, SetObject, SetPlan, Wall, IMAGE_WIDTH_M, MAX_FIGURE_M, MAX_GROUP, MIN_FIGURE_M,
    MIN_GROUP, MIN_IMAGE_OPACITY, MIN_OPENING_WIDTH_M, NOTE_MIN_WIDTH_M, NOTE_WIDTH_M,
};
pub use register::{
    free_random_id, id_from_seed, random_id, BookState, IdCase, Kind, LockedNumber, Print, SceneKey,
};
pub use saved_view::SavedView;
pub use script_setup::{SceneNumbering, ScriptLanguage, ScriptPaper, ScriptSetup, ScriptWords};
pub use shot::{Angle, Movement, Numbering, Shot, ShotSize, VersionOf};
pub use site::{Site, SunTime, MAX_UTC_OFFSET_MIN};
pub use sketch::Sketch;
pub use team::{Casting, Contact, Department, Person, Team};
pub use template::{Template, TemplateFigure};

/// The version of the project format this model reads and writes (FORMAT.md).
pub const FORMAT_VERSION: u32 = 4;
