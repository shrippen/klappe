//! Scene numbers in the text (DB-33): sequential while automatic, kept while locked, with a
//! new scene numbered between its neighbours (12A) and a struck one, by the Sperrstand, left
//! as "OMITTED".

use std::collections::HashSet;

use crate::{numbered_head, parse_located, split_number, Element};

/// The word a struck scene's heading is, in a German and in an English book.
pub const OMITTED_DE: &str = "ENTFÄLLT";
pub const OMITTED_EN: &str = "OMITTED";

/// Which numbers `number_scenes` may change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberMode {
    /// All: scene `i` gets `first + i`.
    Automatic,
    /// None; a scene without a number gets one between its neighbours.
    Locked,
}

#[derive(Debug, Clone, Copy)]
pub struct NumberRules<'a> {
    pub mode: NumberMode,
    /// The first number while automatic.
    pub first: u32,
    /// The heading of a struck scene (`OMITTED_DE`, `OMITTED_EN`).
    pub omitted: &'a str,
    /// The Sperrstand while locked: the numbers of the saved book in order. One the text no
    /// longer carries, on a scene or on an "OMITTED" heading, is struck: its heading is put
    /// in front of the next number of the Sperrstand the text still carries, else at the end.
    pub locked: &'a [String],
    /// The headings (counted from 0 among the scene headings of the text) that keep a number
    /// other headings carry too, the holders of the Sperrstand; the others are inserted
    /// scenes. A number carried twice without a holder stays on all of them.
    pub holders: &'a [usize],
}

/// Whether the heading is a struck scene's (`.OMITTED #12#`), in either language.
pub fn is_omitted(heading: &str) -> bool {
    let word = heading.trim().to_uppercase();

    word == OMITTED_DE || word == OMITTED_EN
}

struct Heading {
    line: usize,
    text: String,
    number: Option<String>,
}

fn headings(text: &str) -> Vec<Heading> {
    let found = parse_located(text).into_iter().filter_map(|located| {
        let Element::SceneHeading { text, number } = located.element else {
            return None;
        };
        let line = *located.lines.first()?;

        Some(Heading { line, text, number })
    });

    found.collect()
}

/// Numbers the scene headings of `text` by `rules` and returns the text; the lines it does
/// not change stay as they are. Without a scene heading the text is returned unchanged.
pub fn number_scenes(text: &str, rules: &NumberRules) -> String {
    // Automatic numbering counts anyway: a struck heading has no use.
    let text = match rules.mode {
        NumberMode::Automatic => without_omitted(text),
        NumberMode::Locked => text.to_owned(),
    };
    let heads = headings(&text);
    if heads.is_empty() {
        return text;
    }

    let wanted = assign(&heads, rules);
    let mut lines: Vec<String> = text.split('\n').map(str::to_owned).collect();
    for (head, number) in heads.iter().zip(&wanted) {
        if head.number.as_ref() != Some(number) && !number.is_empty() {
            lines[head.line] = with_number(&lines[head.line], number);
        }
    }

    if rules.mode == NumberMode::Locked {
        insert_struck(&mut lines, &heads, rules);
    }

    lines.join("\n")
}

/// `text` without its struck headings and the blank line behind each.
fn without_omitted(text: &str) -> String {
    let heads = headings(text);
    let struck: Vec<usize> = heads
        .iter()
        .filter(|h| is_omitted(&h.text))
        .map(|h| h.line)
        .collect();
    if struck.is_empty() {
        return text.to_owned();
    }

    let lines: Vec<&str> = text.split('\n').collect();
    let blank_after = |at: &usize| lines.get(at + 1).is_some_and(|l| l.trim().is_empty());
    let dropped: HashSet<usize> = struck
        .iter()
        .flat_map(|at| std::iter::once(*at).chain(blank_after(at).then_some(at + 1)))
        .collect();
    let kept = lines
        .iter()
        .enumerate()
        .filter(|(at, _)| !dropped.contains(at))
        .map(|(_, line)| *line);

    let mut joined = kept.collect::<Vec<_>>().join("\n");
    // The dropped last blank line leaves the `\r` of the one before it behind.
    if joined.ends_with('\r') && !text.ends_with('\r') {
        joined.pop();
    }

    joined
}

