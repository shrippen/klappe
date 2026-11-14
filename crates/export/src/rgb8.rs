//! A decoded picture: 8-bit RGB.

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Rgb8 {
    pub width: u32,
    pub height: u32,
    /// Row by row, 3 bytes (R, G, B) per pixel.
    pub pixels: Vec<u8>,
}
