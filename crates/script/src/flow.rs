//! The element flow of the editor: the element a line becomes on Enter at its end, and
//! on Tab on an empty line. `assist.rs` offers the suggestions for the same line.

use crate::Element;

/// The kind of element a line of the editor is meant to be, as far as Enter and Tab care.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slot {
    Heading,
    Action,
    Character,
    Parenthetical,
    Dialogue,
    Transition,
}

/// Whether a blank line stands between an element and the next one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gap {
    Blank,
    None,
}

impl Slot {
    /// The slot of a parsed element; none for those that take no part in the flow.
    pub fn of(element: &Element) -> Option<Self> {
        match element {
            Element::SceneHeading { .. } => Some(Self::Heading),
            Element::Action(_) => Some(Self::Action),
            Element::Character { .. } => Some(Self::Character),
            Element::Parenthetical(_) => Some(Self::Parenthetical),
            Element::Dialogue(_) => Some(Self::Dialogue),
            Element::Transition(_) => Some(Self::Transition),
            _ => None,
        }
    }

    /// Enter at the end of the element: what comes next and how it is set off.
    ///
    /// ```text
    /// heading ─► action ─► action        character, parenthetical ─► dialogue
    /// dialogue ─► action                 transition ─► heading
    /// ```
    pub fn after_enter(self) -> (Self, Gap) {
        match self {
            Self::Heading | Self::Action | Self::Dialogue => (Self::Action, Gap::Blank),
            Self::Character | Self::Parenthetical => (Self::Dialogue, Gap::None),
            Self::Transition => (Self::Heading, Gap::Blank),
        }
    }

    /// Tab on an empty line: the element to turn it into; none for a heading or a transition.
    pub fn on_tab(self) -> Option<Self> {
        match self {
            Self::Action => Some(Self::Character),
            Self::Character => Some(Self::Action),
            Self::Parenthetical => Some(Self::Dialogue),
            Self::Dialogue => Some(Self::Parenthetical),
            Self::Heading | Self::Transition => None,
        }
    }
}

/// An empty parenthetical as Tab leaves it.
const EMPTY_PARENTHETICAL: &str = "()";
const PAREN_CLOSE: char = ')';

/// What Enter does on a line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnterStep {
    /// A plain line break.
    Break,
    /// The line has no text of its own: it changes its element, as Tab does.
    Retag,
    /// Text stands below: the cursor goes to the end of the next line.
    GoBelow,
    /// The element ends here: the next one, after the gap.
    Next(Slot, Gap),
}

/// The line Tab leaves: its element, text and cursor column.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TabStep {
    pub slot: Slot,
    pub text: &'static str,
    pub col: usize,
}

/// Whether the line has no text of its own: empty, or the `()` of an empty parenthetical.
fn is_empty_slot(text: &str, slot: Slot) -> bool {
    let text = text.trim();

    text.is_empty() || slot == Slot::Parenthetical && text == EMPTY_PARENTHETICAL
}

/// The end of the element: the end of the line, or before the `)` of a parenthetical.
fn at_element_end(text: &str, col: usize, slot: Slot) -> bool {
    let len = text.chars().count();
    let before_close = slot == Slot::Parenthetical && text.trim_end().ends_with(PAREN_CLOSE);

    col == len || before_close && col + 1 == len
}

/// Enter at column `col` of a line of element `slot` (none: not part of the flow);
/// `next_blank` tells whether the line below is blank or missing.
pub fn enter_step(text: &str, col: usize, slot: Option<Slot>, next_blank: bool) -> EnterStep {
    let Some(slot) = slot.filter(|_| !text.trim().is_empty()) else {
        return EnterStep::Break;
    };

    if is_empty_slot(text, slot) {
        return EnterStep::Retag;
    }
    if !at_element_end(text, col, slot) {
        return EnterStep::Break;
    }
    if next_blank {
        let (next, gap) = slot.after_enter();

        return EnterStep::Next(next, gap);
    }

    // Below a cue or parenthetical the dialogue already stands: go there.
    match slot {
        Slot::Character | Slot::Parenthetical => EnterStep::GoBelow,
        _ => EnterStep::Break,
    }
}

/// Tab on a line of element `slot` (none counts as action): what it turns into; none when
/// the line has text or its element does not change.
pub fn tab_step(text: &str, slot: Option<Slot>) -> Option<TabStep> {
    let slot = slot.unwrap_or(Slot::Action);
    if !is_empty_slot(text, slot) {
        return None;
    }

    Some(match slot.on_tab()? {
        Slot::Parenthetical => TabStep {
            slot: Slot::Parenthetical,
            text: EMPTY_PARENTHETICAL,
            col: 1,
        },
        to => TabStep {
            slot: to,
            text: "",
            col: 0,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enter_follows_the_element() {
        use Slot::*;

        for (text, col, slot, blank, want) in [
            ("", 0, Some(Action), true, EnterStep::Break),
            ("x", 0, None, true, EnterStep::Break),
            ("()", 1, Some(Parenthetical), true, EnterStep::Retag),
            ("Hi", 1, Some(Action), true, EnterStep::Break),
            (
                "Hi",
                2,
                Some(Action),
                true,
                EnterStep::Next(Action, Gap::Blank),
            ),
            (
                "ANNA",
                4,
                Some(Character),
                true,
                EnterStep::Next(Dialogue, Gap::None),
            ),
            (
                "(leise)",
                6,
                Some(Parenthetical),
                true,
                EnterStep::Next(Dialogue, Gap::None),
            ),
            ("(leise)", 3, Some(Parenthetical), true, EnterStep::Break),
            ("ANNA", 4, Some(Character), false, EnterStep::GoBelow),
            ("(leise)", 6, Some(Parenthetical), false, EnterStep::GoBelow),
            ("Hi", 2, Some(Action), false, EnterStep::Break),
            (
                "CUT TO:",
                7,
                Some(Transition),
                true,
                EnterStep::Next(Heading, Gap::Blank),
            ),
        ] {
            assert_eq!(enter_step(text, col, slot, blank), want, "{text:?} {col}");
        }
    }

    #[test]
    fn tab_turns_an_empty_line() {
        use Slot::*;

        let step = |slot, text, col| Some(TabStep { slot, text, col });

        assert_eq!(tab_step("", None), step(Character, "", 0));
        assert_eq!(tab_step("  ", Some(Character)), step(Action, "", 0));
        assert_eq!(tab_step("", Some(Dialogue)), step(Parenthetical, "()", 1));
        assert_eq!(tab_step("()", Some(Parenthetical)), step(Dialogue, "", 0));
        assert_eq!(tab_step("", Some(Heading)), None);
        assert_eq!(tab_step("x", Some(Action)), None);
        assert_eq!(tab_step("()", Some(Dialogue)), None);
    }
}
