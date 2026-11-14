//! zlib compression (RFC 1950/1951) for the PNG writer.
//!
//! LZ77 with hash chains finds repeats; every block of up to `BLOCK_TOKENS` tokens gets its own
//! dynamic Huffman codes. Dynamic over fixed codes because a plan is mostly paper colour and
//! flat fills: the literal and distance statistics are very skewed and per-block tables beat
//! the fixed ones by a wide margin; the table header costs about 100 bytes per block.
//!
//! ```text
//! bytes ─► tokens (literal | length+distance) ─► per block: code lengths ─► bits
//! ```

use std::cmp::Reverse;
use std::collections::BinaryHeap;

const WINDOW: usize = 1 << 15;
const WINDOW_MASK: usize = WINDOW - 1;
const MIN_MATCH: usize = 3;
const MAX_MATCH: usize = 258;
const HASH_BITS: u32 = 15;
const HASH_MUL: u32 = 0x9E37_79B1;
/// Candidates looked at per position: speed against size.
const MAX_CHAIN: usize = 64;
/// A match this long is taken without looking further.
const NICE_MATCH: usize = 128;
/// A 3-byte match that far back costs more bits than three literals.
const FAR_SHORT_MATCH: usize = 4096;
const BLOCK_TOKENS: usize = 1 << 16;
const NO_POS: u32 = u32::MAX;

const END_OF_BLOCK: usize = 256;
const LITERAL_CODES: usize = 286;
const DISTANCE_CODES: usize = 30;
const LENGTH_FIRST_CODE: usize = 257;
const LENGTH_258_CODE: usize = 285;
const MAX_CODE_BITS: u8 = 15;
const MAX_LENGTH_CODE_BITS: u8 = 7;
const LENGTH_CODES: usize = 19;
const CODE_LENGTH_ORDER: [usize; LENGTH_CODES] = [
    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
];
const REPEAT_PREVIOUS: usize = 16;
const REPEAT_ZERO_SHORT: usize = 17;
const REPEAT_ZERO_LONG: usize = 18;
/// Run lengths each repeat code covers and the bits it spends on the count (RFC 1951, 3.2.7).
const REPEAT_PREVIOUS_MIN: usize = 3;
const REPEAT_PREVIOUS_MAX: usize = 6;
const REPEAT_PREVIOUS_BITS: u32 = 2;
const REPEAT_ZERO_SHORT_MIN: usize = 3;
const REPEAT_ZERO_SHORT_BITS: u32 = 3;
const REPEAT_ZERO_LONG_MIN: usize = 11;
const REPEAT_ZERO_LONG_MAX: usize = 138;
const REPEAT_ZERO_LONG_BITS: u32 = 7;

const ZLIB_HEADER: [u8; 2] = [0x78, 0xDA];
const ADLER_MOD: u32 = 65521;
/// Bytes that can be summed before the 32-bit sums could overflow.
const ADLER_CHUNK: usize = 5552;

const BLOCK_DYNAMIC: u32 = 2;

const LENGTH_EXTRA: [u8; 29] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
];
const DISTANCE_EXTRA: [u8; DISTANCE_CODES] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
    13,
];

/// First value of each code: each starts where the one before it ends.
const fn bases<const N: usize>(first: u16, extra: &[u8; N]) -> [u16; N] {
    let mut out = [0_u16; N];
    let mut next = first;
    let mut i = 0;
    while i < N {
        out[i] = next;
        next += 1 << extra[i];
        i += 1;
    }

    out
}

const LENGTH_BASE: [u16; 29] = {
    let mut base = bases(MIN_MATCH as u16, &LENGTH_EXTRA);
    // Code 285 is the length 258 alone; code 284 stops one short of it.
    base[28] = MAX_MATCH as u16;
    base
};
const DISTANCE_BASE: [u16; DISTANCE_CODES] = bases(1, &DISTANCE_EXTRA);

#[derive(Debug, Clone, Copy)]
enum Token {
    Literal(u8),
    Copy { length: u16, distance: u16 },
}

