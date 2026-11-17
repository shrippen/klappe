//! Search and replace (`Ctrl+F`), in all text or only in the dialogue of one character.

use std::ops::Range;

use klappe_services::{find_all, parse_located, replace_hits, Case, Hit, Query};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::{App, Flow, Mode, Notice};
use crate::buffer::Pos;

/// Which way a choice steps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Way {
    Next,
    Previous,
}

/// Which box of the form takes the typing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum FindField {
    #[default]
    Find,
    Replace,
    Speaker,
}

impl FindField {
    fn next(self) -> Self {
        match self {
            Self::Find => Self::Replace,
            Self::Replace => Self::Speaker,
            Self::Speaker => Self::Find,
        }
    }

    fn previous(self) -> Self {
        match self {
            Self::Find => Self::Speaker,
            Self::Replace => Self::Find,
            Self::Speaker => Self::Replace,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Find {
    pub find: String,
    pub replace: String,
    /// Only the dialogue of this character; empty searches everything.
    pub speaker: String,
    pub field: FindField,
    pub case: Case,
    pub hits: Vec<Hit>,
    /// The match the cursor is on.
    pub sel: usize,
}

impl Default for Find {
    fn default() -> Self {
        Self {
            find: String::new(),
            replace: String::new(),
            speaker: String::new(),
            field: FindField::Find,
            case: Case::Ignore,
            hits: Vec::new(),
            sel: 0,
        }
    }
}

impl App {
    /// The columns of the matches on a line to mark while searching (a match over a line
    /// break marks the end of one line and the start of the next), and whether each is the
    /// current one.
    pub(crate) fn find_hits_on(&self, line: usize) -> Vec<(Range<usize>, bool)> {
        if self.mode != Mode::Find {
            return Vec::new();
        }

        self.find
            .hits
            .iter()
            .enumerate()
            .filter(|(_, h)| h.line <= line && line <= h.end_line)
            .map(|(i, h)| {
                let start = if h.line == line { h.start } else { 0 };
                let end = if h.end_line == line {
                    h.end
                } else {
                    usize::MAX
                };
                (start..end, i == self.find.sel)
            })
            .collect()
    }

    pub(super) fn open_find(&mut self) -> Flow {
        self.mode = Mode::Find;
        self.focus = super::Focus::Editor;
        self.suggest = None;
        self.find_update();

        Flow::Continue
    }

    /// Looks for the matches in the text as it is now.
    fn find_hits(&mut self) {
        let text = self.editor.text();
        let speaker = self.find.speaker.trim();
        let query = Query {
            find: &self.find.find,
            case: self.find.case,
            speaker: (!speaker.is_empty()).then_some(speaker),
        };
        let located = parse_located(&text);
        self.find.hits = find_all(self.editor.lines(), &located, &query);
    }

    /// Looks for the matches again, from the cursor on.
    fn find_update(&mut self) {
        self.find_hits();

        let cursor = self.editor.cursor();
        let from = |h: &Hit| (h.line, h.start) >= (cursor.line, cursor.col);
        self.find.sel = self.find.hits.iter().position(from).unwrap_or(0);
    }

    /// Puts the cursor on match `sel`.
    fn find_show(&mut self) {
        let Some(hit) = self.find.hits.get(self.find.sel).copied() else {
            return;
        };
        self.editor.go_to(Pos {
            line: hit.line,
            col: hit.start,
        });
    }

    /// Moves to the next or previous match, wrapping round; also `F3` and `Shift+F3`.
    pub(super) fn find_step(&mut self, back: bool) -> Flow {
        // The text may have changed since the box was closed.
        self.find_hits();
        let count = self.find.hits.len();
        if count == 0 {
            return Flow::Continue;
        }

        // From a cursor off the matches, "next" is the first one after it.
        let at = self.find.hits.iter().position(|h| {
            let c = self.editor.cursor();
            (h.line, h.start) == (c.line, c.col)
        });
        self.find.sel = match (at, back) {
            (Some(at), false) => (at + 1) % count,
            (Some(at), true) => (at + count - 1) % count,
            (None, false) => self.after_cursor(),
            (None, true) => (self.after_cursor() + count - 1) % count,
        };
        self.find_show();

        Flow::Continue
    }

    /// Index of the first match behind the cursor, wrapping to the first.
    fn after_cursor(&self) -> usize {
        let cursor = self.editor.cursor();

        self.find
            .hits
            .iter()
            .position(|h| (h.line, h.start) >= (cursor.line, cursor.col))
            .unwrap_or(0)
    }

    fn field_mut(&mut self) -> &mut String {
        match self.find.field {
            FindField::Find => &mut self.find.find,
            FindField::Replace => &mut self.find.replace,
            FindField::Speaker => &mut self.find.speaker,
        }
    }

    pub(super) fn find_key(&mut self, key: KeyEvent) -> Flow {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

        match key.code {
            KeyCode::Esc => self.mode = Mode::Normal,
            KeyCode::Tab => self.find.field = self.find.field.next(),
            KeyCode::BackTab => self.find.field = self.find.field.previous(),
            KeyCode::Left | KeyCode::Right if self.find.field == FindField::Speaker => {
                let way = match key.code == KeyCode::Left {
                    true => Way::Previous,
                    false => Way::Next,
                };
                self.pick_speaker(way);
            }
            KeyCode::Down | KeyCode::Up => return self.find_step_in(key.code == KeyCode::Up),
            KeyCode::Char(c) if ctrl => return self.find_ctrl(c),
            KeyCode::Enter if self.find.field == FindField::Replace => self.replace_one(),
            KeyCode::Enter => return self.find_step_in(false),
            KeyCode::Backspace => self.edit_field(None),
            KeyCode::Char(c) => self.edit_field(Some(c)),
            _ => {}
        }
        self.show_cursor();

        Flow::Continue
    }

    /// The Ctrl keys of the search box.
    fn find_ctrl(&mut self, c: char) -> Flow {
        match c {
            'n' => return self.find_step_in(false),
            'p' => return self.find_step_in(true),
            'r' => self.replace_one(),
            'a' => self.replace_every(),
            't' => self.toggle_case(),
            _ => {}
        }
        self.show_cursor();

        Flow::Continue
    }

    /// Step within the box: the matches were found for what is typed now.
    fn find_step_in(&mut self, back: bool) -> Flow {
        let flow = self.find_step(back);
        self.show_cursor();

        flow
    }

    fn edit_field(&mut self, typed: Option<char>) {
        match typed {
            Some(c) => self.field_mut().push(c),
            None => {
                self.field_mut().pop();
            }
        }
        // The replacement changes no match.
        if self.find.field == FindField::Replace {
            return;
        }
        self.find_update();
        self.find_show();
    }

    fn toggle_case(&mut self) {
        self.find.case = match self.find.case {
            Case::Ignore => Case::Match,
            Case::Match => Case::Ignore,
        };
        self.find_update();
        self.find_show();
    }

    /// Steps the "Only dialogue of" box through the characters of the script, most frequent
    /// first, and the empty choice (everyone) before the first and after the last.
    fn pick_speaker(&mut self, way: Way) {
        let names = self.analysis.vocab.characters();
        let at = names
            .iter()
            .position(|n| n.to_lowercase() == self.find.speaker.trim().to_lowercase());
        // Slot 0 is "everyone", slot n is `names[n - 1]`.
        let count = names.len() + 1;
        let now = at.map_or(0, |i| i + 1);
        let next = match way {
            Way::Previous => (now + count - 1) % count,
            Way::Next => (now + 1) % count,
        };
        self.find.speaker = match next {
            0 => String::new(),
            n => names[n - 1].clone(),
        };
        self.find_update();
        self.find_show();
    }

    /// Replaces the match the cursor is on and goes on to the next.
    fn replace_one(&mut self) {
        let Some(hit) = self.find.hits.get(self.find.sel).copied() else {
            return;
        };
        let new = self.find.replace.clone();
        let col = hit.start + new.chars().count();
        self.splice_hits(
            &[hit],
            Pos {
                line: hit.line,
                col,
            },
        );
        self.find_update();
        self.find_show();
        self.notice = None;
    }

    /// Replaces every match, as one undo step.
    fn replace_every(&mut self) {
        let hits = self.find.hits.clone();
        let cursor = self.cursor_after(&hits);
        self.splice_hits(&hits, cursor);
        self.find_update();
        self.notice = Some(Notice::Replaced(hits.len()));
    }

    /// Where the cursor is once `hits` are replaced, on the same text: lines joined by a
    /// hit before it move it up, and on the line a hit ends on, its column follows the
    /// change in length. A cursor inside a hit is left where it is (the editor clamps it).
    fn cursor_after(&self, hits: &[Hit]) -> Pos {
        let cursor = self.editor.cursor();
        let width = self.find.replace.chars().count() as isize;
        let mut joined = 0;
        // Original line a column shift holds on, and the shift.
        let mut shift = (usize::MAX, 0isize);

        for hit in hits {
            if (hit.end_line, hit.end) > (cursor.line, cursor.col) {
                break;
            }
            // Earlier hits on the line where this one starts moved its start.
            let before = if shift.0 == hit.line { shift.1 } else { 0 };
            let new_end = hit.start as isize + before + width;
            shift = (hit.end_line, new_end - hit.end as isize);
            joined += hit.end_line - hit.line;
        }

        let col = match shift.0 == cursor.line {
            true => cursor.col.saturating_add_signed(shift.1),
            false => cursor.col,
        };

        Pos {
            line: cursor.line - joined,
            col,
        }
    }

    /// Puts the replacement over `hits` in the editor, one undo step.
    fn splice_hits(&mut self, hits: &[Hit], cursor: Pos) {
        let Some(edit) = replace_hits(self.editor.lines(), hits, &self.find.replace) else {
            return;
        };

        self.editor
            .set_lines(edit.from, edit.to, edit.lines, cursor);
    }
}
