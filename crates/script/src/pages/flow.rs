//! Puts blocks on pages: fills a page, cuts a block where the rules allow, starts the next.

use super::block::{blocks, Block, MIN_SPLIT_ROWS};
use super::{Line, Page, PageKind, PageOptions, Pagination, SceneLength};
use crate::Element;

/// Column of `(MORE)`: under the name of a dialogue.
const MORE_X: usize = 22;

struct Flow<'a> {
    options: &'a PageOptions,
    pages: Vec<Page>,
    /// The page being filled and the rows it has used.
    lines: Vec<Line>,
    used: usize,
    scenes: Vec<SceneLength>,
    total: usize,
}

pub(super) fn run(elements: &[Element], options: &PageOptions) -> Pagination {
    let blocks = blocks(elements, options);
    let mut flow = Flow {
        options,
        pages: Vec::new(),
        lines: Vec::new(),
        used: 0,
        scenes: Vec::new(),
        total: 0,
    };

    for (i, block) in blocks.iter().enumerate() {
        flow.place(block.clone(), blocks.get(i + 1));
    }
    flow.end_page();

    let mut result = Pagination {
        size: options.size,
        pages: flow.pages,
        scenes: flow.scenes,
        lines: flow.total,
        body_lines: options.body_lines,
    };
    for i in 0..result.scenes.len() {
        result.scenes[i].eighths = result.eighths_of(result.scenes[i].lines).max(1);
    }

    result
}

impl Flow<'_> {
    fn space(&self) -> usize {
        self.options.body_lines.saturating_sub(self.used)
    }

    /// Finishes the page, if it has anything on it.
    fn end_page(&mut self) {
        self.used = 0;
        if self.lines.is_empty() {
            return;
        }

        let number = self.pages.len() + 1;
        let lines = std::mem::take(&mut self.lines);
        self.pages.push(Page { number, lines });
    }

    fn place(&mut self, mut block: Block, next: Option<&Block>) {
        if block.page_break {
            self.end_page();
            return;
        }

        loop {
            let at_top = self.used == 0;
            let before = if at_top { 0 } else { block.before };
            let rows = block.rows();
            if rows.is_empty() {
                return;
            }

            let left = self.space().saturating_sub(before + rows.len());
            // A heading stays only where the block after it can start on the page.
            let company = !block.heading || at_top || self.can_start(next, left);
            if before + rows.len() <= self.space() && company {
                self.put(&block, before, rows);
                return;
            }

            let room = self.space().saturating_sub(before);
            if let Some((first, rest)) = block.split(room, self.options.words) {
                self.put_first(&first, before);
                self.end_page();
                block = rest;
                continue;
            }

            if !at_top {
                self.end_page();
                continue;
            }

            // Taller than a page, with no place to cut: cut at a row.
            let mut rows = rows;
            let rest = rows.split_off(self.options.body_lines.max(MIN_SPLIT_ROWS).min(rows.len()));
            self.put(&block, 0, rows);
            self.end_page();
            block = Block::from_rows(rest);
        }
    }

    /// Whether `next` can start in `left` rows: whole, or cut where the rules allow.
    fn can_start(&self, next: Option<&Block>, left: usize) -> bool {
        let Some(next) = next.filter(|n| !n.page_break) else {
            return true;
        };
        let rows = next.rows().len();
        if rows == 0 || next.before + rows <= left {
            return true;
        }

        let room = left.saturating_sub(next.before);
        next.split(room, self.options.words).is_some()
    }

    /// The first half of a cut block; a dialogue ends in `(MORE)`.
    fn put_first(&mut self, first: &Block, before: usize) {
        let mut rows = first.rows();
        let dialogue = rows
            .iter()
            .any(|r| r.first().is_some_and(|l| l.kind == PageKind::Character));
        if dialogue {
            let element = first.element().unwrap_or_default();
            rows.push(vec![Line {
                row: 0,
                x: MORE_X,
                kind: PageKind::More,
                text: format!("({})", self.options.words.more),
                element,
                offset: 0,
                number: None,
                added: true,
            }]);
        }

        self.put(first, before, rows);
    }

    fn put(&mut self, block: &Block, before: usize, rows: Vec<Vec<Line>>) {
        self.used += before;
        let first_row = self.used;

        if block.heading {
            self.scenes.push(SceneLength {
                element: block.element().unwrap_or_default(),
                page: self.pages.len() + 1,
                row: first_row,
                lines: 0,
                eighths: 0,
            });
        }

        let count = before + rows.len();
        for mut row in rows {
            for line in &mut row {
                line.row = self.used;
            }
            self.lines.extend(row);
            self.used += 1;
        }

        self.total += count;
        if let Some(scene) = self.scenes.last_mut() {
            scene.lines += count;
        }
    }
}
