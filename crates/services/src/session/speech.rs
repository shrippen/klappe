//! The spoken dialogue of the scene (LH-64): which line a speech voice reads at which time.

use klappe_model::ScriptLanguage;
use klappe_script::inline_runs;
use serde::Serialize;

use super::Session;
use crate::views::ScriptLine;
use crate::Language;

const DIALOGUE: &str = "dialogue";
const CUE: &str = "character";
const PARENTHETICAL: &str = "parenthetical";
/// The characters that mark emphasis in Fountain.
const EMPHASIS: [char; 2] = ['*', '_'];
/// Fountain's escape: `\*` is a star, not emphasis.
const ESCAPE: char = '\\';

/// A line a voice reads when the playhead gets to `at_s`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SpokenLine {
    /// Index within the scene, as `ScriptLine::index`.
    pub index: usize,
    pub at_s: f64,
    pub text: String,
}

/// What the voice reads in the scene on view, in the order of the time.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Speech {
    /// `de` or `en`: the project's language, the surface's while it has none.
    pub language: &'static str,
    pub lines: Vec<SpokenLine>,
}

impl Session {
    /// The dialogue of the scene on view with the time each line is read at.
    pub fn speech(&self, fallback: Language) -> Speech {
        let language = match (self.project().script.language, fallback) {
            (Some(ScriptLanguage::De), _) | (None, Language::German) => ScriptLanguage::De,
            _ => ScriptLanguage::En,
        };

        Speech {
            language: language.name(),
            lines: schedule(&self.script_lines()),
        }
    }
}

/// When each dialogue is read.
///
/// A dialogue starts with its own link, else with the link of its cue (the usual place of
/// a link). Without a link it starts with the nearest earlier line that has one, action
/// included (Elisabeth walks to the table, then speaks), and at 0 s before any link. Lines
/// that start together are read one after another, in the script's order; a line that
/// starts while another is read waits. Parentheticals and actions are not read.
fn schedule(lines: &[ScriptLine]) -> Vec<SpokenLine> {
    let mut spoken = Vec::new();
    let mut last_s = 0.0;

    for (i, line) in lines.iter().enumerate() {
        let cue_s = match line.kind == DIALOGUE {
            true => cue_of(lines, i).and_then(link_s),
            false => None,
        };
        let own_s = link_s(line).or(cue_s);
        if let Some(at_s) = own_s {
            last_s = at_s;
        }
        if line.kind != DIALOGUE {
            continue;
        }
        let text = readable(&line.text);
        if text.is_empty() {
            continue;
        }

        spoken.push(SpokenLine {
            index: line.index,
            at_s: own_s.unwrap_or(last_s),
            text,
        });
    }

    // Stable: equal times keep the script's order.
    spoken.sort_by(|a, b| a.at_s.total_cmp(&b.at_s));
    spoken
}

