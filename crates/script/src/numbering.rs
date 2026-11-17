//! Scene numbers in the text (DB-33): sequential while automatic, kept while locked, with a
//! new scene numbered between its neighbours (12A) and a struck one left as "OMITTED".

use std::collections::HashSet;

use crate::{parse_located, Element, NUMBER_MARK};

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

/// A scene that left the book and keeps its number as an "OMITTED" heading.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Struck {
    /// The heading, counted from 0 among the scene headings of the text, it follows;
    /// `None` puts it before the first.
    pub after: Option<usize>,
    pub number: String,
}

#[derive(Debug, Clone, Copy)]
pub struct NumberRules<'a> {
    pub mode: NumberMode,
    /// The first number while automatic.
    pub first: u32,
    /// The heading of a struck scene (`OMITTED_DE`, `OMITTED_EN`).
    pub omitted: &'a str,
    pub struck: &'a [Struck],
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
    let heads = headings(text);
    if heads.is_empty() {
        return text.to_owned();
    }

    let wanted = assign(&heads, rules);
    let mut lines: Vec<String> = text.split('\n').map(str::to_owned).collect();
    for (head, number) in heads.iter().zip(&wanted) {
        if head.number.as_ref() != Some(number) {
            lines[head.line] = with_number(&lines[head.line], number);
        }
    }

    insert_struck(&mut lines, &heads, rules);

    lines.join("\n")
}

/// The number of every heading. A struck scene's heading keeps its number in both modes and
/// takes no part in the counting.
fn assign(heads: &[Heading], rules: &NumberRules) -> Vec<String> {
    let numbered = heads
        .iter()
        .any(|h| h.number.is_some() && !is_omitted(&h.text));
    if rules.mode == NumberMode::Automatic || !numbered {
        return sequential(heads, rules.first);
    }

    let mut used: HashSet<String> = heads.iter().filter_map(|h| h.number.clone()).collect();
    let mut last: Option<Parts> = None;
    let mut out = Vec::new();

    for (i, head) in heads.iter().enumerate() {
        let number = match &head.number {
            Some(number) => number.clone(),
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

/// The next letters after `letters`: `A`, `B` .. `Z`, `AA`, `AB` ..
fn bump(letters: &str) -> String {
    let mut chars: Vec<char> = letters.chars().collect();

    for at in (0..chars.len()).rev() {
        if chars[at] != 'Z' {
            chars[at] = char::from(chars[at] as u8 + 1);
            return chars.into_iter().collect();
        }
        chars[at] = 'A';
    }

    format!("A{}", chars.into_iter().collect::<String>())
}

/// The number of a scene that has none: after `last` it is `last` with the next letters
/// (12 then 12A, 12A then 12B); before every number it is `A` and the number of the next
/// scene that has one (A1, then B1). A number in use is skipped.
fn next_free(last: Option<&Parts>, later: Option<&str>, used: &HashSet<String>) -> String {
    let start = last.cloned().or_else(|| {
        let mut parts = later.and_then(Parts::of)?;
        parts.before = String::new();
        parts.after = String::new();
        Some(parts)
    });
    let Some(mut parts) = start else {
        return "1".to_owned();
    };

    // Leading scenes count up in front of the digits, the others behind them.
    let in_front = last.is_none() || !parts.before.is_empty();
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
    let line = line.trim_end();
    let base = line
        .strip_suffix(NUMBER_MARK)
        .and_then(|body| {
            let at = body.rfind(NUMBER_MARK)?;
            let old = &body[at + 1..];
            let plain = !old.is_empty() && !old.contains(char::is_whitespace);

            plain.then(|| body[..at].trim_end())
        })
        .unwrap_or(line);

    format!("{base} {NUMBER_MARK}{number}{NUMBER_MARK}")
}

/// Puts a heading for every struck scene of `rules` that the text lacks behind the scene it
/// followed, so the number stays in the book.
fn insert_struck(lines: &mut Vec<String>, heads: &[Heading], rules: &NumberRules) {
    let have = |number: &str| {
        heads
            .iter()
            .any(|h| is_omitted(&h.text) && h.number.as_deref() == Some(number))
    };
    // The text's last newline stays last.
    let newline = lines.len() > 1 && lines.last().is_some_and(String::is_empty);
    if newline {
        lines.pop();
    }
    let end = lines.len();
    let mut places: Vec<(usize, &str)> = Vec::new();

    for struck in rules.struck.iter().filter(|s| !have(&s.number)) {
        let next = struck.after.map_or(0, |after| after + 1);
        let line = heads.get(next).map_or(end, |head| head.line);
        places.push((line, &struck.number));
    }
    places.sort_by_key(|(line, _)| *line);

    // In order of the lines; `shift` is what the headings before added.
    let mut shift = 0;
    for (line, number) in places {
        let heading = format!(".{} {NUMBER_MARK}{number}{NUMBER_MARK}", rules.omitted);
        if line == end {
            if lines.last().is_some_and(|l| !l.is_empty()) {
                lines.push(String::new());
            }
            lines.push(heading);
            continue;
        }

        let at = line + shift;
        lines.splice(at..at, [heading, String::new()]);
        shift += 2;
    }

    if newline {
        lines.push(String::new());
    }
}
