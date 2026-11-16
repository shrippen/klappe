//! Enter and Tab change the element of a line; suggestions complete what is typed.
//!
//! A line that has been turned into an element by Enter or Tab, and has no text yet that
//! tells what it is, is `pending`: it shows as that element (indent, colour) until the
//! cursor leaves it.

use klappe_services::{suggest, Context, Gap, Slot, Suggestions};

use super::{App, Mode};
use crate::buffer::Pos;
use crate::fountain::LineKind;

/// An empty parenthetical as Tab leaves it.
const EMPTY_PARENTHETICAL: &str = "()";
const PAREN_CLOSE: char = ')';

/// The suggestions open at the cursor and the one chosen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Suggest {
    pub items: Suggestions,
    pub sel: usize,
}

/// Whether the line has no text of its own: empty, or the `()` of an empty parenthetical.
fn is_empty_slot(text: &str, slot: Slot) -> bool {
    let text = text.trim();

    text.is_empty() || slot == Slot::Parenthetical && text == EMPTY_PARENTHETICAL
}

impl App {
    /// The element a line is: what the reader says, or the slot it was turned into.
    fn slot_of(&self, line: usize) -> Option<Slot> {
        self.analysis.kind(line).slot()
    }

    pub(super) fn is_parenthetical(&self, line: usize) -> bool {
        self.slot_of(line) == Some(Slot::Parenthetical)
    }

    /// The kind of the pending line, if `line` is it.
    pub(super) fn pending_kind(&self, line: usize) -> Option<LineKind> {
        match self.pending {
            Some((at, slot)) if at == line => Some(LineKind::of_slot(slot)),
            _ => None,
        }
    }

    fn set_pending(&mut self, line: usize, slot: Slot) {
        self.pending = Some((line, slot));
        self.built_for = None;
    }

    /// Lets go of the pending line once the cursor has left it.
    pub(super) fn leave_pending(&mut self) {
        let Some((line, _)) = self.pending else {
            return;
        };
        if line != self.editor.cursor().line {
            self.pending = None;
            self.built_for = None;
        }
    }

    /// A pending line with no recognised element shows as its slot.
    pub(super) fn apply_pending(&mut self) {
        let Some((line, slot)) = self.pending else {
            return;
        };
        let Some(kind) = self.analysis.kinds.get_mut(line) else {
            return;
        };
        if matches!(kind, LineKind::Blank | LineKind::Action) {
            *kind = LineKind::of_slot(slot);
        }
    }

    fn next_is_blank(&self, line: usize) -> bool {
        self.editor
            .lines()
            .get(line + 1)
            .is_none_or(|l| l.trim().is_empty())
    }

    /// Replaces the cursor line by `lines` (the first is its text) and starts a pending line
    /// after them.
    fn open_line(&mut self, line: usize, text: String, next: Slot, gap: Gap) {
        let mut lines = vec![text];
        if gap == Gap::Blank {
            lines.push(String::new());
        }
        lines.push(String::new());
        let at = Pos {
            line: line + lines.len() - 1,
            col: 0,
        };

        self.editor.set_lines(line, line + 1, lines, at);
        self.set_pending(at.line, next);
    }

    /// Enter: at the end of an element, the next element (Anhang A of the Lastenheft); with a
    /// suggestion open, takes it; elsewhere a plain line break.
    pub(super) fn enter(&mut self) {
        if self.suggest.is_some() {
            self.accept();
            return;
        }

        let Pos { line, col } = self.editor.cursor();
        let text = self.editor.lines()[line].clone();
        let slot = self.slot_of(line);
        let Some(slot) = slot.filter(|_| !text.trim().is_empty()) else {
            self.editor.newline();
            return;
        };

        if is_empty_slot(&text, slot) {
            self.tab();
            return;
        }
        if !self.at_element_end(&text, col, slot) {
            self.editor.newline();
            return;
        }
        if !self.next_is_blank(line) {
            // Below a cue or parenthetical the dialogue already stands: go there.
            self.go_below(line, slot);
            return;
        }

        let (next, gap) = slot.after_enter();
        self.open_line(line, text, next, gap);
    }

