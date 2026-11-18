//! The title page form (`Ctrl+T`): Title, Credit, Author, Source, Draft date, Contact,
//! stored as the Fountain title page at the top of the text.

use klappe_services::{set_title_fields, title_fields};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::{App, Flow, Mode};

const LINE: &str = "\n";

#[derive(Debug)]
pub(crate) struct TitleField {
    /// The Fountain key.
    pub key: String,
    /// What the box holds; lines are separated by `\n`.
    pub shown: String,
    /// What the text had; kept when the box is left as it was.
    raw: String,
}

#[derive(Debug)]
pub(crate) struct TitleForm {
    pub fields: Vec<TitleField>,
    pub sel: usize,
}

impl App {
    pub(super) fn open_title(&mut self) -> Flow {
        let fields = title_fields(&self.editor.text())
            .into_iter()
            .map(|(key, raw)| TitleField {
                key,
                shown: raw.clone(),
                raw,
            })
            .collect();

        self.title = Some(TitleForm { fields, sel: 0 });
        self.mode = Mode::Title;
        self.suggest = None;

        Flow::Continue
    }

    pub(super) fn title_key(&mut self, key: KeyEvent) -> Flow {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let Some(form) = self.title.as_mut() else {
            self.mode = Mode::Normal;
            return Flow::Continue;
        };
        let count = form.fields.len();

        match key.code {
            KeyCode::Esc => self.close_title(),
            KeyCode::Enter
                if key
                    .modifiers
                    .intersects(KeyModifiers::ALT | KeyModifiers::SHIFT) =>
            {
                form.fields[form.sel].shown.push_str(LINE);
            }
            KeyCode::Enter => self.apply_title(),
            KeyCode::Tab | KeyCode::Down => form.sel = (form.sel + 1) % count,
            KeyCode::BackTab | KeyCode::Up => form.sel = (form.sel + count - 1) % count,
            KeyCode::Char('u') if ctrl => form.fields[form.sel].shown.clear(),
            KeyCode::Backspace => {
                form.fields[form.sel].shown.pop();
            }
            KeyCode::Char(c) if !ctrl => form.fields[form.sel].shown.push(c),
            _ => {}
        }

        Flow::Continue
    }

    fn close_title(&mut self) {
        self.title = None;
        self.mode = Mode::Normal;
    }

    /// Writes the fields into the text, as one undo step.
    fn apply_title(&mut self) {
        let Some(form) = self.title.take() else {
            return;
        };
        let fields: Vec<(String, String)> = form
            .fields
            .into_iter()
            .map(|f| {
                let value = match f.shown == f.raw {
                    true => f.raw,
                    false => without_blank_lines(&f.shown),
                };
                (f.key, value)
            })
            .collect();

        let text = set_title_fields(&self.editor.text(), &fields);
        self.editor.set_text(&text);
        self.mode = Mode::Normal;
        self.pending = None;
        self.built_for = None;
    }
}

/// The value without empty lines (a new line left empty, `Contact:` with a gap).
fn without_blank_lines(value: &str) -> String {
    let lines: Vec<&str> = value.lines().filter(|l| !l.trim().is_empty()).collect();

    lines.join(LINE)
}
