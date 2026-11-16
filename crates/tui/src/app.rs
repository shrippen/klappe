//! The editor's state and what keys and the mouse do to it; no drawing, no terminal.

use std::path::Path;
use std::time::{Duration, Instant};

use klappe_services::{ServiceError, Session};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};

use crate::buffer::Pos;
use crate::editor::Editor;
use crate::fountain::{analyse, typed_caps, Analysis};
use crate::i18n::{Language, Texts};
use crate::layout::Layout;
use crate::links::Links;

/// Rows one mouse wheel notch scrolls.
const WHEEL_ROWS: usize = 3;
/// Entries a PageUp/PageDown moves in the scene list.
const LIST_PAGE: usize = 10;
/// Text width and height before the first frame says the real ones.
const DEFAULT_WIDTH: usize = 80;
const DEFAULT_HEIGHT: usize = 24;
/// The scene list shows by itself from this terminal width on (`Ctrl+B` overrides).
const LIST_AUTO_MIN_COLS: u16 = 100;
/// Terminal width before the first frame says the real one.
const DEFAULT_SCREEN_COLS: u16 = 80;

/// Time without a key before the pages are laid out again (DN-02).
pub const IDLE_BEFORE_PAGES: Duration = Duration::from_secs(5);

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
    /// `Ctrl+P`: search box over the scenes.
    Goto,
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
    pub(crate) texts: Texts,
    pub(crate) links: Links,
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
    /// What the search box has been typed, and its selected match.
    pub(crate) goto_query: String,
    pub(crate) goto_sel: usize,
    /// `Ctrl+B`; `None` follows the terminal width.
    list_override: Option<bool>,
    screen_cols: u16,
    width: usize,
    height: usize,
    saved_revision: u64,
    /// Revision, width and page computation the analysis and layout were built for.
    built_for: Option<(u64, usize, u64)>,
    lang: Language,
    /// Counts the page computations; the layout follows it (`built_for`).
    pages_run: u64,
    /// The editor's revision the last page computation read.
    pages_for: u64,
    /// When the last key came; the pages wait `IDLE_BEFORE_PAGES` after it.
    last_key: Option<Instant>,
}

impl App {
    /// Opens the project folder through the session.
    pub fn open(dir: &Path, lang: Language) -> Result<Self, ServiceError> {
        let mut session = Session::open(dir)?;
        let links = Links::read(&mut session);
        let editor = Editor::new(session.script_text());
        let file = dir.file_name().map_or_else(
            || dir.display().to_string(),
            |n| n.to_string_lossy().into_owned(),
        );

        let mut app = Self {
            session,
            file,
            texts: Texts::new(lang),
            links,
            saved_revision: editor.revision(),
            editor,
            analysis: Analysis::default(),
            layout: Layout::default(),
            focus: Focus::Editor,
            mode: Mode::Normal,
            notice: None,
            list_sel: 0,
            scroll: 0,
            goto_query: String::new(),
            goto_sel: 0,
            list_override: None,
            screen_cols: DEFAULT_SCREEN_COLS,
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
            built_for: None,
            lang,
            pages_run: 0,
            pages_for: 0,
            last_key: None,
        };
        app.recompute_pages();
        app.refresh();

        Ok(app)
    }

    /// The pages of the last computation (DN-02), `None` never in practice: `open` lays out.
    pub(crate) fn pages(&self) -> Option<&klappe_services::ScriptPages> {
        self.session.script_pages()
    }

