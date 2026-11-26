//! Lays a screenplay out on pages: Courier 12 pt, industry margins, the break rules of
//! DB-31 and the length of each scene in eighths of a page (DB-30, DB-32).
//!
//! ```text
//! elements ─► blocks (wrapped rows, split points) ─► flow (page by page) ─► Pagination
//! ```
//!
//! One column is 1/10 inch, one row 1/6 inch. Positions are in those units, so the printer
//! and the editor translate them to points or cells.

mod block;
mod flow;
mod locked;
mod wrap;

#[cfg(test)]
mod flow_tests;
#[cfg(test)]
mod locked_tests;
#[cfg(test)]
mod tests;

use crate::Element;
use klappe_model::PageStart;

pub use locked::{carry, page_label_after, page_starts};

/// Columns of the text area: 6 inches.
pub const TEXT_COLUMNS: usize = 60;
/// Lines of body text on a US Letter page.
const LETTER_LINES: usize = 55;
/// Lines of body text on an A4 page.
const A4_LINES: usize = 59;
/// Eighths of a page.
const EIGHTHS: usize = 8;

/// The paper a screenplay is paged for (F-01).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PageSize {
    #[default]
    A4,
    Letter,
}

impl PageSize {
    /// Lines of body text on a page of this paper.
    pub fn body_lines(self) -> usize {
        match self {
            PageSize::A4 => A4_LINES,
            PageSize::Letter => LETTER_LINES,
        }
    }
}

/// The words on a page of a screenplay that are not the author's (F-04, DB-34).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Words {
    /// At the foot of the page: `(MORE)`.
    pub more: &'static str,
    /// After the name on the next page: `NAME (CONT'D)`.
    pub contd: &'static str,
    /// On the sheet that stands for struck pages: `12-13 ENTFÄLLT`.
    pub omitted: &'static str,
}

impl Words {
    pub const ENGLISH: Words = Words {
        more: "MORE",
        contd: "CONT'D",
        omitted: "OMITTED",
    };
    pub const GERMAN: Words = Words {
        more: "WEITER",
        contd: "FORTS.",
        omitted: "ENTFÄLLT",
    };
}

/// Whether a character speaking again after action gets `(CONT'D)` (DB-31).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ContdAfterAction {
    #[default]
    Off,
    On,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageOptions {
    pub size: PageSize,
    pub words: Words,
    pub contd_after_action: ContdAfterAction,
    /// Lines of body text; from `size`, a test sets small pages.
    pub(crate) body_lines: usize,
}

impl PageOptions {
    pub fn new(size: PageSize, words: Words) -> Self {
        Self {
            size,
            words,
            contd_after_action: ContdAfterAction::Off,
            body_lines: size.body_lines(),
        }
    }
}

/// What a line of a page is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageKind {
    Heading,
    Action,
    Character,
    Parenthetical,
    Dialogue,
    /// `(MORE)` at the foot of a broken dialogue.
    More,
    Transition,
    Centered,
    Lyrics,
}

/// How a line is set within its columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    Left,
    Right,
    Center,
}

/// The columns a kind of line takes on the page: from `x`, `width` wide, in columns of the
/// text area (`TEXT_COLUMNS`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Margin {
    pub x: usize,
    pub width: usize,
    pub align: Align,
}

/// A piece of text on a row. Two dual dialogues share a row: two lines with the same `row`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    /// From 0 below the top of the body text.
    pub row: usize,
    /// Columns from the left edge of the text area.
    pub x: usize,
    pub kind: PageKind,
    pub text: String,
    /// The index of the element in the script this comes from.
    pub element: usize,
    /// Byte offset in the element's text where the line starts; 0 for added text.
    pub offset: usize,
    /// On the first line of a scene heading: its number, if the script has one.
    pub number: Option<String>,
    /// Text that is not in the script: `(MORE)` and the name repeated on the next page.
    pub added: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page {
    /// From 1: the place of the page among the pages of the layout.
    pub number: usize,
    /// What the page is called: `3`, and `3A` for a page added to a locked script (DB-34). The
    /// same as `number` while no pages are locked. Pages after the first show it (top right).
    pub label: String,
    pub lines: Vec<Line>,
    /// The labels of the struck pages this sheet stands for; empty for a page of text.
    pub omitted: Vec<String>,
}

/// Where a scene starts and how long it is (DB-32).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneLength {
    /// The index of its heading in the script.
    pub element: usize,
    /// Page number from 1 and row of the heading.
    pub page: usize,
    pub row: usize,
    /// Rows the scene takes with its spacing, over all pages.
    pub lines: usize,
    /// Rounded up to the next eighth of a page, at least 1.
    pub eighths: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pagination {
    pub size: PageSize,
    pub pages: Vec<Page>,
    /// In script order; text before the first heading belongs to no scene.
    pub scenes: Vec<SceneLength>,
    /// Rows of all text with spacing.
    pub lines: usize,
    body_lines: usize,
}

impl Pagination {
    /// Eighths of a page for `lines` rows, rounded up.
    fn eighths_of(&self, lines: usize) -> usize {
        (lines * EIGHTHS).div_ceil(self.body_lines)
    }

    /// The whole script in eighths of a page.
    pub fn eighths(&self) -> usize {
        self.eighths_of(self.lines)
    }

    /// One page of the paper, one minute (DB-32); a page of A4 is as long as one of Letter.
    pub fn minutes(&self) -> f64 {
        self.eighths() as f64 / EIGHTHS as f64
    }
}

/// `3 2/8`: pages and eighths the way a scene list shows a length, `2/8` for under a page.
pub fn length_text(eighths: usize) -> String {
    let (pages, rest) = (eighths / EIGHTHS, eighths % EIGHTHS);

    match (pages, rest) {
        (0, r) => format!("{r}/{EIGHTHS}"),
        (p, 0) => p.to_string(),
        (p, r) => format!("{p} {r}/{EIGHTHS}"),
    }
}

/// Pages the elements of a screenplay.
pub fn paginate(elements: &[Element], options: &PageOptions) -> Pagination {
    flow::run(elements, options, &[])
}

/// Pages the elements the way a locked version had them (DB-34): a page starts at each of
/// `starts` (from `carry`, ascending, the first at the top), the text between two starts is
/// paged for itself, its first page carries the locked label and further ones the labels
/// after it (`12`, `12A`, `12B`). Starts that no text follows are struck pages.
pub fn paginate_locked(
    elements: &[Element],
    located: &[crate::Located],
    options: &PageOptions,
    starts: &[PageStart],
) -> Pagination {
    let spots = locked::spots(located, starts);

    flow::run(elements, options, &spots)
}