/// The number of every heading, empty for a struck heading that has none. A struck scene's
/// heading keeps its number in both modes and takes no part in the counting.
fn assign(heads: &[Heading], rules: &NumberRules) -> Vec<String> {
    if rules.mode == NumberMode::Automatic {
        return sequential(heads, rules.first);
    }

    let mut used: HashSet<String> = heads.iter().filter_map(|h| h.number.clone()).collect();
    let mut last: Option<Parts> = None;
    let mut out = Vec::new();

    for (i, head) in heads.iter().enumerate() {
        // A struck heading without a number (from another program) stays without one.
        if head.number.is_none() && is_omitted(&head.text) {
            out.push(String::new());
            continue;
        }

        let own = head
            .number
            .as_deref()
            .filter(|n| is_omitted(&head.text) || keeps(heads, rules.holders, i, n));
        let number = match own {
            Some(number) => number.to_owned(),
            None => {
                let later = heads[i..].iter().find_map(|h| h.number.as_deref());
                let number = next_free(last.as_ref(), later, &used);
                used.insert(number.clone());
                number
            }
        };
        last = Parts::of(&number).or(last);
        out.push(number);
    }

    out
}

/// Whether heading `i` keeps its number `n`: unless another heading carries it and one of
/// them, not `i`, is the holder (a copy then is the inserted scene).
fn keeps(heads: &[Heading], holders: &[usize], i: usize, n: &str) -> bool {
    let same: Vec<usize> = (0..heads.len())
        .filter(|&j| heads[j].number.as_deref() == Some(n) && !is_omitted(&heads[j].text))
        .collect();
    let known: Vec<&usize> = same.iter().filter(|j| holders.contains(j)).collect();

    known.len() != 1 || *known[0] == i
}

fn sequential(heads: &[Heading], first: u32) -> Vec<String> {
    let mut next = first;

    heads
        .iter()
        .map(|head| match (&head.number, is_omitted(&head.text)) {
            (Some(number), true) => number.clone(),
            _ => {
                next += 1;
                (next - 1).to_string()
            }
        })
        .collect()
}

/// A number split into `A`, `12`, `B` of `A12B`.
#[derive(Debug, Clone)]
struct Parts {
    before: String,
    digits: String,
    after: String,
}

impl Parts {
    fn of(number: &str) -> Option<Self> {
        let before: String = number
            .chars()
            .take_while(|c| c.is_ascii_alphabetic())
            .collect();
        let rest = &number[before.len()..];
        let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
        let after = &rest[digits.len()..];
        let plain = !digits.is_empty() && after.chars().all(|c| c.is_ascii_alphabetic());

        plain.then(|| Self {
            before,
            digits,
            after: after.to_owned(),
        })
    }

    fn text(&self) -> String {
        format!("{}{}{}", self.before, self.digits, self.after)
    }
}

/// The next letters after `letters`: `A`, `B` .. `Z`, `AA`, `AB` .. (lower case counts as upper).
fn bump(letters: &str) -> String {
    let mut chars: Vec<char> = letters.chars().collect();

    for at in (0..chars.len()).rev() {
        let upper = chars[at].to_ascii_uppercase();
        if upper != 'Z' {
            chars[at] = char::from(upper as u8 + 1);
            return chars.into_iter().collect();
        }
        chars[at] = 'A';
    }

    format!("A{}", chars.into_iter().collect::<String>())
}

