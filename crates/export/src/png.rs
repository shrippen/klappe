//! A PNG as a PDF image, without an image library.
//!
//! PDF's Flate filter with predictor 15 reads PNG's own scanline format, so a PNG without an
//! alpha channel (up to 8 bits per sample) goes in as it is: the compressed data is copied,
//! never decoded. The rest (alpha, 16 bits) is inflated, unfiltered and laid over the paper
//! colour as plain RGB.

use std::fmt::Write;

use crate::inflate;
use crate::palette::Rgb;

pub(crate) const SIGNATURE: [u8; 8] = *b"\x89PNG\r\n\x1a\n";
const IHDR_LEN: usize = 13;
const CHUNK_HEAD_LEN: usize = 8;
const CHUNK_TAIL_LEN: usize = 4;
/// Longest decoded image accepted, so a small file cannot ask for gigabytes.
const MAX_DECODED_BYTES: usize = 1 << 29;
const BYTE_BITS: usize = 8;
const FILTER_SUB: u8 = 1;
const FILTER_UP: u8 = 2;
const FILTER_AVERAGE: u8 = 3;
const FILTER_PAETH: u8 = 4;
const OPAQUE: u32 = 255;
const RGB_CHANNELS: usize = 3;

type Result<T> = std::result::Result<T, String>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Color {
    Gray,
    Rgb,
    Indexed,
    GrayAlpha,
    Rgba,
}

impl Color {
    fn from_code(code: u8) -> Result<Self> {
        match code {
            0 => Ok(Color::Gray),
            2 => Ok(Color::Rgb),
            3 => Ok(Color::Indexed),
            4 => Ok(Color::GrayAlpha),
            6 => Ok(Color::Rgba),
            _ => Err(format!("unknown PNG colour type {code}")),
        }
    }

    fn channels(self) -> usize {
        match self {
            Color::Gray | Color::Indexed => 1,
            Color::GrayAlpha => 2,
            Color::Rgb => 3,
            Color::Rgba => 4,
        }
    }

    fn has_alpha(self) -> bool {
        matches!(self, Color::GrayAlpha | Color::Rgba)
    }

