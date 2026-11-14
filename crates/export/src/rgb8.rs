//! A decoded picture: 8-bit RGB, row by row.

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Rgb8 {
    pub width: u32,
    pub height: u32,
    /// 3 bytes per pixel.
    pub pixels: Vec<u8>,
}
