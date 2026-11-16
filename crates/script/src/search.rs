//! Search and replace in the lines of a screenplay, optionally only in the dialogue of one
//! character.

use std::collections::HashSet;

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

/// A match: characters `start..end` of line `line`, all counted from 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hit {
    pub line: usize,
    pub start: usize,
    pub end: usize,
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

/// Lines that hold the dialogue (not the cue or parentheticals) of the character.
fn dialogue_lines(located: &[Located], speaker: &str) -> HashSet<usize> {
    let mut lines = HashSet::new();
    let mut speaking = false;

    for item in located {
        match &item.element {
            Element::Character { name, .. } => speaking = name.eq_ignore_ascii_case(speaker),
            Element::Dialogue(_) if speaking => lines.extend(item.lines.iter().copied()),
            Element::Dialogue(_) | Element::Parenthetical(_) => {}
            _ => speaking = false,
        }
    }

    lines
}

/// Every match in text order, not overlapping. `located` is the screenplay's reading of
/// `lines`; it is used only for a query with a speaker.
pub fn find_all(lines: &[String], located: &[Located], query: &Query) -> Vec<Hit> {
    if query.find.is_empty() {
        return Vec::new();
    }

    let allowed = query.speaker.map(|s| dialogue_lines(located, s));
    let key = |c: char| match query.case {
        Case::Ignore => fold(c),
        Case::Match => c,
    };
    let needle: Vec<char> = query.find.chars().map(key).collect();
    let mut hits = Vec::new();

    for (line, text) in lines.iter().enumerate() {
        if allowed.as_ref().is_some_and(|set| !set.contains(&line)) {
            continue;
        }

        let chars: Vec<char> = text.chars().map(key).collect();
        let mut at = 0;
        while at + needle.len() <= chars.len() {
            if chars[at..at + needle.len()] != needle[..] {
                at += 1;
                continue;
            }
            hits.push(Hit {
                line,
                start: at,
                end: at + needle.len(),
            });
            at += needle.len();
        }
    }

    hits
}

/// The line with each hit that is on it replaced. Hits of other lines are ignored.
pub fn replace_line(line: &str, line_no: usize, hits: &[Hit], with: &str) -> String {
    let chars: Vec<char> = line.chars().collect();
    let mut out = String::new();
    let mut at = 0;

    for hit in hits.iter().filter(|h| h.line == line_no) {
        out.extend(&chars[at..hit.start]);
        out.push_str(with);
        at = hit.end;
    }
    out.extend(&chars[at..]);

    out
}

/// The lines that change when every hit is replaced: line number and new text.
pub fn replace_all(lines: &[String], hits: &[Hit], with: &str) -> Vec<(usize, String)> {
    let mut changed: Vec<(usize, String)> = Vec::new();

    for hit in hits {
        if changed.last().is_some_and(|(n, _)| *n == hit.line) {
            continue;
        }
        let Some(text) = lines.get(hit.line) else {
            continue;
        };
        changed.push((hit.line, replace_line(text, hit.line, hits, with)));
    }

    changed
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

    #[test]
    fn replaces_every_hit_of_a_line() {
        let l = lines(TEXT);
        let h = find_all(&l, &[], &query("anna", Case::Ignore, None));

        assert_eq!(replace_line(&l[0], 0, &h, "Eva"), "Eva sieht Eva.");
        assert_eq!(
            replace_all(&l, &h, "Eva"),
            [
                (0, "Eva sieht Eva.".to_owned()),
                (2, "Eva".to_owned()),
                (3, "Ich bin Eva.".to_owned()),
                (6, "Eva ist da.".to_owned()),
            ]
        );
    }

    #[test]
    fn matches_do_not_overlap() {
        let l = lines("aaaa");
        let h = find_all(&l, &[], &query("aa", Case::Match, None));

        assert_eq!(h.len(), 2);
        assert_eq!(replace_line(&l[0], 0, &h, "b"), "bb");
    }
}
