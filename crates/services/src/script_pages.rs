//! The screenplay on pages (DB-30 to DB-32, DB-70): the layout of `script/` as data for the
//! surfaces, and as the input of export's screenplay PDF.
//!
//! ```text
//! text ─► parse_located ─► script::paginate ─► Layout ─┬─► ScriptPages (page breaks, lengths)
//!                                                      └─► export::ScriptSheet (the PDF)
//! ```

use klappe_export::{Face, Paper, ScriptLine, ScriptPage, ScriptSheet, TitlePage};
use klappe_model::{ScriptLanguage, ScriptPaper, ScriptSetup, ScriptWords};
use klappe_script::{
    length_text, paginate, parse_located, title_fields, ContdAfterAction, Element, Located,
    PageKind, PageOptions, PageSize, Pagination, Words, TEXT_COLUMNS, TITLE_KEYS,
};

pub use klappe_export::SceneNumbers;

use crate::catalog::Language;

/// The first line of a page after the first (DB-30).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageBreak {
    /// The page that starts here, from 2.
    pub page: usize,
    /// The text line (from 0) the page starts at; the break is above it.
    pub line: usize,
}

/// A scene as the last layout has it (DB-20, DB-32).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneSpan {
    /// The text line (from 0) of its heading.
    pub line: usize,
    pub page: usize,
    /// Length in eighths of a page, at least 1.
    pub eighths: usize,
    /// The length as a list shows it: `2 3/8`.
    pub length: String,
}

/// What the last layout of the screenplay found; the editor shows it until the next one.
#[derive(Debug, Clone, PartialEq)]
pub struct ScriptPages {
    pub breaks: Vec<PageBreak>,
    /// In script order.
    pub scenes: Vec<SceneSpan>,
    pub pages: usize,
    pub eighths: usize,
    /// One page of the paper, one minute (DB-32).
    pub minutes: f64,
    /// The length of the whole script as a list shows it.
    pub length: String,
}

/// The line above a screenplay's pages in the PDF.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeaderLine {
    Off,
    /// The title, and the draft date if the title page has one.
    Title,
}

/// What a screenplay PDF is asked to be (DB-70).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScriptPrint {
    pub scene_numbers: SceneNumbers,
    pub header: HeaderLine,
    /// Used for the words (MORE) and (CONT'D) when the project has no language (DN-05).
    pub language: Language,
}

/// A laid-out screenplay, kept for the next export and for the editor.
#[derive(Debug, Clone)]
pub(crate) struct Layout {
    pagination: Pagination,
    title_page: Vec<(String, String)>,
    pub(crate) view: ScriptPages,
}

/// International words, or those of the project's language; the surface's language stands in
/// while the project has none.
fn words(setup: &ScriptSetup, fallback: Language) -> Words {
    let german = match (setup.words, setup.language) {
        (ScriptWords::International, _) => false,
        (ScriptWords::Local, Some(language)) => language == ScriptLanguage::De,
        (ScriptWords::Local, None) => fallback == Language::German,
    };

    if german {
        Words::GERMAN
    } else {
        Words::ENGLISH
    }
}

fn size(setup: &ScriptSetup) -> PageSize {
    match setup.paper {
        ScriptPaper::A4 => PageSize::A4,
        ScriptPaper::Letter => PageSize::Letter,
    }
}

/// Pages `text`: parse, lay out, find where each page starts in the text.
pub(crate) fn lay_out(text: &str, setup: &ScriptSetup, fallback: Language) -> Layout {
    let located = parse_located(text);
    let elements: Vec<Element> = located.iter().map(|l| l.element.clone()).collect();
    let mut options = PageOptions::new(size(setup), words(setup, fallback));
    if setup.contd_after_action {
        options.contd_after_action = ContdAfterAction::On;
    }
    let pagination = paginate(&elements, &options);

    let view = ScriptPages {
        breaks: breaks(&pagination, &located),
        scenes: scenes(&pagination, &located),
        pages: pagination.pages.len(),
        eighths: pagination.eighths(),
        minutes: pagination.minutes(),
        length: length_text(pagination.eighths()),
    };

    Layout {
        pagination,
        title_page: title_fields(text),
        view,
    }
}

