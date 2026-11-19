//! Which scene of the book a register entry is. The number is not enough: a book that was
//! never numbered gives every entry the number `""`, and two scenes may carry the same one.

use klappe_model::Kind;
use klappe_script::Element;

use super::Session;
use crate::book::{self, Plan};

impl Session {
    /// Works out which scene of the book each entry of the register is and where its heading
    /// stands, and keeps both (`book_scene`, `heading_line`). Run where the book or the register
    /// moves (with the open questions, `refresh_pending`), never on a refresh of the window.
    /// `plan` is the alignment of the current state if the caller has it; without one it is run
    /// only when an entry cannot be told by its number.
    pub(super) fn refresh_scene_map(&mut self, plan: Option<&Plan>) {
        let scenes = self.script.scenes();
        let entries = &self.project.scenes;
        let alone = |number: &str| {
            let mine = entries.iter().filter(|e| e.number == number).count();
            let found: Vec<usize> = (0..scenes.len())
                .filter(|&n| scenes[n].number == Some(number))
                .collect();

            (mine == 1 && found.len() == 1).then(|| found[0])
        };
        let by_number: Vec<Option<usize>> = entries
            .iter()
            .map(|e| match e.kind {
                Kind::Scene if !e.number.is_empty() => alone(&e.number),
                _ => None,
            })
            .collect();
        let needs_plan = !scenes.is_empty()
            && entries
                .iter()
                .zip(&by_number)
                .any(|(e, n)| e.kind == Kind::Scene && n.is_none());

        let owned;
        let plan = match plan {
            Some(plan) => Some(plan),
            None if needs_plan => {
                owned = book::plan(&self.project, &self.script);
                Some(&owned)
            }
            None => None,
        };
        let at: Vec<Option<usize>> = by_number
            .into_iter()
            .enumerate()
            .map(|(i, n)| n.or_else(|| plan.and_then(|p| p.scene_of(i))))
            .collect();

        // The k-th heading of the text is the k-th scene that has one; text before the first
        // heading is a scene without a heading and counts no line.
        let headings: Vec<usize> = klappe_script::parse_located(self.script_text())
            .iter()
            .filter(|l| matches!(l.element, Element::SceneHeading { .. }))
            .filter_map(|l| l.lines.first().copied())
            .collect();
        let line = at.iter().map(|scene| {
            let n = (*scene)?;
            scenes[n].heading?;
            let before = scenes[..n].iter().filter(|s| s.heading.is_some()).count();

            headings.get(before).copied()
        });

        self.heading_line = line.collect();
        self.book_scene = at;
    }

    /// The scene of the book (position among `Script::scenes`) that the entry at `entry` in
    /// `Project::scenes` is, as last worked out.
    pub(super) fn book_scene_of(&self, entry: usize) -> Option<usize> {
        self.book_scene.get(entry).copied().flatten()
    }

    /// The heading line of each entry, in the order of `Project::scenes`; short or empty
    /// when the entries have not been mapped yet.
    pub(super) fn heading_lines(&self) -> Vec<Option<usize>> {
        let mut lines = self.heading_line.clone();
        lines.resize(self.project.scenes.len(), None);

        lines
    }
}
