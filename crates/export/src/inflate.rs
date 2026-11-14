//! Inflate (RFC 1951) in a zlib wrapper (RFC 1950), for PNG images with an alpha channel: the
//! PDF cannot take those as they are. Small and plain instead of a dependency; the table walk
//! follows zlib's `puff.c`.

const MAX_BITS: usize = 15;
const END_OF_BLOCK: u16 = 256;
const LENGTH_FIRST: u16 = 257;
const FIXED_LITERALS: usize = 288;
const FIXED_DISTANCES: usize = 30;
/// zlib header: compression method 8 (deflate) in the low nibble of the first byte.
const ZLIB_METHOD_MASK: u8 = 0x0F;
const ZLIB_DEFLATE: u8 = 8;
const ZLIB_HEADER_LEN: usize = 2;

const BLOCK_STORED: u32 = 0;
const BLOCK_FIXED: u32 = 1;
const BLOCK_DYNAMIC: u32 = 2;

const LENGTH_BASE: [u16; 29] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131,
    163, 195, 227, 258,
];
const LENGTH_EXTRA: [u8; 29] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
];
const DISTANCE_BASE: [u16; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
    2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
];
const DISTANCE_EXTRA: [u8; 30] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
    13,
];
/// The order in which a dynamic block lists the code lengths of its length code.
const CODE_LENGTH_ORDER: [usize; 19] = [
    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
];
const REPEAT_PREVIOUS: u16 = 16;
const REPEAT_ZERO_SHORT: u16 = 17;

type Result<T> = std::result::Result<T, String>;

struct Bits<'a> {
    data: &'a [u8],
    at: usize,
    buffer: u32,
    count: u32,
}

impl Bits<'_> {
    /// `n` bits (at most 16), least significant first.
    fn take(&mut self, n: u32) -> Result<u32> {
        while self.count < n {
            let byte = *self.data.get(self.at).ok_or("data ends early")?;
            self.buffer |= u32::from(byte) << self.count;
            self.count += 8;
            self.at += 1;
        }

        let value = self.buffer & ((1 << n) - 1);
        self.buffer >>= n;
        self.count -= n;

        Ok(value)
    }
}

/// A canonical Huffman code: how many codes of each length, and the symbols in code order.
struct Huffman {
    count: [u16; MAX_BITS + 1],
    symbol: Vec<u16>,
}

impl Huffman {
    fn new(lengths: &[u8]) -> Result<Self> {
        let mut count = [0_u16; MAX_BITS + 1];
        for &len in lengths {
            count[usize::from(len)] += 1;
        }

        // More codes of a length than the shorter codes leave room for: not a prefix code.
        let mut left: i32 = 1;
        for &n in &count[1..] {
            left = (left << 1) - i32::from(n);
            if left < 0 {
                return Err("over-subscribed code".to_owned());
            }
        }

        let mut offset = [0_u16; MAX_BITS + 2];
        for len in 1..=MAX_BITS {
            offset[len + 1] = offset[len] + count[len];
        }
        let mut symbol = vec![0_u16; lengths.len()];
        for (sym, &len) in lengths.iter().enumerate() {
            if len == 0 {
                continue;
            }
            symbol[usize::from(offset[usize::from(len)])] = sym as u16;
            offset[usize::from(len)] += 1;
        }

        Ok(Self { count, symbol })
    }

    fn decode(&self, bits: &mut Bits) -> Result<u16> {
        let (mut code, mut first, mut index) = (0_i32, 0_i32, 0_i32);

        for len in 1..=MAX_BITS {
            code |= bits.take(1)? as i32;
            let count = i32::from(self.count[len]);
            if code - count < first {
                let at = usize::try_from(index + (code - first)).map_err(|e| e.to_string())?;
                return self.symbol.get(at).copied().ok_or("bad code".to_owned());
            }
            index += count;
            first = (first + count) << 1;
            code <<= 1;
        }

        Err("bad code".to_owned())
    }
}

