//! What each line of the text is, for an editor that sets the elements as the page does
//! (D-15): a cue indented like on paper, a transition at the right.
//!
//! ```text
//! INT. SAAL - TAG       Heading
//!                       Blank
//!         ANNA          Character   (columns of `PageKind::margin`)
//!     Ihr kommt spät.   Dialogue
//! ```

use std::cell::RefCell;

use crate::{parse_located, title_page_lines, Element, Located, PageKind, Slot};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextKind {
    Blank,
    /// A line of the title page (`Title: …`).
    Title,
    Heading,
    Action,
    Character,
    Parenthetical,
    Dialogue,
    Transition,
    Centered,
    Lyrics,
    Section,
    Synopsis,
    Note,
    PageBreak,
}

impl TextKind {
    /// The kind it is printed as; `None` for what the page does not show.
    pub fn page_kind(self) -> Option<PageKind> {
        match self {
            TextKind::Heading => Some(PageKind::Heading),
            TextKind::Action => Some(PageKind::Action),
            TextKind::Character => Some(PageKind::Character),
            TextKind::Parenthetical => Some(PageKind::Parenthetical),
            TextKind::Dialogue => Some(PageKind::Dialogue),
            TextKind::Transition => Some(PageKind::Transition),
            TextKind::Centered => Some(PageKind::Centered),
            TextKind::Lyrics => Some(PageKind::Lyrics),
            TextKind::Blank
            | TextKind::Title
            | TextKind::Section
            | TextKind::Synopsis
            | TextKind::Note
            | TextKind::PageBreak => None,
        }
    }

    /// The element the line is for Enter and Tab; `None` for those outside the flow.
    pub fn slot(self) -> Option<Slot> {
        match self {
            TextKind::Heading => Some(Slot::Heading),
            TextKind::Action => Some(Slot::Action),
            TextKind::Character => Some(Slot::Character),
            TextKind::Parenthetical => Some(Slot::Parenthetical),
            TextKind::Dialogue => Some(Slot::Dialogue),
            TextKind::Transition => Some(Slot::Transition),
            _ => None,
        }
    }

    /// The kind of line an element of the flow is.
    pub fn of_slot(slot: Slot) -> Self {
        match slot {
            Slot::Heading => TextKind::Heading,
            Slot::Action => TextKind::Action,
            Slot::Character => TextKind::Character,
            Slot::Parenthetical => TextKind::Parenthetical,
            Slot::Dialogue => TextKind::Dialogue,
            Slot::Transition => TextKind::Transition,
        }
    }

    fn of(element: &Element) -> Self {
        match element {
            Element::SceneHeading { .. } => TextKind::Heading,
            Element::Action(_) => TextKind::Action,
            Element::Character { .. } => TextKind::Character,
            Element::Parenthetical(_) => TextKind::Parenthetical,
            Element::Dialogue(_) => TextKind::Dialogue,
            Element::Transition(_) => TextKind::Transition,
            Element::Centered(_) => TextKind::Centered,
            Element::Lyrics(_) => TextKind::Lyrics,
            Element::Section { .. } => TextKind::Section,
            Element::Synopsis(_) => TextKind::Synopsis,
            Element::Note(_) => TextKind::Note,
            Element::PageBreak => TextKind::PageBreak,
        }
    }
}

thread_local! {
    /// The text `text_kinds` read last, and its kinds.
    static LAST_KINDS: RefCell<(String, Vec<TextKind>)> = const { RefCell::new((String::new(), Vec::new())) };
}

/// The kind of every line of `text` (split at `\n`).
///
/// The editor asks for the kinds of the same text again and again: after an edit for the new
/// text, and before the next one for what has become the old; the last answer is kept.
pub fn text_kinds(text: &str) -> Vec<TextKind> {
    LAST_KINDS.with(|last| {
        let mut last = last.borrow_mut();
        if last.0 != text || last.1.is_empty() {
            last.1 = located_kinds(text, &parse_located(text));
            last.0 = text.to_owned();
        }

        last.1.clone()
    })
}

/// `text_kinds` for a text already read as `located`. The reader says which lines each
/// element stands on; a line of no element (a boneyard) is action, or blank when empty.
pub fn located_kinds(text: &str, located: &[Located]) -> Vec<TextKind> {
    let mut kinds: Vec<TextKind> = text
        .split('\n')
        .map(|line| match line.trim().is_empty() {
            true => TextKind::Blank,
            false => TextKind::Action,
        })
        .collect();

    for item in located {
        let kind = TextKind::of(&item.element);
        for &at in &item.lines {
            if let Some(slot) = kinds.get_mut(at) {
                *slot = kind;
            }
        }
    }

    for kind in kinds.iter_mut().take(title_page_lines(text)) {
        *kind = TextKind::Title;
    }

    kinds
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Align;

    /// The kept answer is for the text it was made for only.
    #[test]
    fn the_last_answer_is_kept_for_the_same_text_only() {
        let heading = "INT. SAAL - TAG";
        let action = "Anna tritt ein.";

        assert_eq!(text_kinds(heading), [TextKind::Heading]);
        assert_eq!(text_kinds(heading), [TextKind::Heading]);
        assert_eq!(text_kinds(action), [TextKind::Action]);
        assert_eq!(text_kinds(heading), [TextKind::Heading]);
    }

    #[test]
    fn kinds_of_a_dialogue_scene() {
        use TextKind::*;

        let text = "INT. SAAL - TAG #12#\n\nAnna tritt ein.\n\nANNA\n(leise)\nIhr kommt\nspät.\n\nCUT TO:\n";

        assert_eq!(
            text_kinds(text),
            [
                Heading,
                Blank,
                Action,
                Blank,
                Character,
                Parenthetical,
                Dialogue,
                Dialogue,
                Blank,
                Transition,
                Blank
            ]
        );
    }

    #[test]
    fn a_title_page_and_notes_are_not_printed() {
        use TextKind::*;

        let kinds = text_kinds("Title: Nachtwache\n\n# Akt I\n\n= Sie kommt an.\n\n[[Licht?]]\n");

        assert_eq!(
            kinds,
            [Title, Blank, Section, Blank, Synopsis, Blank, Note, Blank]
        );
        assert!(kinds.iter().all(|k| k.page_kind().is_none()));
    }

    /// The editor indents a line as the page does: one table for both.
    #[test]
    fn a_cue_takes_the_columns_of_the_page() {
        let cue = TextKind::Character.page_kind().map(PageKind::margin);
        let transition = TextKind::Transition.page_kind().map(PageKind::margin);

        assert_eq!(cue.map(|m| (m.x, m.width)), Some((22, 38)));
        assert_eq!(transition.map(|m| m.align), Some(Align::Right));
    }

    #[test]
    fn an_empty_text_is_one_blank_line() {
        assert_eq!(text_kinds(""), [TextKind::Blank]);
    }

    /// The terminal and the window take the role of a line from here.
    #[test]
    fn a_kind_and_its_slot_go_both_ways() {
        use TextKind::*;

        for kind in [
            Heading,
            Action,
            Character,
            Parenthetical,
            Dialogue,
            Transition,
        ] {
            let slot = kind.slot().expect("part of the flow");
            assert_eq!(TextKind::of_slot(slot), kind);
        }

        let outside = [
            Blank, Centered, Section, Synopsis, Note, PageBreak, Lyrics, Title,
        ];
        assert!(outside.iter().all(|k| k.slot().is_none()));
    }
}