/// `data` as a zlib stream.
pub(crate) fn zlib(data: &[u8]) -> Vec<u8> {
    let mut out = BitWriter::default();
    out.bytes.extend_from_slice(&ZLIB_HEADER);

    let mut finder = Finder::new(data.len());
    let mut pos = 0;
    let mut tokens: Vec<Token> = Vec::with_capacity(BLOCK_TOKENS);

    loop {
        tokens.clear();
        while tokens.len() < BLOCK_TOKENS && pos < data.len() {
            let (length, distance) = finder.find(data, pos);
            let end = match length {
                0 => {
                    tokens.push(Token::Literal(data[pos]));
                    finder.insert(data, pos);
                    pos + 1
                }
                _ => {
                    tokens.push(Token::Copy {
                        length: length as u16,
                        distance: distance as u16,
                    });
                    for p in pos..pos + length {
                        finder.insert(data, p);
                    }
                    pos + length
                }
            };
            pos = end;
        }

        let last = match pos >= data.len() {
            true => Final::Yes,
            false => Final::No,
        };
        block(&mut out, &tokens, last);
        if last == Final::Yes {
            break;
        }
    }

    out.align();
    out.bytes.extend_from_slice(&adler32(data).to_be_bytes());

    out.bytes
}

pub(crate) fn adler32(data: &[u8]) -> u32 {
    let (mut a, mut b) = (1_u32, 0_u32);

    for chunk in data.chunks(ADLER_CHUNK) {
        for &byte in chunk {
            a += u32::from(byte);
            b += a;
        }
        a %= ADLER_MOD;
        b %= ADLER_MOD;
    }

    b << 16 | a
}

/// Finds the longest earlier copy of the bytes at a position, through hash chains of
/// 3-byte prefixes.
struct Finder {
    head: Vec<u32>,
    prev: Vec<u32>,
}

impl Finder {
    fn new(len: usize) -> Self {
        Self {
            head: vec![NO_POS; 1 << HASH_BITS],
            prev: vec![NO_POS; len.min(WINDOW)],
        }
    }

    fn hash(data: &[u8], pos: usize) -> usize {
        let v =
            u32::from(data[pos]) << 16 | u32::from(data[pos + 1]) << 8 | u32::from(data[pos + 2]);

        (v.wrapping_mul(HASH_MUL) >> (32 - HASH_BITS)) as usize
    }

    fn insert(&mut self, data: &[u8], pos: usize) {
        if pos + MIN_MATCH > data.len() {
            return;
        }

        let h = Self::hash(data, pos);
        self.prev[pos & WINDOW_MASK] = self.head[h];
        self.head[h] = pos as u32;
    }

    /// (length, distance); length 0 when nothing worth a copy is there.
    fn find(&self, data: &[u8], pos: usize) -> (usize, usize) {
        if pos + MIN_MATCH > data.len() {
            return (0, 0);
        }

        let max = MAX_MATCH.min(data.len() - pos);
        let mut candidate = self.head[Self::hash(data, pos)];
        let (mut best_len, mut best_dist) = (MIN_MATCH - 1, 0);
        let mut chain = MAX_CHAIN;

        while candidate != NO_POS && chain > 0 {
            let at = candidate as usize;
            let distance = pos - at;
            if distance > WINDOW {
                break;
            }

            // The byte past the best match so far rules most candidates out cheaply.
            if best_len >= max || data[at + best_len] == data[pos + best_len] {
                let len = data[at..at + max]
                    .iter()
                    .zip(&data[pos..pos + max])
                    .take_while(|(a, b)| a == b)
                    .count();
                if len > best_len {
                    (best_len, best_dist) = (len, distance);
                    if len >= NICE_MATCH || len == max {
                        break;
                    }
                }
            }

            // Slots are reused every `WINDOW` positions: a link that does not go back is stale.
            let next = self.prev[at & WINDOW_MASK];
            if next != NO_POS && next as usize >= at {
                break;
            }
            candidate = next;
            chain -= 1;
        }

        match best_len < MIN_MATCH || (best_len == MIN_MATCH && best_dist > FAR_SHORT_MATCH) {
            true => (0, 0),
            false => (best_len, best_dist),
        }
    }
}

#[derive(Default)]
struct BitWriter {
    bytes: Vec<u8>,
    buffer: u64,
    count: u32,
}

impl BitWriter {
    /// `n` bits (at most 32) of `value`, least significant first.
    fn put(&mut self, value: u32, n: u32) {
        self.buffer |= u64::from(value) << self.count;
        self.count += n;

        while self.count >= 8 {
            self.bytes.push(self.buffer as u8);
            self.buffer >>= 8;
            self.count -= 8;
        }
    }

    /// A Huffman code goes out most significant bit first, so it is reversed once here.
    fn code(&mut self, code: u16, len: u8) {
        self.put(
            u32::from(code.reverse_bits() >> (16 - u16::from(len))),
            u32::from(len),
        );
    }