/// The number of a scene that has none: after `last` it is `last` with the next letters
/// (12 then 12A, 12A then 12B); before every number it is `A` and the number of the next
/// scene that has one (A1, then B1), before a prefixed one (`A1`) it is `A0`, then `A0A`.
/// Between two prefixed numbers the letters go behind the digits (A1A). A number in use is
/// skipped, so with `12` and `12A` side by side the new scene gets the next free letter
/// (12B): nothing sorts between those two.
fn next_free(last: Option<&Parts>, later: Option<&str>, used: &HashSet<String>) -> String {
    let later = later.and_then(Parts::of);
    let later_plain = later.as_ref().is_some_and(|l| l.before.is_empty());
    let mut parts = match (last, &later) {
        (Some(last), _) => last.clone(),
        (None, Some(later)) => Parts {
            before: if later_plain {
                String::new()
            } else {
                later.before.clone()
            },
            digits: if later_plain {
                later.digits.clone()
            } else {
                "0".to_owned()
            },
            after: String::new(),
        },
        (None, None) => Parts {
            before: String::new(),
            digits: "1".to_owned(),
            after: String::new(),
        },
    };

    // Only the scenes in front of a plain number count up in front of the digits.
    let in_front = match last {
        None => later_plain,
        Some(_) => !parts.before.is_empty() && later_plain,
    };
    if last.is_none() && !later_plain && !used.contains(&parts.text()) {
        return parts.text();
    }

    loop {
        match in_front {
            true => parts.before = bump(&parts.before),
            false => parts.after = bump(&parts.after),
        }
        if !used.contains(&parts.text()) {
            return parts.text();
        }
    }
}

/// `line` with its number set: a `#..#` at its end is replaced, a heading without one gets it.
fn with_number(line: &str, number: &str) -> String {
    let (line, ret) = line
        .strip_suffix('\r')
        .map_or((line, ""), |line| (line, "\r"));
    let line = line.trim_end();
    let (base, _) = split_number(line);

    numbered_head(base, number) + ret
}

/// The struck numbers of the Sperrstand with the line each heading goes in front of; `None`
/// for the end of the text. In the order of the Sperrstand.
fn struck_places(heads: &[Heading], locked: &[String]) -> Vec<(Option<usize>, String)> {
    let present = |number: &String| heads.iter().position(|h| h.number.as_ref() == Some(number));
    let mut places = Vec::new();

    for (at, number) in locked.iter().enumerate() {
        let done = places.iter().any(|(_, n): &(_, String)| n == number);
        if present(number).is_some() || done {
            continue;
        }

        let next = locked[at + 1..].iter().find_map(present);
        places.push((next.map(|head| heads[head].line), number.clone()));
    }

    places
}

/// Gives the last line its ending when the text has no final newline to give it one.
fn end_last_line(lines: &mut [String], newline: bool, ret: &str) {
    let Some(last) = lines.last_mut().filter(|_| !newline) else {
        return;
    };

    last.push_str(ret);
}

/// Puts a heading for every struck number of the Sperrstand the text lacks in front of the
/// next number it still has, so the number stays in the book.
fn insert_struck(lines: &mut Vec<String>, heads: &[Heading], rules: &NumberRules) {
    // The line ending of the book, for the lines made here.
    let ret = if lines.iter().any(|l| l.ends_with('\r')) {
        "\r"
    } else {
        ""
    };
    // The text's last newline stays last.
    let newline = lines.len() > 1 && lines.last().is_some_and(String::is_empty);
    if newline {
        lines.pop();
    }
    let end = lines.len();
    let mut places: Vec<(usize, String)> = struck_places(heads, rules.locked)
        .into_iter()
        .map(|(line, number)| (line.unwrap_or(end), number))
        .collect();
    places.sort_by_key(|(line, _)| *line);

    // In order of the lines; `shift` is what the headings before added.
    let mut shift = 0;
    for (line, number) in places {
        let heading = numbered_head(&format!(".{}", rules.omitted), &number);
        if line == end {
            if lines.last().is_some_and(|l| !l.trim().is_empty()) {
                end_last_line(lines, newline, ret);
                lines.push(ret.to_owned());
            }
            // The last line has an ending only if the text's last newline follows it.
            lines.push(format!("{heading}{}", if newline { ret } else { "" }));
            continue;
        }

        let at = line + shift;
        lines.splice(at..at, [format!("{heading}{ret}"), ret.to_owned()]);
        shift += 2;
    }

    if newline {
        lines.push(String::new());
    }
}
