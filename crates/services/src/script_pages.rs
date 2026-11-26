//! The screenplay on pages (DB-30 to DB-32, DB-70): the layout of `script/` as data for the
//! surfaces, and as the input of export's screenplay PDF.
//!
//! ```text
//! text ─► parse_located ─► script::paginate ─► Layout ─┬─► ScriptPages (page breaks, lengths)
//!                                                      └─► export::ScriptSheet (the PDF)
//! ```

use klappe_export::{Face, Paper, Revised, ScriptLine, ScriptPage, ScriptSheet, TitlePage};
use klappe_model::{PageStart, ScriptLanguage, ScriptPaper, ScriptSetup, ScriptWords};
use klappe_script::{
    carry, length_text, lines_text, page_starts, paginate_locked, parse_located, title_fields,
    ContdAfterAction, Element, Located, PageKind, PageOptions, PageSize, Pagination, Words,
    TEXT_COLUMNS, TITLE_KEYS,
};

pub use klappe_export::SceneNumbers;

use crate::catalog::Language;
use crate::session::{Revision, Revisions};

/// The first line of a page after the first (DB-30).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageBreak {
    /// The page that starts here, from 2 (its place among the pages, struck ones counted).
    pub page: usize,
    /// What the page is called: `3`, or `3A` once the pages are locked (DB-34).
    pub label: String,
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
    /// Mark what changed since the last locked version (DB-41).
    pub revisions: Revisions,
}

/// A laid-out screenplay, kept for the next export and for the editor.
#[derive(Debug, Clone)]
pub(crate) struct Layout {
    pagination: Pagination,
    /// The elements with their text lines, to say which line of the text a printed row is from.
    located: Vec<Located>,
    title_page: Vec<(String, String)>,
    pub(crate) view: ScriptPages,
    /// What it was laid out from: the text (as a hash), the setup, the fallback language and
    /// the locked version.
    made_for: Source,
}

/// The pages of the last locked version (DB-34): its text and where each page started.
#[derive(Debug, Clone)]
pub(crate) struct PageLock {
    pub(crate) id: String,
    pub(crate) text: String,
    pub(crate) starts: Vec<PageStart>,
}

/// The input of a layout, to see whether it is still the one for the text.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Source {
    text: u64,
    setup: ScriptSetup,
    fallback: Language,
    lock: Option<String>,
}

impl Source {
    pub(crate) fn of(
        text: &str,
        setup: &ScriptSetup,
        fallback: Language,
        lock: Option<&str>,
    ) -> Self {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        std::hash::Hash::hash(text, &mut hasher);

        Self {
            text: std::hash::Hasher::finish(&hasher),
            setup: *setup,
            fallback,
            lock: lock.map(str::to_owned),
        }
    }
}

