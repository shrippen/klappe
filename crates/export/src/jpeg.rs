//! A baseline JPEG as 8-bit RGB, without an image library.
//!
//! Supported: SOF0 and SOF1 with 8 bits per sample, one interleaved scan, Huffman coding,
//! restart intervals, 1 (gray) or 3 (YCbCr; RGB if Adobe says so or the ids are `R G B`)
//! components, any sampling factors that divide the largest one (4:4:4, 4:2:2, 4:2:0, 4:4:0,
//! 4:1:1). Refused with a message: progressive, lossless, hierarchical and arithmetic coding,
//! several scans, 12 bits, 4 components (CMYK/YCCK).
//!
//! Choices that differ from libjpeg:
//! - The IDCT is a separable `f32` one. It is exact up to rounding, cannot overflow on
//!   corrupt coefficients, and needs no scaling constants; libjpeg's integer "islow" differs
//!   by at most 1 per sample.
//! - Subsampled chroma is repeated (box filter), not interpolated: libjpeg's default "fancy"
//!   upsampling smooths the edges, so 4:2:0 pictures differ from it there. `djpeg -nosmooth`
//!   is the reference.
//! - The EXIF orientation is ignored: pixels come out as stored.
//! - Data that ends before the scan is complete is an error, not padded with gray.

use crate::rgb8::Rgb8;

const MARKER_PREFIX: u8 = 0xFF;
const SOI: u8 = 0xD8;
const EOI: u8 = 0xD9;
const SOF0: u8 = 0xC0;
const SOF1: u8 = 0xC1;
const SOF2: u8 = 0xC2;
const DHT: u8 = 0xC4;
const JPG: u8 = 0xC8;
const DAC: u8 = 0xCC;
const SOF_LAST: u8 = 0xCF;
const SOS: u8 = 0xDA;
const DQT: u8 = 0xDB;
const DRI: u8 = 0xDD;
const APP14: u8 = 0xEE;
const RST_FIRST: u8 = 0xD0;
const RST_LAST: u8 = 0xD7;
const STUFFED_ZERO: u8 = 0x00;
/// Markers that stand alone, without a length: TEM and RSTn.
const TEM: u8 = 0x01;

const SAMPLE_BITS: u8 = 8;
const BLOCK: usize = 8;
const BLOCK_LEN: usize = BLOCK * BLOCK;
const MAX_TABLES: usize = 4;
const MAX_CODE_BITS: usize = 16;
const MAX_SYMBOLS: usize = 256;
/// Largest sampling factor, and the most blocks in one MCU (T.81, B.2.3).
const MAX_SAMPLING: u8 = 4;
const MAX_MCU_BLOCKS: usize = 10;
const MAX_DC_CATEGORY: u8 = 11;
const MAX_AC_CATEGORY: u8 = 10;
const EOB: u8 = 0x00;
const ZRL: u8 = 0xF0;
const GRAY_COMPONENTS: usize = 1;
const YCC_COMPONENTS: usize = 3;
const CMYK_COMPONENTS: usize = 4;
const RGB_CHANNELS: usize = 3;
const ADOBE_ID: &[u8] = b"Adobe";
/// Adobe APP14: "Adobe", version, two flag words, then the transform byte.
const ADOBE_TRANSFORM_AT: usize = 11;
const ADOBE_NO_TRANSFORM: u8 = 0;
const RGB_IDS: [u8; 3] = *b"RGB";
const LEVEL_SHIFT: f32 = 128.0;
/// Most bytes each buffer of the decoder may hold (the component planes together, the RGB
/// output), like `png`; it keeps a small file from asking for gigabytes.
const MAX_DECODED_BYTES: usize = 1 << 28;

/// Zigzag position to row-major index (T.81, figure A.6).
const ZIGZAG: [usize; BLOCK_LEN] = [
    0, 1, 8, 16, 9, 2, 3, 10, 17, 24, 32, 25, 18, 11, 4, 5, 12, 19, 26, 33, 40, 48, 41, 34, 27, 20,
    13, 6, 7, 14, 21, 28, 35, 42, 49, 56, 57, 50, 43, 36, 29, 22, 15, 23, 30, 37, 44, 51, 58, 59,
    52, 45, 38, 31, 39, 46, 53, 60, 61, 54, 47, 55, 62, 63,
];

type Result<T> = std::result::Result<T, String>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ColorSpace {
    Gray,
    YCbCr,
    Rgb,
}

