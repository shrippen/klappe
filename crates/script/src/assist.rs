//! Writing aids on Fountain: what the elements of a screenplay are named so far (for
//! completion) and the suggestions for the line being typed. Surfaces show them; the
//! rules live here. The element flow of Enter and Tab is in `flow.rs`.

use std::collections::HashMap;

use crate::parse::{heading_prefix, DUAL_MARK, FORCE_CHARACTER, FORCE_HEADING, NUMBER_MARK};
use crate::{Element, Located};

/// Scene heading prefixes offered; the forced German ones start with the `.` that makes a
/// heading of any line.
const PREFIXES: [&str; 8] = [
    "INT.",
    "EXT.",
    "INT./EXT.",
    "I/E",
    "EST.",
    ".INNEN",
    ".AUSSEN",
    ".INNEN/AUSSEN",
];
/// Times of day offered after the location, German first.
const TIMES: [&str; 20] = [
    "TAG",
    "NACHT",
    "MORGEN",
    "ABEND",
    "DÄMMERUNG",
    "MORGENDÄMMERUNG",
    "ABENDDÄMMERUNG",
    "SPÄTER",
    "KONTINUIERLICH",
    "GLEICHZEITIG",
    "DAY",
    "NIGHT",
    "MORNING",
    "EVENING",
    "DAWN",
    "DUSK",
    "LATER",
    "CONTINUOUS",
    "SAME",
    "MOMENTS LATER",
];
/// Extensions of a cue, between the parentheses.
const EXTENSIONS: [&str; 7] = ["OFF", "V.O.", "O.S.", "O.C.", "CONT'D", "WEITER", "FORTS."];
/// Between the location and the time of a heading.
const SEPARATORS: [&str; 3] = [" - ", " – ", " — "];
/// What a suggestion for a location puts behind it, so the time of day can follow.
const LOCATION_END: &str = " - ";
const OPEN: char = '(';
const CLOSE: char = ')';

/// What the screenplay already names, for completion.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Vocabulary {
    /// Line and name of every cue, in text order.
    cues: Vec<(usize, String)>,
    places: Vec<(usize, String)>,
    times: Vec<(usize, String)>,
}

/// The words most frequent first, the earlier one first among equals; the words of line
/// `skip` do not count (the line being typed would suggest itself).
fn ranked(words: &[(usize, String)], skip: Option<usize>) -> Vec<String> {
    let mut seen: HashMap<String, usize> = HashMap::new();
    let mut counted: Vec<(&str, usize)> = Vec::new();

    for (_, word) in words.iter().filter(|(line, _)| Some(*line) != skip) {
        let at = *seen.entry(word.to_lowercase()).or_insert_with(|| {
            counted.push((word, 0));
            counted.len() - 1
        });
        counted[at].1 += 1;
    }
    counted.sort_by_key(|(_, n)| std::cmp::Reverse(*n));

    counted.into_iter().map(|(w, _)| w.to_owned()).collect()
}

/// Whether two names are the same in any case, umlauts included.
pub(crate) fn same_name(a: &str, b: &str) -> bool {
    a.to_lowercase() == b.to_lowercase()
}

/// A heading's text cut into location and time of day, without its prefix.
fn split_heading(text: &str) -> (&str, Option<&str>) {
    let prefix = heading_prefix(text).map_or(0, str::len);
    let rest = text.get(prefix..).unwrap_or(text);
    let rest = rest
        .strip_prefix(FORCE_HEADING)
        .unwrap_or(rest)
        .trim_start();

    let cut = SEPARATORS
        .iter()
        .filter_map(|s| rest.rfind(s).map(|at| (at, s.len())))
        .max_by_key(|(at, _)| *at);
    match cut {
        Some((at, len)) => (rest[..at].trim(), Some(rest[at + len..].trim())),
        None => (rest.trim(), None),
    }
}

