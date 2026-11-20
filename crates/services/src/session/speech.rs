//! The spoken dialogue of the scene (LH-64): which line a speech voice reads at which time.

use klappe_script::{plain_text, Element};
use serde::Serialize;

use super::Session;
use crate::Language;

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
        let language = self
            .project()
            .script
            .language
            .unwrap_or_else(|| fallback.into());

        let items: Vec<Item> = self
            .scene_elements()
            .iter()
            .enumerate()
            .map(|(i, element)| Item {
                element,
                link_s: self.link_s(i),
            })
            .collect();

        Speech {
            language: language.name(),
            lines: schedule(&items),
        }
    }
}

/// An element of the scene with the earliest time it is linked to.
struct Item<'a> {
    element: &'a Element,
    link_s: Option<f64>,
}

/// When each dialogue is read.
///
/// A dialogue starts with its own link, else with the link of its cue (the usual place of
/// a link). Without a link it starts with the nearest earlier line that has one, action
/// included (Elisabeth walks to the table, then speaks), and at 0 s before any link. Lines
/// that start together are read one after another, in the script's order; a line that
/// starts while another is read waits. Parentheticals and actions are not read.
fn schedule(items: &[Item]) -> Vec<SpokenLine> {
    let mut spoken = Vec::new();
    let mut last_s = 0.0;

    for (i, item) in items.iter().enumerate() {
        let Element::Dialogue(dialogue) = item.element else {
            if let Some(at_s) = item.link_s {
                last_s = at_s;
            }
            continue;
        };
        let own_s = item.link_s.or(cue_of(items, i).and_then(|cue| cue.link_s));
        if let Some(at_s) = own_s {
            last_s = at_s;
        }
        let text = plain_text(dialogue);
        if text.is_empty() {
            continue;
        }

        spoken.push(SpokenLine {
            index: i,
            at_s: own_s.unwrap_or(last_s),
            text,
        });
    }

    // Stable: equal times keep the script's order.
    spoken.sort_by(|a, b| a.at_s.total_cmp(&b.at_s));
    spoken
}

/// The cue above the dialogue at `i`, past parentheticals.
fn cue_of<'a>(items: &'a [Item], i: usize) -> Option<&'a Item<'a>> {
    items[..i]
        .iter()
        .rev()
        .find(|l| !matches!(l.element, Element::Parenthetical(_)))
        .filter(|l| matches!(l.element, Element::Character { .. }))
}

#[cfg(test)]
mod tests {
    use super::*;
    /// An element of the scene and its links.
    type Entry = (Element, Option<f64>);

    fn line(kind: &str, text: &str, links: &[f64]) -> Entry {
        let element = match kind {
            "character" => Element::Character {
                name: text.to_owned(),
                extension: None,
                dual: false,
            },
            "parenthetical" => Element::Parenthetical(text.to_owned()),
            "dialogue" => Element::Dialogue(text.to_owned()),
            "transition" => Element::Transition(text.to_owned()),
            _ => Element::Action(text.to_owned()),
        };

        (element, links.iter().copied().min_by(f64::total_cmp))
    }

    /// Leaks the texts: test code only.
    fn read(lines: &[Entry]) -> Vec<(f64, &'static str)> {
        let items: Vec<Item> = lines
            .iter()
            .map(|(element, link_s)| Item {
                element,
                link_s: *link_s,
            })
            .collect();

        schedule(&items)
            .into_iter()
            .map(|s| (s.at_s, &*Box::leak(s.text.into_boxed_str())))
            .collect()
    }

    #[test]
    fn a_link_on_the_cue_times_the_dialogue() {
        let lines = [
            line("character", "ANNA", &[6.0]),
            line("parenthetical", "(leise)", &[]),
            line("dialogue", "Hallo.", &[]),
        ];

        assert_eq!(read(&lines), [(6.0, "Hallo.")]);
    }

    #[test]
    fn its_own_link_wins_over_the_cues() {
        let lines = [
            line("character", "ANNA", &[6.0]),
            line("dialogue", "Hallo.", &[4.0, 5.0]),
        ];

        assert_eq!(read(&lines), [(4.0, "Hallo.")]);
    }

    #[test]
    fn an_unlinked_dialogue_follows_the_last_linked_line() {
        let lines = [
            line("dialogue", "Davor.", &[]),
            line("action", "Sie geht.", &[3.0]),
            line("character", "ANNA", &[]),
            line("dialogue", "Danach.", &[]),
            line("character", "BEN", &[8.0]),
            line("dialogue", "Später.", &[]),
        ];

        assert_eq!(
            read(&lines),
            [(0.0, "Davor."), (3.0, "Danach."), (8.0, "Später.")]
        );
    }

    #[test]
    fn lines_of_one_time_keep_the_script_order_and_time_sorts() {
        let lines = [
            line("character", "ANNA", &[9.0]),
            line("dialogue", "Spät.", &[]),
            line("character", "BEN", &[2.0]),
            line("dialogue", "Eins.", &[]),
            line("dialogue", "Zwei.", &[]),
        ];

        assert_eq!(
            read(&lines),
            [(2.0, "Eins."), (2.0, "Zwei."), (9.0, "Spät.")]
        );
    }

    #[test]
    fn only_dialogue_is_read() {
        let lines = [
            line("action", "Regen.", &[1.0]),
            line("character", "ANNA", &[]),
            line("parenthetical", "(leise)", &[]),
            line("dialogue", "  ", &[]),
            line("transition", "SCHNITT", &[]),
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
            "dialogue",
            "Hallo [[Notiz an Regie]] *laut* _jetzt_.\nNoch **mehr**.",
            &[],
        )];

        assert_eq!(read(&lines), [(0.0, "Hallo laut jetzt. Noch mehr.")]);
    }

    #[test]
    fn an_escaped_mark_is_read_without_its_backslash() {
        let lines = [line("dialogue", "Zwei \\* drei", &[])];

        assert_eq!(read(&lines), [(0.0, "Zwei * drei")]);
    }

    #[test]
    fn a_boneyard_opener_in_the_text_is_read() {
        let text = parsed("Hallo /*weg*/ *laut* Und/\\* oder.");
        assert_eq!(text, "Hallo  *laut* Und/* oder.");

        let lines = [line("dialogue", &text, &[])];

        assert_eq!(read(&lines), [(0.0, "Hallo laut Und/* oder.")]);
    }

    #[test]
    fn a_dialogue_of_notes_only_is_not_read() {
        let lines = [line("dialogue", "[[Nur eine Notiz]]", &[])];

        assert!(read(&lines).is_empty());
    }
}