/// Decodes a baseline JPEG; see the module comment for what that covers.
pub(crate) fn decode_jpeg(bytes: &[u8]) -> Result<Rgb8> {
    if bytes.get(..2) != Some(&[MARKER_PREFIX, SOI]) {
        return Err("not a JPEG".to_owned());
    }

    let mut dec = Decoder::default();
    let mut at = 2;

    loop {
        let marker = next_marker(bytes, &mut at)?;
        match marker {
            EOI => return Err("JPEG has no scan".to_owned()),
            TEM | RST_FIRST..=RST_LAST | SOI => continue,
            _ => {}
        }

        let len = bytes
            .get(at..at + 2)
            .map(|b| usize::from(u16::from_be_bytes([b[0], b[1]])))
            .filter(|l| *l >= 2)
            .ok_or("JPEG segment is cut off")?;
        let data = bytes
            .get(at + 2..at + len)
            .ok_or("JPEG segment is cut off")?;
        at += len;

        match marker {
            DQT => dec.quant(data)?,
            DHT => dec.huffman(data)?,
            SOF0 | SOF1 => dec.frame(data)?,
            SOF2 => return Err("progressive JPEG is not supported".to_owned()),
            JPG | DAC => {}
            m if (SOF0..=SOF_LAST).contains(&m) => {
                return Err(
                    "this JPEG coding (arithmetic, lossless or hierarchical) is not supported"
                        .to_owned(),
                )
            }
            DRI => dec.restart(data)?,
            APP14 => dec.adobe(data),
            SOS => return dec.scan(data, bytes.get(at..).unwrap_or_default()),
            _ => {}
        }
    }
}

/// The marker after `at`, skipping bytes that are no marker and fill bytes; `at` ends after it.
fn next_marker(bytes: &[u8], at: &mut usize) -> Result<u8> {
    while let Some(&b) = bytes.get(*at) {
        *at += 1;
        if b != MARKER_PREFIX {
            continue;
        }

        while bytes.get(*at) == Some(&MARKER_PREFIX) {
            *at += 1;
        }
        let Some(&m) = bytes.get(*at) else {
            break;
        };
        *at += 1;
        if m != STUFFED_ZERO {
            return Ok(m);
        }
    }

    Err("JPEG ends early".to_owned())
}

#[derive(Debug, Clone, Copy, Default)]
struct Component {
    id: u8,
    h: usize,
    v: usize,
    quant: usize,
}

#[derive(Debug, Default)]
struct Decoder {
    width: usize,
    height: usize,
    comps: Vec<Component>,
    quant: [Option<[u16; BLOCK_LEN]>; MAX_TABLES],
    dc: [Option<Huffman>; MAX_TABLES],
    ac: [Option<Huffman>; MAX_TABLES],
    restart_interval: usize,
    adobe_transform: Option<u8>,
}

impl Decoder {
    fn quant(&mut self, mut data: &[u8]) -> Result<()> {
        while let Some((&head, rest)) = data.split_first() {
            let (wide, id) = (head >> 4, usize::from(head & 0x0F));
            let size = BLOCK_LEN * (usize::from(wide) + 1);
            if wide > 1 || id >= MAX_TABLES {
                return Err("JPEG quantisation table is broken".to_owned());
            }

            let raw = rest
                .get(..size)
                .ok_or("JPEG quantisation table is cut off")?;
            let mut table = [0_u16; BLOCK_LEN];
            for (i, slot) in ZIGZAG.iter().enumerate() {
                table[*slot] = match wide {
                    0 => u16::from(raw[i]),
                    _ => u16::from_be_bytes([raw[2 * i], raw[2 * i + 1]]),
                };
            }

            self.quant[id] = Some(table);
            data = &rest[size..];
        }

        Ok(())
    }

    fn huffman(&mut self, mut data: &[u8]) -> Result<()> {
        while let Some((&head, rest)) = data.split_first() {
            let (class, id) = (head >> 4, usize::from(head & 0x0F));
            if class > 1 || id >= MAX_TABLES {
                return Err("JPEG Huffman table is broken".to_owned());
            }

            let counts = rest
                .get(..MAX_CODE_BITS)
                .ok_or("JPEG Huffman table is cut off")?;
            let total: usize = counts.iter().map(|c| usize::from(*c)).sum();
            let symbols = rest
                .get(MAX_CODE_BITS..MAX_CODE_BITS + total)
                .ok_or("JPEG Huffman table is cut off")?;
            let table = Huffman::new(counts, symbols)?;

            match class {
                0 => self.dc[id] = Some(table),
                _ => self.ac[id] = Some(table),
            }
            data = &rest[MAX_CODE_BITS + total..];
        }

        Ok(())
    }

