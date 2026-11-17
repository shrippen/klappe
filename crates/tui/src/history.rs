//! Undo and redo without a limit: a step keeps only the lines it changed, so a long
//! screenplay costs a few lines per step, not a copy of the text.

use crate::buffer::{Buffer, Pos};

/// Lines before a change: where they start, the lines, and the line count of the text.
#[derive(Debug)]
pub(crate) struct Before {
    start: usize,
    lines: Vec<String>,
    count: usize,
}

impl Before {
    /// Whether `lines` are the lines as they are.
    pub(crate) fn is(&self, lines: &[String]) -> bool {
        self.lines == lines
    }

    /// The lines `from..to` of the buffer, as they are now.
    pub(crate) fn of(buffer: &Buffer, from: usize, to: usize) -> Self {
        let count = buffer.lines().len();
        let to = to.min(count);
        let start = from.min(to);

        Self {
            start,
            lines: buffer.lines()[start..to].to_vec(),
            count,
        }
    }
}

/// The lines a step changed: `removed` are the old ones, `len` is how many stand there now.
#[derive(Debug)]
struct Step {
    start: usize,
    removed: Vec<String>,
    len: usize,
    /// Cursor before the step and after it.
    cursors: (Pos, Pos),
}

impl Step {
    /// The region of this step after `next` followed it, if the two touch: the first change
    /// and the second one are then one step.
    fn join(&mut self, next: Step) -> Option<Step> {
        let end = self.start + self.len;
        let next_end = next.start + next.removed.len();
        if next.start > end || self.start > next_end {
            return Some(next);
        }

        // Lines of `next` outside this step's region are unchanged by it: they are original.
        if next.start < self.start {
            let head = self.start - next.start;
            let mut removed = next.removed[..head].to_vec();
            removed.append(&mut self.removed);
            self.removed = removed;
            self.start = next.start;
        }
        if next_end > end {
            let tail = next.removed.len() - (next_end - end);
            self.removed.extend_from_slice(&next.removed[tail..]);
        }

        let union = end.max(next_end) - self.start;
        let grown = next.len as isize - next.removed.len() as isize;
        self.len = (union as isize + grown).max(0) as usize;
        self.cursors.1 = next.cursors.1;

        None
    }
}

#[derive(Debug, Default)]
pub(crate) struct History {
    undo: Vec<Step>,
    redo: Vec<Step>,
}

/// Swaps the lines of the step with the ones in the buffer; returns the step that undoes it.
fn swap(buffer: &mut Buffer, step: Step) -> Step {
    let end = step.start + step.len;
    let taken = buffer.splice(step.start, end, step.removed.clone());

    Step {
        start: step.start,
        len: step.removed.len(),
        removed: taken,
        cursors: (step.cursors.1, step.cursors.0),
    }
}

impl History {
    /// Notes a change. `joins` makes it part of the step before, when the two touch.
    pub(crate) fn record(
        &mut self,
        before: Before,
        buffer: &Buffer,
        cursors: (Pos, Pos),
        joins: bool,
    ) {
        let grown = buffer.lines().len() as isize - before.count as isize;
        let len = (before.lines.len() as isize + grown).max(0) as usize;
        let step = Step {
            start: before.start,
            removed: before.lines,
            len,
            cursors,
        };

        self.redo.clear();
        let step = match self.undo.last_mut().filter(|_| joins) {
            Some(last) => last.join(step),
            None => Some(step),
        };
        if let Some(step) = step {
            self.undo.push(step);
        }
    }

    /// Takes back the last step; returns where the cursor was before it.
    pub(crate) fn undo(&mut self, buffer: &mut Buffer) -> Option<Pos> {
        let step = self.undo.pop()?;
        let back = swap(buffer, step);
        let cursor = back.cursors.1;
        self.redo.push(back);

        Some(cursor)
    }

    /// Does the step again; returns where the cursor was after it.
    pub(crate) fn redo(&mut self, buffer: &mut Buffer) -> Option<Pos> {
        let step = self.redo.pop()?;
        let again = swap(buffer, step);
        let cursor = again.cursors.1;
        self.undo.push(again);

        Some(cursor)
    }

    /// Lines held by all steps, old and new: what the history costs.
    #[cfg(test)]
    pub(crate) fn lines_kept(&self) -> usize {
        let held = |steps: &[Step]| steps.iter().map(|s| s.removed.len() + s.len).sum::<usize>();

        held(&self.undo) + held(&self.redo)
    }

    /// Steps that can be undone.
    #[cfg(test)]
    pub(crate) fn depth(&self) -> usize {
        self.undo.len()
    }
}