    fn align(&mut self) {
        if self.count > 0 {
            self.bytes.push(self.buffer as u8);
            self.buffer = 0;
            self.count = 0;
        }
    }
}

fn length_code(length: usize) -> usize {
    match length {
        MAX_MATCH => LENGTH_258_CODE - LENGTH_FIRST_CODE,
        _ => LENGTH_BASE.partition_point(|b| usize::from(*b) <= length) - 1,
    }
}

fn distance_code(distance: usize) -> usize {
    DISTANCE_BASE.partition_point(|b| usize::from(*b) <= distance) - 1
}

/// Code lengths for `freq`, none longer than `limit`. A lone symbol gets one bit.
fn code_lengths(freq: &[u32], limit: u8) -> Vec<u8> {
    let mut freq = freq.to_vec();

    loop {
        let lengths = huffman_lengths(&freq);
        if lengths.iter().all(|l| *l <= limit) {
            return lengths;
        }

        // Too deep: flatten the counts and build again; equal counts give a balanced tree.
        for f in freq.iter_mut().filter(|f| **f > 0) {
            *f = (*f / 2).max(1);
        }
    }
}

fn huffman_lengths(freq: &[u32]) -> Vec<u8> {
    let mut lengths = vec![0_u8; freq.len()];
    let used: Vec<usize> = (0..freq.len()).filter(|i| freq[*i] > 0).collect();
    match used.len() {
        0 => return lengths,
        1 => {
            lengths[used[0]] = 1;
            return lengths;
        }
        _ => {}
    }

    // Nodes 0..n are the leaves; a merged node's parent is filled in once it is merged.
    let mut parent: Vec<usize> = vec![usize::MAX; used.len()];
    let mut heap: BinaryHeap<Reverse<(u64, usize)>> = used
        .iter()
        .enumerate()
        .map(|(node, sym)| Reverse((u64::from(freq[*sym]), node)))
        .collect();

    while heap.len() > 1 {
        let (Some(Reverse((fa, a))), Some(Reverse((fb, b)))) = (heap.pop(), heap.pop()) else {
            break;
        };
        let node = parent.len();
        parent.push(usize::MAX);
        parent[a] = node;
        parent[b] = node;
        heap.push(Reverse((fa + fb, node)));
    }

    for (leaf, sym) in used.iter().enumerate() {
        let (mut depth, mut at) = (0_u8, leaf);
        while parent[at] != usize::MAX {
            at = parent[at];
            depth = depth.saturating_add(1);
        }
        lengths[*sym] = depth;
    }

    lengths
}

/// Canonical codes for `lengths` (RFC 1951, 3.2.2).
fn canonical(lengths: &[u8]) -> Vec<u16> {
    let mut per_length = [0_u16; MAX_CODE_BITS as usize + 1];
    for &l in lengths.iter().filter(|l| **l > 0) {
        per_length[usize::from(l)] += 1;
    }

    let mut next = [0_u16; MAX_CODE_BITS as usize + 2];
    for bits in 1..=usize::from(MAX_CODE_BITS) {
        next[bits + 1] = (next[bits] + per_length[bits]) << 1;
    }

    lengths
        .iter()
        .map(|&l| {
            if l == 0 {
                return 0;
            }
            let code = next[usize::from(l)];
            next[usize::from(l)] += 1;

            code
        })
        .collect()
}

/// The lengths as the code-length alphabet sees them: (symbol, extra bits value, extra bits).
fn run_lengths(lengths: &[u8]) -> Vec<(usize, u32, u32)> {
    let mut out = Vec::new();
    let mut i = 0;

    while i < lengths.len() {
        let value = lengths[i];
        let run = lengths[i..].iter().take_while(|l| **l == value).count();

        if value == 0 && run >= REPEAT_ZERO_SHORT_MIN {
            let n = run.min(REPEAT_ZERO_LONG_MAX);
            match n >= REPEAT_ZERO_LONG_MIN {
                true => out.push((
                    REPEAT_ZERO_LONG,
                    (n - REPEAT_ZERO_LONG_MIN) as u32,
                    REPEAT_ZERO_LONG_BITS,
                )),
                false => out.push((
                    REPEAT_ZERO_SHORT,
                    (n - REPEAT_ZERO_SHORT_MIN) as u32,
                    REPEAT_ZERO_SHORT_BITS,
                )),
            }
            i += n;
            continue;
        }

        out.push((usize::from(value), 0, 0));
        i += 1;

        // Further copies of a non-zero length repeat the one just written.
        let mut repeats = run - 1;
        while value != 0 && repeats >= REPEAT_PREVIOUS_MIN {
            let n = repeats.min(REPEAT_PREVIOUS_MAX);
            out.push((
                REPEAT_PREVIOUS,
                (n - REPEAT_PREVIOUS_MIN) as u32,
                REPEAT_PREVIOUS_BITS,
            ));
            repeats -= n;
            i += n;
        }
    }

    out
}

