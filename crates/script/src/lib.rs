//! Fountain screenplay parser and writer, and the layout of a screenplay on pages.
//!
//! `parse` turns Fountain text into a [`Script`]; `write` turns it back into text that
//! parses to the same elements. Parsing never fails: input that matches no other rule
//! becomes an [`Element::Action`].

mod align;
mod anchor;
mod assist;
mod caps;
mod compare;
mod conflict;
mod flow;
mod inline;
mod kinds;
mod lcs;
mod numbering;
mod outline;
mod pages;
mod parse;
mod quote;
mod search;
mod text;
mod title;
mod write;

pub use align::{align, heading_print, scene_states, Alignment, Fingerprint, Outcome, SceneState};
pub use assist::{heading_place, suggest, Context, Stage, Suggestions, Vocabulary};
pub use caps::typed_caps;
pub use compare::{compare, Comparison, Edit, LineChange, SceneDiff, ScenePlace, SceneStatus};
pub use conflict::conflict_line;
pub use flow::{enter_step, force_line, tab_step, upper, EnterStep, Forced, Gap, Slot, TabStep};
pub use inline::{inline_runs, plain_text, Marks, Run};
pub use kinds::{located_kinds, text_kinds, TextKind};
pub use numbering::{is_omitted, number_scenes, NumberMode, NumberRules, OMITTED_DE, OMITTED_EN};
pub use outline::{move_entry, outline, outline_of, visible, Entry, EntryKind, Moved, Way};
pub use pages::{
    carry, compare_labels, length_text, lines_text, page_label_after, page_starts, paginate,
    paginate_locked, Align, ContdAfterAction, Line, Margin, Page, PageKind, PageOptions, PageSize,
    Pagination, SceneLength, Words, TEXT_COLUMNS,
};
pub use parse::{
    boneyard_lines, parse, parse_located, title_page_lines, FORCE_CHARACTER, NUMBER_MARK,
};
pub use quote::{find_quote, find_quotes, quote_at, Found, LineKind, Quote};
pub use search::{find_all, replace_hits, Case, Hit, Query, Splice};
pub use text::normal;
pub use title::{set_title_fields, title_fields, without_blank_lines, TITLE_KEYS};
pub use write::write;

/// One block of a screenplay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Element {
    /// `INT. SÄNGERSAAL – TAG #12#`; `text` has the number cut off.
    SceneHeading {
        text: String,
        number: Option<String>,
    },
    /// Lines of description; several lines are joined with `\n`.
    Action(String),
    /// Cue of a dialogue: `ELISABETH (V.O.)`, trailing `^` marks the second voice of a
    /// dual dialogue.
    Character {
        name: String,
        extension: Option<String>,
        dual: bool,
    },
    /// Text between the parentheses of `(ohne aufzusehen)`.
    Parenthetical(String),
    Dialogue(String),
    Transition(String),
    Centered(String),
    Section {
        depth: usize,
        text: String,
    },
    Synopsis(String),
    /// A `[[note]]` on a line of its own.
    Note(String),
    PageBreak,
    Lyrics(String),
}

/// An element and the lines of the text it was read from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Located {
    pub element: Element,
    /// From 0, in text order: one per line the element takes, several for an action or
    /// dialogue of more than one line.
    pub lines: Vec<usize>,
}

/// A parsed screenplay.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Script {
    /// `Key: value` pairs in file order; multi-line values are joined with `\n`.
    pub title_page: Vec<(String, String)>,
    pub elements: Vec<Element>,
}

/// The elements of one scene: everything after its heading up to the next heading.
///
/// The position in `elements` is the index a `ScriptLink` refers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SceneView<'a> {
    /// `None` for text before the first heading.
    pub heading: Option<&'a str>,
    pub number: Option<&'a str>,
    pub elements: &'a [Element],
}

impl Script {
    /// Splits the script at its scene headings.
    ///
    /// Text before the first heading forms a scene without heading, if there is any.
    pub fn scenes(&self) -> Vec<SceneView<'_>> {
        let mut scenes = Vec::new();
        let mut start = 0;
        let mut head: Option<(&str, Option<&str>)> = None;

        for (i, el) in self.elements.iter().enumerate() {
            let Element::SceneHeading { text, number } = el else {
                continue;
            };

            // Close the scene before this heading.
            if head.is_some() || i > start {
                scenes.push(view(head, &self.elements[start..i]));
            }
            head = Some((text, number.as_deref()));
            start = i + 1;
        }

        if head.is_some() || start < self.elements.len() {
            scenes.push(view(head, &self.elements[start..]));
        }

        scenes
    }

    /// The scene with this number, e.g. `"12"` or `"12A"`.
    pub fn scene(&self, number: &str) -> Option<SceneView<'_>> {
        self.scenes().into_iter().find(|s| s.number == Some(number))
    }
}

fn view<'a>(head: Option<(&'a str, Option<&'a str>)>, elements: &'a [Element]) -> SceneView<'a> {
    SceneView {
        heading: head.map(|h| h.0),
        number: head.and_then(|h| h.1),
        elements,
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod outline_tests;
