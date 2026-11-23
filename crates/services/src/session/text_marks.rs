//! Elements marked in a scene's text (D-23): marking, the text with its marks, the marks no
//! longer found and the elements whose name stands in the text.

use std::collections::HashSet;

use klappe_model::{Scene, TextMark};

use super::Session;
use crate::error::ServiceError;
use crate::field::Field;
use crate::project_edit::{BreakdownEdit, MarkTarget, ProjectEdit};
use crate::scene_text::{self, Shown};
use crate::views::{Suggestion, TextLine, TextRun};

/// A mark found in the text: its line, where its words start and end there, and the element.
struct Placed<'a> {
    line: usize,
    start: usize,
    end: usize,
    element: &'a str,
}

impl Session {
    /// The position of the scene `id` in `Project::scenes`.
    fn entry_of(&self, id: &str) -> Result<usize, ServiceError> {
        self.project
            .scenes
            .iter()
            .position(|s| s.id == id)
            .ok_or_else(|| ServiceError::NoScene(id.to_owned()))
    }

    /// The lines of the scene at `entry` in `Project::scenes`; none when it is not a scene of
    /// the book.
    pub(super) fn book_lines(&self, entry: usize) -> Vec<Shown> {
        let Some(book) = self.book_scene_of(entry) else {
            return Vec::new();
        };

        self.script
            .scenes()
            .get(book)
            .map(|s| scene_text::shown(s.elements))
            .unwrap_or_default()
    }

    /// Marks the words `text` that stand in line `line` of the scene's text right after
    /// `prefix` (D-23); undoable. Refused when they do not stand there.
    pub fn mark_text(
        &mut self,
        scene: &str,
        (line, prefix, text): (usize, &str, &str),
        target: MarkTarget,
    ) -> Result<(), ServiceError> {
        let entry = self.entry_of(scene)?;
        let lines = self.book_lines(entry);
        let occurrence = scene_text::occurrence(&lines, line, prefix, text).ok_or_else(|| {
            ServiceError::BadValue {
                field: Field::Mark,
                value: text.to_owned(),
            }
        })?;

        let mark = TextMark {
            element: String::new(),
            text: text.to_owned(),
            occurrence,
        };
        self.edit_project(ProjectEdit::Breakdown(BreakdownEdit::MarkText {
            scene: scene.to_owned(),
            mark,
            target,
        }))
    }

    /// Drops the element's marks in the scene that are no longer found in its text; undoable.
    pub fn forget_lost(&mut self, scene: &str, element: &str) -> Result<(), ServiceError> {
        let entry = self.entry_of(scene)?;
        let lines = self.book_lines(entry);
        let marks: Vec<TextMark> = self.project.scenes[entry]
            .sheet
            .marks
            .iter()
            .filter(|m| m.element == element && scene_text::locate(&lines, m).is_none())
            .cloned()
            .collect();

        self.edit_project(ProjectEdit::Breakdown(BreakdownEdit::ForgetMarks {
            scene: scene.to_owned(),
            marks,
        }))
    }

    /// The elements of the scene marked in its text of which no mark is found there any more.
    pub(super) fn lost_marks(scene: &Scene, lines: &[Shown]) -> HashSet<String> {
        let found: HashSet<&str> = scene
            .sheet
            .marks
            .iter()
            .filter(|m| scene_text::locate(lines, m).is_some())
            .map(|m| m.element.as_str())
            .collect();

        scene
            .sheet
            .marks
            .iter()
            .filter(|m| !found.contains(m.element.as_str()))
            .map(|m| m.element.clone())
            .collect()
    }

    /// The scene's lines with their marks; a mark that overlaps one before it in its line is
    /// not shown.
    pub(super) fn marked_text(&self, scene: &Scene, lines: &[Shown]) -> Vec<TextLine> {
        let mut placed: Vec<Placed> = scene
            .sheet
            .marks
            .iter()
            .filter_map(|m| {
                let (line, start) = scene_text::locate(lines, m)?;
                Some(Placed {
                    line,
                    start,
                    end: start + m.text.len(),
                    element: &m.element,
                })
            })
            .collect();
        placed.sort_by_key(|p| (p.line, p.start));

        lines
            .iter()
            .enumerate()
            .map(|(i, l)| TextLine {
                kind: l.kind,
                runs: self.runs(&l.text, placed.iter().filter(|p| p.line == i)),
            })
            .collect()
    }

    /// A line cut at its marks, in order of their start.
    fn runs<'a>(&self, text: &str, marks: impl Iterator<Item = &'a Placed<'a>>) -> Vec<TextRun> {
        let mut runs = Vec::new();
        let mut at = 0;

        for mark in marks {
            if mark.start < at {
                continue;
            }
            if mark.start > at {
                runs.push(plain(&text[at..mark.start]));
            }
            let element = self.project.elements.iter().find(|e| e.id == mark.element);
            runs.push(TextRun {
                text: text[mark.start..mark.end].to_owned(),
                element: element.map(|e| e.id.clone()),
                category: element.map(|e| e.category.name()),
            });
            at = mark.end;
        }
        if at < text.len() {
            runs.push(plain(&text[at..]));
        }

        runs
    }

    /// The elements whose name stands in the scene's text that the sheet neither lists nor
    /// turned down and that `offered` does not hold yet.
    pub(super) fn named_in_text(
        &self,
        scene: &Scene,
        lines: &[Shown],
        offered: &[Suggestion],
    ) -> Vec<Suggestion> {
        let sheet = &scene.sheet;

        self.project
            .elements
            .iter()
            .filter(|e| !sheet.elements.contains(&e.id) && !sheet.excluded.contains(&e.id))
            .filter(|e| !offered.iter().any(|s| s.element == e.id))
            .filter(|e| scene_text::names(lines, &e.name))
            .map(|e| Suggestion {
                element: e.id.clone(),
                name: e.name.clone(),
                category: e.category.name(),
                object: None,
            })
            .collect()
    }
}

fn plain(text: &str) -> TextRun {
    TextRun {
        text: text.to_owned(),
        element: None,
        category: None,
    }
}
