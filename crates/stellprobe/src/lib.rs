//! Stellprobe's core, without UI or files (planung/KONZEPT.md, section 4).
//!
//! ```text
//! model::FloorPlan ──► motion   where is everyone at t?
//!                  ──► optics   what does a camera see?
//!                  ──► axis     which shot jumps the line?
//!                  ──► setup    which shots share camera, place and lens?
//!                  └─► display  a display list in metres: the GUI draws it,
//!                               the export writes it as SVG or PDF
//! ```

mod axis;
mod display;
mod hit;
mod motion;
mod optics;
mod setup;

pub use axis::{axis_at, axis_jumps, Axis, Side};
pub use display::{handles, render, DisplayList, Item, Role, Shape, Source, FIGURE_RADIUS_M};
pub use hit::hit;
pub use motion::{bend_point, camera_at, figure_at, via_through, way_length_m, Placement};
pub use optics::{frustum, horizontal_fov_deg, lens_at};
pub use setup::{ordered_shots, shot_setups, ShotOrder, SETUP_TOLERANCE_M};
