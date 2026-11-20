//! The scene numbering form (`Ctrl+N`, DB-33): automatic or locked, and the first number.

use klappe_services::{error_text, SceneNumbering};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::{App, Flow, Mode, Notice};

/// The rows of the form.
const ROW_NUMBERING: usize = 0;
const ROW_FIRST: usize = 1;
const ROWS: usize = 2;

#[derive(Debug)]
pub(crate) struct NumbersForm {
    pub numbering: SceneNumbering,
    /// What the box shows of the first number.
    pub first: String,
    pub sel: usize,
}

fn other(numbering: SceneNumbering) -> SceneNumbering {
    match numbering {
        SceneNumbering::Automatic => SceneNumbering::Locked,
        SceneNumbering::Locked => SceneNumbering::Automatic,
    }
}

impl App {
    pub(super) fn open_numbers(&mut self) -> Flow {
        let setup = self.session.project().script;

        self.numbers = Some(NumbersForm {
            numbering: setup.scene_numbering,
            first: setup.first_scene.to_string(),
            sel: ROW_NUMBERING,
        });
        self.mode = Mode::Numbers;
        self.suggest = None;

        Flow::Continue
    }

    pub(super) fn numbers_key(&mut self, key: KeyEvent) -> Flow {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let Some(form) = self.numbers.as_mut() else {
            self.mode = Mode::Normal;
            return Flow::Continue;
        };

        match (form.sel, key.code) {
            (_, KeyCode::Esc) => self.close_numbers(),
            (_, KeyCode::Enter) => self.apply_numbers(),
            (_, KeyCode::Tab | KeyCode::Down | KeyCode::BackTab | KeyCode::Up) => {
                form.sel = (form.sel + 1) % ROWS;
            }
            (ROW_NUMBERING, KeyCode::Left | KeyCode::Right | KeyCode::Char(' ')) => {
                form.numbering = other(form.numbering);
            }
            (ROW_FIRST, KeyCode::Char('u')) if ctrl => form.first.clear(),
            (ROW_FIRST, KeyCode::Backspace) => {
                form.first.pop();
            }
            (ROW_FIRST, KeyCode::Char(c)) if c.is_ascii_digit() && !ctrl => form.first.push(c),
            _ => {}
        }

        Flow::Continue
    }

    fn close_numbers(&mut self) {
        self.numbers = None;
        self.mode = Mode::Normal;
    }

    /// Keeps the choice in the project; a first number that is no number stays as it was.
    fn apply_numbers(&mut self) {
        let Some(form) = self.numbers.take() else {
            return;
        };
        self.mode = Mode::Normal;

        let before = self.session.project().script.first_scene;
        let first = form.first.parse().unwrap_or(before);
        if let Err(e) = self.session.set_scene_numbering(form.numbering, first) {
            self.notice = Some(Notice::SaveFailed(error_text(self.lang, &e)));
        }
    }
}
