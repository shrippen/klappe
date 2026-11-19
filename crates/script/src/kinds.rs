//! What each line of the text is, for an editor that sets the elements as the page does
//! (D-15): a cue indented like on paper, a transition at the right.
//!
//! ```text
//! INT. SAAL - TAG       Heading
//!                       Blank
//!         ANNA          Character   (columns of `PageKind::margin`)
//!     Ihr kommt spät.   Dialogue
//! ```

use crate::{parse_located, title_page_lines, Element, PageKind};

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

/// The kind of every line of `text` (split at `\n`). The reader says which lines each
/// element stands on; a line of no element (a boneyard) is action, or blank when empty.
pub fn text_kinds(text: &str) -> Vec<TextKind> {
    let mut kinds: Vec<TextKind> = text
        .split('\n')
        .map(|line| match line.trim().is_empty() {
            true => TextKind::Blank,
            false => TextKind::Action,
        })
        .collect();

    for located in parse_located(text) {
        let kind = TextKind::of(&located.element);
        for &at in &located.lines {
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
}