    /// The bit depths the PNG standard allows for this colour type.
    fn allows(self, depth: u8) -> bool {
        match self {
            Color::Gray => matches!(depth, 1 | 2 | 4 | 8 | 16),
            Color::Indexed => matches!(depth, 1 | 2 | 4 | 8),
            _ => matches!(depth, 8 | 16),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Png {
    pub width: u32,
    pub height: u32,
    depth: u8,
    color: Color,
    interlaced: bool,
    palette: Vec<u8>,
    /// The `tRNS` chunk: alpha per palette entry, or one colour that is see-through.
    transparency: Vec<u8>,
    /// All IDAT chunks joined: one zlib stream.
    idat: Vec<u8>,
}

pub(crate) fn parse(bytes: &[u8]) -> Result<Png> {
    let mut at = SIGNATURE.len();
    let mut header: Option<&[u8]> = None;
    let mut palette = Vec::new();
    let mut transparency = Vec::new();
    let mut idat = Vec::new();

    while let Some(head) = bytes.get(at..at + CHUNK_HEAD_LEN) {
        let len = u32::from_be_bytes([head[0], head[1], head[2], head[3]]) as usize;
        let start = at + CHUNK_HEAD_LEN;
        let data = bytes.get(start..start + len).ok_or("PNG ends early")?;

        match &head[4..] {
            b"IHDR" => header = Some(data),
            b"PLTE" => palette = data.to_vec(),
            b"tRNS" => transparency = data.to_vec(),
            b"IDAT" => idat.extend_from_slice(data),
            b"IEND" => break,
            _ => {}
        }
        at = start + len + CHUNK_TAIL_LEN;
    }

    let h = header
        .filter(|h| h.len() == IHDR_LEN)
        .ok_or("no PNG header")?;
    let be = |i: usize| u32::from_be_bytes([h[i], h[i + 1], h[i + 2], h[i + 3]]);
    let color = Color::from_code(h[9])?;
    let png = Png {
        width: be(0),
        height: be(4),
        depth: h[8],
        color,
        interlaced: h[12] == 1,
        palette,
        transparency,
        idat,
    };

    if !color.allows(png.depth) {
        return Err("bit depth does not fit the colour type".to_owned());
    }
    if png.width == 0 || png.height == 0 || png.idat.is_empty() {
        return Err("PNG without pixels".to_owned());
    }
    if color == Color::Indexed && (png.palette.is_empty() || !png.palette.len().is_multiple_of(3)) {
        return Err("PNG palette is missing or broken".to_owned());
    }

    Ok(png)
}

impl Png {
    /// The image XObject of the PDF: dictionary entries and stream data.
    pub(crate) fn xobject(&self, paper: Rgb) -> Result<(String, Vec<u8>)> {
        if self.interlaced {
            return Err("interlaced PNG; save it without interlacing".to_owned());
        }

        let (w, h) = (self.width, self.height);
        let size = format!("/Width {w} /Height {h}");
        if self.color.has_alpha() || self.depth > 8 || !self.transparency.is_empty() {
            let rgb = self.rgb_over(paper)?;
            return Ok((
                format!("{size} /ColorSpace /DeviceRGB /BitsPerComponent 8"),
                rgb,
            ));
        }

        let space = match self.color {
            Color::Gray => "/DeviceGray".to_owned(),
            Color::Rgb => "/DeviceRGB".to_owned(),
            _ => indexed(&self.palette),
        };
        let (colors, bits) = (self.color.channels(), self.depth);
        let dict = format!(
            "{size} /ColorSpace {space} /BitsPerComponent {bits} /Filter /FlateDecode \
             /DecodeParms << /Predictor 15 /Colors {colors} /BitsPerComponent {bits} /Columns {w} >>"
        );

        Ok((dict, self.idat.clone()))
    }

    /// 8-bit RGB, alpha (channel or `tRNS`) laid over `paper`; 16-bit samples keep their high
    /// byte, samples under 8 bits are stretched to 0..255.
    fn rgb_over(&self, paper: Rgb) -> Result<Vec<u8>> {
        let (w, h) = (self.width as usize, self.height as usize);
        let depth = usize::from(self.depth);
        let channels = self.color.channels();
        let row_bits = w.checked_mul(channels * depth).ok_or("PNG too large")?;
        let row_bytes = row_bits.div_ceil(BYTE_BITS);
        let expected = row_bytes
            .checked_add(1)
            .and_then(|r| r.checked_mul(h))
            .filter(|n| *n <= MAX_DECODED_BYTES)
            .ok_or("PNG too large")?;

        let raw = inflate::zlib(&self.idat, expected)?;
        if raw.len() != expected {
            return Err("PNG data is shorter than its size says".to_owned());
        }
        let pixel_bytes = (channels * depth / BYTE_BITS).max(1);
        let rows = unfilter(&raw, row_bytes, pixel_bytes, h)?;

        let max = (1_u32 << depth) - 1;
        let mut out = Vec::with_capacity(w * h * RGB_CHANNELS);
        for row in rows.chunks_exact(row_bytes) {
            for x in 0..w {
                // Raw samples at their own depth.
                let s: Vec<u32> = (0..channels)
                    .map(|c| sample(row, x * channels + c, depth))
                    .collect();
                let to8 = |v: u32| match depth {
                    16 => v >> 8,
                    8 => v,
                    _ => v * OPAQUE / max,
                };
                let (rgb, alpha) = match self.color {
                    Color::Gray => ([to8(s[0]); 3], self.key_alpha(&s)),
                    Color::GrayAlpha => ([to8(s[0]); 3], to8(s[1])),
                    Color::Rgb => ([to8(s[0]), to8(s[1]), to8(s[2])], self.key_alpha(&s)),
                    Color::Rgba => ([to8(s[0]), to8(s[1]), to8(s[2])], to8(s[3])),
                    Color::Indexed => {
                        let i = s[0] as usize;
                        let c =
                            |k: usize| u32::from(self.palette.get(i * 3 + k).copied().unwrap_or(0));
                        let a = self.transparency.get(i).map_or(OPAQUE, |a| u32::from(*a));
                        ([c(0), c(1), c(2)], a)
                    }
                };
                for (c, p) in rgb.into_iter().zip([paper.r, paper.g, paper.b]) {
                    let mixed = (c * alpha + u32::from(p) * (OPAQUE - alpha) + OPAQUE / 2) / OPAQUE;
                    out.push(mixed as u8);
                }
            }
        }

        Ok(out)
    }

    /// Gray and RGB: the pixel is see-through when it equals the colour `tRNS` names (one
    /// big-endian 16-bit value per channel).
    fn key_alpha(&self, samples: &[u32]) -> u32 {
        let key: Vec<u32> = self
            .transparency
            .as_chunks::<2>()
            .0
            .iter()
            .map(|b| u32::from(u16::from_be_bytes(*b)))
            .collect();

        match !key.is_empty() && key == samples {
            true => 0,
            false => OPAQUE,
        }
    }
}

/// Sample number `index` of a row at `depth` bits (1, 2, 4, 8 or 16), most significant first.
fn sample(row: &[u8], index: usize, depth: usize) -> u32 {
    let at = |i: usize| u32::from(row.get(i).copied().unwrap_or(0));
    match depth {
        16 => at(index * 2) << 8 | at(index * 2 + 1),
        8 => at(index),
        _ => {
            let bit = index * depth;
            let shift = BYTE_BITS - depth - bit % BYTE_BITS;
            (at(bit / BYTE_BITS) >> shift) & ((1 << depth) - 1)
        }
    }
}

/// `[/Indexed /DeviceRGB last <hex>]`.
fn indexed(palette: &[u8]) -> String {
    let mut hex = String::new();
    for b in palette {
        // Writing to a String cannot fail.
        let _ = write!(hex, "{b:02x}");
    }

    format!("[/Indexed /DeviceRGB {} <{hex}>]", palette.len() / 3 - 1)
}

/// Undoes the five scanline filters. `raw` holds, per row, a filter byte and `row_bytes`
/// bytes; `pixel_bytes` is the distance to the pixel on the left (at least one byte).
fn unfilter(raw: &[u8], row_bytes: usize, pixel_bytes: usize, rows: usize) -> Result<Vec<u8>> {
    let mut out = vec![0_u8; row_bytes * rows];

    for y in 0..rows {
        let line = &raw[y * (row_bytes + 1)..(y + 1) * (row_bytes + 1)];
        let kind = line[0];
        let (before, row) = out.split_at_mut(y * row_bytes);
        let above = match y {
            0 => &[][..],
            _ => &before[(y - 1) * row_bytes..],
        };

        for x in 0..row_bytes {
            let left = if x >= pixel_bytes {
                row[x - pixel_bytes]
            } else {
                0
            };
            let up = above.get(x).copied().unwrap_or(0);
            let corner = match x >= pixel_bytes {
                true => above.get(x - pixel_bytes).copied().unwrap_or(0),
                false => 0,
            };
            let predicted = match kind {
                0 => 0,
                FILTER_SUB => left,
                FILTER_UP => up,
                FILTER_AVERAGE => ((u16::from(left) + u16::from(up)) / 2) as u8,
                FILTER_PAETH => paeth(left, up, corner),
                other => return Err(format!("unknown PNG filter {other}")),
            };
            row[x] = line[1 + x].wrapping_add(predicted);
        }
    }

    Ok(out)
}

/// The neighbour (left, above, corner) closest to left + above - corner.
fn paeth(a: u8, b: u8, c: u8) -> u8 {
    let (ia, ib, ic) = (i16::from(a), i16::from(b), i16::from(c));
    let p = ia + ib - ic;
    let (da, db, dc) = ((p - ia).abs(), (p - ib).abs(), (p - ic).abs());

    match (da <= db && da <= dc, db <= dc) {
        (true, _) => a,
        (false, true) => b,
        (false, false) => c,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    const WHITE: Rgb = Rgb {
        r: 255,
        g: 255,
        b: 255,
    };

    /// zlib stream of stored blocks: the PDF reader does not care how it was compressed.
    /// The checksum is not checked by the inflater, so zero will do.
    fn zlib_stored(data: &[u8]) -> Vec<u8> {
        let len = data.len() as u16;
        let mut z = vec![0x78, 0x01, 0x01];
        z.extend_from_slice(&len.to_le_bytes());
        z.extend_from_slice(&(!len).to_le_bytes());
        z.extend_from_slice(data);
        z.extend_from_slice(&[0; 4]);
        z
    }

    fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
        out.extend_from_slice(&(data.len() as u32).to_be_bytes());
        out.extend_from_slice(kind);
        out.extend_from_slice(data);
        out.extend_from_slice(&[0; 4]);
    }

    /// A `w` x `h` PNG of the given type and depth whose scanlines are `raw` (filter bytes in).
    fn png(w: u32, h: u32, depth: u8, color: u8, raw: &[u8], palette: &[u8]) -> Vec<u8> {
        let mut head = Vec::new();
        head.extend_from_slice(&w.to_be_bytes());
        head.extend_from_slice(&h.to_be_bytes());
        head.extend_from_slice(&[depth, color, 0, 0, 0]);

        let mut out = SIGNATURE.to_vec();
        chunk(&mut out, b"IHDR", &head);
        if !palette.is_empty() {
            chunk(&mut out, b"PLTE", palette);
        }
        let z = zlib_stored(raw);
        let (first, second) = z.split_at(z.len() / 2);
        chunk(&mut out, b"IDAT", first);
        chunk(&mut out, b"IDAT", second);
        chunk(&mut out, b"IEND", &[]);
        out
    }

    /// RGBA, 3 x 1: opaque red stays; green at alpha 0 is the paper; black at alpha 128 over
    /// white is (0 * 128 + 255 * 127 + 127) / 255 = 127 (rounded).
    #[test]
    fn alpha_is_laid_over_the_paper() {
        let raw = [0, 255, 0, 0, 255, 0, 255, 0, 0, 0, 0, 0, 128];
        let png = parse(&png(3, 1, 8, 6, &raw, &[])).unwrap();

        let rgb = png.rgb_over(WHITE).unwrap();

        assert_eq!(rgb, [255, 0, 0, 255, 255, 255, 127, 127, 127]);
    }

    #[test]
    fn unfilters_sub_average_and_paeth() {
        // One row of 3 gray bytes, Sub: 10, +5, +5 -> 10, 15, 20.
        assert_eq!(unfilter(&[1, 10, 5, 5], 3, 1, 1), Ok(vec![10, 15, 20]));

        // Two rows of 2 bytes. Row 1 Average: floor((left + up) / 2) added to the stored byte.
        // Row 0 = 10, 20. Row 1 stored 4, 4: x0 = 4 + (0+10)/2 = 9; x1 = 4 + (9+20)/2 = 18.
        let raw = [0, 10, 20, FILTER_AVERAGE, 4, 4];
        assert_eq!(unfilter(&raw, 2, 1, 2), Ok(vec![10, 20, 9, 18]));

        // Paeth: row 1 x0: left 0, up 10, corner 0 -> p = 10, nearest is up (10); 3 + 10 = 13.
        // x1: left 13, up 20, corner 10 -> p = 23; distances 10, 3, 13 -> up (20); 3 + 20 = 23.
        let raw = [0, 10, 20, FILTER_PAETH, 3, 3];
        assert_eq!(unfilter(&raw, 2, 1, 2), Ok(vec![10, 20, 13, 23]));
    }

    #[test]
    fn unknown_filters_are_errors() {
        assert!(unfilter(&[9, 1], 1, 1, 1).is_err());
    }

    #[test]
    fn rgb_without_alpha_is_passed_through() {
        let raw = [0, 1, 2, 3];
        let png = parse(&png(1, 1, 8, 2, &raw, &[])).unwrap();

        let (dict, data) = png.xobject(WHITE).unwrap();

        assert!(dict.contains("/ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /FlateDecode"));
        assert!(dict.contains("/Predictor 15 /Colors 3 /BitsPerComponent 8 /Columns 1"));
        // The IDAT chunks joined are the zlib stream, untouched.
        assert_eq!(data, zlib_stored(&raw));
    }

    #[test]
    fn a_palette_becomes_an_indexed_space() {
        let palette = [255, 0, 0, 0, 0, 255];
        let png = parse(&png(2, 1, 1, 3, &[0, 0b0100_0000], &palette)).unwrap();

        let (dict, _) = png.xobject(WHITE).unwrap();

        assert!(dict.contains("/ColorSpace [/Indexed /DeviceRGB 1 <ff00000000ff>]"));
        assert!(dict.contains("/BitsPerComponent 1"));
        assert!(dict.contains("/Colors 1"));
    }

    /// Palette PNG, 1 bit, two pixels: index 0 (black, `tRNS` alpha 0) and index 1 (red).
    /// The see-through pixel prints as the paper, not as black.
    #[test]
    fn trns_makes_palette_pixels_see_through() {
        let mut bytes = png(2, 1, 1, 3, &[0, 0b0100_0000], &[0, 0, 0, 255, 0, 0]);
        // Right after PLTE: the chunk list is IHDR, PLTE, IDAT, IDAT, IEND.
        let at = bytes.windows(4).position(|w| w == b"IDAT").unwrap() - 4;
        let mut trns = Vec::new();
        chunk(&mut trns, b"tRNS", &[0]);
        bytes.splice(at..at, trns);
        let paper = Rgb {
            r: 250,
            g: 240,
            b: 230,
        };

        let (dict, data) = parse(&bytes).unwrap().xobject(paper).unwrap();

        assert!(dict.contains("/ColorSpace /DeviceRGB /BitsPerComponent 8"));
        assert_eq!(data, [250, 240, 230, 255, 0, 0]);
    }

    #[test]
    fn sixteen_bit_keeps_the_high_byte() {
        // Gray 16-bit, one pixel 0xAB12: output (0xAB, 0xAB, 0xAB).
        let png = parse(&png(1, 1, 16, 0, &[0, 0xAB, 0x12], &[])).unwrap();

        assert_eq!(png.rgb_over(WHITE).unwrap(), [0xAB; 3]);
    }

    #[test]
    fn interlaced_and_broken_files_are_errors() {
        let mut interlaced = png(1, 1, 8, 2, &[0, 1, 2, 3], &[]);
        // IHDR data starts at 16; the interlace byte is its last one.
        interlaced[16 + 12] = 1;
        let parsed = parse(&interlaced).unwrap();

        assert!(parsed.xobject(WHITE).is_err());
        assert!(parse(&SIGNATURE).is_err());
        assert!(parse(&png(1, 1, 3, 2, &[0, 1, 2, 3], &[])).is_err());
        assert!(parse(&png(1, 1, 8, 3, &[0, 0], &[])).is_err());
    }
}