/// The data of a zlib stream. Fails on bad data and when more than `limit` bytes come out
/// (a stream that claims to be small can inflate to gigabytes).
pub(crate) fn zlib(data: &[u8], limit: usize) -> Result<Vec<u8>> {
    let method = data.first().ok_or("empty zlib stream")? & ZLIB_METHOD_MASK;
    if method != ZLIB_DEFLATE {
        return Err("not a deflate stream".to_owned());
    }

    let mut bits = Bits {
        data,
        at: ZLIB_HEADER_LEN,
        buffer: 0,
        count: 0,
    };
    let mut out = Vec::new();

    loop {
        let last = bits.take(1)?;
        match bits.take(2)? {
            BLOCK_STORED => stored(&mut bits, &mut out)?,
            BLOCK_FIXED => {
                let (literals, distances) = fixed()?;
                codes(&mut bits, &mut out, &literals, &distances, limit)?;
            }
            BLOCK_DYNAMIC => {
                let (literals, distances) = dynamic(&mut bits)?;
                codes(&mut bits, &mut out, &literals, &distances, limit)?;
            }
            _ => return Err("bad block type".to_owned()),
        }

        if out.len() > limit {
            return Err("inflates to more than expected".to_owned());
        }
        if last == 1 {
            return Ok(out);
        }
    }
}

fn stored(bits: &mut Bits, out: &mut Vec<u8>) -> Result<()> {
    // The rest of the current byte is padding.
    bits.buffer = 0;
    bits.count = 0;

    let head = bits
        .data
        .get(bits.at..bits.at + 4)
        .ok_or("data ends early")?;
    let len = usize::from(u16::from_le_bytes([head[0], head[1]]));
    let not_len = u16::from_le_bytes([head[2], head[3]]);
    if not_len != !(len as u16) {
        return Err("bad stored block".to_owned());
    }

    let start = bits.at + 4;
    let block = bits.data.get(start..start + len).ok_or("data ends early")?;
    out.extend_from_slice(block);
    bits.at = start + len;

    Ok(())
}

fn fixed() -> Result<(Huffman, Huffman)> {
    let mut lengths = [0_u8; FIXED_LITERALS];
    for (sym, len) in lengths.iter_mut().enumerate() {
        *len = match sym {
            0..=143 => 8,
            144..=255 => 9,
            256..=279 => 7,
            _ => 8,
        };
    }

    Ok((
        Huffman::new(&lengths)?,
        Huffman::new(&[5; FIXED_DISTANCES])?,
    ))
}

fn dynamic(bits: &mut Bits) -> Result<(Huffman, Huffman)> {
    let literal_count = bits.take(5)? as usize + 257;
    let distance_count = bits.take(5)? as usize + 1;
    let code_count = bits.take(4)? as usize + 4;
    if literal_count > FIXED_LITERALS - 2 || distance_count > FIXED_DISTANCES {
        return Err("too many codes".to_owned());
    }

    let mut code_lengths = [0_u8; 19];
    for &slot in CODE_LENGTH_ORDER.iter().take(code_count) {
        code_lengths[slot] = bits.take(3)? as u8;
    }
    let length_code = Huffman::new(&code_lengths)?;

    let total = literal_count + distance_count;
    let mut lengths: Vec<u8> = Vec::with_capacity(total);
    while lengths.len() < total {
        let sym = length_code.decode(bits)?;
        let (value, repeat) = match sym {
            0..=15 => (sym as u8, 1),
            REPEAT_PREVIOUS => {
                let previous = *lengths.last().ok_or("repeat without a length")?;
                (previous, bits.take(2)? as usize + 3)
            }
            REPEAT_ZERO_SHORT => (0, bits.take(3)? as usize + 3),
            _ => (0, bits.take(7)? as usize + 11),
        };
        if lengths.len() + repeat > total {
            return Err("too many lengths".to_owned());
        }
        lengths.resize(lengths.len() + repeat, value);
    }

    let (literals, distances) = lengths.split_at(literal_count);
    Ok((Huffman::new(literals)?, Huffman::new(distances)?))
}