impl Vocabulary {
    /// Reads the names from the elements of a located screenplay.
    pub fn from_located(located: &[Located]) -> Self {
        let mut vocab = Self::default();

        for item in located {
            let line = item.lines.first().copied().unwrap_or(0);
            match &item.element {
                Element::Character { name, .. } => vocab.cues.push((line, name.clone())),
                Element::SceneHeading { text, .. } => {
                    let (place, time) = split_heading(text);
                    if !place.is_empty() {
                        vocab.places.push((line, place.to_owned()));
                    }
                    if let Some(time) = time.filter(|t| !t.is_empty()) {
                        vocab.times.push((line, time.to_owned()));
                    }
                }
                _ => {}
            }
        }

        vocab
    }

    /// Cue names, most frequent first.
    pub fn characters(&self) -> Vec<String> {
        ranked(&self.cues, None)
    }

    pub fn locations(&self) -> Vec<String> {
        ranked(&self.places, None)
    }

    /// In a conversation the other one: the last speaker above `line` who is not the one
    /// who spoke last. `None` before two have spoken.
    pub fn other_speaker(&self, line: usize) -> Option<&str> {
        let mut above = self.cues.iter().rev().filter(|(at, _)| *at < line);
        let last = &above.next()?.1;

        above
            .find(|(_, name)| !same_name(name, last))
            .map(|(_, name)| name.as_str())
    }
}

/// Which element the line being typed is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Context {
    Heading,
    Cue,
    /// Anything else: only a line that has become a heading or a forced cue gets suggestions.
    Line,
}

/// What part of the line a suggestion completes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Prefix,
    Location,
    Time,
    Name,
    Extension,
}

/// One thing to put in place of the line from `Suggestions::from` on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Suggestions {
    pub stage: Stage,
    /// Character column the replaced text starts at; it runs to the end of the line.
    pub from: usize,
    pub options: Vec<String>,
}

impl Stage {
    /// Whether taking a suggestion of this stage ends the element: the writer goes on
    /// with the next one (a time of day ends a heading, a name or extension a cue).
    pub fn finishes(self) -> bool {
        matches!(self, Self::Time | Self::Name | Self::Extension)
    }
}

/// Words of `pool` that begin with `typed`, in any case. An exact hit means the writer is done:
/// nothing is offered then.
fn matching<'a>(pool: impl Iterator<Item = &'a str>, typed: &str) -> Vec<String> {
    let typed = typed.to_lowercase();
    let mut out: Vec<String> = Vec::new();

    for word in pool {
        let lower = word.to_lowercase();
        if lower == typed {
            return Vec::new();
        }
        let known = out.iter().any(|o| o.to_lowercase() == lower);
        if lower.starts_with(&typed) && !known {
            out.push(word.to_owned());
        }
    }

    out
}

fn chars_before(line: &str, byte: usize) -> usize {
    line[..byte].chars().count()
}

/// Suggestions for the end of `line`, or `None` when there are none.
///
/// `line` is the text as typed; `line_no` places it in the screenplay for the conversation.
pub fn suggest(
    vocab: &Vocabulary,
    line: &str,
    line_no: usize,
    ctx: Context,
) -> Option<Suggestions> {
    let trimmed = line.trim_start();
    let ctx = context_of(trimmed, ctx);
    let skip = line.len() - trimmed.len() + line_marker(trimmed, ctx);
    let body = &line[skip..];
    if body.contains(NUMBER_MARK) || body.contains(DUAL_MARK) {
        return None;
    }

    let suggestions = match ctx {
        Context::Heading => heading_stage(vocab, line, line_no),
        Context::Cue => cue_stage(vocab, line, skip, line_no),
        Context::Line => None,
    }?;

    (!suggestions.options.is_empty()).then_some(suggestions)
}

/// Bytes of the line that are the forcing character of the element.
fn line_marker(trimmed: &str, ctx: Context) -> usize {
    let forced = match ctx {
        Context::Heading => FORCE_HEADING,
        Context::Cue => FORCE_CHARACTER,
        Context::Line => return 0,
    };

    usize::from(trimmed.starts_with(forced))
}

