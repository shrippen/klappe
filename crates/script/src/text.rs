//! Text normalisation shared by the comparison code.

const DASHES: [char; 4] = ['–', '—', '‑', '−'];
const FNV_OFFSET: u32 = 0x811c_9dc5;
const FNV_PRIME: u32 = 0x0100_0193;

/// Lower case, runs of whitespace collapsed, dash variants equal and set off by spaces:
/// `"INT.  Küche – Tag"`, `"int. küche - tag"` and `"INT. KÜCHE—TAG"` are equal.
pub(crate) fn normal(text: &str) -> String {
    let mut spaced = String::with_capacity(text.len() + 8);
    for c in text.chars() {
        if c == '-' || DASHES.contains(&c) {
            spaced.push_str(" - ");
            continue;
        }
        spaced.push(c);
    }

    spaced
        .split_whitespace()
        .map(str::to_lowercase)
        .collect::<Vec<_>>()
        .join(" ")
}

/// FNV-1a over a kind tag and the normalised text; the same on every machine and
/// release, so it can be saved.
pub(crate) fn hash(tag: u8, text: &str) -> u32 {
    let mut hash = FNV_OFFSET;
    for byte in std::iter::once(tag).chain(normal(text).bytes()) {
        hash = (hash ^ u32::from(byte)).wrapping_mul(FNV_PRIME);
    }
    hash
}

/// Distinct lower-case words, sorted; punctuation separates words.
pub(crate) fn words(text: &str) -> Vec<String> {
    let lower = text.to_lowercase();
    let mut words: Vec<String> = lower
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_string)
        .collect();
    words.sort();
    words.dedup();
    words
}