    fn frame(&mut self, data: &[u8]) -> Result<()> {
        if !self.comps.is_empty() {
            return Err("JPEG has two frame headers".to_owned());
        }

        let head = data.get(..6).ok_or("JPEG frame header is cut off")?;
        let count = usize::from(head[5]);
        let specs = data
            .get(6..6 + 3 * count)
            .ok_or("JPEG frame header is cut off")?;
        let (height, width) = (
            usize::from(u16::from_be_bytes([head[1], head[2]])),
            usize::from(u16::from_be_bytes([head[3], head[4]])),
        );

        if head[0] != SAMPLE_BITS {
            return Err(format!(
                "JPEG with {} bits per sample is not supported",
                head[0]
            ));
        }
        if width == 0 || height == 0 {
            return Err("JPEG without pixels".to_owned());
        }
        if count == CMYK_COMPONENTS {
            return Err("JPEG with 4 colour channels (CMYK) is not supported".to_owned());
        }
        if count != GRAY_COMPONENTS && count != YCC_COMPONENTS {
            return Err(format!(
                "JPEG with {count} colour channels is not supported"
            ));
        }

        let mut comps = Vec::with_capacity(count);
        for spec in specs.as_chunks::<3>().0 {
            let (h, v) = (spec[1] >> 4, spec[1] & 0x0F);
            let quant = usize::from(spec[2]);
            if !(1..=MAX_SAMPLING).contains(&h) || !(1..=MAX_SAMPLING).contains(&v) {
                return Err("JPEG sampling factor is out of range".to_owned());
            }
            if quant >= MAX_TABLES || comps.iter().any(|c: &Component| c.id == spec[0]) {
                return Err("JPEG component is broken".to_owned());
            }
            comps.push(Component {
                id: spec[0],
                h: usize::from(h),
                v: usize::from(v),
                quant,
            });
        }

        // A scan of one component is not interleaved: its MCU is a single block.
        if count == GRAY_COMPONENTS {
            comps[0].h = 1;
            comps[0].v = 1;
        }

        let (hmax, vmax) = (
            comps.iter().map(|c| c.h).max(),
            comps.iter().map(|c| c.v).max(),
        );
        let (hmax, vmax) = (hmax.unwrap_or(1), vmax.unwrap_or(1));
        if comps.iter().any(|c| hmax % c.h != 0 || vmax % c.v != 0) {
            return Err("JPEG sampling factors do not divide evenly".to_owned());
        }
        if comps.iter().map(|c| c.h * c.v).sum::<usize>() > MAX_MCU_BLOCKS {
            return Err("JPEG MCU has too many blocks".to_owned());
        }

        self.width = width;
        self.height = height;
        self.comps = comps;

        Ok(())
    }

    fn restart(&mut self, data: &[u8]) -> Result<()> {
        let b = data.get(..2).ok_or("JPEG restart interval is cut off")?;
        self.restart_interval = usize::from(u16::from_be_bytes([b[0], b[1]]));

        Ok(())
    }

    /// Only the transform byte matters; a malformed segment is ignored like any other APP.
    fn adobe(&mut self, data: &[u8]) {
        if data.starts_with(ADOBE_ID) {
            self.adobe_transform = data.get(ADOBE_TRANSFORM_AT).copied();
        }
    }

    fn color_space(&self) -> ColorSpace {
        if self.comps.len() == GRAY_COMPONENTS {
            return ColorSpace::Gray;
        }

        let ids: Vec<u8> = self.comps.iter().map(|c| c.id).collect();
        if self.adobe_transform == Some(ADOBE_NO_TRANSFORM) || ids == RGB_IDS {
            return ColorSpace::Rgb;
        }

        ColorSpace::YCbCr
    }

