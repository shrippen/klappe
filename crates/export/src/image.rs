//! A background image as the export holds it: the file's bytes with their size and format.
//! SVG embeds the bytes as they are; the PDF gets a JPEG as it is (DCTDecode) and a PNG
//! through `png`.

use crate::error::ExportError;
use crate::palette::Rgb;
use crate::png;

const JPEG_SOI: [u8; 2] = [0xFF, 0xD8];
const JPEG_MARKER: u8 = 0xFF;
/// Markers in the SOFn range that are no frame header: DHT, JPG, DAC.
const JPEG_NOT_SOF: [u8; 3] = [0xC4, 0xC8, 0xCC];
const JPEG_SOF: std::ops::RangeInclusive<u8> = 0xC0..=0xCF;
/// Markers without a length: padding, TEM, SOI, EOI and RSTn.
const JPEG_STANDALONE: [u8; 4] = [0x00, 0x01, 0xD8, 0xD9];
const JPEG_RST: std::ops::RangeInclusive<u8> = 0xD0..=0xD7;
const JPEG_SAMPLE_BITS: u8 = 8;
const BASE64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const BASE64_PAD: char = '=';

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Format {
    Png,
    Jpeg,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Picture {
    /// As the display list names it, for error messages.
    file: String,
    format: Format,
    pub bytes: Vec<u8>,
    frame: Option<JpegFrame>,
}

/// What the PDF needs to know of a JPEG.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct JpegFrame {
    width: u32,
    height: u32,
    precision: u8,
    components: u8,
}

impl Picture {
    pub(crate) fn parse(file: &str, bytes: &[u8]) -> Result<Self, ExportError> {
        let bad = |message: &str| ExportError::Image {
            file: file.to_owned(),
            message: message.to_owned(),
        };

        let (format, frame) = match bytes {
            b if b.starts_with(&png::SIGNATURE) => {
                png::parse(b).map_err(|m| bad(&m))?;
                (Format::Png, None)
            }
            b if b.starts_with(&JPEG_SOI) => {
                let frame = jpeg_frame(b).ok_or_else(|| bad("JPEG without a frame header"))?;
                (Format::Jpeg, Some(frame))
            }
            _ => return Err(bad("neither PNG nor JPEG")),
        };

        Ok(Self {
            file: file.to_owned(),
            format,
            bytes: bytes.to_vec(),
            frame,
        })
    }

    pub(crate) fn mime(&self) -> &'static str {
        match self.format {
            Format::Png => "image/png",
            Format::Jpeg => "image/jpeg",
        }
    }

    /// The body of the PDF's image object (dictionary and stream); a PNG with an alpha
    /// channel is laid over `paper`.
    pub(crate) fn pdf_object(&self, paper: Rgb) -> Result<Vec<u8>, ExportError> {
        let bad = |message: String| ExportError::Image {
            file: self.file.clone(),
            message,
        };

        let (entries, data) = match (self.format, self.frame) {
            (Format::Jpeg, Some(frame)) => (jpeg_entries(frame).map_err(bad)?, self.bytes.clone()),
            _ => png::parse(&self.bytes)
                .and_then(|p| p.xobject(paper))
                .map_err(bad)?,
        };

        let mut out = format!(
            "<< /Type /XObject /Subtype /Image {entries} /Length {} >>\nstream\n",
            data.len()
        )
        .into_bytes();
        out.extend_from_slice(&data);
        out.extend_from_slice(b"\nendstream");

        Ok(out)
    }
}

fn jpeg_entries(frame: JpegFrame) -> Result<String, String> {
    let space = match frame.components {
        1 => "/DeviceGray",
        3 => "/DeviceRGB",
        n => {
            return Err(format!(
                "JPEG with {n} colour channels (CMYK) is not supported"
            ))
        }
    };
    if frame.precision != JPEG_SAMPLE_BITS {
        return Err(format!("JPEG with {} bits per sample", frame.precision));
    }

    Ok(format!(
        "/Width {} /Height {} /ColorSpace {space} /BitsPerComponent {JPEG_SAMPLE_BITS} /Filter /DCTDecode",
        frame.width, frame.height
    ))
}