/// International words, or those of the project's language; the surface's language stands in
/// while the project has none.
fn words(setup: &ScriptSetup, fallback: Language) -> Words {
    let language = setup.language.unwrap_or_else(|| fallback.into());
    let local = setup.words == ScriptWords::Local;

    if local && language == ScriptLanguage::De {
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

/// Pages `text`: parse, lay out (on the pages of `lock`, if the script is locked), find where
/// each page starts in the text.
pub(crate) fn lay_out(
    text: &str,
    setup: &ScriptSetup,
    fallback: Language,
    lock: Option<&PageLock>,
) -> Layout {
    let located = parse_located(text);
    let elements: Vec<Element> = located.iter().map(|l| l.element.clone()).collect();
    let mut options = PageOptions::new(size(setup), words(setup, fallback));
    if setup.contd_after_action {
        options.contd_after_action = ContdAfterAction::On;
    }
    let starts = lock.map(|l| carry(&l.text, text, &l.starts));
    let pagination = paginate_locked(&elements, &located, &options, &starts.unwrap_or_default());

    let view = ScriptPages {
        breaks: breaks(&pagination, &located),
        scenes: scenes(&pagination, &located),
        pages: pagination.pages.len(),
        eighths: pagination.eighths(),
        minutes: pagination.minutes(),
        length: length_text(pagination.eighths()),
    };

    Layout {
        made_for: Source::of(text, setup, fallback, lock.map(|l| l.id.as_str())),
        pagination,
        located,
        title_page: title_fields(text),
        view,
    }
}

/// Bytes of the text before `at`, as the line of the element it falls on.
fn text_line(located: &[Located], element: usize, offset: usize) -> Option<usize> {
    let located = located.get(element)?;
    let text = lines_text(&located.element);
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
                label: page.label.clone(),
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
    /// Where each page starts in the text it was laid out from, to lock them (DB-34).
    pub(crate) fn page_starts(&self, text: &str) -> Vec<PageStart> {
        let lines = text
            .replace("\r\n", "\n")
            .replace('\r', "\n")
            .matches('\n')
            .count()
            + 1;

        page_starts(&self.pagination, &self.located, lines)
    }

    /// Whether it was laid out from `source`.
    pub(crate) fn is_for(&self, source: &Source) -> bool {
        self.made_for == *source
    }

    /// The sheet export prints: the rows of every page as columns, the page number from
    /// page 2, the title page from the title page keys of the text; with a `revision`, the
    /// marks of its lines and its header.
    pub(crate) fn sheet(
        &self,
        setup: &ScriptSetup,
        print: &ScriptPrint,
        project_title: &str,
        revision: Option<&Revision>,
    ) -> ScriptSheet {
        let only_changed = print.revisions == Revisions::ChangedPages && revision.is_some();
        let pages = self.script_pages(revision, only_changed);

        // The fields in the order of `TITLE_KEYS`, so a new spelling there reaches the PDF.
        let [title, credit, author, source, draft_date, contact] =
            TITLE_KEYS.map(|name| title_key(&self.title_page, name));
        let header = match print.header {
            HeaderLine::Off => None,
            HeaderLine::Title => Some(header_text(&title, project_title, &draft_date)),
        };

        // The pages of a revision are loose sheets for the ones who have the script already.
        let title_page = match only_changed {
            true => TitlePage::default(),
            false => TitlePage {
                title,
                credit,
                author,
                source,
                draft_date,
                contact,
            },
        };

        ScriptSheet {
            paper: match setup.paper {
                ScriptPaper::A4 => Paper::A4,
                ScriptPaper::Letter => Paper::Letter,
            },
            columns: TEXT_COLUMNS,
            title_page,
            pages,
            scene_numbers: print.scene_numbers,
            header,
            revision: revision.map(|r| r.header.clone()),
        }
    }

    /// The pages as export prints them; with `only_changed` those with a marked row.
    fn script_pages(&self, revision: Option<&Revision>, only_changed: bool) -> Vec<ScriptPage> {
        let ordinal = |element: usize| {
            self.pagination
                .scenes
                .iter()
                .position(|s| s.element == element)
                .map(|i| (i + 1).to_string())
        };

        self.pagination
            .pages
            .iter()
            .filter(|page| !only_changed || self.page_changed(page, revision))
            .map(|page| ScriptPage {
                label: (page.number > 1).then(|| format!("{}.", page.label)),
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
                        revised: self.revised(line, revision),
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
            .collect()
    }

    /// Whether whoever holds the last locked version needs the page (DB-43): a line is marked,
    /// the page is new (12A), a sheet of struck pages, or a page that lost text.
    fn page_changed(&self, page: &klappe_script::Page, revision: Option<&Revision>) -> bool {
        let Some(revision) = revision else {
            return false;
        };
        let marked = || {
            let revised = |line| self.revised(line, Some(revision)) == Revised::Yes;

            page.lines.iter().any(revised)
        };

        !page.omitted.is_empty()
            || !revision.old_pages.contains(&page.label)
            || revision.shortened.contains(&page.label)
            || marked()
    }

    /// Whether the text line a printed row is from is new or changed. A repeated name and
    /// (MORE) are not in the text.
    fn revised(&self, line: &klappe_script::Line, revision: Option<&Revision>) -> Revised {
        let Some(revision) = revision.filter(|_| !line.added) else {
            return Revised::No;
        };
        let at = text_line(&self.located, line.element, line.offset);

        match at.is_some_and(|l| revision.marked.contains(&l)) {
            true => Revised::Yes,
            false => Revised::No,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn words_follow_the_project_language_else_the_surface() {
        let setup = |language, words| ScriptSetup {
            language,
            words,
            ..ScriptSetup::default()
        };
        let local = ScriptWords::Local;

        for (language, fallback, german) in [
            (Some(ScriptLanguage::De), Language::English, true),
            (Some(ScriptLanguage::En), Language::German, false),
            (None, Language::German, true),
            (None, Language::English, false),
        ] {
            let want = if german {
                Words::GERMAN
            } else {
                Words::ENGLISH
            };
            assert_eq!(words(&setup(language, local), fallback), want);
        }
        let international = setup(Some(ScriptLanguage::De), ScriptWords::International);
        assert_eq!(words(&international, Language::German), Words::ENGLISH);
    }

    #[test]
    fn a_line_of_a_lyrics_block_is_found_like_one_of_an_action() {
        let lyrics = Located {
            element: Element::Lyrics("a\nb\nc".into()),
            lines: vec![4, 5, 6],
        };

        assert_eq!(text_line(&[lyrics], 0, 2), Some(5));
    }
}
