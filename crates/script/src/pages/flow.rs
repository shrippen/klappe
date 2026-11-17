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
        flow.place(block.clone(), &blocks[i + 1..]);
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

    fn place(&mut self, mut block: Block, rest: &[Block]) {
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
            // A heading is never the last thing on a page: it stays only where what follows
            // can start on the page.
            let company = !block.heading || at_top || self.can_start(rest, left);
            if before + rows.len() <= self.space() && company {
                self.put(&block, before, rows);
                return;
            }

            let room = self.space().saturating_sub(before);
            if let Some((first, tail)) = block.split(room, self.options.words) {
                self.put_first(&first, before);
                self.end_page();
                block = tail;
                continue;
            }

            // Taller than a page, with no place to cut: cut at a row, right here if two
            // rows fit (under the heading before it), else at the top of the next page.
            let tall = rows.len() > self.options.body_lines;
            if !at_top && !(tall && room >= MIN_SPLIT_ROWS) {
                self.end_page();
                continue;
            }

            let take = match at_top {
                true => self.options.body_lines.max(MIN_SPLIT_ROWS),
                false => room,
            };
            let mut rows = rows;
            let tail = rows.split_off(take.min(rows.len()));
            self.put(&block, before, rows);
            self.end_page();
            block = Block::from_rows(tail);
        }
    }

    /// Whether what follows can start in `left` rows: whole, cut where the rules allow, or a
    /// block taller than a page, which is cut at a row. A heading after a heading needs
    /// the one after it to start too.
    fn can_start(&self, rest: &[Block], left: usize) -> bool {
        let Some(next) = rest.first().filter(|n| !n.page_break) else {
            return true;
        };
        let rows = next.rows().len();
        if rows == 0 {
            return self.can_start(&rest[1..], left);
        }

        let whole = next.before + rows <= left;
        if next.heading {
            return whole && self.can_start(&rest[1..], left - next.before - rows);
        }

        let room = left.saturating_sub(next.before);
        let tall = rows > self.options.body_lines && room >= MIN_SPLIT_ROWS;

        whole || tall || next.split(room, self.options.words).is_some()
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
