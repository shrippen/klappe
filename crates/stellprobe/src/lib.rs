//! Stellprobe's core, without UI or files (planung/KONZEPT.md, section 4).
//!
//! ```text
//! model::FloorPlan ──► motion   where is everyone at t?
//!                  ──► height   how tall is everything? (defaults, stacking)
//!                  ──► optics   what does a camera see?
//!                  ──► scene    the set as bodies ──► picture  pixels through `raster`
//!                                   └─► vector_view  the faces far to near, for print
//!                  ──► movement what does a camera do during a shot?
//!                  ──► axis     which shot jumps the line?
//!                  ──► setup    which shots share camera, place and lens?
//!                  └─► display  a display list in metres: the GUI draws it,
//!                               the export writes it as SVG or PDF
//! ```

mod axis;
mod card;
mod display;
mod eye;
mod figure;
mod forms;
mod height;
mod hit;
mod light;
mod look;
mod motion;
mod movement;
mod opening;
mod optics;
mod picture;
mod rig;
mod scene;
mod setup;
mod triangulate;

pub use axis::{axis_at, axis_jumps, axis_pair, Axis, AxisJump, Side};
pub use card::NOTE_TEXT_M;
pub use display::{
    drawing_edge_places, drawing_handle_places, handles, label_point, object_grip_place, render,
    Bend, DisplayList, Item, LightGrip, ObjectGrip, OpeningEnd, Role, Shape, Source,
    FIGURE_RADIUS_M,
};
pub use eye::{Eye, Orbit};
pub use forms::Forms;
pub use height::{object_extent, opening_span, wall_top};
pub use hit::{anchor, hit, in_lasso, within};
pub use klappe_raster::Frame;
pub use light::{light_grip_place, Lamp};
pub use look::{look, scene_look, Area, Hue, Look};
pub use motion::{
    aim_at, bend_follow, bend_grips, camera_at, figure_at, walked_ways, way_length_m, Placement,
};
pub use movement::movement_in;
pub use optics::{frustum, horizontal_fov_deg, lens_at, shot_lens, ShotLens};
pub use picture::{
    floor_at, grab_at, level_at, lift_at, picture, source_at, vector_view, Picture, PictureLabel,
    VectorFace, VectorView,
};
pub use scene::{Cut, SceneRole};
pub use setup::{
    is_setup_model, ordered_shots, setup_choices, setup_target, shot_setups, SetupChoice,
    ShotOrder, SETUP_TOLERANCE_M,
};
