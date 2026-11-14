//! RGB pixels as a PNG file: 8 bits, no alpha, no interlacing.
//!
//! Each row is filtered with the one of the five filters whose output has the smallest sum of
//! absolute values (read as signed bytes), the heuristic the PNG specification suggests:
//! small bytes compress best. Flat areas come out as zeros under Up or Sub.

use crate::deflate;

const BPP: usize = 3;
const BIT_DEPTH: u8 = 8;
const COLOR_RGB: u8 = 2;
const UNIT_METRE: u8 = 1;
const INCH_M: f64 = 0.0254;

const FILTER_NONE: u8 = 0;
const FILTER_SUB: u8 = 1;
const FILTER_UP: u8 = 2;
const FILTER_AVERAGE: u8 = 3;
const FILTER_PAETH: u8 = 4;
const FILTERS: [u8; 5] = [
    FILTER_NONE,
    FILTER_SUB,
    FILTER_UP,
    FILTER_AVERAGE,
    FILTER_PAETH,
];

const CRC_POLY: u32 = 0xEDB8_8320;
const CRC_TABLE: [u32; 256] = {
    let mut table = [0_u32; 256];
    let mut n = 0;
    while n < 256 {
        let mut c = n as u32;
        let mut k = 0;
        while k < 8 {
            c = match c & 1 {
                1 => CRC_POLY ^ (c >> 1),
                _ => c >> 1,
            };
            k += 1;
        }
        table[n] = c;
        n += 1;
    }

    table
};

/// `rgb` is `width * height * 3` bytes, row by row; `dpi` goes into the `pHYs` chunk so a
/// printout has the size of the sheet.
pub(crate) fn encode(width: u32, height: u32, rgb: &[u8], dpi: f64) -> Vec<u8> {
    let mut out = crate::png::SIGNATURE.to_vec();

    let mut header = Vec::new();
    header.extend_from_slice(&width.to_be_bytes());
    header.extend_from_slice(&height.to_be_bytes());
    header.extend_from_slice(&[BIT_DEPTH, COLOR_RGB, 0, 0, 0]);
    chunk(&mut out, b"IHDR", &header);

    let per_metre = ((dpi / INCH_M).round() as u32).to_be_bytes();
    let mut physical = Vec::new();
    physical.extend_from_slice(&per_metre);
    physical.extend_from_slice(&per_metre);
    physical.push(UNIT_METRE);
    chunk(&mut out, b"pHYs", &physical);

    let row_bytes = width as usize * BPP;
    chunk(&mut out, b"IDAT", &deflate::zlib(&filtered(rgb, row_bytes)));
    chunk(&mut out, b"IEND", &[]);

    out
}

fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let start = out.len();
    out.extend_from_slice(kind);
    out.extend_from_slice(data);
    let crc = crc32(&out[start..]);
    out.extend_from_slice(&crc.to_be_bytes());
}

fn crc32(data: &[u8]) -> u32 {
    !data.iter().fold(!0_u32, |crc, &byte| {
        CRC_TABLE[usize::from(crc as u8 ^ byte)] ^ (crc >> 8)
    })
}

/// Every row behind its filter byte.
fn filtered(rgb: &[u8], row_bytes: usize) -> Vec<u8> {
    if row_bytes == 0 {
        return Vec::new();
    }

    let zeros = vec![0_u8; row_bytes];
    let mut out = Vec::with_capacity(rgb.len() + rgb.len() / row_bytes);
    let mut candidate = vec![0_u8; row_bytes];
    let mut best = vec![0_u8; row_bytes];

    for (y, row) in rgb.chunks_exact(row_bytes).enumerate() {
        let above = match y {
            0 => &zeros[..],
            _ => &rgb[(y - 1) * row_bytes..y * row_bytes],
        };

        let (mut best_filter, mut best_cost) = (FILTER_NONE, u64::MAX);
        for filter in FILTERS {
            apply(filter, row, above, &mut candidate);
            let cost = candidate
                .iter()
                .map(|b| u64::from((*b as i8).unsigned_abs()))
                .sum();
            if cost < best_cost {
                (best_filter, best_cost) = (filter, cost);
                std::mem::swap(&mut best, &mut candidate);
            }
        }

        out.push(best_filter);
        out.extend_from_slice(&best);
    }

    out
}

