//! The element flow of the editor: the element a line becomes on Enter at its end, and
//! on Tab on an empty line. `assist.rs` offers the suggestions for the same line.

use crate::parse::{
    parse_located, title_page_lines, FORCE_ACTION, FORCE_CHARACTER, FORCE_HEADING,
    FORCE_TRANSITION, NUMBER_MARK,
};
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

const SHARP_S: char = 'ß';
const CENTER_END: char = '<';
/// A heading that starts so is an ellipsis; `.` before it would not force it.
const ELLIPSIS: &str = "..";

/// Uppercase as the parser counts it: `ß` stays, and a letter whose capital is longer
/// than one character stays, so the cursor column holds.
pub fn upper(text: &str) -> String {
    let one = |c: char| {
        let mut up = c.to_uppercase();
        match (up.next(), up.next()) {
            (Some(u), None) if c != SHARP_S => u,
            _ => c,
        }
    };

    text.chars().map(one).collect()
}

/// `text` in capitals, a trailing scene number `#12a#` as typed.
fn capitals(text: &str) -> String {
    let body = text.strip_suffix(NUMBER_MARK).unwrap_or(text);
    let cut = match body.rfind(NUMBER_MARK) {
        Some(at) if text.ends_with(NUMBER_MARK) && !body[at..].contains(char::is_whitespace) => at,
        _ => text.len(),
    };

    upper(&text[..cut]) + &text[cut..]
}

/// `text` without a leading forcing marker (and the `<` of a centered `>text<`).
fn unforced(text: &str) -> &str {
    let text = text.trim();
    let marked = [
        FORCE_HEADING,
        FORCE_ACTION,
        FORCE_CHARACTER,
        FORCE_TRANSITION,
    ];
    let Some(first) = text.chars().next().filter(|c| marked.contains(c)) else {
        return text;
    };
    let rest = &text[first.len_utf8()..];
    if first == FORCE_HEADING && rest.starts_with(FORCE_HEADING) {
        return text;
    }

    match first == FORCE_TRANSITION {
        true => rest.strip_suffix(CENTER_END).unwrap_or(rest).trim(),
        false => rest.trim(),
    }
}

/// The text after [`force_line`] and the line the forced element stands on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Forced {
    pub text: String,
    pub line: usize,
}

/// Whether line `at` of `lines` is, alone, an element of `slot`.
fn stands_as(lines: &[String], at: usize, slot: Slot) -> bool {
    parse_located(&lines.join("\n"))
        .iter()
        .any(|l| l.lines == [at] && Slot::of(&l.element) == Some(slot))
}

fn is_blank(line: Option<&String>) -> bool {
    line.is_none_or(|l| l.trim().is_empty())
}

/// Makes line `line` of `text` a scene heading or transition (`slot`; none for other
/// slots): capitals, the number of a heading as typed, a blank line before and after where
/// missing (so the line stands alone), and the forcing marker (`.`, `>`) only where
/// Fountain would not read the line as that element in this place. None for a line of the
/// title page, and for a heading that starts with `..` (an ellipsis cannot be forced).
///
/// ```text
/// "kueche tag"  ─► ".KUECHE TAG"      "int. küche"  ─► "INT. KÜCHE"
/// "fade out."   ─► ">FADE OUT."       "cut to:"     ─► "CUT TO:"
/// "Anna geht.⏎int. a" ─► "Anna geht.⏎⏎INT. A"
/// ```
pub fn force_line(text: &str, line: usize, slot: Slot) -> Option<Forced> {
    let marker = match slot {
        Slot::Heading => FORCE_HEADING,
        Slot::Transition => FORCE_TRANSITION,
        _ => return None,
    };
    let mut lines: Vec<String> = text.split('\n').map(str::to_owned).collect();
    if line >= lines.len() || line < title_page_lines(text) {
        return None;
    }

    let plain = match slot {
        Slot::Heading => capitals(unforced(&lines[line])),
        _ => upper(unforced(&lines[line])),
    };
    if slot == Slot::Heading && plain.starts_with(ELLIPSIS) {
        return None;
    }

    // An empty line only becomes the pending element: nothing to set apart.
    let mut at = line;
    lines[at] = plain.clone();
    if !plain.is_empty() {
        // Alone between blank lines; an element needs that to be read as one.
        if at > 0 && !is_blank(lines.get(at - 1)) {
            lines.insert(at, String::new());
            at += 1;
        }
        if !is_blank(lines.get(at + 1)) {
            lines.insert(at + 1, String::new());
        }
        if !stands_as(&lines, at, slot) {
            lines[at] = format!("{marker}{plain}");
        }
    }

    Some(Forced {
        text: lines.join("\n"),
        line: at,
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

    #[test]
    fn force_line_marks_only_where_needed() {
        for (text, slot, want) in [
            ("int. küche - tag", Slot::Heading, "INT. KÜCHE - TAG"),
            ("küche", Slot::Heading, ".KÜCHE"),
            (".küche", Slot::Heading, ".KÜCHE"),
            ("  >Fade out. ", Slot::Heading, ".FADE OUT."),
            ("Gasse #3#", Slot::Heading, ".GASSE #3#"),
            ("int. a #12a#", Slot::Heading, "INT. A #12a#"),
            ("cut to:", Slot::Transition, "CUT TO:"),
            ("fade out.", Slot::Transition, ">FADE OUT."),
            (".INT. A", Slot::Transition, ">INT. A"),
            (">FADE OUT.<", Slot::Transition, ">FADE OUT."),
            ("...und dann", Slot::Transition, ">...UND DANN"),
            ("straße", Slot::Heading, ".STRAßE"),
            ("", Slot::Heading, ""),
        ] {
            let forced = force_line(&format!("x\n\n{text}\n\ny"), 2, slot);

            assert_eq!(
                forced.map(|f| f.text),
                Some(format!("x\n\n{want}\n\ny")),
                "{text:?}"
            );
        }
        assert_eq!(force_line("x", 0, Slot::Action), None);
        assert_eq!(force_line("x\n\n...und dann", 2, Slot::Heading), None);
    }

    #[test]
    fn force_line_sets_the_line_apart_and_spares_the_title_page() {
        let forced = |text, line, slot| force_line(text, line, slot).map(|f| (f.text, f.line));
        let own = |s: &str, line| Some((s.to_owned(), line));

        assert_eq!(
            forced("x\n\nAnna geht.\nint. a\n\ny", 3, Slot::Heading),
            own("x\n\nAnna geht.\n\nINT. A\n\ny", 4)
        );
        assert_eq!(
            forced("x\n\nANNA\nhallo du.\n\ny", 3, Slot::Transition),
            own("x\n\nANNA\n\n>HALLO DU.\n\ny", 4)
        );
        assert_eq!(forced("Title: T\n\nx", 0, Slot::Heading), None);
        assert_eq!(
            forced("Title: T\nAuthor: A\n\nx", 1, Slot::Transition),
            None
        );
    }
}