    /// Reads the scan header and the entropy-coded data after it, and builds the picture.
    fn scan(&self, head: &[u8], data: &[u8]) -> Result<Rgb8> {
        if self.comps.is_empty() {
            return Err("JPEG scan before the frame header".to_owned());
        }

        let count = usize::from(*head.first().ok_or("JPEG scan header is cut off")?);
        let specs = head
            .get(1..1 + 2 * count)
            .ok_or("JPEG scan header is cut off")?;
        if count != self.comps.len() {
            return Err("JPEG with several scans is not supported".to_owned());
        }

        let mut tables = Vec::with_capacity(count);
        for comp in &self.comps {
            let spec = specs
                .as_chunks::<2>()
                .0
                .iter()
                .find(|s| s[0] == comp.id)
                .ok_or("JPEG scan names an unknown component")?;
            let (dc, ac) = (usize::from(spec[1] >> 4), usize::from(spec[1] & 0x0F));
            let dc = self
                .dc
                .get(dc)
                .and_then(Option::as_ref)
                .ok_or("JPEG lacks a Huffman table")?;
            let ac = self
                .ac
                .get(ac)
                .and_then(Option::as_ref)
                .ok_or("JPEG lacks a Huffman table")?;
            let quant = self.quant[comp.quant]
                .as_ref()
                .ok_or("JPEG lacks a quantisation table")?;
            tables.push((dc, ac, quant));
        }

        let planes = self.decode_planes(&tables, data)?;

        self.to_rgb(&planes)
    }

    fn decode_planes(
        &self,
        tables: &[(&Huffman, &Huffman, &[u16; BLOCK_LEN])],
        data: &[u8],
    ) -> Result<Vec<Plane>> {
        let (hmax, vmax) = self.max_sampling();
        let mcus_x = self.width.div_ceil(BLOCK * hmax);
        let mcus_y = self.height.div_ceil(BLOCK * vmax);

        // Sizes first: nothing is allocated before they are known to be fine.
        let sizes: Vec<(usize, usize)> = self
            .comps
            .iter()
            .map(|c| (mcus_x * c.h * BLOCK, mcus_y * c.v * BLOCK))
            .collect();
        let total = sizes.iter().try_fold(0_usize, |sum, (w, h)| {
            w.checked_mul(*h).and_then(|n| sum.checked_add(n))
        });
        let output = self
            .width
            .checked_mul(self.height)
            .and_then(|n| n.checked_mul(RGB_CHANNELS));
        if total.is_none_or(|t| t > MAX_DECODED_BYTES)
            || output.is_none_or(|o| o > MAX_DECODED_BYTES)
        {
            return Err("JPEG is too large to decode".to_owned());
        }

        let mut planes: Vec<Plane> = sizes.iter().map(|(w, h)| Plane::new(*w, *h)).collect();
        let mut bits = Bits::new(data);
        let mut preds = vec![0_i32; self.comps.len()];
        let idct = Idct::new();
        let mut count = 0_usize;

        for my in 0..mcus_y {
            for mx in 0..mcus_x {
                if self.restart_interval > 0
                    && count > 0
                    && count.is_multiple_of(self.restart_interval)
                {
                    bits.restart()?;
                    preds.fill(0);
                }
                count += 1;

                for (i, comp) in self.comps.iter().enumerate() {
                    let (dc, ac, quant) = tables[i];
                    for by in 0..comp.v {
                        for bx in 0..comp.h {
                            let coefs = block(&mut bits, dc, ac, quant, &mut preds[i])?;
                            let (x, y) = ((mx * comp.h + bx) * BLOCK, (my * comp.v + by) * BLOCK);
                            idct.write(&coefs, &mut planes[i], x, y);
                        }
                    }
                }
            }
        }

        Ok(planes)
    }

    fn max_sampling(&self) -> (usize, usize) {
        let h = self.comps.iter().map(|c| c.h).max().unwrap_or(1);
        let v = self.comps.iter().map(|c| c.v).max().unwrap_or(1);

        (h, v)
    }

    /// Repeats subsampled planes up to full size and converts to RGB.
    fn to_rgb(&self, planes: &[Plane]) -> Result<Rgb8> {
        let (hmax, vmax) = self.max_sampling();
        let space = self.color_space();
        let mut pixels = Vec::with_capacity(self.width * self.height * RGB_CHANNELS);

        let columns: Vec<Vec<usize>> = self
            .comps
            .iter()
            .map(|c| (0..self.width).map(|x| x * c.h / hmax).collect())
            .collect();

        for y in 0..self.height {
            let rows: Vec<&[u8]> = self
                .comps
                .iter()
                .zip(planes)
                .map(|(c, p)| p.row(y * c.v / vmax))
                .collect();

            for x in 0..self.width {
                let sample = |i: usize| rows[i][columns[i][x]];
                pixels.extend_from_slice(&match space {
                    ColorSpace::Gray => [sample(0); RGB_CHANNELS],
                    ColorSpace::Rgb => [sample(0), sample(1), sample(2)],
                    ColorSpace::YCbCr => ycc_to_rgb(sample(0), sample(1), sample(2)),
                });
            }
        }

        Ok(Rgb8 {
            width: u32::try_from(self.width).map_err(|e| e.to_string())?,
            height: u32::try_from(self.height).map_err(|e| e.to_string())?,
            pixels,
        })
    }
}

