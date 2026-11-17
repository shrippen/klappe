//! Puts blocks on pages. One placement function, `plan`, serves the real run and the look
//! ahead on a copy of the position, so both decide the same way.
//!
//! For a block with room left on the page:
//! 1. it fits whole: place it;
//! 2. else the last allowed cut with `MIN_SPLIT_ROWS` rows on both sides (a dialogue gets
//!    `(MORE)` and `(CONT'D)`): place the first part, end the page, go on with the rest;
//! 3. else, for a block taller than a page that has no cut that works on a fresh page, or
//!    one that would leave a heading alone: cut at a row;
//! 4. else end the page and try again on the fresh one (at the top of a page, a block that
//!    still does not fit is cut at a row, so the flow always advances).
//!
//! A heading is never the last thing on a page: before the first heading of a chain,
//! the chain and the block after it are placed on a copy; if a heading would end a page,
//! the page ends first, once.

use super::block::{blocks, Block, MIN_SPLIT_ROWS};
use super::{Line, Page, PageKind, PageOptions, Pagination, SceneLength};
use crate::Element;

/// Column of `(MORE)`: under the name of a dialogue.
const MORE_X: usize = 22;

/// What the placement does, in order; the real run turns it into pages.
enum Step {
    Put {
        heading: bool,
        element: usize,
        before: usize,
        rows: Vec<Vec<Line>>,
    },
    EndPage,
}

/// Where the next row goes, and what the pages so far would look like.
#[derive(Clone)]
struct Sim {
    used: usize,
    /// The last thing put on this page is a heading.
    last_heading: bool,
    /// Only headings are on this page so far: moving on would not give them company.
    only_headings: bool,
    /// A page ended on a heading.
    bad: bool,
    /// Pages ended.
    ended: usize,
}

impl Sim {
    fn end_page(&mut self, out: &mut Vec<Step>) {
        if self.used == 0 {
            return;
        }

        self.bad |= self.last_heading;
        self.ended += 1;
        self.used = 0;
        self.last_heading = false;
        self.only_headings = true;
        out.push(Step::EndPage);
    }

    fn put(&mut self, out: &mut Vec<Step>, block: &Block, before: usize, rows: Vec<Vec<Line>>) {
        self.used += before + rows.len();
        self.last_heading = block.heading;
        self.only_headings &= block.heading;
        out.push(Step::Put {
            heading: block.heading,
            element: block.element().unwrap_or_default(),
            before,
            rows,
        });
    }

    /// The first part of a cut block; a dialogue ends in `(MORE)`.
    fn put_first(&mut self, out: &mut Vec<Step>, first: &Block, before: usize, o: &PageOptions) {
        let mut rows = first.rows();
        let dialogue = rows
            .iter()
            .any(|r| r.first().is_some_and(|l| l.kind == PageKind::Character));
        if dialogue {
            rows.push(vec![Line {
                row: 0,
                x: MORE_X,
                kind: PageKind::More,
                text: format!("({})", o.words.more),
                element: first.element().unwrap_or_default(),
                offset: 0,
                number: None,
                added: true,
            }]);
        }

        self.put(out, first, before, rows);
    }
}

