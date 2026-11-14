//! Placeholder until the JPEG decoder is merged in.

use crate::rgb8::Rgb8;

pub(crate) fn decode_jpeg(_bytes: &[u8]) -> Result<Rgb8, String> {
    Err("JPEG decoding not built yet".into())
}
