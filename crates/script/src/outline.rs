//! The outline of a screenplay (DB-20, DB-21): sections, scenes and synopses as a tree, and
//! moving a scene or a section.
//!
//! ```text
//! # Akt I                 Section(1)   depth 0 ─┐ block of the section
//! ## Ankunft              Section(2)   depth 1  │ runs to the next section of
//! INT. SAAL - TAG         Scene        depth 2  │ the same or a higher level
//! = Sie kommt an.         Synopsis     depth 3  │
//! ```

use std::ops::Range;

use crate::{Element, Located};

/// What an outline entry is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    /// `#` is level 1 (act), `##` level 2 (sequence), and so on.
    Section(usize),
    Scene,
    Synopsis,
}

/// One line of the outline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub kind: EntryKind,
    /// How deep the entry sits in the tree: 0 on top. A scene is a child of the section it
    /// stands in, a synopsis of the scene or section above it.
    pub depth: usize,
    /// The text line (from 0) the entry starts at.
    pub line: usize,
    /// The line after its block. A section's block holds its scenes and subsections, a
    /// scene's runs to the next scene or section.
    pub end: usize,
    /// Position among the scenes, from 0; `None` for sections and synopses.
    pub scene: Option<usize>,
    /// The `#12#` number of a scene.
    pub number: Option<String>,
    /// Heading without number, section title or synopsis.
    pub text: String,
}

impl Entry {
    fn is_unit(&self) -> bool {
        self.kind != EntryKind::Synopsis
    }
}

/// Which way to move.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Way {
    Up,
    Down,
}

/// New text for the lines `from..to`; one edit for the surface to apply.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Moved {
    pub from: usize,
    pub to: usize,
    pub lines: Vec<String>,
    /// Where the moved entry starts now.
    pub line: usize,
}

/// The outline of parsed text with `line_count` lines.
pub fn outline_of(located: &[Located], line_count: usize) -> Vec<Entry> {
    let mut entries: Vec<Entry> = Vec::new();
    let mut scenes = 0;
    // Source levels of the sections still open.
    let mut open: Vec<usize> = Vec::new();

    for item in located {
        let Some(&line) = item.lines.first() else {
            continue;
        };
        let end = item.lines.last().map_or(line + 1, |l| l + 1);
        let entry = match &item.element {
            Element::Section { depth, text } => {
                open.retain(|level| level < depth);
                open.push(*depth);
                entry(EntryKind::Section(*depth), open.len() - 1, line, end, text)
            }
            Element::SceneHeading { text, number } => {
                let mut e = entry(EntryKind::Scene, open.len(), line, end, text);
                e.scene = Some(scenes);
                e.number = number.clone();
                scenes += 1;
                e
            }
            Element::Synopsis(text) => {
                let depth = entries.last().map_or(0, |p| match p.kind {
                    EntryKind::Synopsis => p.depth,
                    _ => p.depth + 1,
                });
                entry(EntryKind::Synopsis, depth, line, end, text)
            }
            _ => continue,
        };
        entries.push(entry);
    }

    close_blocks(&mut entries, line_count);
    entries
}

/// The outline of Fountain text.
pub fn outline(text: &str) -> Vec<Entry> {
    outline_of(&crate::parse_located(text), text.lines().count())
}

fn entry(kind: EntryKind, depth: usize, line: usize, end: usize, text: &str) -> Entry {
    Entry {
        kind,
        depth,
        line,
        end,
        scene: None,
        number: None,
        text: text.to_string(),
    }
}

/// Sets the end of each scene and section block.
fn close_blocks(entries: &mut [Entry], line_count: usize) {
    for i in 0..entries.len() {
        let next = entries[i + 1..].iter().find(|e| match entries[i].kind {
            EntryKind::Section(level) => matches!(e.kind, EntryKind::Section(l) if l <= level),
            EntryKind::Scene => e.is_unit(),
            EntryKind::Synopsis => false,
        });
        match entries[i].kind {
            EntryKind::Synopsis => {}
            _ => entries[i].end = next.map_or(line_count, |n| n.line),
        }
    }
}

/// Indices of the entries to show. A collapsed section hides what is inside it. A `filter`
/// keeps the entries whose text or number contains it (any case) and the sections around
/// them, and ignores what is collapsed.
pub fn visible(entries: &[Entry], collapsed: &dyn Fn(&Entry) -> bool, filter: &str) -> Vec<usize> {
    let filter = filter.to_lowercase();
    if !filter.is_empty() {
        let hit = |e: &Entry| {
            e.text.to_lowercase().contains(&filter)
                || e.number
                    .as_deref()
                    .is_some_and(|n| n.to_lowercase().contains(&filter))
        };
        return (0..entries.len())
            .filter(|&i| {
                let e = &entries[i];
                hit(e) || (matches!(e.kind, EntryKind::Section(_)) && inside_hit(entries, i, &hit))
            })
            .collect();
    }

    let mut shown = Vec::new();
    let mut hidden_to = 0;
    for (i, e) in entries.iter().enumerate() {
        if e.line < hidden_to {
            continue;
        }
        shown.push(i);
        if matches!(e.kind, EntryKind::Section(_)) && collapsed(e) {
            hidden_to = e.end;
        }
    }

    shown
}