/// JFIF's YCbCr, full range (T.871).
fn ycc_to_rgb(y: u8, cb: u8, cr: u8) -> [u8; 3] {
    let (y, cb, cr) = (
        f32::from(y),
        f32::from(cb) - LEVEL_SHIFT,
        f32::from(cr) - LEVEL_SHIFT,
    );

    [
        to_byte(y + 1.402 * cr),
        to_byte(y - 0.344_136 * cb - 0.714_136 * cr),
        to_byte(y + 1.772 * cb),
    ]
}

fn to_byte(v: f32) -> u8 {
    v.round().clamp(0.0, 255.0) as u8
}

/// One component's samples, padded to whole MCUs.
#[derive(Debug)]
struct Plane {
    stride: usize,
    samples: Vec<u8>,
}

impl Plane {
    fn new(width: usize, height: usize) -> Self {
        Self {
            stride: width,
            samples: vec![0; width * height],
        }
    }

    fn row(&self, y: usize) -> &[u8] {
        &self.samples[y * self.stride..(y + 1) * self.stride]
    }
}

/// Canonical Huffman code: per length the smallest and largest code and where its symbols
/// start (T.81, F.2.2.3).
#[derive(Debug, Clone)]
struct Huffman {
    min_code: [i32; MAX_CODE_BITS + 1],
    max_code: [i32; MAX_CODE_BITS + 1],
    first: [usize; MAX_CODE_BITS + 1],
    symbols: Vec<u8>,
}

impl Huffman {
    fn new(counts: &[u8], symbols: &[u8]) -> Result<Self> {
        if symbols.len() > MAX_SYMBOLS {
            return Err("JPEG Huffman table has too many symbols".to_owned());
        }

        let mut table = Self {
            min_code: [0; MAX_CODE_BITS + 1],
            max_code: [-1; MAX_CODE_BITS + 1],
            first: [0; MAX_CODE_BITS + 1],
            symbols: symbols.to_vec(),
        };
        let (mut code, mut next) = (0_i32, 0_usize);

        for len in 1..=MAX_CODE_BITS {
            let n = usize::from(counts[len - 1]);
            table.first[len] = next;
            table.min_code[len] = code;
            code += n as i32;
            next += n;
            if n > 0 {
                table.max_code[len] = code - 1;
            }
            // More codes than the length has room for.
            if code > 1 << len {
                return Err("JPEG Huffman table is over-subscribed".to_owned());
            }
            code <<= 1;
        }

        Ok(table)
    }

    fn decode(&self, bits: &mut Bits) -> Result<u8> {
        let mut code = 0_i32;

        for len in 1..=MAX_CODE_BITS {
            code = (code << 1) | bits.take(1)? as i32;
            if code > self.max_code[len] {
                continue;
            }

            let index = self.first[len] + (code - self.min_code[len]) as usize;
            return self
                .symbols
                .get(index)
                .copied()
                .ok_or_else(|| "JPEG Huffman code is broken".to_owned());
        }

        Err("JPEG Huffman code is broken".to_owned())
    }
}

/// The entropy-coded data: bits from the most significant end, `FF 00` read as `FF`.
#[derive(Debug)]
struct Bits<'a> {
    data: &'a [u8],
    at: usize,
    buffer: u32,
    held: u32,
}