/// A line of no known element becomes one when it starts with a heading prefix or `@`.
fn context_of(trimmed: &str, ctx: Context) -> Context {
    if ctx != Context::Line {
        return ctx;
    }
    if trimmed.starts_with(FORCE_CHARACTER) {
        return Context::Cue;
    }

    let heading = heading_prefix(trimmed).is_some();
    if heading || trimmed.starts_with(FORCE_HEADING) && !trimmed.starts_with("..") {
        return Context::Heading;
    }

    Context::Line
}

fn heading_stage(vocab: &Vocabulary, line: &str, line_no: usize) -> Option<Suggestions> {
    // The forcing dot belongs to the German prefixes, so it is part of what is typed.
    let lead = line.len() - line.trim_start().len();
    let typed = &line[lead..];

    let Some(space) = typed.find(' ') else {
        let options = matching(PREFIXES.iter().copied(), typed.trim_end())
            .into_iter()
            .map(|p| format!("{p} "))
            .collect();
        return Some(Suggestions {
            stage: Stage::Prefix,
            from: chars_before(line, lead),
            options,
        });
    };

    let after = lead + space + 1;
    let rest = &line[after..];
    let separated = SEPARATORS
        .iter()
        .filter_map(|s| rest.rfind(s).map(|at| (at, s.len())))
        .max_by_key(|(at, _)| *at);

    let Some((at, len)) = separated else {
        let place = rest.trim_start();
        let places = ranked(&vocab.places, Some(line_no));
        let options = matching(places.iter().map(String::as_str), place.trim_end())
            .into_iter()
            .map(|l| format!("{l}{LOCATION_END}"))
            .collect();
        return Some(Suggestions {
            stage: Stage::Location,
            from: chars_before(line, line.len() - place.len()),
            options,
        });
    };

    let used = ranked(&vocab.times, Some(line_no));
    let pool = used.iter().map(String::as_str).chain(TIMES.iter().copied());
    Some(Suggestions {
        stage: Stage::Time,
        from: chars_before(line, after + at + len),
        options: matching(pool, rest[at + len..].trim_end()),
    })
}

