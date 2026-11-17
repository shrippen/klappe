//! Search and replace in the lines of a screenplay, optionally only in the dialogue of one
//! character.

use std::collections::HashSet;

use crate::assist::same_name;
use crate::{Element, Located};

/// Whether upper and lower case count as different.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Case {
    Ignore,
    Match,
}

#[derive(Debug, Clone, Copy)]
pub struct Query<'a> {
    pub find: &'a str,
    pub case: Case,
    /// Only the dialogue of this character (any case); `None` searches every line.
    pub speaker: Option<&'a str>,
}

/// A match from character `start` of line `line` to just before character `end` of line
/// `end_line`, all counted from 0. A match that ends on a line break ends at column 0 of the
/// next line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hit {
    pub line: usize,
    pub start: usize,
    pub end_line: usize,
    pub end: usize,
}

/// New text for the lines `from..to` (`to` exclusive) after replacing hits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Splice {
    pub from: usize,
    pub to: usize,
    pub lines: Vec<String>,
}

/// A character as it compares when case does not count; one that would change its length
/// stays as it is, so columns hold.
fn fold(c: char) -> char {
    let mut lower = c.to_lowercase();
    match (lower.next(), lower.next()) {
        (Some(one), None) => one,
        _ => c,
    }
}

/// A character as it is compared under the case option.
fn keyed(c: char, case: Case) -> char {
    match case {
        Case::Ignore => fold(c),
        Case::Match => c,
    }
}

/// Lines that hold the dialogue (not the cue or parentheticals) of the character.
fn dialogue_lines(located: &[Located], speaker: &str) -> HashSet<usize> {
    let mut lines = HashSet::new();
    let mut speaking = false;

    for item in located {
        match &item.element {
            Element::Character { name, .. } => speaking = same_name(name, speaker),
            Element::Dialogue(_) if speaking => lines.extend(item.lines.iter().copied()),
            Element::Dialogue(_) | Element::Parenthetical(_) => {}
            _ => speaking = false,
        }
    }

    lines
}

/// One character of the text as the search sees it: `None` is a character the search must
/// not match (a line outside the speaker's dialogue).
struct Cell {
    key: Option<char>,
    line: usize,
    col: usize,
}

const SPACE: char = ' ';
const BREAK: char = '\n';

/// The text as cells, a line break between every two lines.
fn cells(lines: &[String], allowed: Option<&HashSet<usize>>, case: Case) -> Vec<Cell> {
    let mut out = Vec::new();

    for (line, text) in lines.iter().enumerate() {
        let open = allowed.is_none_or(|set| set.contains(&line));
        for (col, c) in text.chars().enumerate() {
            let key = open.then(|| keyed(c, case));
            out.push(Cell { key, line, col });
        }
        // No break after the last line.
        if line + 1 == lines.len() {
            break;
        }
        let col = text.chars().count();
        let key = open.then_some(BREAK);
        out.push(Cell { key, line, col });
    }

    out
}

/// Whether the cells match the needle. A space of the needle also matches a line break, so a
/// phrase broken over two lines is found; a blank line (two breaks in a row) never matches.
fn matches_at(cells: &[Cell], needle: &[char]) -> bool {
    let mut broke = false;

    for (cell, n) in cells.iter().zip(needle) {
        let is_break = cell.key == Some(BREAK);
        let same = cell.key == Some(*n) || (is_break && *n == SPACE);
        if !same || (is_break && broke) {
            return false;
        }
        broke = is_break;
    }

    true
}

/// Every match in text order, not overlapping. `located` is the screenplay's reading of
/// `lines`; it is used only for a query with a speaker, whose matches must lie within the
/// lines of her dialogue. A space in `query.find` also matches a line break.
pub fn find_all(lines: &[String], located: &[Located], query: &Query) -> Vec<Hit> {
    if query.find.is_empty() {
        return Vec::new();
    }

    let allowed = query.speaker.map(|s| dialogue_lines(located, s));
    let text = cells(lines, allowed.as_ref(), query.case);
    let needle: Vec<char> = query.find.chars().map(|c| keyed(c, query.case)).collect();
    let mut hits = Vec::new();
    let mut at = 0;

    while at + needle.len() <= text.len() {
        let span = &text[at..at + needle.len()];
        if !matches_at(span, &needle) {
            at += 1;
            continue;
        }

        let (first, last) = (&span[0], &span[span.len() - 1]);
        let ends_on_break = last.key == Some(BREAK);
        let (end_line, end) = match ends_on_break {
            true => (last.line + 1, 0),
            false => (last.line, last.col + 1),
        };
        hits.push(Hit {
            line: first.line,
            start: first.col,
            end_line,
            end,
        });
        at += needle.len();
    }

    hits
}

