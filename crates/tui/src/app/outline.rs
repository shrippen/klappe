//! The outline in the list pane (DB-20, DB-21): the tree of sections, scenes and
//! synopses, collapsing, the filter, jumping and moving blocks.

use klappe_services::{
    length_text, move_entry, visible_entries, MoveWay, OutlineEntry, OutlineKind,
};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::Rect;

use super::{App, Focus, LIST_PAGE};
use crate::buffer::Pos;

/// Columns a tree level is indented in the list.
pub(crate) const INDENT: usize = 2;
/// Columns the fold marker of a section takes (`▾ `).
const MARKER_COLS: usize = 2;

/// What `Left` and `Right` do to the selected section.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Fold {
    Close,
    Open,
    Toggle,
}

/// The first row of `height` rows that shows row `pos`, the selection at the bottom edge
/// once the list is longer than the pane.
pub(crate) fn list_top(pos: usize, height: usize) -> usize {
    (pos + 1).saturating_sub(height)
}

/// What a collapsed section is remembered by: its level and title. Typing above it does
/// not move it; renaming it expands it.
fn fold_key(entry: &OutlineEntry) -> String {
    format!("{:?}{}", entry.kind, entry.text)
}

fn is_section(entry: &OutlineEntry) -> bool {
    matches!(entry.kind, OutlineKind::Section(_))
}

impl App {
    /// Indices into `analysis.outline` of the rows on screen.
    pub(crate) fn list_rows(&self) -> Vec<usize> {
        let collapsed = |e: &OutlineEntry| self.collapsed.contains(&fold_key(e));

        visible_entries(&self.analysis.outline, &collapsed, &self.list_filter)
    }

    /// The row the selection is on: a selected entry that is not shown (collapsed, filtered
    /// out) counts as the row above it.
    pub(crate) fn list_pos(&self, rows: &[usize]) -> usize {
        rows.iter().rposition(|&i| i <= self.list_sel).unwrap_or(0)
    }

    pub(crate) fn is_collapsed(&self, entry: &OutlineEntry) -> bool {
        self.collapsed.contains(&fold_key(entry))
    }

    /// The length an entry shows: a scene's, a section's total of the scenes in it.
    pub(crate) fn entry_length(&self, index: usize) -> Option<String> {
        let entries = &self.analysis.outline;
        let entry = entries.get(index)?;
        if let Some(scene) = entry.scene {
            return self.scene_span(scene).map(|s| s.length.clone());
        }
        if !is_section(entry) {
            return None;
        }

        let total: usize = entries[index + 1..]
            .iter()
            .take_while(|e| e.line < entry.end)
            .filter_map(|e| self.scene_span(e.scene?))
            .map(|s| s.eighths)
            .sum();

        (total > 0).then(|| length_text(total))
    }

    /// The entry of the outline that is scene `scene`.
    pub(crate) fn entry_of_scene(&self, scene: usize) -> Option<usize> {
        self.analysis
            .outline
            .iter()
            .position(|e| e.scene == Some(scene))
    }

    /// The list's pane for the mouse, as the last frame drew it.
    pub(crate) fn set_list_area(&mut self, area: Rect) {
        self.list_area = area;
    }

    /// Moves the cursor to the line of the selected entry.
    pub(super) fn jump(&mut self) {
        self.jump_entry(self.list_sel);
    }

    fn jump_entry(&mut self, index: usize) {
        let Some(entry) = self.analysis.outline.get(index) else {
            return;
        };
        self.editor.go_to_line(entry.line);
        self.focus = Focus::Editor;
        self.list_sel = index;

        // The line goes to the top of the view.
        self.scroll = self.layout.row_of(self.editor.cursor());
    }

    pub(super) fn list_key(&mut self, key: KeyEvent) {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let alt = key.modifiers.contains(KeyModifiers::ALT);

        match key.code {
            KeyCode::Up if alt => self.move_block(MoveWay::Up),
            KeyCode::Down if alt => self.move_block(MoveWay::Down),
            KeyCode::Left => self.fold(Fold::Close),
            KeyCode::Right => self.fold(Fold::Open),
            KeyCode::Enter => self.jump(),
            KeyCode::Esc => self.leave_list(),
            KeyCode::Backspace => {
                self.list_filter.pop();
                self.select_first();
            }
            KeyCode::Char(c) if !ctrl && !alt => {
                self.list_filter.push(c);
                self.select_first();
            }
            code => self.list_move(code),
        }
    }

