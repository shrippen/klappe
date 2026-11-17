//! Text normalisation shared by the comparison code.

/// Lower case, runs of whitespace collapsed: `"INT.  Küche"` and `"int. küche"` are equal.
pub(crate) fn normal(text: &str) -> String {
    text.split_whitespace()
        .map(str::to_lowercase)
        .collect::<Vec<_>>()
        .join(" ")
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
