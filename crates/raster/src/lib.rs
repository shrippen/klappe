//! Fills triangles on the screen into a frame with a depth buffer (3D-KONZEPT.md, section 5).
//! It knows nothing about sets: a triangle brings its pixel corners, their depth and what to
//! write. A pixel holds no colour, only a role, a shade and where it came from; whoever shows
//! the frame colours it.
//!
//! ```text
//!   triangles (pixels, depth)
//!      │  per pixel centre inside: depth from 1/depth (perspective right), nearest wins
//!      ▼
//!   Frame { role, shade, source, face, depth }
//!      │  edge where the face changes to the neighbour, on the nearer pixel
//!      ▼
//!   Frame { … edge }
//! ```

mod fill;

pub use fill::rasterize;

/// The role of a pixel no triangle covers.
pub const EMPTY: u8 = 0;
/// The source of a pixel no triangle covers.
pub const NO_SOURCE: u32 = u32::MAX;

/// A corner on the screen: pixels from the top left, and how far it lies from the eye.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Corner {
    pub x: f32,
    pub y: f32,
    /// Distance along the view, more than 0 (the caller clips at the near plane).
    pub depth: f32,
}

/// What a triangle writes into the pixels it wins.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Paint {
    /// What it is, e.g. wall or figure; never `EMPTY`.
    pub role: u8,
    /// How much light the face gets, 0 dark to 255 full.
    pub shade: u8,
    /// What a click there hits, e.g. an index into the caller's list of elements.
    pub source: u32,
    /// The flat face it belongs to: edges appear where the face changes.
    pub face: u32,
    /// Fills only every other pixel (a checkerboard): see-through without sorting.
    pub sparse: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Triangle {
    pub corners: [Corner; 3],
    pub paint: Paint,
}

/// The picture as data, row by row from the top left: `width * height` entries each.
#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    pub width: usize,
    pub height: usize,
    pub role: Vec<u8>,
    pub shade: Vec<u8>,
    pub source: Vec<u32>,
    pub face: Vec<u32>,
    /// Distance of the nearest triangle; infinite where there is none.
    pub depth: Vec<f32>,
    /// The pixel lies on an outline or a crease.
    pub edge: Vec<bool>,
}

impl Frame {
    /// The index of pixel (`x`, `y`); `None` outside the frame.
    pub fn index(&self, x: usize, y: usize) -> Option<usize> {
        (x < self.width && y < self.height).then_some(y * self.width + x)
    }
}