fn inside_hit(entries: &[Entry], section: usize, hit: &dyn Fn(&Entry) -> bool) -> bool {
    let end = entries[section].end;

    entries[section + 1..]
        .iter()
        .take_while(|e| e.line < end)
        .any(hit)
}

/// Moves the scene or section `index` one place: a scene past its neighbour scene or section
/// heading, a section past the section of its own level next to it. The text of the block
/// goes with it; scene numbers stay as they are. `None` where it cannot move (first or last
/// place, a synopsis).
pub fn move_entry(lines: &[String], entries: &[Entry], index: usize, way: Way) -> Option<Moved> {
    let entry = entries.get(index)?;
    let own = entry.line..entry.end;
    let other = match (entry.kind, way) {
        (EntryKind::Synopsis, _) => return None,
        (EntryKind::Scene, _) => unit_next_to(entries, index, way)?,
        (EntryKind::Section(level), _) => sibling(entries, index, level, way)?,
    };

    let (first, second) = match way {
        Way::Up => (other, own),
        Way::Down => (own, other),
    };

    Some(swap(lines, first, second, way))
}

/// The block of the scene or section heading before or after `index`.
fn unit_next_to(entries: &[Entry], index: usize, way: Way) -> Option<Range<usize>> {
    let mut units = entries.iter().enumerate().filter(|(_, e)| e.is_unit());
    let at = units.clone().position(|(i, _)| i == index)?;
    let near = match way {
        Way::Up => at.checked_sub(1)?,
        Way::Down => at + 1,
    };
    let (_, e) = units.nth(near)?;
    // A section as neighbour counts by its heading alone: a scene steps into or out of it.
    let end = match e.kind {
        EntryKind::Section(_) => entries[..]
            .iter()
            .find(|n| n.is_unit() && n.line > e.line)
            .map_or(e.end, |n| n.line),
        _ => e.end,
    };

    Some(e.line..end)
}

/// The block of the section of `level` that follows or precedes, inside the same parent.
fn sibling(entries: &[Entry], index: usize, level: usize, way: Way) -> Option<Range<usize>> {
    let level_of = |e: &Entry| match e.kind {
        EntryKind::Section(l) => Some(l),
        _ => None,
    };
    let found = match way {
        Way::Up => entries[..index]
            .iter()
            .rev()
            .filter_map(|e| Some((e, level_of(e)?)))
            .find(|(_, l)| *l <= level),
        Way::Down => entries[index + 1..]
            .iter()
            .filter_map(|e| Some((e, level_of(e)?)))
            .find(|(_, l)| *l <= level),
    };
    let (e, l) = found?;

    (l == level).then_some(e.line..e.end)
}

/// Swaps two blocks that touch. Blank lines after a block stay with its place, so the
/// gaps between blocks do not change.
fn swap(lines: &[String], first: Range<usize>, second: Range<usize>, way: Way) -> Moved {
    let core = |r: &Range<usize>| {
        let mut end = r.end;
        while end > r.start && lines[end - 1].trim().is_empty() {
            end -= 1;
        }
        r.start..end
    };
    let (a, b) = (core(&first), core(&second));
    let gap = first.end - a.end;
    let tail = second.end - b.end;

    // A heading only reads as one after a blank line, and the block after the pair needs
    // one before it: a move must not turn a scene into action.
    let lead = usize::from(first.start > 0 && !lines[first.start - 1].trim().is_empty());
    let after =
        usize::from(tail == 0 && lines.get(second.end).is_some_and(|l| !l.trim().is_empty()));

    let mut out: Vec<String> = vec![String::new(); lead];
    out.extend_from_slice(&lines[b.clone()]);
    let a_start = first.start + out.len() + gap.max(1);
    out.extend(std::iter::repeat_n(String::new(), gap.max(1)));
    out.extend_from_slice(&lines[a.clone()]);
    out.extend(std::iter::repeat_n(String::new(), tail.max(after)));

    Moved {
        from: first.start,
        to: second.end,
        lines: out,
        line: match way {
            Way::Up => first.start + lead,
            Way::Down => a_start,
        },
    }
}
