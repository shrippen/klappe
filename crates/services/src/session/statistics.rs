//! The statistics (DB-53) and the dialogue of one character (DB-52): who says how much, where
//! the book is shot and when, how long it is. Counted over the scenes in the book; a cue that
//! no character of the project answers to has no row, as in the cast.
//!
//! ```text
//! book scenes ─► replies (cue + brackets + lines) ─┬─► per character, share of lines
//!             │                                    └─► one character's rows
//!             └► length in eighths ─► per place, per time of day
//! ```

use std::collections::HashMap;

use klappe_export::ReportTable;
use klappe_model::{Character, Scene};
use klappe_script::{length_text, Element};
use serde::Serialize;

use super::uses::time_of;
use super::Session;
use crate::error::ServiceError;
use crate::Language;

const NUMBER_SEPARATOR: &str = "  ";
const WORD_SEPARATOR: &str = " ";
const PERCENT: f64 = 100.0;
/// A share shows one decimal; a German one with a comma.
const SHARE_DECIMALS: usize = 1;
const GERMAN_DECIMAL: char = ',';

/// What a character says in a row: its cue, the brackets and the lines under it.
struct Reply<'a> {
    scene: &'a Scene,
    character: &'a Character,
    brackets: Vec<String>,
    /// A line of the script each, as written.
    lines: Vec<String>,
}

/// A character of the dialogue choice: who speaks in the book, and how often.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DialogueFigure {
    pub id: String,
    pub name: String,
    pub replies: usize,
}

/// What a character said, over the whole book.
#[derive(Default)]
struct Tally<'a> {
    scenes: Vec<&'a str>,
    replies: usize,
    lines: usize,
}

impl Session {
    /// The characters with a reply in the book, most replies first, then by name; for the
    /// choice before the dialogue report.
    pub fn dialogue_figures(&self) -> Vec<DialogueFigure> {
        let replies = self.replies();
        let mut figures: Vec<DialogueFigure> = self
            .project
            .characters
            .iter()
            .map(|c| DialogueFigure {
                id: c.id.clone(),
                name: c.name.clone(),
                replies: replies.iter().filter(|r| r.character.id == c.id).count(),
            })
            .filter(|f| f.replies > 0)
            .collect();
        figures.sort_by(|a, b| b.replies.cmp(&a.replies).then_with(|| a.name.cmp(&b.name)));

        figures
    }

    /// The replies of the book scenes in book order.
    fn replies(&self) -> Vec<Reply<'_>> {
        let scenes = self.script.scenes();
        let mut out = Vec::new();

        for (entry, scene) in self.book_scenes() {
            let Some(view) = self.book_scene_of(entry).and_then(|b| scenes.get(b)) else {
                continue;
            };

            self.replies_in(scene, view.elements, &mut out);
        }

