//! Pages of a locked screenplay (DB-34): where the locked pages start in the book now, and
//! the blocks cut at those places.
//!
//! ```text
//! locked text + starts ─► carry ─► starts in the text now ─► element and byte (`Start`)
//!                                                        └─► blocks cut there, a page break
//!                                                            with the locked label at each
//! ```

use klappe_model::PageStart;

use super::block::{Block, Hit};
use super::{Pagination, Words};
use crate::lcs::unpaired;
use crate::{Element, Located};

/// A locked page starting in the element `element` (an index past the last: nothing follows),
/// `offset` bytes into its text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Start {
    pub label: String,
    pub element: usize,
    pub offset: usize,
}

const LETTERS: usize = 26;

/// The label of the page added after `label`: `12` gets `12A`, `12A` gets `12B`, `12Z` gets
/// `12AA`.
pub fn page_label_after(label: &str) -> String {
    let digits: String = label.chars().take_while(char::is_ascii_digit).collect();
    let letters = label[digits.len()..]
        .bytes()
        .filter(u8::is_ascii_uppercase)
        .fold(0, |n, b| n * LETTERS + usize::from(b - b'A') + 1);

    // Bijective base 26: A is 1, Z is 26, AA is 27.
    let mut rest = letters + 1;
    let mut out = Vec::new();
    while rest > 0 {
        rest -= 1;
        out.push(b'A' + (rest % LETTERS) as u8);
        rest /= LETTERS;
    }
    out.reverse();

    format!("{digits}{}", String::from_utf8_lossy(&out))
}

/// The place of a page label in the script: by its number, then by its letters (`A` before
/// `Z` before `AA`).
pub fn compare_labels(a: &str, b: &str) -> std::cmp::Ordering {
    let split = |label: &str| {
        let digits: String = label.chars().take_while(char::is_ascii_digit).collect();
        let letters = label[digits.len()..].len();
        let value = label[digits.len()..]
            .bytes()
            .filter(u8::is_ascii_uppercase)
            .fold(0, |n: usize, b| n * LETTERS + usize::from(b - b'A') + 1);

        (digits.parse::<usize>().unwrap_or_default(), letters, value)
    };

    split(a).cmp(&split(b))
}

/// The lines of `text` as the parser counts them: CR and CRLF are one break.
fn lines_of(text: &str) -> Vec<String> {
    let text = text.replace("\r\n", "\n").replace('\r', "\n");

    text.split('\n').map(str::to_owned).collect()
}

/// Per line of `old`: the line of `new` the longest common subsequence pairs it with.
fn line_map(old: &[String], new: &[String]) -> Vec<Option<usize>> {
    let (removed, added) = unpaired(old, new);
    let mut gone = vec![false; old.len()];
    let mut fresh = vec![false; new.len()];
    removed.iter().for_each(|&i| gone[i] = true);
    added.iter().for_each(|&j| fresh[j] = true);

    // The paired lines keep their order: the n-th kept of `old` is the n-th kept of `new`.
    let mut kept = (0..new.len()).filter(|&j| !fresh[j]);

    (0..old.len())
        .map(|i| match gone[i] {
            true => None,
            false => kept.next(),
        })
        .collect()
}

/// `offset` in `old` as the same share of `new`, on a character boundary.
fn scaled(offset: usize, old: &str, new: &str) -> usize {
    let mut at = (offset * new.len()).checked_div(old.len()).unwrap_or(0);
    while !new.is_char_boundary(at) {
        at -= 1;
    }

    at
}

/// The starts of a locked version's pages (`starts`, in the text `old`) as they stand in the
/// text `new`: a start moves with its line, one on a line that is gone moves to the next
/// line that is kept, one with no such line (and the pages struck at the end) goes to the
/// end. The first page starts at the top. A start on a changed line moves to the changed
/// lines right before the next kept one. A start within a line keeps its place only if the
/// line is unchanged, else it moves to the start of the line.
pub fn carry(old: &str, new: &str, starts: &[PageStart]) -> Vec<PageStart> {
    let (old, new) = (lines_of(old), lines_of(new));
    let map = line_map(&old, &new);
    let mut kept = vec![false; new.len()];
    map.iter().flatten().for_each(|&j| kept[j] = true);

    let moved = |start: &PageStart| {
        let crowded = starts.iter().filter(|s| s.line == start.line).count() > 1;
        let found = (start.line..old.len()).find_map(|i| Some((i, map[i]?)));
        let Some((at, line)) = found else {
            return (new.len(), 0);
        };

        if at == start.line {
            return (line, start.offset);
        }

        // The line is gone or changed: the page starts at the changed lines that stand
        // right before the next kept one (a paragraph, no blank line), so a changed heading
        // keeps its page (DB-31) and text added before it stays on the page before.
        let mut first = line;
        while first > 0 && !kept[first - 1] && !new[first - 1].is_empty() {
            first -= 1;
        }

        // Several starts in one changed line, once it is a single line again: they keep
        // their share of it, else they all fall on its start and leave pages empty.
        if crowded && line - first == 1 {
            return (first, scaled(start.offset, &old[start.line], &new[first]));
        }

        (first, 0)
    };

    starts
        .iter()
        .enumerate()
        .map(|(i, start)| {
            let (line, offset) = match i {
                0 => (0, 0),
                _ => moved(start),
            };

            PageStart {
                label: start.label.clone(),
                line,
                offset,
            }
        })
        .collect()
}