    /// The keys that move the selection.
    fn list_move(&mut self, code: KeyCode) {
        let rows = self.list_rows();
        let pos = self.list_pos(&rows);
        let last = rows.len().saturating_sub(1);

        match code {
            KeyCode::Up => self.select_row(&rows, pos.saturating_sub(1)),
            KeyCode::Down => self.select_row(&rows, (pos + 1).min(last)),
            KeyCode::PageUp => self.select_row(&rows, pos.saturating_sub(LIST_PAGE)),
            KeyCode::PageDown => self.select_row(&rows, (pos + LIST_PAGE).min(last)),
            KeyCode::Home => self.select_row(&rows, 0),
            KeyCode::End => self.select_row(&rows, last),
            _ => {}
        }
    }

    /// Esc clears the filter first, then leaves the list.
    fn leave_list(&mut self) {
        if self.list_filter.is_empty() {
            self.focus = Focus::Editor;
            return;
        }
        self.list_filter.clear();
    }

    fn select_row(&mut self, rows: &[usize], pos: usize) {
        if let Some(&index) = rows.get(pos) {
            self.list_sel = index;
        }
    }

    fn select_first(&mut self) {
        let rows = self.list_rows();
        self.select_row(&rows, 0);
    }

    /// Collapses or expands the selected section. `Left` on what is no open section goes to
    /// the section around it, as in a tree. A filter shows no tree: nothing folds.
    fn fold(&mut self, how: Fold) {
        if !self.list_filter.is_empty() {
            return;
        }
        let rows = self.list_rows();
        let Some(&index) = rows.get(self.list_pos(&rows)) else {
            return;
        };
        let entry = &self.analysis.outline[index];
        let key = fold_key(entry);
        let folded = self.collapsed.contains(&key);

        // A selection inside what just collapsed rests on its section.
        self.list_sel = index;
        match (how, is_section(entry), folded) {
            (Fold::Close | Fold::Toggle, true, false) => {
                self.collapsed.insert(key);
            }
            (Fold::Open | Fold::Toggle, true, true) => {
                self.collapsed.remove(&key);
            }
            (Fold::Close, _, _) => self.select_parent(index),
            _ => {}
        }
    }

    fn select_parent(&mut self, index: usize) {
        let entries = &self.analysis.outline;
        let depth = entries[index].depth;
        let parent = entries[..index]
            .iter()
            .rposition(|e| is_section(e) && e.depth < depth);

        if let Some(parent) = parent {
            self.list_sel = parent;
        }
    }

    /// A click in the list: the fold marker of a section folds it, any other place jumps.
    pub(super) fn click_list(&mut self, column: u16, row: u16) {
        let area = self.list_area;
        let inside = column >= area.x
            && column < area.x + area.width
            && row >= area.y
            && row < area.y + area.height;
        if !inside {
            return;
        }

        let rows = self.list_rows();
        let top = list_top(self.list_pos(&rows), usize::from(area.height));
        let Some(&index) = rows.get(top + usize::from(row - area.y)) else {
            return;
        };
        let entry = &self.analysis.outline[index];
        // The list shows a bar or blank (2 columns), then the indent, then the marker.
        let marker_at = usize::from(area.x) + 2 + entry.depth * INDENT;
        let on_marker = (marker_at..marker_at + MARKER_COLS).contains(&usize::from(column));

        self.list_sel = index;
        if !(is_section(entry) && on_marker) {
            self.jump_entry(index);
            return;
        }
        self.focus = Focus::List;
        self.fold(Fold::Toggle);
    }

    /// Moves the selected scene or section one place, as one undo step. The cursor goes with
    /// it. Not while a filter hides the order.
    fn move_block(&mut self, way: MoveWay) {
        if !self.list_filter.is_empty() {
            return;
        }
        let entries = &self.analysis.outline;
        let Some(moved) = move_entry(self.editor.lines(), entries, self.list_sel, way) else {
            return;
        };
        let cursor = Pos {
            line: moved.line,
            col: 0,
        };

        self.editor
            .set_lines(moved.from, moved.to, moved.lines, cursor);
        self.pending = None;
        self.refresh();
        let at = self
            .analysis
            .outline
            .iter()
            .position(|e| e.line == moved.line);
        self.list_sel = at.unwrap_or(self.list_sel);
    }
}
