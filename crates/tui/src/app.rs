//! The editor's state and what keys and the mouse do to it; no drawing, no terminal.

use std::path::Path;

use klappe_services::{ServiceError, Session};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};

use crate::buffer::Pos;
use crate::editor::Editor;
use crate::fountain::{analyse, Analysis};
use crate::i18n::Language;
use crate::layout::Layout;

/// Rows one mouse wheel notch scrolls.
const WHEEL_ROWS: usize = 3;
/// Entries a PageUp/PageDown moves in the scene list.
const LIST_PAGE: usize = 10;
/// Text width and height before the first frame says the real ones.
const DEFAULT_WIDTH: usize = 80;
const DEFAULT_HEIGHT: usize = 24;

/// Whether the program goes on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flow {
    Continue,
    Quit,
}

/// Which pane the keys go to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Focus {
    List,
    Editor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mode {
    Normal,
    Help,
    ConfirmQuit,
}

/// A message for the status line, until the next key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Notice {
    Saved,
    SaveFailed(String),
}

#[derive(Debug)]
pub struct App {
    session: Session,
    file: String,
    pub(crate) lang: Language,
    pub(crate) editor: Editor,
    pub(crate) analysis: Analysis,
    pub(crate) layout: Layout,
    pub(crate) focus: Focus,
    pub(crate) mode: Mode,
    pub(crate) notice: Option<Notice>,
    /// Selected entry of the scene list.
    pub(crate) list_sel: usize,
    /// First visible screen row of the editor.
    pub(crate) scroll: usize,
    width: usize,
    height: usize,
    saved_revision: u64,
    /// Revision and width the analysis and layout were built for.
    built_for: Option<(u64, usize)>,
}

impl App {
    /// Opens the project folder through the session.
    pub fn open(dir: &Path, lang: Language) -> Result<Self, ServiceError> {
        let session = Session::open(dir)?;
        let editor = Editor::new(session.script_text());
        let file = dir.file_name().map_or_else(
            || dir.display().to_string(),
            |n| n.to_string_lossy().into_owned(),
        );

        let mut app = Self {
            session,
            file,
            lang,
            saved_revision: editor.revision(),
            editor,
            analysis: Analysis::default(),
            layout: Layout::default(),
            focus: Focus::Editor,
            mode: Mode::Normal,
            notice: None,
            list_sel: 0,
            scroll: 0,
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
            built_for: None,
        };
        app.refresh();

        Ok(app)
    }

    pub(crate) fn file_name(&self) -> &str {
        &self.file
    }

    /// The text as it is in the editor, saved or not.
    pub fn text(&self) -> String {
        self.editor.text()
    }

    pub fn is_modified(&self) -> bool {
        self.editor.revision() != self.saved_revision
    }

    pub(crate) fn cursor(&self) -> Pos {
        self.editor.cursor()
    }

    /// The text size the next frame has; rebuilds the layout when it changed.
    pub(crate) fn set_view(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height.max(1);
        self.refresh();
        self.clamp_scroll();
    }

    /// Rebuilds what depends on the text or the width, if either changed.
    fn refresh(&mut self) {
        let key = (self.editor.revision(), self.width);
        if self.built_for == Some(key) {
            return;
        }
        self.built_for = Some(key);

        let lines = self.editor.lines();
        self.analysis = analyse(lines);
        self.layout = Layout::build(lines, &self.analysis.kinds, self.width);
        self.list_sel = self
            .list_sel
            .min(self.analysis.scenes.len().saturating_sub(1));
    }

    fn clamp_scroll(&mut self) {
        self.scroll = self.scroll.min(self.layout.len().saturating_sub(1));
    }