/// Where each page of `pages` starts in the text of `located` (of `line_count` lines), to lock
/// them: the first line of text on the page and the byte in its line. A struck page starts
/// where the next page of text does, or at `line_count` when none follows.
pub fn page_starts(pages: &Pagination, located: &[Located], line_count: usize) -> Vec<PageStart> {
    let mut starts = Vec::new();
    let mut struck: Vec<String> = Vec::new();

    for page in &pages.pages {
        struck.extend(page.omitted.iter().cloned());

        // A repeated name and (MORE) are not in the text.
        let Some(first) = page.lines.iter().find(|l| !l.added) else {
            continue;
        };
        let (line, offset) = spot_of(located, first.element, first.offset).unwrap_or_default();

        for label in struck.drain(..).chain([page.label.clone()]) {
            starts.push(PageStart {
                label,
                line,
                offset,
            });
        }
    }

    starts.extend(struck.into_iter().map(|label| PageStart {
        label,
        line: line_count,
        offset: 0,
    }));

    starts
}

/// The text line of byte `offset` of the text of `element`, and the byte in that line.
fn spot_of(located: &[Located], element: usize, offset: usize) -> Option<(usize, usize)> {
    let located = located.get(element)?;
    let text = lines_text(&located.element);
    let before = text.get(..offset).unwrap_or_default();
    let nth = before.matches('\n').count();
    let column = before.len() - before.rfind('\n').map_or(0, |at| at + 1);
    let line = located.lines.get(nth).or(located.lines.last())?;

    Some((*line, column))
}

/// The text of an element whose lines may be several.
pub fn lines_text(element: &Element) -> &str {
    match element {
        Element::Action(t) | Element::Dialogue(t) | Element::Lyrics(t) => t,
        _ => "",
    }
}

/// The starts as places in the elements of the text.
pub(super) fn spots(located: &[Located], starts: &[PageStart]) -> Vec<Start> {
    starts
        .iter()
        .map(|start| {
            let at = located
                .iter()
                .position(|l| l.lines.last().is_some_and(|&last| last >= start.line));
            let (element, offset) = at
                .and_then(|e| Some((e, in_element(&located[e], start)?)))
                .unwrap_or((located.len(), 0));

            Start {
                label: start.label.clone(),
                element,
                offset,
            }
        })
        .collect()
}

/// Bytes into the text of `located` where the line of `start` begins, plus its column; a
/// line that is not the element's (a blank one before it) starts the element.
fn in_element(located: &Located, start: &PageStart) -> Option<usize> {
    let nth = located.lines.iter().position(|&l| l >= start.line)?;
    if located.lines[nth] != start.line {
        return Some(0);
    }

    let before: usize = lines_text(&located.element)
        .split('\n')
        .take(nth)
        .map(|line| line.len() + 1)
        .sum();

    Some(before + start.offset)
}

/// The blocks with a page break, carrying the locked label, at each start; a block is cut
/// where a start falls inside it.
pub(super) fn split_blocks(blocks: Vec<Block>, starts: &[Start], words: Words) -> Vec<Block> {
    if starts.is_empty() {
        return blocks;
    }

    let mut out = Vec::with_capacity(blocks.len() + starts.len());
    let mut next = 0;

    for mut block in blocks {
        while let Some(start) = starts.get(next) {
            let Some((first, last)) = block.span() else {
                break;
            };
            if start.element > last {
                break;
            }

            next += 1;
            let inside = start.element >= first && (start.element > first || start.offset > 0);
            let hit = match inside {
                true => block.hit(start.element, start.offset, words),
                false => Hit::Before,
            };
            match hit {
                Hit::Before => out.push(Block::marker(start.label.clone())),
                Hit::Inside(head, rest) => {
                    out.push(head);
                    out.push(Block::marker(start.label.clone()));
                    block = rest;
                }
                Hit::Nowhere => {}
            }
        }
        out.push(block);
    }

    out.extend(
        starts[next..]
            .iter()
            .map(|s| Block::marker(s.label.clone())),
    );

    out
}