/// Places `block` from the position `sim`, appending the steps.
fn plan(sim: &mut Sim, o: &PageOptions, mut block: Block, out: &mut Vec<Step>) {
    if block.page_break {
        sim.end_page(out);
        return;
    }

    loop {
        let at_top = sim.used == 0;
        let before = if at_top { 0 } else { block.before };
        let rows = block.rows();
        if rows.is_empty() {
            return;
        }

        let space = o.body_lines.saturating_sub(sim.used);
        if before + rows.len() <= space {
            sim.put(out, &block, before, rows);
            return;
        }

        let room = space.saturating_sub(before);
        if let Some((first, rest)) = block.split(room, o.words) {
            sim.put_first(out, &first, before, o);
            sim.end_page(out);
            block = rest;
            continue;
        }

        // A dialogue cut at a row also gets `(MORE)` below and its name(s) again above.
        let extra = usize::from(cue_row(&rows).is_some());
        let tall = rows.len() > o.body_lines;
        let fresh = block.split(o.body_lines, o.words).is_some();
        // A block that cannot start here is cut at a row if it is taller than a page and
        // would not be cut on a fresh page either, or if headings would be left alone on a page.
        let wanted = (tall && !fresh) || (sim.last_heading && sim.only_headings);
        let here = wanted && room >= MIN_SPLIT_ROWS + extra && rows.len() >= 2 * MIN_SPLIT_ROWS;
        if !at_top && !here {
            sim.end_page(out);
            continue;
        }

        // Cut at a row, both parts keeping `MIN_SPLIT_ROWS`.
        let take = if at_top { o.body_lines } else { room };
        let take = take.saturating_sub(extra);
        let take = take.min(rows.len().saturating_sub(MIN_SPLIT_ROWS)).max(1);
        let mut first = rows.clone();
        let mut rest = first.split_off(take);
        if let Some(cues) = cue_row(&rows) {
            first.push(marked(cues, PageKind::More, &format!("({})", o.words.more)));
            rest.insert(0, marked(cues, PageKind::Character, o.words.contd));
        }
        sim.put(out, &block, before, first);
        sim.end_page(out);
        block = Block::from_rows(rest);
    }
}

/// The name row of a dialogue: its first row, if it holds a name (two for a dual dialogue).
fn cue_row(rows: &[Vec<Line>]) -> Option<&Vec<Line>> {
    rows.first()
        .filter(|r| r.iter().any(|l| l.kind == PageKind::Character))
}

/// A row from the names of `cues`: `(MORE)` under each, or the name again with `(CONT'D)`.
fn marked(cues: &[Line], kind: PageKind, word: &str) -> Vec<Line> {
    cues.iter()
        .filter(|c| c.kind == PageKind::Character)
        .map(|c| {
            let mut text = c.text.clone();
            let mark = format!(" ({word})");
            if kind == PageKind::Character && !text.ends_with(&mark) {
                text.push_str(&mark);
            }
            if kind == PageKind::More {
                text = word.to_owned();
            }

            Line {
                text,
                kind,
                offset: 0,
                number: None,
                added: true,
                ..c.clone()
            }
        })
        .collect()
}

struct Flow<'a> {
    options: &'a PageOptions,
    sim: Sim,
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
        sim: Sim {
            used: 0,
            last_heading: false,
            only_headings: true,
            bad: false,
            ended: 0,
        },
        pages: Vec::new(),
        lines: Vec::new(),
        used: 0,
        scenes: Vec::new(),
        total: 0,
    };

    for (i, block) in blocks.iter().enumerate() {
        let chain_start = block.heading && (i == 0 || !blocks[i - 1].heading);
        let mut steps = Vec::new();
        if chain_start && flow.sim.used > 0 && flow.chain_ends_a_page(&blocks[i..]) {
            flow.sim.end_page(&mut steps);
        }

        plan(&mut flow.sim, options, block.clone(), &mut steps);
        flow.apply(steps);
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
    /// Places the chain of headings at the start of `rest` and the block after it on a copy
    /// of the position, up to the end of the current page, and tells whether a heading
    /// would end a page. A forced page break or the end of the script is no such end.
    fn chain_ends_a_page(&self, rest: &[Block]) -> bool {
        let mut sim = Sim {
            bad: false,
            ended: 0,
            ..self.sim.clone()
        };
        let mut sink = Vec::new();

        for block in rest {
            if block.page_break {
                break;
            }

            plan(&mut sim, self.options, block.clone(), &mut sink);
            sink.clear();
            if !block.heading || sim.ended > 0 {
                break;
            }
        }
        sim.bad
    }

    fn apply(&mut self, steps: Vec<Step>) {
        for step in steps {
            match step {
                Step::EndPage => self.end_page(),
                Step::Put {
                    heading,
                    element,
                    before,
                    rows,
                } => self.put(heading, element, before, rows),
            }
        }
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

    fn put(&mut self, heading: bool, element: usize, before: usize, rows: Vec<Vec<Line>>) {
        self.used += before;
        if heading {
            self.scenes.push(SceneLength {
                element,
                page: self.pages.len() + 1,
                row: self.used,
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