fn cue_stage(vocab: &Vocabulary, line: &str, skip: usize, line_no: usize) -> Option<Suggestions> {
    let body = &line[skip..];
    let name_from = skip + (body.len() - body.trim_start().len());

    if let Some(open) = body.rfind(OPEN).filter(|_| !body.contains(CLOSE)) {
        let typed = body[open + 1..].trim_end();
        let options = matching(EXTENSIONS.iter().copied(), typed)
            .into_iter()
            .map(|e| format!("{OPEN}{e}{CLOSE}"))
            .collect();
        return Some(Suggestions {
            stage: Stage::Extension,
            from: chars_before(line, skip + open),
            options,
        });
    }
    if body.contains(CLOSE) {
        return None;
    }

    let typed = line[name_from..].trim_end();
    let options = if typed.is_empty() {
        vocab
            .other_speaker(line_no)
            .map(str::to_owned)
            .into_iter()
            .collect()
    } else {
        let names = ranked(&vocab.cues, Some(line_no));
        matching(names.iter().map(String::as_str), typed)
    };

    Some(Suggestions {
        stage: Stage::Name,
        from: chars_before(line, name_from),
        options,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flow::{Gap, Slot};
    use crate::parse_located;

    fn vocab(text: &str) -> Vocabulary {
        Vocabulary::from_located(&parse_located(text))
    }

    const SAMPLE: &str = "INT. KÜCHE - TAG\n\nA.\n\nANNA\nHi.\n\nBEN\nHo.\n\nANNA\nNa.\n\nEXT. HOF - NACHT\n\nCARL\nHm.\n\nINT. KÜCHE - NACHT\n\nB.\n";

    fn options(v: &Vocabulary, line: &str, ctx: Context) -> Vec<String> {
        suggest(v, line, 99, ctx)
            .map(|s| s.options)
            .unwrap_or_default()
    }

    #[test]
    fn names_come_most_frequent_first() {
        let v = vocab(SAMPLE);

        assert_eq!(v.characters(), ["ANNA", "BEN", "CARL"]);
        assert_eq!(v.locations(), ["KÜCHE", "HOF"]);
        assert_eq!(options(&v, "", Context::Cue), ["ANNA"]);
        assert_eq!(options(&v, "B", Context::Cue), ["BEN"]);
        assert_eq!(options(&v, "@c", Context::Line), ["CARL"]);
    }

    #[test]
    fn an_empty_cue_proposes_the_other_speaker() {
        let v = vocab("A\n\nANNA\nx\n\nBEN\ny\n");

        assert_eq!(v.other_speaker(99), Some("ANNA"));
        assert_eq!(options(&v, "", Context::Cue), ["ANNA"]);
        // Above the second cue only one has spoken.
        assert_eq!(v.other_speaker(5), None);
    }

    #[test]
    fn the_other_speaker_skips_repeats() {
        let v = vocab("ANNA\nx\n\nBEN\ny\n\nBEN\nz\n");

        assert_eq!(v.other_speaker(99), Some("ANNA"));
    }

    #[test]
    fn the_line_being_typed_does_not_suggest_itself() {
        // Line 4 is the cue being typed; it is a cue already, with its dialogue below.
        let v = vocab("INT. KÜ\n\nx\n\nAN\nHi.\n\nANNA\nHo.\n");

        assert_eq!(options_at(&v, "AN", 4, Context::Cue), ["ANNA"]);
        assert_eq!(
            options_at(&v, "INT. KÜ", 0, Context::Heading),
            Vec::<String>::new()
        );

        let v = vocab("INT. KÜCHE\n\nx\n\nINT. KÜ\n\ny\n");
        assert_eq!(options_at(&v, "INT. KÜ", 4, Context::Heading), ["KÜCHE - "]);
    }

    fn options_at(v: &Vocabulary, line: &str, at: usize, ctx: Context) -> Vec<String> {
        suggest(v, line, at, ctx)
            .map(|s| s.options)
            .unwrap_or_default()
    }

    #[test]
    fn an_exact_name_offers_nothing() {
        let v = vocab(SAMPLE);

        assert!(options(&v, "ANNA", Context::Cue).is_empty());
        assert_eq!(options(&v, "AN", Context::Cue), ["ANNA"]);
    }

    #[test]
    fn extensions_follow_an_open_parenthesis() {
        let v = vocab(SAMPLE);
        let s = suggest(&v, "ANNA (v", 0, Context::Cue);

        assert_eq!(s.as_ref().map(|s| s.stage), Some(Stage::Extension));
        assert_eq!(s.as_ref().map(|s| s.from), Some(5));
        assert_eq!(s.map(|s| s.options), Some(vec!["(V.O.)".to_string()]));
        assert!(options(&v, "ANNA (", Context::Cue).contains(&"(CONT'D)".to_string()));
        assert!(options(&v, "ANNA (OFF)", Context::Cue).is_empty());
    }

    #[test]
    fn prefixes_complete_with_a_space() {
        let v = vocab(SAMPLE);

        assert_eq!(options(&v, "IN", Context::Heading), ["INT. ", "INT./EXT. "]);
        assert_eq!(options(&v, "e", Context::Heading), ["EXT. ", "EST. "]);
        assert_eq!(
            options(&v, ".inn", Context::Heading),
            [".INNEN ", ".INNEN/AUSSEN "]
        );
        assert!(options(&v, "", Context::Heading).len() >= 5);
    }

    #[test]
    fn locations_then_times() {
        let v = vocab(SAMPLE);
        let s = suggest(&v, "INT. kü", 0, Context::Heading);

        assert_eq!(s.as_ref().map(|s| s.stage), Some(Stage::Location));
        assert_eq!(s.as_ref().map(|s| s.from), Some(5));
        assert_eq!(s.map(|s| s.options), Some(vec!["KÜCHE - ".to_string()]));

        let s = suggest(&v, "INT. KÜCHE - N", 0, Context::Heading);
        assert_eq!(s.as_ref().map(|s| s.stage), Some(Stage::Time));
        assert_eq!(s.as_ref().map(|s| s.from), Some(13));
        // The script's own times first, then German, then English.
        assert_eq!(
            s.map(|s| s.options),
            Some(vec!["NACHT".to_string(), "NIGHT".to_string()])
        );
    }

    #[test]
    fn every_time_of_day_is_offered_after_the_dash() {
        let v = Vocabulary::default();
        let all = options(&v, "INT. X – ", Context::Heading);

        assert_eq!(all.first().map(String::as_str), Some("TAG"));
        assert!(all.contains(&"CONTINUOUS".to_string()));
        assert!(options(&v, "INT. X – TAG", Context::Heading).is_empty());
    }

    #[test]
    fn a_plain_line_gets_suggestions_only_when_it_became_a_heading_or_cue() {
        let v = vocab(SAMPLE);

        assert!(options(&v, "Anna geht", Context::Line).is_empty());
        assert!(options(&v, "Ein Tag", Context::Line).is_empty());
        assert_eq!(options(&v, "int. h", Context::Line), ["HOF - "]);
        assert_eq!(options(&v, "@b", Context::Line), ["BEN"]);
        assert!(options(&v, "INT. KÜCHE #3#", Context::Line).is_empty());
        assert!(options(&v, "...und", Context::Line).is_empty());
    }

    #[test]
    fn the_enter_and_tab_table_of_fade_in() {
        use Gap::{Blank, None as Close};
        use Slot::*;

        for (slot, enter, tab) in [
            (Heading, (Action, Blank), None),
            (Action, (Action, Blank), Some(Character)),
            (Character, (Dialogue, Close), Some(Action)),
            (Parenthetical, (Dialogue, Close), Some(Dialogue)),
            (Dialogue, (Action, Blank), Some(Parenthetical)),
            (Transition, (Heading, Blank), None),
        ] {
            assert_eq!(slot.after_enter(), enter, "{slot:?}");
            assert_eq!(slot.on_tab(), tab, "{slot:?}");
        }
    }

    #[test]
    fn slots_of_elements() {
        let els = crate::parse("INT. A\n\nx\n\nCUT TO:\n\nB\n(c)\nd\n").elements;
        let slots: Vec<_> = els.iter().map(Slot::of).collect();

        assert_eq!(
            slots,
            [
                Some(Slot::Heading),
                Some(Slot::Action),
                Some(Slot::Transition),
                Some(Slot::Character),
                Some(Slot::Parenthetical),
                Some(Slot::Dialogue),
            ]
        );
        assert_eq!(Slot::of(&Element::PageBreak), None);
    }

    #[test]
    fn the_other_speaker_is_found_with_unicode_case() {
        let v = vocab("ÄRZTIN\nx\n\nBEN\ny\n\n@ärztin\nz\n");

        assert_eq!(v.other_speaker(99), Some("BEN"));
        let v = vocab("BEN\nx\n\nÄRZTIN\ny\n\n@ärztin\nz\n");
        assert_eq!(v.other_speaker(99), Some("BEN"));
    }

    #[test]
    fn forced_german_headings_give_their_place() {
        let v = vocab(".INNEN KÜCHE - TAG\n\nx\n\n.AUSSEN/INNEN HOF - NACHT\n\ny\n\n.INNEN/AUSSEN BAD\n\nz\n\n.GARTEN - TAG\n\nw\n");

        assert_eq!(v.locations(), ["KÜCHE", "HOF", "BAD", "GARTEN"]);
        assert_eq!(options(&v, ".innen k", Context::Line), ["KÜCHE - "]);
        assert_eq!(options(&v, ".innen ", Context::Heading).len(), 4);
    }
}
