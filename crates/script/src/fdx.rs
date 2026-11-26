//! Final Draft (FDX): the screenplay as XML, read into a [`Script`] and written from one.
//!
//! Written from the public description of the format (paragraph types, text runs with a
//! style, `DualDialogue`, `TitlePage`, the scene number as attribute) and from own files.

use crate::xml::Node;

pub(crate) const ROOT: &str = "FinalDraft";
pub(crate) const CONTENT: &str = "Content";
pub(crate) const PARAGRAPH: &str = "Paragraph";
pub(crate) const DUAL: &str = "DualDialogue";
pub(crate) const TITLE_PAGE: &str = "TitlePage";
pub(crate) const TEXT: &str = "Text";

pub(crate) const TYPE: &str = "Type";
pub(crate) const STYLE: &str = "Style";
pub(crate) const NUMBER: &str = "Number";
pub(crate) const ALIGNMENT: &str = "Alignment";
pub(crate) const NEW_PAGE: &str = "StartsNewPage";
/// Names the title page key of a paragraph; FDX has paragraphs only, no keys.
pub(crate) const KEY: &str = "KlappeKey";
pub(crate) const YES: &str = "Yes";

pub(crate) const HEADING: &str = "Scene Heading";
pub(crate) const ACTION: &str = "Action";
pub(crate) const CHARACTER: &str = "Character";
pub(crate) const PARENTHETICAL: &str = "Parenthetical";
pub(crate) const DIALOGUE: &str = "Dialogue";
pub(crate) const TRANSITION: &str = "Transition";

pub(crate) const CENTER: &str = "Center";
pub(crate) const RIGHT: &str = "Right";
pub(crate) const LEFT: &str = "Left";

const BOLD: &str = "Bold";
const ITALIC: &str = "Italic";
const UNDERLINE: &str = "Underline";
const STYLE_JOIN: char = '+';

/// What the FDX file could not take from a screenplay, counted; nothing here is in the file
/// except the lyrics, which stand there as action.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FdxLoss {
    pub sections: usize,
    pub synopses: usize,
    /// Notes on a line of their own and `[[notes]]` inside a line.
    pub notes: usize,
    /// Lyrics, written as action.
    pub lyrics: usize,
    /// `/* … */` comments; the parser cuts them, so the caller counts them in the text
    /// (`boneyard_count`) and sets this.
    pub boneyards: usize,
    /// Page breaks with no paragraph after them to start a page.
    pub page_breaks: usize,
}

impl FdxLoss {
    pub fn is_empty(&self) -> bool {
        *self == FdxLoss::default()
    }
}

/// The emphasis of a text run.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Style {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

impl Style {
    /// `Bold+Italic+Underline` as FDX spells it; other words (`AllCaps`) are no emphasis.
    pub fn parse(attr: &str) -> Style {
        let has = |word: &str| attr.split(STYLE_JOIN).any(|w| w == word);

        Style {
            bold: has(BOLD),
            italic: has(ITALIC),
            underline: has(UNDERLINE),
        }
    }

    /// The attribute value, or `None` for plain text.
    pub fn attr(self) -> Option<String> {
        let words: Vec<&str> = [
            (self.bold, BOLD),
            (self.italic, ITALIC),
            (self.underline, UNDERLINE),
        ]
        .iter()
        .filter(|(on, _)| *on)
        .map(|(_, word)| *word)
        .collect();

        if words.is_empty() {
            return None;
        }

        Some(words.join(&STYLE_JOIN.to_string()))
    }
}

/// The paragraphs below a `Content` element.
pub(crate) fn paragraphs(content: &Node) -> impl Iterator<Item = &Node> {
    content.nodes().filter(|n| n.name == PARAGRAPH)
}
