//! Stellprobe's core, without UI or files (planung/KONZEPT.md, section 4).
//!
//! ```text
//! model::FloorPlan ──► motion   where is everyone at t?
//!                  ──► optics   what does a camera see?
//!                  ──► movement what does a camera do during a shot?
//!                  ──► axis     which shot jumps the line?
//!                  ──► setup    which shots share camera, place and lens?
//!                  └─► display  a display list in metres: the GUI draws it,
//!                               the export writes it as SVG or PDF
//! ```

mod axis;
mod card;
mod display;
mod figure;
mod hit;
mod light;
mod look;
mod motion;
mod movement;
mod optics;
mod rig;
mod setup;

pub use axis::{axis_at, axis_jumps, axis_pair, Axis, AxisJump, Side};
pub use card::NOTE_TEXT_M;
pub use display::{
    drawing_handle_places, handles, render, DisplayList, Item, Role, Shape, Source, FIGURE_RADIUS_M,
};
pub use hit::{anchor, hit, in_lasso, within};
pub use light::Lamp;
pub use look::{look, Area, Hue, Look};
pub use motion::{
    aim_at, bend_follow, bend_point, camera_at, figure_at, via_through, walked_ways, way_length_m,
    Placement,
};
pub use movement::movement_in;
pub use optics::{frustum, horizontal_fov_deg, lens_at, shot_lens, ShotLens};
pub use setup::{ordered_shots, shot_setups, ShotOrder, SETUP_TOLERANCE_M};
