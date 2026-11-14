//! The one reader of a JPEG file's head. `store` checks an image with it when it is added,
//! `export` reads the colour model from it; both must agree on what a JPEG is.

const SOI: [u8; 2] = [0xFF, 0xD8];
const MARKER: u8 = 0xFF;
/// Markers in the SOFn range that are no frame header: DHT, JPG, DAC.
const NOT_SOF: [u8; 3] = [0xC4, 0xC8, 0xCC];
const SOF: std::ops::RangeInclusive<u8> = 0xC0..=0xCF;
/// Markers without a length: padding, TEM, SOI, EOI.
const STANDALONE: [u8; 4] = [0x00, 0x01, 0xD8, 0xD9];
const RST: std::ops::RangeInclusive<u8> = 0xD0..=0xD7;
/// After the segment length (2): precision (1), height (2), width (2), components (1).
const PRECISION_AT: usize = 2;
const HEIGHT_AT: usize = 3;
const WIDTH_AT: usize = 5;
const COMPONENTS_AT: usize = 7;

/// What a JPEG's frame header (SOFn) says.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JpegFrame {
    pub width: u32,
    pub height: u32,
    /// Bits per sample.
    pub precision: u8,
    /// Colour components: 1 gray, 3 colour, 4 CMYK.
    pub components: u8,
}

/// The first frame header of `bytes`; `None` if it is no JPEG, the header is cut off or a
/// side has zero pixels.
pub fn jpeg_frame(bytes: &[u8]) -> Option<JpegFrame> {
    if !bytes.starts_with(&SOI) {
        return None;
    }

    let mut at = SOI.len();

    loop {
        // Fill bytes may pad before a marker.
        while *bytes.get(at)? == MARKER && *bytes.get(at + 1)? == MARKER {
            at += 1;
        }
        if *bytes.get(at)? != MARKER {
            return None;
        }

        let marker = *bytes.get(at + 1)?;
        at += 2;
        if STANDALONE.contains(&marker) || RST.contains(&marker) {
            continue;
        }

        let be = |i: usize| {
            Some(u32::from(u16::from_be_bytes([
                *bytes.get(i)?,
                *bytes.get(i + 1)?,
            ])))
        };
        if SOF.contains(&marker) && !NOT_SOF.contains(&marker) {
            let frame = JpegFrame {
                precision: *bytes.get(at + PRECISION_AT)?,
                height: be(at + HEIGHT_AT)?,
                width: be(at + WIDTH_AT)?,
                components: *bytes.get(at + COMPONENTS_AT)?,
            };
            return Some(frame).filter(|f| f.width > 0 && f.height > 0);
        }

        at += usize::try_from(be(at)?).ok()?;
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// SOI, a comment segment, then SOF0: 8 bits, `h` high, `w` wide, 3 components.
    fn jpeg_head(w: u16, h: u16) -> Vec<u8> {
        let mut b = vec![0xFF, 0xD8, 0xFF, 0xFE, 0x00, 0x04, b'h', b'i'];
        b.extend_from_slice(&[0xFF, 0xC0, 0x00, 0x0B, 0x08]);
        b.extend_from_slice(&h.to_be_bytes());
        b.extend_from_slice(&w.to_be_bytes());
        b.extend_from_slice(&[0x03, 0x01, 0x11, 0x00]);
        b
    }

    #[test]
    fn reads_the_frame_past_other_segments() {
        let frame = jpeg_frame(&jpeg_head(800, 600)).unwrap();

        assert_eq!(
            frame,
            JpegFrame {
                width: 800,
                height: 600,
                precision: 8,
                components: 3
            }
        );
    }

    #[test]
    fn tables_are_no_frame_header() {
        // DHT (0xC4) of 2 bytes, then SOF2 (progressive) 30 x 20.
        let mut b = vec![0xFF, 0xD8, 0xFF, 0xC4, 0x00, 0x02];
        b.extend_from_slice(&[0xFF, 0xC2, 0x00, 0x0B, 0x08, 0x00, 0x14, 0x00, 0x1E, 0x01]);

        let frame = jpeg_frame(&b);

        assert_eq!(frame.map(|f| (f.width, f.height)), Some((30, 20)));
    }

    #[test]
    fn rejects_other_data_cut_off_heads_and_zero_size() {
        assert_eq!(jpeg_frame(b"GIF89a"), None);
        assert_eq!(jpeg_frame(&[0xFF, 0xD8, 0xFF]), None);
        assert_eq!(jpeg_frame(&jpeg_head(0, 5)), None);

        // Cut off right after the width: no component count.
        let head = jpeg_head(8, 8);
        assert_eq!(jpeg_frame(&head[..head.len() - 4]), None);
    }
}