/// What a voice reads of Fountain text: notes and the marks of emphasis (`*`, `_`, and the
/// backslash of `\*`) are left out. Example: `Hallo [[Regie]] *laut*` is read as `Hallo laut`.
/// Boneyards are gone before (the parser cuts them); a `/*` here is text, as in `Und/* oder`.
fn readable(text: &str) -> String {
    let mut out = String::new();

    for line in text.lines() {
        let (runs, _) = inline_runs(line, false);
        let chars: Vec<char> = line.chars().collect();

        for run in runs {
            let marks = run.marks;
            if marks.note {
                continue;
            }
            let emphasis = marks.bold || marks.italic || marks.underline;
            let part = &chars[run.start..run.end];
            for (i, c) in part.iter().enumerate() {
                let escapes = *c == ESCAPE && part.get(i + 1).is_some_and(|n| EMPHASIS.contains(n));
                if escapes || (emphasis && EMPHASIS.contains(c)) {
                    continue;
                }
                out.push(*c);
            }
        }
        out.push(' ');
    }

    // Left-out parts leave double spaces.
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The earliest time the line is linked to.
fn link_s(line: &ScriptLine) -> Option<f64> {
    line.chips.iter().map(|c| c.at_s).min_by(f64::total_cmp)
}

/// The cue above the dialogue at `i`, past parentheticals.
fn cue_of(lines: &[ScriptLine], i: usize) -> Option<&ScriptLine> {
    lines[..i]
        .iter()
        .rev()
        .find(|l| l.kind != PARENTHETICAL)
        .filter(|l| l.kind == CUE)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::views::{Chip, LinkKind};

    fn chip(at_s: f64) -> Chip {
        Chip {
            kind: LinkKind::Mark,
            text: String::new(),
            target: String::new(),
            label: String::new(),
            at_s,
        }
    }

    fn line(index: usize, kind: &'static str, text: &str, links: &[f64]) -> ScriptLine {
        ScriptLine {
            index,
            kind,
            text: text.to_owned(),
            chips: links.iter().map(|t| chip(*t)).collect(),
            current: false,
            character: String::new(),
        }
    }

    /// Leaks the texts: test code only.
    fn read(lines: &[ScriptLine]) -> Vec<(f64, &'static str)> {
        schedule(lines)
            .into_iter()
            .map(|s| (s.at_s, &*Box::leak(s.text.into_boxed_str())))
            .collect()
    }

    #[test]
    fn a_link_on_the_cue_times_the_dialogue() {
        let lines = [
            line(0, "character", "ANNA", &[6.0]),
            line(1, "parenthetical", "(leise)", &[]),
            line(2, "dialogue", "Hallo.", &[]),
        ];

        assert_eq!(read(&lines), [(6.0, "Hallo.")]);
    }

    #[test]
    fn its_own_link_wins_over_the_cues() {
        let lines = [
            line(0, "character", "ANNA", &[6.0]),
            line(1, "dialogue", "Hallo.", &[4.0, 5.0]),
        ];

        assert_eq!(read(&lines), [(4.0, "Hallo.")]);
    }

    #[test]
    fn an_unlinked_dialogue_follows_the_last_linked_line() {
        let lines = [
            line(0, "dialogue", "Davor.", &[]),
            line(1, "action", "Sie geht.", &[3.0]),
            line(2, "character", "ANNA", &[]),
            line(3, "dialogue", "Danach.", &[]),
            line(4, "character", "BEN", &[8.0]),
            line(5, "dialogue", "Später.", &[]),
        ];

        assert_eq!(
            read(&lines),
            [(0.0, "Davor."), (3.0, "Danach."), (8.0, "Später.")]
        );
    }

    #[test]
    fn lines_of_one_time_keep_the_script_order_and_time_sorts() {
        let lines = [
            line(0, "character", "ANNA", &[9.0]),
            line(1, "dialogue", "Spät.", &[]),
            line(2, "character", "BEN", &[2.0]),
            line(3, "dialogue", "Eins.", &[]),
            line(4, "dialogue", "Zwei.", &[]),
        ];

        assert_eq!(
            read(&lines),
            [(2.0, "Eins."), (2.0, "Zwei."), (9.0, "Spät.")]
        );
    }

    #[test]
    fn only_dialogue_is_read() {
        let lines = [
            line(0, "action", "Regen.", &[1.0]),
            line(1, "character", "ANNA", &[]),
            line(2, "parenthetical", "(leise)", &[]),
            line(3, "dialogue", "  ", &[]),
            line(4, "transition", "SCHNITT", &[]),
        ];

        assert!(read(&lines).is_empty());
    }

    /// The dialogue text as the parser hands it over (boneyards cut, `/\*` unescaped).
    fn parsed(source: &str) -> String {
        let script = klappe_script::parse(&format!("ANNA\n{source}"));
        let text = script.elements.iter().find_map(|e| match e {
            klappe_script::Element::Dialogue(text) => Some(text.clone()),
            _ => None,
        });

        text.expect("a dialogue")
    }

    #[test]
    fn notes_and_emphasis_are_not_read() {
        let lines = [line(
            0,
            "dialogue",
            "Hallo [[Notiz an Regie]] *laut* _jetzt_.\nNoch **mehr**.",
            &[],
        )];

        assert_eq!(read(&lines), [(0.0, "Hallo laut jetzt. Noch mehr.")]);
    }

    #[test]
    fn an_escaped_mark_is_read_without_its_backslash() {
        let lines = [line(0, "dialogue", "Zwei \\* drei", &[])];

        assert_eq!(read(&lines), [(0.0, "Zwei * drei")]);
    }

    #[test]
    fn a_boneyard_opener_in_the_text_is_read() {
        let text = parsed("Hallo /*weg*/ *laut* Und/\\* oder.");
        assert_eq!(text, "Hallo  *laut* Und/* oder.");

        let lines = [line(0, "dialogue", &text, &[])];

        assert_eq!(read(&lines), [(0.0, "Hallo laut Und/* oder.")]);
    }

    #[test]
    fn a_dialogue_of_notes_only_is_not_read() {
        let lines = [line(0, "dialogue", "[[Nur eine Notiz]]", &[])];

        assert!(read(&lines).is_empty());
    }
}