/// Literals and (length, distance) copies up to the end of the block.
fn codes(
    bits: &mut Bits,
    out: &mut Vec<u8>,
    literals: &Huffman,
    distances: &Huffman,
    limit: usize,
) -> Result<()> {
    loop {
        let sym = literals.decode(bits)?;
        if sym < END_OF_BLOCK {
            out.push(sym as u8);
            continue;
        }
        if sym == END_OF_BLOCK {
            return Ok(());
        }

        let l = usize::from(sym - LENGTH_FIRST);
        let extra = u32::from(*LENGTH_EXTRA.get(l).ok_or("bad length code")?);
        let length = usize::from(LENGTH_BASE[l]) + bits.take(extra)? as usize;

        let d = usize::from(distances.decode(bits)?);
        let extra = u32::from(*DISTANCE_EXTRA.get(d).ok_or("bad distance code")?);
        let distance = usize::from(DISTANCE_BASE[d]) + bits.take(extra)? as usize;
        if distance > out.len() {
            return Err("copy from before the start".to_owned());
        }
        if out.len() + length > limit {
            return Err("inflates to more than expected".to_owned());
        }

        // Byte by byte: a copy may overlap what it writes (distance 1 repeats one byte).
        for _ in 0..length {
            out.push(out[out.len() - distance]);
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    const TEXT: &str = "the quick brown fox jumps over the lazy dog ";

    /// `zlib.compress("the quick brown fox jumps over the lazy dog ")`, level 9: one fixed block.
    const FIXED: &str =
        "78da2bc94855282ccd4cce56482aca2fcf5348cbaf50c82acd2d2856c82f4b2d5228014ae72456552aa4e4a72b00007156101a";
    /// Compressed `(0..600).map(|i| b'a' + (i * i / 7 % 13))`, level 9: one dynamic block.
    const DYNAMIC: &str = "78daedcc410ec0300803c1b71a0cb884fcffda54ea2fda3d8f168039533d161aafe5da39b162121747dc5dc4b42a83a7c83a16acded4f0c28bb77c958fc28e4dbae1e95f7f757d039fe7f0bd";

    fn hex(text: &str) -> Vec<u8> {
        (0..text.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
            .collect()
    }

    #[test]
    fn inflates_a_fixed_block() {
        let out = zlib(&hex(FIXED), 1000).unwrap();

        assert_eq!(String::from_utf8_lossy(&out), TEXT);
    }

    #[test]
    fn inflates_a_dynamic_block() {
        let expected: Vec<u8> = (0..600_u32)
            .map(|i| b'a' + (i * i / 7 % 13) as u8)
            .collect();

        assert_eq!(zlib(&hex(DYNAMIC), 1000), Ok(expected));
    }

    /// A stored block: header, then LEN and NLEN, then the bytes as they are.
    #[test]
    fn inflates_a_stored_block() {
        let data = [0x78, 0x01, 0x01, 0x03, 0x00, 0xFC, 0xFF, b'a', b'b', b'c'];

        assert_eq!(zlib(&data, 10), Ok(b"abc".to_vec()));
    }

    #[test]
    fn stops_at_the_limit() {
        let data = hex(FIXED);

        assert!(zlib(&data, 10).is_err());
    }

    #[test]
    fn bad_data_is_an_error_not_a_panic() {
        assert!(zlib(&[], 10).is_err());
        assert!(zlib(&[0x78, 0xDA, 0xFF], 10).is_err());
        assert!(zlib(&[0x78, 0xDA], 10).is_err());
        assert!(zlib(&hex(FIXED)[..10], 100).is_err());
    }
}
