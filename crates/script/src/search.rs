//! Search and replace in the lines of a screenplay, optionally only in the dialogue of one
//! character.

use std::collections::HashSet;

use crate::{Element, Located};
use klappe_model::same_character_name;

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
            Element::Character { name, .. } => speaking = same_character_name(name, speaker),
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

/// The line breaks that may stand for a space: after line `n`, when `n` and the next line
/// are lines of one action paragraph or one speech, and neither is blank. Never at the start
/// or end of an element, next to a blank line or between cue, parenthetical and dialogue.
fn spaced_breaks(lines: &[String], located: &[Located]) -> HashSet<usize> {
    let mut breaks = HashSet::new();

    for item in located {
        if !matches!(item.element, Element::Action(_) | Element::Dialogue(_)) {
            continue;
        }
        for pair in item.lines.windows(2) {
            let blank = |n: usize| lines.get(n).is_none_or(|l| l.trim().is_empty());
            if pair[1] == pair[0] + 1 && !blank(pair[0]) && !blank(pair[1]) {
                breaks.insert(pair[0]);
            }
        }
    }

    breaks
}

/// The text as cells, a line break after each line of `breaks`; other line ends are cells
/// no needle matches.
fn cells(
    lines: &[String],
    allowed: Option<&HashSet<usize>>,
    breaks: &HashSet<usize>,
    case: Case,
) -> Vec<Cell> {
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
        let key = (open && breaks.contains(&line)).then_some(BREAK);
        out.push(Cell { key, line, col });
    }

    out
}

/// Whether the cells match the needle. A space of the needle also matches a line break, so a
/// phrase broken over two lines is found; see `spaced_breaks` for which breaks count.
fn matches_at(cells: &[Cell], needle: &[char]) -> bool {
    cells.iter().zip(needle).all(|(cell, n)| {
        let is_break = cell.key == Some(BREAK);

        cell.key == Some(*n) || (is_break && *n == SPACE)
    })
}

/// Every match in text order, not overlapping. `located` is the screenplay's reading of
/// `lines`; it tells the speaker's dialogue and which line breaks a space may match: a space
/// in `query.find` also matches a break between two lines of one action or speech, so a
/// match with a speaker never leaves one speech.
pub fn find_all(lines: &[String], located: &[Located], query: &Query) -> Vec<Hit> {
    if query.find.is_empty() {
        return Vec::new();
    }

    let allowed = query.speaker.map(|s| dialogue_lines(located, s));
    let breaks = spaced_breaks(lines, located);
    let text = cells(lines, allowed.as_ref(), &breaks, query.case);
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
        let h = find_all(&l, &parse_located(text), &query(find, Case::Ignore, None));

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
        let h = find_all(
            &l,
            &parse_located(text),
            &query("die tür", Case::Ignore, None),
        );

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
        let h = find_all(
            &l,
            &parse_located("ab \ncd"),
            &query("b ", Case::Match, None),
        );

        assert_eq!(h.len(), 1);
        assert_eq!((h[0].end_line, h[0].end), (0, 3));

        let l = lines("ab\ncd");
        let h = find_all(
            &l,
            &parse_located("ab\ncd"),
            &query("b ", Case::Match, None),
        );
        assert_eq!((h[0].end_line, h[0].end), (1, 0));
        assert_eq!(replace_hits(&l, &h, "-").unwrap().lines, ["a-cd"]);
    }

    fn hits_in(text: &str, find: &str, speaker: Option<&str>) -> Vec<Hit> {
        let query = query(find, Case::Ignore, speaker);

        find_all(&lines(text), &parse_located(text), &query)
    }

    #[test]
    fn a_blank_line_stops_a_match() {
        assert!(hits_in("Ende.\n\nAnfang", "Ende. Anfang", None).is_empty());
        assert!(hits_in("Ende.\n\nAnfang", "Ende.  Anfang", None).is_empty());
    }

    #[test]
    fn a_break_at_the_end_of_an_element_is_no_space() {
        let text = "Ende.\n\nAnfang";
        assert!(hits_in(text, "ende. ", None).is_empty());
        assert!(hits_in(text, " anfang", None).is_empty());
        assert!(hits_in("Er sagt\n\nANNA\nHallo", " ", None)
            .iter()
            .all(|h| h.line == h.end_line));
    }

    #[test]
    fn a_break_next_to_a_cue_is_no_space() {
        let text = "Draußen.\n\nANNA\nGuten Tag.\n\nANNA\nNa?";
        assert!(hits_in(text, "anna ", None).is_empty());
        assert!(hits_in(text, "anna ", Some("anna")).is_empty());
    }

    #[test]
    fn a_speaker_match_never_leaves_one_speech() {
        let text = "ANNA\nHallo Ben\n\nBEN\nJa.";
        assert!(hits_in(text, "ben ", Some("anna")).is_empty());
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

    /// The search finds a cue's lines exactly when the project calls the names one character.
    #[test]
    fn a_speaker_is_the_project_s_character() {
        let names = ["WEISS", "WEIẞ", "Weiß", " weiss ", "WEISS 2"];

        for cue in ["WEISS", "WEIẞ"] {
            let text = format!("{cue}\nHallo.");

            for speaker in names {
                let found = !hits_in(&text, "hallo", Some(speaker)).is_empty();

                assert_eq!(
                    found,
                    klappe_model::same_character_name(cue, speaker),
                    "cue {cue:?}, speaker {speaker:?}"
                );
            }
        }
    }
}
