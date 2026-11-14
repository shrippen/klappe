//! A PNG as a PDF image, without an image library.
//!
//! PDF's Flate filter with predictor 15 reads PNG's own scanline format, so a PNG without an
//! alpha channel (up to 8 bits per sample) goes in as it is: the compressed data is copied,
//! never decoded. `tRNS` stays that way where the PDF can say it with a colour key mask
//! (`/Mask`): one colour of a gray or RGB image, or a run of palette entries that are fully
//! see-through. The rest is inflated, unfiltered and laid over the paper colour as plain RGB:
//! an alpha channel, partial alpha in a palette, scattered see-through entries, 16 bits. That
//! costs memory, so it is refused above `MAX_DECODED_BYTES`.

use std::fmt::Write;

use crate::inflate;
use crate::palette::Rgb;

pub(crate) const SIGNATURE: [u8; 8] = *b"\x89PNG\r\n\x1a\n";
const IHDR_LEN: usize = 13;
const CHUNK_HEAD_LEN: usize = 8;
const CHUNK_TAIL_LEN: usize = 4;
/// Most bytes each buffer of the decoding path may hold (inflated scanlines, unfiltered rows,
/// RGB output of 3 bytes per pixel). All three exist together, so the peak is about three
/// times this; it keeps a small file from asking for gigabytes.
const MAX_DECODED_BYTES: usize = 1 << 28;
const BYTE_BITS: usize = 8;
const FILTER_SUB: u8 = 1;
const FILTER_UP: u8 = 2;
const FILTER_AVERAGE: u8 = 3;
const FILTER_PAETH: u8 = 4;
const OPAQUE: u32 = 255;
const RGB_CHANNELS: usize = 3;
const MAX_CHANNELS: usize = 4;

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
    if !png.trns_fits() {
        return Err("PNG tRNS chunk does not fit the image".to_owned());
    }

    Ok(png)
}

impl Png {
    /// Whether `tRNS` is valid: a palette has at most one alpha per entry, a colour key stays
    /// within the sample range. Images with an alpha channel ignore `tRNS`.
    fn trns_fits(&self) -> bool {
        if self.color.has_alpha() {
            return true;
        }
        if self.color == Color::Indexed {
            return self.transparency.len() <= self.palette.len() / 3;
        }

        let max = (1_u32 << self.depth) - 1;

        self.transparency
            .as_chunks::<2>()
            .0
            .iter()
            .all(|b| u32::from(u16::from_be_bytes(*b)) <= max)
    }

    /// The image XObject of the PDF: dictionary entries and stream data.
    pub(crate) fn xobject(&self, paper: Rgb) -> Result<(String, Vec<u8>)> {
        if self.interlaced {
            return Err("interlaced PNG; save it without interlacing".to_owned());
        }

        let (w, h) = (self.width, self.height);
        let size = format!("/Width {w} /Height {h}");
        let mask = match self.color.has_alpha() || self.depth > 8 {
            true => None,
            false => self.mask(),
        };
        let Some(mask) = mask else {
            let rgb = self.rgb_over(paper)?;
            return Ok((
                format!("{size} /ColorSpace /DeviceRGB /BitsPerComponent 8"),
                rgb,
            ));
        };

        let space = match self.color {
            Color::Gray => "/DeviceGray".to_owned(),
            Color::Rgb => "/DeviceRGB".to_owned(),
            _ => indexed(&self.palette),
        };
        let (colors, bits) = (self.color.channels(), self.depth);
        let dict = format!(
            "{size} /ColorSpace {space} /BitsPerComponent {bits} /Filter /FlateDecode \
             /DecodeParms << /Predictor 15 /Colors {colors} /BitsPerComponent {bits} /Columns {w} >>{mask}"
        );

        Ok((dict, self.idat.clone()))
    }

    /// The `/Mask` entry that says what `tRNS` says, `""` without see-through pixels; `None`
    /// when a mask cannot say it. A mask is one range per colour component, and a pixel is
    /// not painted when all its components are in range: so a palette can only hide a run of
    /// entries, and only entirely.
    fn mask(&self) -> Option<String> {
        let key: Vec<u16> = self
            .transparency
            .as_chunks::<2>()
            .0
            .iter()
            .map(|b| u16::from_be_bytes(*b))
            .collect();
        let range = |values: &[u16]| {
            let pairs: Vec<String> = values.iter().map(|v| format!("{v} {v}")).collect();
            format!(" /Mask [{}]", pairs.join(" "))
        };

        match self.color {
            Color::Gray => Some(key.get(..1).map(range).unwrap_or_default()),
            Color::Rgb => Some(key.get(..3).map(range).unwrap_or_default()),
            Color::Indexed => {
                let hidden: Vec<usize> = (0..self.transparency.len())
                    .filter(|i| self.transparency[*i] == 0)
                    .collect();
                let partial = self
                    .transparency
                    .iter()
                    .any(|a| *a != 0 && *a != OPAQUE as u8);
                let (Some(first), Some(last)) = (hidden.first(), hidden.last()) else {
                    return (!partial).then(String::new);
                };
                let one_run = last - first + 1 == hidden.len();

                (one_run && !partial).then(|| format!(" /Mask [{first} {last}]"))
            }
            _ => Some(String::new()),
        }
    }