impl<'a> Bits<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            at: 0,
            buffer: 0,
            held: 0,
        }
    }

    /// The next `n` (at most 16) bits. A marker inside the data counts as its end.
    fn take(&mut self, n: u32) -> Result<u32> {
        while self.held < n {
            let byte = *self.data.get(self.at).ok_or("JPEG ends early")?;
            self.at += 1;

            if byte == MARKER_PREFIX {
                let next = self.data.get(self.at).copied();
                if next != Some(STUFFED_ZERO) {
                    return Err("JPEG data ends early".to_owned());
                }
                self.at += 1;
            }
            self.buffer = (self.buffer << 8) | u32::from(byte);
            self.held += 8;
        }

        self.held -= n;

        Ok((self.buffer >> self.held) & ((1 << n) - 1))
    }

    /// Drops the bits left in the last byte and steps over the RSTn marker.
    fn restart(&mut self) -> Result<()> {
        self.held = 0;
        self.buffer = 0;

        let marker = self
            .data
            .get(self.at..self.at + 2)
            .ok_or("JPEG ends early")?;
        if marker[0] != MARKER_PREFIX || !(RST_FIRST..=RST_LAST).contains(&marker[1]) {
            return Err("JPEG restart marker is missing".to_owned());
        }
        self.at += 2;

        Ok(())
    }
}

/// The sign-extended value of `n` bits `v` (T.81, F.2.2.1).
fn extend(v: u32, n: u8) -> i32 {
    if n == 0 {
        return 0;
    }

    let v = v as i32;
    if v < 1 << (n - 1) {
        return v - (1 << n) + 1;
    }

    v
}

/// One block's coefficients, dequantised, in row-major order.
fn block(
    bits: &mut Bits,
    dc: &Huffman,
    ac: &Huffman,
    quant: &[u16; BLOCK_LEN],
    pred: &mut i32,
) -> Result<[f32; BLOCK_LEN]> {
    let mut out = [0.0_f32; BLOCK_LEN];

    let category = dc.decode(bits)?;
    if category > MAX_DC_CATEGORY {
        return Err("JPEG DC value is out of range".to_owned());
    }
    *pred = pred.saturating_add(extend(bits.take(u32::from(category))?, category));
    out[0] = *pred as f32 * f32::from(quant[0]);

    let mut k = 1;
    while k < BLOCK_LEN {
        let symbol = ac.decode(bits)?;
        if symbol == EOB {
            break;
        }
        if symbol == ZRL {
            k += 16;
            continue;
        }

        let (run, size) = (usize::from(symbol >> 4), symbol & 0x0F);
        k += run;
        if size > MAX_AC_CATEGORY || k >= BLOCK_LEN {
            return Err("JPEG AC value is out of range".to_owned());
        }

        let index = ZIGZAG[k];
        out[index] = extend(bits.take(u32::from(size))?, size) as f32 * f32::from(quant[index]);
        k += 1;
    }

    Ok(out)
}

/// Separable 8x8 inverse DCT: `out[y][x] = sum c[y][v] c[x][u] F[v][u]`.
#[derive(Debug)]
struct Idct {
    /// `cos[x][u] = C(u)/2 * cos((2x+1) u pi / 16)`, with `C(0) = 1/sqrt 2`.
    cos: [[f32; BLOCK]; BLOCK],
}

impl Idct {
    fn new() -> Self {
        let mut cos = [[0.0_f32; BLOCK]; BLOCK];

        for (x, row) in cos.iter_mut().enumerate() {
            for (u, slot) in row.iter_mut().enumerate() {
                let scale = if u == 0 {
                    std::f64::consts::FRAC_1_SQRT_2
                } else {
                    1.0
                };
                let angle = (2 * x + 1) as f64 * u as f64 * std::f64::consts::PI / 16.0;
                *slot = (scale * 0.5 * angle.cos()) as f32;
            }
        }

        Self { cos }
    }