/// Bytes of the text before `at`, as the line of the element it falls on.
fn text_line(located: &[Located], element: usize, offset: usize) -> Option<usize> {
    let located = located.get(element)?;
    let text = match &located.element {
        Element::Action(t) | Element::Dialogue(t) => t.as_str(),
        _ => "",
    };
    let before = text.get(..offset).unwrap_or_default();
    let nth = before.matches('\n').count();

    located.lines.get(nth).or(located.lines.last()).copied()
}

fn breaks(pagination: &Pagination, located: &[Located]) -> Vec<PageBreak> {
    pagination
        .pages
        .iter()
        .skip(1)
        .filter_map(|page| {
            // A repeated name and (MORE) are not in the text; the break is above what is.
            let first = page.lines.iter().find(|l| !l.added)?;
            let line = text_line(located, first.element, first.offset)?;

            Some(PageBreak {
                page: page.number,
                line,
            })
        })
        .collect()
}

fn scenes(pagination: &Pagination, located: &[Located]) -> Vec<SceneSpan> {
    pagination
        .scenes
        .iter()
        .filter_map(|scene| {
            Some(SceneSpan {
                line: text_line(located, scene.element, 0)?,
                page: scene.page,
                eighths: scene.eighths,
                length: length_text(scene.eighths),
            })
        })
        .collect()
}

/// The value of the form field `name` (a `TITLE_KEYS` spelling).
fn title_key(title_page: &[(String, String)], name: &str) -> String {
    title_page
        .iter()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value.clone())
        .unwrap_or_default()
}

impl Layout {
    /// The sheet export prints: the rows of every page as columns, the page number from
    /// page 2, the title page from the title page keys of the text.
    pub(crate) fn sheet(
        &self,
        setup: &ScriptSetup,
        print: &ScriptPrint,
        project_title: &str,
    ) -> ScriptSheet {
        let ordinal = |element: usize| {
            self.pagination
                .scenes
                .iter()
                .position(|s| s.element == element)
                .map(|i| (i + 1).to_string())
        };

        let pages = self
            .pagination
            .pages
            .iter()
            .map(|page| ScriptPage {
                label: (page.number > 1).then(|| format!("{}.", page.number)),
                lines: page
                    .lines
                    .iter()
                    .map(|line| ScriptLine {
                        row: line.row,
                        col: line.x,
                        text: line.text.clone(),
                        face: match line.kind {
                            PageKind::Heading => Face::Bold,
                            _ => Face::Regular,
                        },
                        // A heading without a number gets its place in the script.
                        scene_number: match (line.kind, line.offset) {
                            (PageKind::Heading, 0) => {
                                line.number.clone().or_else(|| ordinal(line.element))
                            }
                            _ => None,
                        },
                    })
                    .collect(),
            })
            .collect();

        // The fields in the order of `TITLE_KEYS`, so a new spelling there reaches the PDF.
        let [title, credit, author, source, draft_date, contact] =
            TITLE_KEYS.map(|name| title_key(&self.title_page, name));
        let header = match print.header {
            HeaderLine::Off => None,
            HeaderLine::Title => Some(header_text(&title, project_title, &draft_date)),
        };

        ScriptSheet {
            paper: match setup.paper {
                ScriptPaper::A4 => Paper::A4,
                ScriptPaper::Letter => Paper::Letter,
            },
            columns: TEXT_COLUMNS,
            title_page: TitlePage {
                title,
                credit,
                author,
                source,
                draft_date,
                contact,
            },
            pages,
            scene_numbers: print.scene_numbers,
            header,
        }
    }
}

/// `Title - Draft date`; the project's title if the title page has none.
fn header_text(title: &str, project_title: &str, draft_date: &str) -> String {
    let title = if title.is_empty() {
        project_title
    } else {
        title
    };

    match draft_date.is_empty() {
        true => title.to_owned(),
        false => format!("{title} - {draft_date}"),
    }
}
