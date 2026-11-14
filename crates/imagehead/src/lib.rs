//! The one reader of the head of a JPEG or PNG file. `store` checks an image with it when it is
//! added, `export` reads the colour model from it; both must agree on what an image is and
//! what can be printed.

/// Start of a JPEG file.
pub const JPEG_SOI: [u8; 2] = [0xFF, 0xD8];
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
/// The only sample precision of a JPEG the export prints.
pub const JPEG_SAMPLE_BITS: u8 = 8;

/// Start of a PNG file.
pub const PNG_SIGNATURE: [u8; 8] = *b"\x89PNG\r\n\x1a\n";
const IHDR: &[u8; 4] = b"IHDR";
/// After the signature: IHDR length (4), type (4).
const IHDR_LEN_AT: usize = 8;
const IHDR_DATA_AT: usize = 16;
/// Data of IHDR: width (4), height (4), bit depth, colour type, compression, filter, interlace.
const IHDR_LEN: usize = 13;
const IHDR_LEN_FIELD: [u8; 4] = (IHDR_LEN as u32).to_be_bytes();
const PNG_HEIGHT_AT: usize = 4;
const PNG_DEPTH_AT: usize = 8;
const PNG_COLOR_AT: usize = 9;
const PNG_INTERLACE_AT: usize = 12;
const PNG_ADAM7: u8 = 1;

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

impl JpegFrame {
    /// Whether the PDF export can print this JPEG: gray or RGB, 8 bits per sample.
    pub fn check_printable(&self) -> Result<(), String> {
        if !matches!(self.components, 1 | 3) {
            return Err(format!(
                "JPEG with {} colour channels (CMYK) is not supported",
                self.components
            ));
        }
        if self.precision != JPEG_SAMPLE_BITS {
            return Err(format!("JPEG with {} bits per sample", self.precision));
        }

        Ok(())
    }
}

/// The first frame header of `bytes`; `None` if it is no JPEG, the header is cut off or a
/// side has zero pixels.
pub fn jpeg_frame(bytes: &[u8]) -> Option<JpegFrame> {
    if !bytes.starts_with(&JPEG_SOI) {
        return None;
    }

    let mut at = JPEG_SOI.len();

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

/// The colour types of PNG that the export knows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PngColor {
    Gray,
    Rgb,
    Indexed,
    GrayAlpha,
    Rgba,
}

impl PngColor {
    fn from_code(code: u8) -> Result<Self, String> {
        match code {
            0 => Ok(PngColor::Gray),
            2 => Ok(PngColor::Rgb),
            3 => Ok(PngColor::Indexed),
            4 => Ok(PngColor::GrayAlpha),
            6 => Ok(PngColor::Rgba),
            _ => Err(format!("unknown PNG colour type {code}")),
        }
    }

    pub fn channels(self) -> usize {
        match self {
            PngColor::Gray | PngColor::Indexed => 1,
            PngColor::GrayAlpha => 2,
            PngColor::Rgb => 3,
            PngColor::Rgba => 4,
        }
    }

    pub fn has_alpha(self) -> bool {
        matches!(self, PngColor::GrayAlpha | PngColor::Rgba)
    }

    /// The bit depths the PNG standard allows for this colour type.
    fn allows(self, depth: u8) -> bool {
        match self {
            PngColor::Gray => matches!(depth, 1 | 2 | 4 | 8 | 16),
            PngColor::Indexed => matches!(depth, 1 | 2 | 4 | 8),
            _ => matches!(depth, 8 | 16),
        }
    }
}

/// What a PNG's `IHDR` chunk says.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PngHead {
    pub width: u32,
    pub height: u32,
    /// Bits per sample.
    pub depth: u8,
    pub color: PngColor,
    pub interlaced: bool,
}

/// The `IHDR` chunk of `bytes`, which must come first; `Err` if it is missing or cut off, the
/// colour type or bit depth is invalid, or a side has zero pixels.
pub fn png_head(bytes: &[u8]) -> Result<PngHead, String> {
    let data = bytes
        .get(IHDR_DATA_AT - IHDR.len()..IHDR_DATA_AT)
        .filter(|kind| *kind == IHDR)
        .filter(|_| bytes.get(IHDR_LEN_AT..IHDR_DATA_AT - IHDR.len()) == Some(&IHDR_LEN_FIELD))
        .and_then(|_| bytes.get(IHDR_DATA_AT..IHDR_DATA_AT + IHDR_LEN))
        .filter(|_| bytes.starts_with(&PNG_SIGNATURE))
        .ok_or("no PNG header")?;
    let be = |at: usize| u32::from_be_bytes([data[at], data[at + 1], data[at + 2], data[at + 3]]);

    let color = PngColor::from_code(data[PNG_COLOR_AT])?;
    let head = PngHead {
        width: be(0),
        height: be(PNG_HEIGHT_AT),
        depth: data[PNG_DEPTH_AT],
        color,
        interlaced: data[PNG_INTERLACE_AT] == PNG_ADAM7,
    };

    if !color.allows(head.depth) {
        return Err("bit depth does not fit the colour type".to_owned());
    }
    if head.width == 0 || head.height == 0 {
        return Err("PNG without pixels".to_owned());
    }

    Ok(head)
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

    /// A PNG head: signature, IHDR of `w` x `h`, `depth` bits, colour type `color`.
    fn png(w: u32, h: u32, depth: u8, color: u8) -> Vec<u8> {
        let mut b = PNG_SIGNATURE.to_vec();
        b.extend_from_slice(&13_u32.to_be_bytes());
        b.extend_from_slice(b"IHDR");
        b.extend_from_slice(&w.to_be_bytes());
        b.extend_from_slice(&h.to_be_bytes());
        b.extend_from_slice(&[depth, color, 0, 0, 0]);
        b
    }

    #[test]
    fn reads_the_png_head() {
        let mut bytes = png(1000, 640, 4, 3);
        bytes[16 + 12] = 1;

        assert_eq!(
            png_head(&bytes),
            Ok(PngHead {
                width: 1000,
                height: 640,
                depth: 4,
                color: PngColor::Indexed,
                interlaced: true
            })
        );
    }

    #[test]
    fn rejects_broken_png_heads() {
        assert!(png_head(b"GIF89a").is_err());
        assert!(png_head(&png(1, 1, 8, 2)[..20]).is_err());
        assert!(png_head(&png(0, 5, 8, 2)).is_err());
        assert!(png_head(&png(5, 0, 8, 2)).is_err());
        // Unknown colour type, depth that does not fit it.
        assert!(png_head(&png(1, 1, 8, 1)).is_err());
        assert!(png_head(&png(1, 1, 3, 2)).is_err());
        assert!(png_head(&png(1, 1, 16, 3)).is_err());

        // IHDR length field must be 13.
        let mut short = png(1, 1, 8, 2);
        short[8..12].copy_from_slice(&12_u32.to_be_bytes());
        assert!(png_head(&short).is_err());

        let mut late = png(1, 1, 8, 2);
        late[12..16].copy_from_slice(b"IDAT");
        assert!(png_head(&late).is_err());
    }

    #[test]
    fn only_gray_and_rgb_jpegs_of_8_bits_print() {
        let frame = |precision, components| JpegFrame {
            width: 1,
            height: 1,
            precision,
            components,
        };

        assert!(frame(8, 1).check_printable().is_ok());
        assert!(frame(8, 3).check_printable().is_ok());
        assert!(frame(8, 4).check_printable().is_err());
        assert!(frame(12, 3).check_printable().is_err());
    }
}