    /// Transforms `coefs` and stores the level-shifted samples at (`x0`, `y0`) in `plane`.
    fn write(&self, coefs: &[f32; BLOCK_LEN], plane: &mut Plane, x0: usize, y0: usize) {
        let mut rows = [[0.0_f32; BLOCK]; BLOCK];

        // Along each row of coefficients.
        for (v, row) in rows.iter_mut().enumerate() {
            for (x, slot) in row.iter_mut().enumerate() {
                *slot = (0..BLOCK)
                    .map(|u| self.cos[x][u] * coefs[v * BLOCK + u])
                    .sum();
            }
        }

        // Down each column.
        for (y, cos) in self.cos.iter().enumerate() {
            let at = (y0 + y) * plane.stride + x0;
            for (x, out) in plane.samples[at..at + BLOCK].iter_mut().enumerate() {
                let sum: f32 = (0..BLOCK).map(|v| cos[v] * rows[v][x]).sum();
                *out = to_byte(sum + LEVEL_SHIFT);
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// Largest allowed difference per sample against `djpeg -nosmooth` (libjpeg-turbo,
    /// integer IDCT): the two IDCTs and colour conversions round differently.
    const TOLERANCE: u8 = 2;

    fn check(jpeg: &[u8], reference: &[u8]) {
        let pic = decode_jpeg(jpeg).unwrap();

        assert_eq!((pic.width, pic.height), (21, 13));
        assert_eq!(pic.pixels.len(), reference.len());

        let worst = pic
            .pixels
            .iter()
            .zip(reference)
            .map(|(a, b)| a.abs_diff(*b))
            .max()
            .unwrap();
        assert!(worst <= TOLERANCE, "largest difference {worst}");
    }

    macro_rules! reference {
        ($name:ident, $file:literal) => {
            #[test]
            fn $name() {
                check(
                    include_bytes!(concat!("../tests/data/", $file, ".jpg")),
                    include_bytes!(concat!("../tests/data/", $file, ".rgb")),
                );
            }
        };
    }

    reference!(matches_444, "jpeg444");
    reference!(matches_422, "jpeg422");
    reference!(matches_420, "jpeg420");
    reference!(matches_440, "jpeg440");
    reference!(matches_411, "jpeg411");
    reference!(matches_gray, "jpeggray");
    reference!(matches_restarts, "jpegrst");

    #[test]
    fn sof1_decodes_like_sof0() {
        let mut jpeg = include_bytes!("../tests/data/jpeg444.jpg").to_vec();
        let at = jpeg
            .windows(2)
            .position(|w| w == [MARKER_PREFIX, SOF0])
            .unwrap();
        jpeg[at + 1] = SOF1;

        check(&jpeg, include_bytes!("../tests/data/jpeg444.rgb"));
    }

    #[test]
    fn progressive_is_refused() {
        let err = decode_jpeg(include_bytes!("../tests/data/jpegprog.jpg")).unwrap_err();

        assert!(err.contains("progressive"), "{err}");
    }

    #[test]
    fn cmyk_is_refused() {
        let err = decode_jpeg(include_bytes!("../tests/data/jpegcmyk.jpg")).unwrap_err();

        assert!(err.contains("CMYK"), "{err}");
    }

    #[test]
    fn not_a_jpeg_is_refused() {
        assert!(decode_jpeg(b"\x89PNG").is_err());
        assert!(decode_jpeg(&[]).is_err());
    }

    #[test]
    fn huge_frame_is_refused_before_allocating() {
        let mut jpeg = include_bytes!("../tests/data/jpeg444.jpg").to_vec();
        let at = jpeg
            .windows(2)
            .position(|w| w == [MARKER_PREFIX, SOF0])
            .unwrap();
        // 65535 x 65535: 12 GB of RGB.
        jpeg[at + 5..at + 9].copy_from_slice(&[0xFF; 4]);

        let err = decode_jpeg(&jpeg).unwrap_err();

        assert!(err.contains("too large"), "{err}");
    }

    /// xorshift: a fixed sequence, so a failure can be repeated.
    fn next(state: &mut u64) -> u64 {
        *state ^= *state << 13;
        *state ^= *state >> 7;
        *state ^= *state << 17;

        *state
    }

    const FILES: [&[u8]; 6] = [
        include_bytes!("../tests/data/jpeg444.jpg"),
        include_bytes!("../tests/data/jpeg420.jpg"),
        include_bytes!("../tests/data/jpeg411.jpg"),
        include_bytes!("../tests/data/jpeggray.jpg"),
        include_bytes!("../tests/data/jpegrst.jpg"),
        include_bytes!("../tests/data/jpegprog.jpg"),
    ];

    #[test]
    fn truncated_files_do_not_panic() {
        for file in FILES {
            for len in 0..file.len() {
                let _ = decode_jpeg(&file[..len]);
            }
        }
    }

    #[test]
    fn damaged_files_do_not_panic() {
        let mut state = 0x9E37_79B9_7F4A_7C15_u64;

        for file in FILES {
            for _ in 0..3000 {
                let mut damaged = file.to_vec();
                for _ in 0..=next(&mut state) % 4 {
                    let at = next(&mut state) as usize % damaged.len();
                    damaged[at] = next(&mut state) as u8;
                }

                let _ = decode_jpeg(&damaged);
            }
        }
    }
}