        out
    }

    /// Reads one scene: a cue opens a reply and the brackets and dialogue after it belong to
    /// it, until a line of another kind. With dual dialogue the second cue follows the first
    /// one's lines, so each of the two gets its own reply.
    fn replies_in<'a>(&'a self, scene: &'a Scene, elements: &[Element], out: &mut Vec<Reply<'a>>) {
        let mut open = false;

        for element in elements {
            match element {
                Element::Character { name, .. } => {
                    let named = self.project.characters.iter().find(|c| c.is_called(name));
                    open = named.is_some();
                    if let Some(character) = named {
                        out.push(Reply {
                            scene,
                            character,
                            brackets: Vec::new(),
                            lines: Vec::new(),
                        });
                    }
                }
                Element::Parenthetical(text) if open => {
                    if let Some(reply) = out.last_mut() {
                        reply.brackets.push(format!("({text})"));
                    }
                }
                Element::Dialogue(text) if open => {
                    if let Some(reply) = out.last_mut() {
                        reply.lines.extend(text.split('\n').map(str::to_owned));
                    }
                }
                // These stand between lines without ending a reply.
                Element::Note(_) | Element::Section { .. } | Element::Synopsis(_) => {}
                Element::PageBreak | Element::Parenthetical(_) | Element::Dialogue(_) => {}
                _ => open = false,
            }
        }
    }

    /// The statistics: the length of the book, the characters, the places and the times of
    /// day.
    pub(super) fn statistic_tables(&self, language: Language) -> Vec<ReportTable> {
        let t = |k: &str| language.text(k);
        let scene_head = t("report.scenes");
        let length_head = t("report.length");
        let (places, times) = self.length_rows(language);

        vec![
            ReportTable {
                title: t("report.statLength"),
                head: vec![t("report.pages"), t("report.minutes")],
                rows: vec![self.total_length(language)],
            },
            ReportTable {
                title: t("report.figures"),
                head: vec![
                    t("report.figure"),
                    t("report.dialogueScenes"),
                    t("report.replies"),
                    t("report.dialogueLines"),
                    t("report.share"),
                ],
                rows: self.figure_rows(language),
            },
            ReportTable {
                title: t("report.statPlaces"),
                head: vec![t("report.place"), scene_head.clone(), length_head.clone()],
                rows: places,
            },
            ReportTable {
                title: t("report.statTimes"),
                head: vec![t("report.time"), scene_head, length_head],
                rows: times,
            },
        ]
    }

    /// The whole screenplay in pages and estimated minutes, as the layout has it (DB-32);
    /// the minutes are rounded up from half a minute and never 0 for a book with text.
    fn total_length(&self, language: Language) -> Vec<String> {
        let Some(pages) = self.script_pages() else {
            return vec![String::new(); 2];
        };
        let minutes = match pages.minutes > 0.0 {
            true => pages.minutes.round().max(1.0),
            false => 0.0,
        };

        vec![
            pages.length.clone(),
            language
                .text("tui.minutes")
                .replace("{count}", &format!("{minutes:.0}")),
        ]
    }

    /// A row per character with a reply: scenes with dialogue, replies, lines and the share of
    /// all lines; most replies first, then by name.
    fn figure_rows(&self, language: Language) -> Vec<Vec<String>> {
        let replies = self.replies();
        let mut tallies: HashMap<&str, Tally> = HashMap::new();
        for reply in &replies {
            let tally = tallies.entry(&reply.character.id).or_default();
            if !tally.scenes.contains(&reply.scene.id.as_str()) {
                tally.scenes.push(&reply.scene.id);
            }
            tally.replies += 1;
            tally.lines += reply.lines.len();
        }

        let all_lines: usize = tallies.values().map(|t| t.lines).sum();
        let mut rows: Vec<(&Character, Tally)> = self
            .project
            .characters
            .iter()
            .filter_map(|c| Some((c, tallies.remove(c.id.as_str())?)))
            .collect();
        rows.sort_by(|(a, x), (b, y)| y.replies.cmp(&x.replies).then_with(|| a.name.cmp(&b.name)));

        rows.into_iter()
            .map(|(c, t)| {
                vec![
                    c.name.clone(),
                    t.scenes.len().to_string(),
                    t.replies.to_string(),
                    t.lines.to_string(),
                    share_text(t.lines, all_lines, language),
                ]
            })
            .collect()
    }

    /// The scenes and their length by place and by time of day, each in the order of first
    /// appearance in the book; a scene without either is counted under "none".
    fn length_rows(&self, language: Language) -> (Vec<Vec<String>>, Vec<Vec<String>>) {
        let eighths = self.scene_eighths();
        let (places, _) = self.shot_scenes();
        let (mut by_place, mut by_time) = (Groups::default(), Groups::default());

        for (entry, scene) in self.book_scenes() {
            let length = eighths.get(&scene.id).copied().unwrap_or_default();
            let place = places.get(entry).copied().flatten();
            let time = time_of(&scene.heading);

            by_place.add(
                place.map(|p| p.id.as_str()).unwrap_or_default(),
                place.map_or_else(|| language.text("report.noPlace"), |p| p.name.clone()),
                length,
            );
            by_time.add(
                &time.to_uppercase(),
                if time.is_empty() {
                    language.text("report.noTime")
                } else {
                    time
                },
                length,
            );
        }

        (by_place.rows(), by_time.rows())
    }

    /// Scene, brackets and text of every reply of the character `figure`, in book order. A
    /// brackets line stays with its reply; the text is the reply's lines in one.
    pub(super) fn dialogue_table(
        &self,
        figure: &str,
        language: Language,
    ) -> Result<ReportTable, ServiceError> {
        let character = self
            .project
            .character(figure)
            .ok_or_else(|| ServiceError::NoCharacter(figure.to_owned()))?;
        let rows = self
            .replies()
            .iter()
            .filter(|r| r.character.id == character.id)
            .map(|r| {
                vec![
                    format!("{}{NUMBER_SEPARATOR}{}", r.scene.number, r.scene.heading)
                        .trim()
                        .to_owned(),
                    r.brackets.join(WORD_SEPARATOR),
                    r.lines.join(WORD_SEPARATOR),
                ]
            })
            .collect();

        Ok(ReportTable {
            title: format!("{} – {}", language.text("report.dialog"), character.name),
            head: vec![
                language.text("report.scene"),
                language.text("report.bracket"),
                language.text("report.speech"),
            ],
            rows,
        })
    }
}

/// Scenes and eighths by a key, in the order the keys first came.
#[derive(Default)]
struct Groups {
    /// Key, shown name, scenes, eighths.
    entries: Vec<(String, String, usize, usize)>,
}

impl Groups {
    fn add(&mut self, key: &str, name: String, eighths: usize) {
        let at = self.entries.iter().position(|e| e.0 == key);
        let entry = match at {
            Some(i) => &mut self.entries[i],
            None => {
                self.entries.push((key.to_owned(), name, 0, 0));
                let last = self.entries.len() - 1;

                &mut self.entries[last]
            }
        };

        entry.2 += 1;
        entry.3 += eighths;
    }

    fn rows(&self) -> Vec<Vec<String>> {
        self.entries
            .iter()
            .map(|(_, name, scenes, eighths)| {
                vec![name.clone(), scenes.to_string(), length_text(*eighths)]
            })
            .collect()
    }
}

/// `part` of `whole` in percent with one decimal, `0.0` for no whole.
fn share_text(part: usize, whole: usize, language: Language) -> String {
    let share = match whole {
        0 => 0.0,
        _ => part as f64 * PERCENT / whole as f64,
    };
    let text = format!("{share:.SHARE_DECIMALS$}");

    match language {
        Language::German => text.replace('.', &GERMAN_DECIMAL.to_string()),
        Language::English => text,
    }
}