    /// The end of the element: the end of the line, or before the `)` of a parenthetical.
    fn at_element_end(&self, text: &str, col: usize, slot: Slot) -> bool {
        let len = text.chars().count();
        let before_close = slot == Slot::Parenthetical && text.trim_end().ends_with(PAREN_CLOSE);

        col == len || before_close && col + 1 == len
    }

    /// Enter at the end of a cue or parenthetical with text below: to the end of the next
    /// line. Anything else with text below is a plain line break.
    fn go_below(&mut self, line: usize, slot: Slot) {
        if !matches!(slot, Slot::Character | Slot::Parenthetical) {
            self.editor.newline();
            return;
        }
        let col = self.editor.lines()[line + 1].chars().count();

        self.editor.go_to(Pos {
            line: line + 1,
            col,
        });
    }

    /// Tab on an empty line changes its element; with a suggestion open it takes it.
    pub(super) fn tab(&mut self) {
        if self.suggest.is_some() {
            self.accept();
            return;
        }

        let Pos { line, .. } = self.editor.cursor();
        let text = self.editor.lines()[line].clone();
        let slot = self.slot_of(line).unwrap_or(Slot::Action);
        if !is_empty_slot(&text, slot) {
            return;
        }
        let Some(to) = slot.on_tab() else {
            return;
        };

        let (text, col) = match to {
            Slot::Parenthetical => (EMPTY_PARENTHETICAL.to_owned(), 1),
            _ => (String::new(), 0),
        };
        self.editor
            .set_lines(line, line + 1, vec![text], Pos { line, col });
        self.set_pending(line, to);
    }

    /// What the line at the cursor is for completion.
    fn context_at(&self, line: usize) -> Context {
        match self.analysis.kind(line) {
            LineKind::Heading => Context::Heading,
            LineKind::Character => Context::Cue,
            _ => Context::Line,
        }
    }

    /// Looks for suggestions at the end of the cursor line; none while one was dismissed for
    /// this text or the cursor is not at the end of the line.
    pub(super) fn update_suggest(&mut self) {
        let found = self.find_suggest();
        let same = self.suggest.as_ref().map(|s| &s.items) == found.as_ref();
        if same {
            return;
        }

        self.suggest = found.map(|items| Suggest { items, sel: 0 });
    }

    fn find_suggest(&self) -> Option<Suggestions> {
        let shown = self.mode == Mode::Normal && self.focus == super::Focus::Editor;
        let Pos { line, col } = self.editor.cursor();
        let text = self.editor.lines().get(line)?;
        if !shown || self.suggest_off == Some(self.editor.revision()) {
            return None;
        }
        if col != text.chars().count() {
            return None;
        }

        suggest(&self.analysis.vocab, text, line, self.context_at(line))
    }

    pub(super) fn dismiss_suggest(&mut self) {
        if self.suggest.take().is_some() {
            self.suggest_off = Some(self.editor.revision());
        }
    }

    pub(super) fn move_suggest(&mut self, delta: isize) {
        let Some(open) = self.suggest.as_mut() else {
            return;
        };
        let last = open.items.options.len().saturating_sub(1);

        open.sel = open.sel.saturating_add_signed(delta).min(last);
    }

    /// Takes the chosen suggestion; one that ends the element goes on to the next.
    pub(super) fn accept(&mut self) {
        let Some(open) = self.suggest.take() else {
            return;
        };
        let Some(choice) = open.items.options.get(open.sel) else {
            return;
        };
        let Pos { line, .. } = self.editor.cursor();
        let head: String = self.editor.lines()[line]
            .chars()
            .take(open.items.from)
            .collect();
        let text = format!("{head}{choice}");

        let next = match open.items.stage.finishes() && self.next_is_blank(line) {
            true => Some(self.after_stage(open.items.stage)),
            false => None,
        };
        let Some((slot, gap)) = next else {
            let col = text.chars().count();
            self.editor
                .set_lines(line, line + 1, vec![text], Pos { line, col });
            return;
        };
        self.open_line(line, text, slot, gap);
    }

    /// The element after a finished stage: a time ends a heading, a name or extension a cue.
    fn after_stage(&self, stage: klappe_services::Stage) -> (Slot, Gap) {
        match stage {
            klappe_services::Stage::Time => Slot::Heading.after_enter(),
            _ => Slot::Character.after_enter(),
        }
    }
}