/// The lines changed by replacing every hit (sorted, apart) with `replacement`; `None` for
/// no hits. Lines between the first and last hit that do not change come back as they are.
pub fn replace_hits(lines: &[String], hits: &[Hit], replacement: &str) -> Option<Splice> {
    let (first, last) = (hits.first()?, hits.last()?);
    let (from, to) = (first.line, last.end_line + 1);

    // The lines as one text; `offset[n]` is where line `from + n` starts.
    let mut chars: Vec<char> = Vec::new();
    let mut offset = Vec::new();
    for line in &lines[from..to] {
        offset.push(chars.len());
        chars.extend(line.chars());
        chars.push(BREAK);
    }
    chars.pop();

    let place = |line: usize, col: usize| offset[line - from] + col;
    let mut out = String::new();
    let mut at = 0;
    for hit in hits {
        out.extend(&chars[at..place(hit.line, hit.start)]);
        out.push_str(replacement);
        at = place(hit.end_line, hit.end).min(chars.len());
    }
    out.extend(&chars[at..]);

    Some(Splice {
        from,
        to,
        lines: out.split(BREAK).map(str::to_owned).collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_located;

    fn lines(text: &str) -> Vec<String> {
        text.split('\n').map(str::to_owned).collect()
    }

    fn query<'a>(find: &'a str, case: Case, speaker: Option<&'a str>) -> Query<'a> {
        Query {
            find,
            case,
            speaker,
        }
    }

    const TEXT: &str = "Anna sieht Anna.\n\nANNA\nIch bin anna.\n\nBEN\nANNA ist da.";

    fn hits(find: &str, case: Case, speaker: Option<&str>) -> Vec<(usize, usize, usize)> {
        let l = lines(TEXT);
        find_all(&l, &parse_located(TEXT), &query(find, case, speaker))
            .iter()
            .map(|h| (h.line, h.start, h.end))
            .collect()
    }

    #[test]
    fn finds_in_text_order_with_or_without_case() {
        assert_eq!(
            hits("anna", Case::Ignore, None),
            [(0, 0, 4), (0, 11, 15), (2, 0, 4), (3, 8, 12), (6, 0, 4)]
        );
        assert_eq!(hits("Anna", Case::Match, None), [(0, 0, 4), (0, 11, 15)]);
        assert!(hits("", Case::Ignore, None).is_empty());
    }

    #[test]
    fn a_speaker_limits_the_search_to_the_dialogue() {
        assert_eq!(hits("anna", Case::Ignore, Some("anna")), [(3, 8, 12)]);
        assert_eq!(hits("anna", Case::Ignore, Some("BEN")), [(6, 0, 4)]);
        assert!(hits("anna", Case::Ignore, Some("CARL")).is_empty());
    }

    #[test]
    fn columns_count_characters() {
        let l = lines("Äpfel und äpfel");
        let h = find_all(&l, &[], &query("ÄPFEL", Case::Ignore, None));

        assert_eq!(
            h.iter().map(|h| (h.start, h.end)).collect::<Vec<_>>(),
            [(0, 5), (10, 15)]
        );
    }

    fn replaced(text: &str, find: &str, with: &str) -> Splice {
        let l = lines(text);
        let h = find_all(&l, &[], &query(find, Case::Ignore, None));

        replace_hits(&l, &h, with).unwrap()
    }

    #[test]
    fn replaces_every_hit_of_a_line() {
        let edit = replaced(TEXT, "anna", "Eva");

        assert_eq!((edit.from, edit.to), (0, 7));
        assert_eq!(
            edit.lines,
            lines("Eva sieht Eva.\n\nEva\nIch bin Eva.\n\nBEN\nEva ist da.")
        );
        assert!(replace_hits(&lines(TEXT), &[], "x").is_none());
    }

    #[test]
    fn a_space_matches_a_line_break() {
        let text = "Sie geht durch die\nTür und bleibt.";
        let l = lines(text);
        let h = find_all(&l, &[], &query("die tür", Case::Ignore, None));

        assert_eq!(
            h,
            [Hit {
                line: 0,
                start: 15,
                end_line: 1,
                end: 3
            }]
        );
        let edit = replaced(text, "die tür", "eine Tür");
        assert_eq!((edit.from, edit.to), (0, 2));
        assert_eq!(edit.lines, ["Sie geht durch eine Tür und bleibt."]);
    }

    #[test]
    fn a_match_may_end_on_a_line_break() {
        let l = lines("ab \ncd");
        let h = find_all(&l, &[], &query("b ", Case::Match, None));

        assert_eq!(h.len(), 1);
        assert_eq!((h[0].end_line, h[0].end), (0, 3));

        let l = lines("ab\ncd");
        let h = find_all(&l, &[], &query("b ", Case::Match, None));
        assert_eq!((h[0].end_line, h[0].end), (1, 0));
        assert_eq!(replace_hits(&l, &h, "-").unwrap().lines, ["a-cd"]);
    }

    #[test]
    fn a_blank_line_stops_a_match() {
        let l = lines("Ende.\n\nAnfang");
        assert!(find_all(&l, &[], &query("Ende. Anfang", Case::Match, None)).is_empty());
        assert!(find_all(&l, &[], &query("Ende.  Anfang", Case::Match, None)).is_empty());
    }

    #[test]
    fn a_speaker_match_stays_in_her_dialogue() {
        let text = "ANNA\nGuten\nTag.\n\nBEN\nGuten\nTag.";
        let l = lines(text);
        let h = find_all(
            &l,
            &parse_located(text),
            &query("guten tag", Case::Ignore, Some("ben")),
        );

        assert_eq!(h.len(), 1);
        assert_eq!((h[0].line, h[0].end_line), (5, 6));

        // The cue is no dialogue: "ANNA Guten" crosses from the cue into the speech.
        let none = find_all(
            &l,
            &parse_located(text),
            &query("anna guten", Case::Ignore, Some("anna")),
        );
        assert!(none.is_empty());
    }

    #[test]
    fn matches_do_not_overlap() {
        let l = lines("aaaa");
        let h = find_all(&l, &[], &query("aa", Case::Match, None));

        assert_eq!(h.len(), 2);
        assert_eq!(replace_hits(&l, &h, "b").unwrap().lines, ["bb"]);
    }

    #[test]
    fn a_speaker_is_matched_with_unicode_case() {
        let text = "ÄRZTIN\nGuten Tag.\n\nBEN\nTag.";
        let l = lines(text);
        let h = find_all(
            &l,
            &parse_located(text),
            &query("tag", Case::Ignore, Some("ärztin")),
        );

        assert_eq!(h.len(), 1);
        assert_eq!(h[0].line, 1);
    }
}