    /// 8-bit RGB, alpha (channel or `tRNS`) laid over `paper`; 16-bit samples keep their high
    /// byte, samples under 8 bits are stretched to 0..255.
    pub(crate) fn rgb_over(&self, paper: Rgb) -> Result<Vec<u8>> {
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

        let out_bytes = w
            .checked_mul(h)
            .and_then(|n| n.checked_mul(RGB_CHANNELS))
            .filter(|n| *n <= MAX_DECODED_BYTES)
            .ok_or("PNG too large to print: its decoded RGB data would exceed the limit")?;

        let raw = inflate::zlib(&self.idat, expected)?;
        if raw.len() != expected {
            return Err("PNG data is shorter than its size says".to_owned());
        }
        let pixel_bytes = (channels * depth / BYTE_BITS).max(1);
        let rows = unfilter(&raw, row_bytes, pixel_bytes, h)?;

        let max = (1_u32 << depth) - 1;
        let key: Vec<u32> = self
            .transparency
            .as_chunks::<2>()
            .0
            .iter()
            .map(|b| u32::from(u16::from_be_bytes(*b)))
            .collect();
        let mut out = Vec::with_capacity(out_bytes);
        for row in rows.chunks_exact(row_bytes) {
            for x in 0..w {
                // Raw samples at their own depth.
                let mut s = [0_u32; MAX_CHANNELS];
                for (c, value) in s.iter_mut().enumerate().take(channels) {
                    *value = sample(row, x * channels + c, depth);
                }
                let keyed = key_alpha(&key, &s[..channels]);
                let to8 = |v: u32| match depth {
                    16 => v >> 8,
                    8 => v,
                    _ => v * OPAQUE / max,
                };
                let (rgb, alpha) = match self.color {
                    Color::Gray => ([to8(s[0]); 3], keyed),
                    Color::GrayAlpha => ([to8(s[0]); 3], to8(s[1])),
                    Color::Rgb => ([to8(s[0]), to8(s[1]), to8(s[2])], keyed),
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
}

/// Gray and RGB: the pixel is see-through when it equals the colour `tRNS` names (one 16-bit
/// value per channel, `key`).
fn key_alpha(key: &[u32], samples: &[u32]) -> u32 {
    match !key.is_empty() && key == samples {
        true => 0,
        false => OPAQUE,
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

    /// A PNG of the given type, depth and size whose `tRNS` chunk is `trns`; the pixel data
    /// is `raw` (scanlines with filter bytes), or just a few bytes for the huge sizes that the
    /// tests only parse.
    fn png_trns(
        w: u32,
        h: u32,
        depth: u8,
        color: u8,
        raw: &[u8],
        palette: &[u8],
        trns: &[u8],
    ) -> Vec<u8> {
        let mut bytes = png(w, h, depth, color, raw, palette);
        // The chunk list is IHDR, [PLTE], IDAT, IDAT, IEND: tRNS goes in before the first IDAT.
        let at = bytes.windows(4).position(|w| w == b"IDAT").unwrap() - 4;
        let mut chunk_bytes = Vec::new();
        chunk(&mut chunk_bytes, b"tRNS", trns);
        bytes.splice(at..at, chunk_bytes);
        bytes
    }

    const PAPER: Rgb = Rgb {
        r: 250,
        g: 240,
        b: 230,
    };

    /// A palette PNG whose see-through entries are a run of indices, and gray and RGB PNGs
    /// with a colour key, keep their compressed data: the PDF masks the colour (`/Mask`).
    #[test]
    fn trns_becomes_a_colour_key_mask() {
        let palette = [0, 0, 0, 255, 0, 0];
        let indexed = png_trns(2, 1, 1, 3, &[0, 0b0100_0000], &palette, &[0]);
        let gray = png_trns(1, 1, 2, 0, &[0, 0b1100_0000], &[], &[0, 3]);
        let rgb = png_trns(1, 1, 8, 2, &[0, 1, 2, 3], &[], &[0, 1, 0, 2, 0, 3]);

        for (bytes, mask) in [
            (indexed, "/Mask [0 0]"),
            (gray, "/Mask [3 3]"),
            (rgb, "/Mask [1 1 2 2 3 3]"),
        ] {
            let (dict, data) = parse(&bytes).unwrap().xobject(PAPER).unwrap();

            assert!(dict.contains(mask), "{dict}");
            assert!(dict.contains("/Filter /FlateDecode"), "{dict}");
            assert_eq!(data, parse(&bytes).unwrap().idat);
        }
    }

    /// The mask also lets 1-bit drawings with a see-through ground through without decoding:
    /// 16000 x 16000 px would be 768 MB as RGB.
    #[test]
    fn a_huge_masked_drawing_is_not_decoded() {
        let bytes = png_trns(16000, 16000, 1, 3, &[0, 0], &[0, 0, 0, 255, 255, 255], &[0]);

        let (dict, data) = parse(&bytes).unwrap().xobject(PAPER).unwrap();

        assert!(dict.contains("/Mask [0 0]"), "{dict}");
        assert!(dict.contains("/Columns 16000"), "{dict}");
        assert!(data.len() < 100);
    }

    /// Real partial alpha has to be decoded: entry 1 is half see-through: black over the
    /// paper (250, 240, 230) is (0 * 128 + p * 127 + 127) / 255 = 125, 120, 115 (rounded).
    #[test]
    fn partial_palette_alpha_is_laid_over_the_paper() {
        let palette = [0, 0, 0, 0, 0, 0, 255, 0, 0];
        let raw = [0, 0b0001_1011];
        let bytes = png_trns(3, 1, 2, 3, &raw, &palette, &[0, 128]);

        let (dict, data) = parse(&bytes).unwrap().xobject(PAPER).unwrap();

        assert!(dict.contains("/DeviceRGB /BitsPerComponent 8"), "{dict}");
        assert_eq!(data, [250, 240, 230, 125, 120, 115, 255, 0, 0]);
    }

    /// Two see-through entries that are not next to each other cannot be one mask range.
    #[test]
    fn scattered_see_through_entries_are_decoded() {
        let palette = [10, 10, 10, 20, 20, 20, 30, 30, 30];
        let bytes = png_trns(3, 1, 2, 3, &[0, 0b0001_1011], &palette, &[0, 255, 0]);

        let (_, data) = parse(&bytes).unwrap().xobject(PAPER).unwrap();

        assert_eq!(data, [250, 240, 230, 20, 20, 20, 250, 240, 230]);
    }

    /// What has to be decoded is refused when its RGB output would not fit the limit, before
    /// anything is allocated; a few bytes of file claim 65536 x 65000 px.
    #[test]
    fn a_huge_image_that_needs_decoding_is_refused() {
        let scattered = png_trns(65536, 65000, 1, 3, &[0, 0], &[0; 9], &[0, 255, 0]);
        let rgba = png(100_000, 100_000, 8, 6, &[0, 0], &[]);

        for bytes in [scattered, rgba] {
            let started = std::time::Instant::now();
            let error = parse(&bytes).unwrap().xobject(PAPER).unwrap_err();

            assert!(error.contains("too large"), "{error}");
            assert!(started.elapsed().as_secs() < 1);
        }
    }

    /// The standard allows no colour key above the sample range and no more alphas than
    /// palette entries; such a `tRNS` is an error, like the other broken chunks.
    #[test]
    fn an_invalid_trns_is_an_error() {
        let palette = [0, 0, 0, 255, 0, 0];
        let long = png_trns(2, 1, 1, 3, &[0, 0b0100_0000], &palette, &[0, 255, 255]);
        let gray = png_trns(1, 1, 2, 0, &[0, 0b1100_0000], &[], &[0, 4]);
        let rgb = png_trns(1, 1, 8, 2, &[0, 1, 2, 3], &[], &[0, 1, 0, 2, 1, 0]);

        for bytes in [long, gray, rgb] {
            let error = parse(&bytes).unwrap_err();

            assert!(error.contains("tRNS"), "{error}");
        }
    }

    /// Review of PR 20: images with an alpha channel ignore `tRNS`, so its value must not
    /// decide whether they load.
    #[test]
    fn an_alpha_image_ignores_trns() {
        let rgba = png_trns(1, 1, 8, 6, &[0, 1, 2, 3, 255], &[], &[1, 0]);

        assert!(parse(&rgba).is_ok());
    }

    /// 16 bits are decoded; the colour key compares the full 16-bit value.
    #[test]
    fn a_sixteen_bit_colour_key_is_see_through() {
        let raw = [0, 0xAB, 0x12, 0xAB, 0x13];
        let bytes = png_trns(2, 1, 16, 0, &raw, &[], &[0xAB, 0x12]);

        let (_, data) = parse(&bytes).unwrap().xobject(PAPER).unwrap();

        assert_eq!(data, [250, 240, 230, 0xAB, 0xAB, 0xAB]);
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