    /// The length of scene `index` of the list as the last computation has it, e.g. `3/8`;
    /// `None` without one. While the text has gained or lost scenes since, the scene is
    /// found by its line, which moves with typing: then a length may be missing, never wrong
    /// by more than the last edit.
    pub(crate) fn scene_length(&self, index: usize) -> Option<&str> {
        let spans = &self.pages()?.scenes;
        let span = match spans.len() == self.analysis.scenes.len() {
            true => spans.get(index),
            false => {
                let line = self.analysis.scenes.get(index)?.line;
                spans.iter().find(|s| s.line == line)
            }
        };

        span.map(|s| s.length.as_str())
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

    /// The terminal width of the frame being drawn. A list that is no longer on screen
    /// gives up the focus: keys would go to something the user cannot see.
    pub(crate) fn set_screen(&mut self, cols: u16) {
        self.screen_cols = cols;
        if !self.list_shown() && self.focus == Focus::List {
            self.focus = Focus::Editor;
        }
    }

    /// Whether the scene list is on screen.
    pub(crate) fn list_shown(&self) -> bool {
        self.list_override
            .unwrap_or(self.screen_cols >= LIST_AUTO_MIN_COLS)
    }

    /// The text size the next frame has; rebuilds the layout when it changed.
    pub(crate) fn set_view(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height.max(1);
        self.refresh();
        self.clamp_scroll();
    }

    /// Lays the pages out again for the text as it is in the editor (DN-02): the page
    /// breaks and scene lengths shown from now on are those of this text.
    pub fn recompute_pages(&mut self) {
        let text = self.editor.text();

        self.session.paginate_text(&text, self.lang);
        self.pages_for = self.editor.revision();
        self.pages_run += 1;
        self.refresh();
    }

    /// How long until the pages are due, `None` if they are up to date. They are due
    /// `IDLE_BEFORE_PAGES` after the last key; a key never waits for them.
    pub fn pages_wait(&self, now: Instant) -> Option<Duration> {
        if !self.pages_stale() {
            return None;
        }
        let since = self
            .last_key
            .map_or(IDLE_BEFORE_PAGES, |at| now.saturating_duration_since(at));

        Some(IDLE_BEFORE_PAGES.saturating_sub(since))
    }

    /// Whether the text changed since the pages were laid out.
    pub(crate) fn pages_stale(&self) -> bool {
        self.pages_for != self.editor.revision()
    }

    /// Rebuilds what depends on the text or the width, if either changed.
    fn refresh(&mut self) {
        let key = (self.editor.revision(), self.width, self.pages_run);
        if self.built_for == Some(key) {
            return;
        }
        self.built_for = Some(key);

        let lines = self.editor.lines();
        self.analysis = analyse(lines);
        let breaks: Vec<(usize, usize)> = self
            .session
            .script_pages()
            .map(|p| p.breaks.iter().map(|b| (b.line, b.page)).collect())
            .unwrap_or_default();
        self.layout = Layout::build(lines, &self.analysis.kinds, self.width, &breaks);
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
        self.last_key = Some(Instant::now());
        match self.mode {
            Mode::Help => {
                self.mode = Mode::Normal;
                return Flow::Continue;
            }
            Mode::ConfirmQuit => return self.answer_quit(key),
            Mode::Goto => return self.goto_key(key),
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
            (true, KeyCode::Char('p')) => self.open_goto(),
            (true, KeyCode::Char('b')) => self.toggle_list(),
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
        // Tab into a hidden list shows it.
        if !self.list_shown() {
            self.list_override = Some(true);
        }
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

    fn toggle_list(&mut self) -> Flow {
        let shown = self.list_shown();
        self.list_override = Some(!shown);
        if shown && self.focus == Focus::List {
            self.focus = Focus::Editor;
        }

        Flow::Continue
    }

    fn open_goto(&mut self) -> Flow {
        self.mode = Mode::Goto;
        self.goto_query.clear();
        self.goto_sel = 0;

        Flow::Continue
    }

    /// Indices of the scenes whose number or heading contain the query, any case.
    pub(crate) fn goto_matches(&self) -> Vec<usize> {
        let query = self.goto_query.to_lowercase();

        self.analysis
            .scenes
            .iter()
            .enumerate()
            .filter(|(i, scene)| {
                let label = format!("{} {}", scene.display_number(*i), scene.heading);
                label.to_lowercase().contains(&query)
            })
            .map(|(i, _)| i)
            .collect()
    }

    fn goto_key(&mut self, key: KeyEvent) -> Flow {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let matches = self.goto_matches();
        let last = matches.len().saturating_sub(1);

        match key.code {
            KeyCode::Esc => self.mode = Mode::Normal,
            KeyCode::Char('p') if ctrl => self.mode = Mode::Normal,
            KeyCode::Up => self.goto_sel = self.goto_sel.saturating_sub(1),
            KeyCode::Down => self.goto_sel = (self.goto_sel + 1).min(last),
            KeyCode::Enter => {
                let Some(&scene) = matches.get(self.goto_sel) else {
                    return Flow::Continue;
                };
                self.mode = Mode::Normal;
                self.jump_to(scene);
            }
            KeyCode::Backspace => {
                self.goto_query.pop();
                self.goto_sel = 0;
            }
            KeyCode::Char(c) if !ctrl => {
                self.goto_query.push(c);
                self.goto_sel = 0;
            }
            _ => {}
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
                self.links = Links::read(&mut self.session);
                Notice::Saved
            }
            Err(e) => Notice::SaveFailed(e.to_string()),
        });

        Flow::Continue
    }

    /// Moves the cursor to the heading of the scene selected in the list.
    fn jump(&mut self) -> Flow {
        self.jump_to(self.list_sel);
        Flow::Continue
    }

    /// Moves the cursor to the heading of scene `index`.
    fn jump_to(&mut self, index: usize) {
        let Some(scene) = self.analysis.scenes.get(index) else {
            return;
        };
        self.editor.go_to_line(scene.line);
        self.focus = Focus::Editor;
        self.list_sel = index;

        // The heading goes to the top of the view.
        self.scroll = self.layout.row_of(self.editor.cursor());
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

    /// Types the character; scene headings and forced cues turn to capitals as they go.
    fn type_char(&mut self, c: char) {
        self.editor.type_char(c);

        let Some(caps) = typed_caps(self.editor.lines(), self.editor.cursor().line) else {
            return;
        };
        self.editor.replace_line(caps);
    }

    fn editor_key(&mut self, key: KeyEvent) {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let alt = key.modifiers.contains(KeyModifiers::ALT);
        let page = isize::try_from(self.height).unwrap_or(isize::MAX);

        match key.code {
            KeyCode::Char(c) if !ctrl && !alt => self.type_char(c),
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