    /// Scrolls just far enough to show the cursor.
    fn show_cursor(&mut self) {
        let row = self.layout.row_of(self.editor.cursor());
        if row < self.scroll {
            self.scroll = row;
        }
        if row >= self.scroll + self.height {
            self.scroll = row + 1 - self.height;
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Flow {
        match self.mode {
            Mode::Help => {
                self.mode = Mode::Normal;
                return Flow::Continue;
            }
            Mode::ConfirmQuit => return self.answer_quit(key),
            Mode::Normal => {}
        }

        self.notice = None;
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let flow = match (ctrl, key.code) {
            (true, KeyCode::Char('q')) => self.ask_quit(),
            (true, KeyCode::Char('s')) => self.save(),
            (true, KeyCode::Char('z')) => self.edit_with(Editor::undo),
            (true, KeyCode::Char('y')) => self.edit_with(Editor::redo),
            (true, KeyCode::Char('g')) => self.jump(),
            (_, KeyCode::F(1)) => self.open_help(),
            (_, KeyCode::Tab) => self.switch_focus(),
            _ => self.pane_key(key),
        };
        self.refresh();
        self.show_cursor_if_editing();

        flow
    }

    fn show_cursor_if_editing(&mut self) {
        if self.focus == Focus::Editor {
            self.show_cursor();
        }
    }

    fn answer_quit(&mut self, key: KeyEvent) -> Flow {
        self.mode = Mode::Normal;
        match key.code {
            KeyCode::Char('y' | 'Y' | 'j' | 'J') => Flow::Quit,
            _ => Flow::Continue,
        }
    }

    fn ask_quit(&mut self) -> Flow {
        if !self.is_modified() {
            return Flow::Quit;
        }
        self.mode = Mode::ConfirmQuit;

        Flow::Continue
    }

    fn open_help(&mut self) -> Flow {
        self.mode = Mode::Help;
        Flow::Continue
    }

    fn switch_focus(&mut self) -> Flow {
        self.focus = match self.focus {
            Focus::List => Focus::Editor,
            Focus::Editor => Focus::List,
        };
        // The list starts at the scene the cursor is in.
        if self.focus == Focus::List {
            self.list_sel = self
                .analysis
                .scene_at(self.editor.cursor().line)
                .unwrap_or(0);
        }

        Flow::Continue
    }

    fn edit_with(&mut self, action: fn(&mut Editor)) -> Flow {
        action(&mut self.editor);
        self.focus = Focus::Editor;
        Flow::Continue
    }

    /// Writes the text through the session.
    fn save(&mut self) -> Flow {
        self.session.set_script_text(&self.editor.text());
        self.notice = Some(match self.session.save_script() {
            Ok(()) => {
                self.saved_revision = self.editor.revision();
                Notice::Saved
            }
            Err(e) => Notice::SaveFailed(e.to_string()),
        });

        Flow::Continue
    }

    /// Moves the cursor to the heading of the scene selected in the list.
    fn jump(&mut self) -> Flow {
        let Some(scene) = self.analysis.scenes.get(self.list_sel) else {
            return Flow::Continue;
        };
        self.editor.go_to_line(scene.line);
        self.focus = Focus::Editor;

        // The heading goes to the top of the view.
        self.scroll = self.layout.row_of(self.editor.cursor());

        Flow::Continue
    }

    fn pane_key(&mut self, key: KeyEvent) -> Flow {
        match self.focus {
            Focus::List => self.list_key(key),
            Focus::Editor => self.editor_key(key),
        }
        Flow::Continue
    }

    fn list_key(&mut self, key: KeyEvent) {
        let last = self.analysis.scenes.len().saturating_sub(1);
        match key.code {
            KeyCode::Up => self.list_sel = self.list_sel.saturating_sub(1),
            KeyCode::Down => self.list_sel = (self.list_sel + 1).min(last),
            KeyCode::PageUp => self.list_sel = self.list_sel.saturating_sub(LIST_PAGE),
            KeyCode::PageDown => self.list_sel = (self.list_sel + LIST_PAGE).min(last),
            KeyCode::Home => self.list_sel = 0,
            KeyCode::End => self.list_sel = last,
            KeyCode::Enter => {
                self.jump();
            }
            _ => {}
        }
    }

    fn editor_key(&mut self, key: KeyEvent) {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let alt = key.modifiers.contains(KeyModifiers::ALT);
        let page = isize::try_from(self.height).unwrap_or(isize::MAX);

        match key.code {
            KeyCode::Char(c) if !ctrl && !alt => self.editor.type_char(c),
            KeyCode::Enter => self.editor.newline(),
            KeyCode::Backspace => self.editor.backspace(),
            KeyCode::Delete => self.editor.delete(),
            KeyCode::Left if ctrl => self.editor.word_left(),
            KeyCode::Right if ctrl => self.editor.word_right(),
            KeyCode::Left => self.editor.left(),
            KeyCode::Right => self.editor.right(),
            KeyCode::Up => self.editor.rows(-1, &self.layout),
            KeyCode::Down => self.editor.rows(1, &self.layout),
            KeyCode::PageUp => self.editor.rows(-page, &self.layout),
            KeyCode::PageDown => self.editor.rows(page, &self.layout),
            KeyCode::Home => self.editor.home(&self.layout),
            KeyCode::End => self.editor.end(&self.layout),
            _ => {}
        }
    }

    /// The wheel scrolls the editor without moving the cursor.
    pub fn handle_mouse(&mut self, event: MouseEvent) {
        if self.mode != Mode::Normal {
            return;
        }
        match event.kind {
            MouseEventKind::ScrollUp => self.scroll = self.scroll.saturating_sub(WHEEL_ROWS),
            MouseEventKind::ScrollDown => self.scroll += WHEEL_ROWS,
            _ => return,
        }
        self.clamp_scroll();
    }
}