/// Walks the segments to the first frame header (SOFn).
fn jpeg_frame(bytes: &[u8]) -> Option<JpegFrame> {
    let mut at = JPEG_SOI.len();

    loop {
        // Fill bytes may pad before a marker.
        while *bytes.get(at)? == JPEG_MARKER && *bytes.get(at + 1)? == JPEG_MARKER {
            at += 1;
        }
        if *bytes.get(at)? != JPEG_MARKER {
            return None;
        }

        let marker = *bytes.get(at + 1)?;
        at += 2;
        if JPEG_STANDALONE.contains(&marker) || JPEG_RST.contains(&marker) {
            continue;
        }

        let be = |i: usize| {
            Some(u32::from(u16::from_be_bytes([
                *bytes.get(i)?,
                *bytes.get(i + 1)?,
            ])))
        };
        if JPEG_SOF.contains(&marker) && !JPEG_NOT_SOF.contains(&marker) {
            // After the length (2): precision (1), height (2), width (2), components (1).
            let frame = JpegFrame {
                precision: *bytes.get(at + 2)?,
                height: be(at + 3)?,
                width: be(at + 5)?,
                components: *bytes.get(at + 7)?,
            };
            return Some(frame).filter(|f| f.width > 0 && f.height > 0);
        }

        at += usize::try_from(be(at)?).ok()?;
    }
}

/// Standard Base64 with padding, for `data:` URIs.
pub(crate) fn base64(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);

    for group in bytes.chunks(3) {
        let n = group
            .iter()
            .enumerate()
            .fold(0_u32, |n, (i, b)| n | u32::from(*b) << (16 - 8 * i));
        for i in 0..4 {
            // A group of one byte has two digits, of two bytes three: the rest is padding.
            match i <= group.len() {
                true => out.push(char::from(BASE64[(n >> (18 - 6 * i) & 63) as usize])),
                false => out.push(BASE64_PAD),
            }
        }
    }

    out
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// SOI, then SOF0: 8 bits, 20 high, 30 wide, 3 components.
    fn jpeg() -> Vec<u8> {
        let mut b = vec![
            0xFF, 0xD8, 0xFF, 0xC0, 0x00, 0x11, 0x08, 0x00, 0x14, 0x00, 0x1E, 0x03,
        ];
        b.extend_from_slice(&[0; 9]);
        b
    }

    #[test]
    fn base64_pads_as_the_standard_says() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foo"), "Zm9v");
        assert_eq!(base64(b"foob"), "Zm9vYg==");
        assert_eq!(base64(&[0xFB, 0xFF]), "+/8=");
    }

    #[test]
    fn a_jpeg_goes_into_the_pdf_as_it_is() {
        let bytes = jpeg();
        let picture = Picture::parse("bilder/a.jpg", &bytes).unwrap();
        let object = picture
            .pdf_object(Rgb {
                r: 255,
                g: 255,
                b: 255,
            })
            .unwrap();
        let text = String::from_utf8_lossy(&object);

        assert!(text.contains("/Width 30 /Height 20 /ColorSpace /DeviceRGB"));
        assert!(text.contains("/Filter /DCTDecode"));
        assert!(object.windows(bytes.len()).any(|w| w == bytes));
    }

    #[test]
    fn cmyk_jpeg_and_other_files_are_errors() {
        let mut cmyk = jpeg();
        cmyk[11] = 4;
        let picture = Picture::parse("bilder/a.jpg", &cmyk).unwrap();

        assert!(picture.pdf_object(Rgb { r: 0, g: 0, b: 0 }).is_err());
        assert!(Picture::parse("bilder/a.gif", b"GIF89a").is_err());
        assert!(Picture::parse("bilder/a.jpg", &[0xFF, 0xD8]).is_err());
    }
}