fn apply(filter: u8, row: &[u8], above: &[u8], out: &mut [u8]) {
    for i in 0..row.len() {
        let left = if i >= BPP { row[i - BPP] } else { 0 };
        let up = above[i];
        let up_left = if i >= BPP { above[i - BPP] } else { 0 };

        let predicted = match filter {
            FILTER_SUB => left,
            FILTER_UP => up,
            FILTER_AVERAGE => ((u16::from(left) + u16::from(up)) / 2) as u8,
            FILTER_PAETH => paeth(left, up, up_left),
            _ => 0,
        };
        out[i] = row[i].wrapping_sub(predicted);
    }
}

fn paeth(a: u8, b: u8, c: u8) -> u8 {
    let (ia, ib, ic) = (i16::from(a), i16::from(b), i16::from(c));
    let p = ia + ib - ic;
    let (pa, pb, pc) = ((p - ia).abs(), (p - ib).abs(), (p - ic).abs());

    match (pa <= pb && pa <= pc, pb <= pc) {
        (true, _) => a,
        (false, true) => b,
        (false, false) => c,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::palette::Rgb;
    use crate::png;

    const PAPER: Rgb = Rgb {
        r: 255,
        g: 255,
        b: 255,
    };

    fn gradient(width: u32, height: u32) -> Vec<u8> {
        let mut out = Vec::new();
        for y in 0..height {
            for x in 0..width {
                out.extend_from_slice(&[(x * 3 + y) as u8, (y * 5) as u8, ((x ^ y) * 7) as u8]);
            }
        }

        out
    }

    fn decode(file: &[u8]) -> (u32, u32, Vec<u8>) {
        let parsed = png::parse(file).unwrap();

        (parsed.width, parsed.height, parsed.rgb_over(PAPER).unwrap())
    }

    #[test]
    fn crc_matches_the_known_value() {
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
        assert_eq!(crc32(b"IEND"), 0xAE42_6082);
    }

    #[test]
    fn the_existing_decoder_reads_back_every_byte() {
        for (w, h) in [(1, 1), (7, 5), (64, 33), (301, 200)] {
            let pixels = gradient(w, h);
            let (dw, dh, back) = decode(&encode(w, h, &pixels, 150.0));

            assert_eq!((dw, dh), (w, h));
            assert!(back == pixels, "{w}x{h}");
        }
    }

    #[test]
    fn noise_comes_back_too() {
        let (w, h) = (97_u32, 61_u32);
        let mut state = 9_u32;
        let pixels: Vec<u8> = (0..w * h * 3)
            .map(|_| {
                state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                (state >> 24) as u8
            })
            .collect();

        assert!(decode(&encode(w, h, &pixels, 150.0)).2 == pixels);
    }

    #[test]
    fn the_file_has_the_chunks_in_order_with_valid_checks() {
        let file = encode(4, 3, &gradient(4, 3), 150.0);
        let mut at = png::SIGNATURE.len();
        let mut kinds = Vec::new();

        while at < file.len() {
            let len = u32::from_be_bytes(file[at..at + 4].try_into().unwrap()) as usize;
            let body = &file[at + 4..at + 8 + len];
            let crc = u32::from_be_bytes(file[at + 8 + len..at + 12 + len].try_into().unwrap());
            assert_eq!(crc, crc32(body));
            kinds.push(String::from_utf8_lossy(&body[..4]).into_owned());
            at += 12 + len;
        }

        assert_eq!(kinds, ["IHDR", "pHYs", "IDAT", "IEND"]);
        // 150 dpi is 5906 pixels per metre.
        assert!(file.windows(4).any(|w| w == 5906_u32.to_be_bytes()));
    }

    #[test]
    fn flat_rows_pick_a_filter_that_leaves_zeros() {
        let pixels = vec![200_u8; 8 * 4 * 3];
        let rows = filtered(&pixels, 8 * 3);

        // Row 0 has nothing above: Sub. Later rows: Up, all zeros but nothing to lose.
        assert_eq!(rows[0], FILTER_SUB);
        assert_eq!(rows[1 + 24], FILTER_UP);
        assert!(rows[1 + 24 + 1..2 + 24 + 24].iter().all(|b| *b == 0));
    }

    #[test]
    fn paeth_picks_the_nearest_predictor() {
        assert_eq!(paeth(10, 20, 10), 20);
        assert_eq!(paeth(20, 10, 10), 20);
        assert_eq!(paeth(10, 10, 20), 10);
    }
}