/// Whether a block is the last of the stream (BFINAL).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Final {
    Yes,
    No,
}

fn block(out: &mut BitWriter, tokens: &[Token], last: Final) {
    let mut literal_freq = vec![0_u32; LITERAL_CODES];
    let mut distance_freq = vec![0_u32; DISTANCE_CODES];
    literal_freq[END_OF_BLOCK] = 1;
    for token in tokens {
        match *token {
            Token::Literal(b) => literal_freq[usize::from(b)] += 1,
            Token::Copy { length, distance } => {
                literal_freq[LENGTH_FIRST_CODE + length_code(usize::from(length))] += 1;
                distance_freq[distance_code(usize::from(distance))] += 1;
            }
        }
    }

    let literal_lengths = code_lengths(&literal_freq, MAX_CODE_BITS);
    let mut distance_lengths = code_lengths(&distance_freq, MAX_CODE_BITS);
    // At least one distance code is declared, even when no copy uses it.
    if distance_lengths.iter().all(|l| *l == 0) {
        distance_lengths[0] = 1;
    }
    let literal_codes = canonical(&literal_lengths);
    let distance_codes = canonical(&distance_lengths);

    let literal_count = literal_lengths
        .iter()
        .rposition(|l| *l > 0)
        .map_or(LENGTH_FIRST_CODE, |i| (i + 1).max(LENGTH_FIRST_CODE));
    let distance_count = distance_lengths
        .iter()
        .rposition(|l| *l > 0)
        .map_or(1, |i| i + 1);

    let mut joined = literal_lengths[..literal_count].to_vec();
    joined.extend_from_slice(&distance_lengths[..distance_count]);
    let runs = run_lengths(&joined);
    let mut run_freq = vec![0_u32; LENGTH_CODES];
    for (symbol, _, _) in &runs {
        run_freq[*symbol] += 1;
    }
    let run_lengths_of = code_lengths(&run_freq, MAX_LENGTH_CODE_BITS);
    let run_codes = canonical(&run_lengths_of);
    let run_count = CODE_LENGTH_ORDER
        .iter()
        .rposition(|s| run_lengths_of[*s] > 0)
        .map_or(4, |i| (i + 1).max(4));

    out.put(u32::from(last == Final::Yes), 1);
    out.put(BLOCK_DYNAMIC, 2);
    out.put((literal_count - LENGTH_FIRST_CODE) as u32, 5);
    out.put((distance_count - 1) as u32, 5);
    out.put((run_count - 4) as u32, 4);
    for symbol in &CODE_LENGTH_ORDER[..run_count] {
        out.put(u32::from(run_lengths_of[*symbol]), 3);
    }
    for (symbol, extra, extra_bits) in runs {
        out.code(run_codes[symbol], run_lengths_of[symbol]);
        out.put(extra, extra_bits);
    }

    for token in tokens {
        match *token {
            Token::Literal(b) => {
                let s = usize::from(b);
                out.code(literal_codes[s], literal_lengths[s]);
            }
            Token::Copy { length, distance } => {
                let (length, distance) = (usize::from(length), usize::from(distance));
                let lc = length_code(length);
                let s = LENGTH_FIRST_CODE + lc;
                out.code(literal_codes[s], literal_lengths[s]);
                out.put(
                    (length - usize::from(LENGTH_BASE[lc])) as u32,
                    u32::from(LENGTH_EXTRA[lc]),
                );

                let dc = distance_code(distance);
                out.code(distance_codes[dc], distance_lengths[dc]);
                out.put(
                    (distance - usize::from(DISTANCE_BASE[dc])) as u32,
                    u32::from(DISTANCE_EXTRA[dc]),
                );
            }
        }
    }
    out.code(literal_codes[END_OF_BLOCK], literal_lengths[END_OF_BLOCK]);
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::inflate;

    /// Cheap deterministic noise.
    fn noise(len: usize, seed: u32) -> Vec<u8> {
        let mut state = seed;

        (0..len)
            .map(|_| {
                state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                (state >> 24) as u8
            })
            .collect()
    }

    fn round_trip(data: &[u8]) -> usize {
        let packed = zlib(data);
        let back = inflate::zlib(&packed, data.len()).unwrap();
        assert!(back == data, "{} bytes did not come back", data.len());

        packed.len()
    }

    #[test]
    fn adler_matches_the_known_value() {
        assert_eq!(adler32(b"Wikipedia"), 0x11E6_0398);
        assert_eq!(adler32(b""), 1);
        // Enough bytes to need the modulo more than once.
        assert_eq!(adler32(&[0xFF; 100_000]), {
            let mut a = 1_u64;
            let mut b = 0_u64;
            for _ in 0..100_000 {
                a = (a + 255) % 65521;
                b = (b + a) % 65521;
            }
            (b << 16 | a) as u32
        });
    }

    #[test]
    fn the_stream_carries_the_adler_sum() {
        let packed = zlib(b"abc");

        assert_eq!(packed[..2], ZLIB_HEADER);
        assert_eq!(packed[packed.len() - 4..], adler32(b"abc").to_be_bytes());
    }

    #[test]
    fn small_and_empty_inputs_round_trip() {
        round_trip(b"");
        round_trip(b"a");
        round_trip(b"ab");
        round_trip(b"aaa");
        round_trip(b"the quick brown fox jumps over the lazy dog ");
    }

    #[test]
    fn long_runs_use_the_longest_copies() {
        let size = round_trip(&[7; 1_000_000]);

        assert!(size < 4_000, "{size}");
        round_trip(&[0; MAX_MATCH]);
        round_trip(&[0; MAX_MATCH + 1]);
    }

    #[test]
    fn noise_round_trips_across_blocks() {
        let data = noise(300_000, 1);
        let size = round_trip(&data);

        // Incompressible: a few percent over at most.
        assert!(size < data.len() + data.len() / 20, "{size}");
    }

    #[test]
    fn structure_far_apart_is_found() {
        let chunk = noise(20_000, 2);
        let mut data = chunk.clone();
        data.extend(noise(5_000, 3));
        data.extend_from_slice(&chunk);
        let size = round_trip(&data);

        assert!(size < 27_000, "{size}");
    }

    #[test]
    fn skewed_symbols_need_the_length_limit() {
        // Fibonacci-like counts give an unlimited Huffman tree deeper than 15.
        let mut data = Vec::new();
        let (mut a, mut b) = (1_usize, 1_usize);
        for symbol in 0..30_u8 {
            data.extend(std::iter::repeat_n(symbol, a));
            (a, b) = (b, a + b);
            if data.len() > 2_000_000 {
                break;
            }
        }
        let mut mixed = data.clone();
        mixed.reverse();
        mixed.extend(noise(1000, 4));
        round_trip(&mixed);

        let freq: Vec<u32> = (0..30).map(|i| 1_u32 << i.min(28)).collect();
        assert!(huffman_lengths(&freq).iter().any(|l| *l > MAX_CODE_BITS));
        assert!(code_lengths(&freq, MAX_CODE_BITS)
            .iter()
            .all(|l| *l <= MAX_CODE_BITS));
    }

    #[test]
    fn a_picture_like_input_shrinks_a_lot() {
        let mut data = Vec::new();
        for row in 0..400_u32 {
            for col in 0..600_u32 {
                let edge = row % 50 == 0 || col % 70 == 0;
                data.extend_from_slice(match edge {
                    true => &[20, 20, 20],
                    false => &[240, 238, 230],
                });
            }
        }
        let size = round_trip(&data);

        assert!(size * 50 < data.len(), "{size}");
    }

    #[test]
    fn length_and_distance_codes_cover_their_ranges() {
        assert_eq!(length_code(3), 0);
        assert_eq!(length_code(10), 7);
        assert_eq!(length_code(227), 27);
        assert_eq!(length_code(257), 27);
        assert_eq!(length_code(258), 28);
        assert_eq!(distance_code(1), 0);
        assert_eq!(distance_code(4), 3);
        assert_eq!(distance_code(5), 4);
        assert_eq!(distance_code(32768), 29);
        assert_eq!(LENGTH_BASE[27], 227);
        assert_eq!(DISTANCE_BASE[29], 24577);
    }
}
