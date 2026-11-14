//! What the project knows about the scenes of the script: shots and marks linked to its
//! lines, and counts per scene. Read-only; the TUI writes the script only.

use std::collections::HashMap;

use klappe_services::{Chip, Session};

/// Counts of a scene's first plan.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Facts {
    pub shots: usize,
    pub figures: usize,
}

/// Per scene number: the chips of its elements, and its counts.
#[derive(Debug, Default)]
pub(crate) struct Links {
    chips: HashMap<(String, usize), Vec<Chip>>,
    facts: HashMap<String, Facts>,
}

impl Links {
    /// Reads every scene of the project through the session, then selects the scene it
    /// had selected again.
    pub(crate) fn read(session: &mut Session) -> Self {
        let before = session.scene().map(|s| s.id.clone());
        let scenes: Vec<(String, String)> = session
            .project()
            .scenes
            .iter()
            .map(|s| (s.id.clone(), s.number.clone()))
            .collect();
        let mut links = Self::default();

        for (id, number) in scenes {
            if session.select_scene(&id).is_err() {
                continue;
            }
            links.read_scene(session, &number);
        }

        if let Some(id) = before {
            // The scene was there a moment ago; if it is gone nothing is selected to restore.
            let _ = session.select_scene(&id);
        }

        links
    }

    fn read_scene(&mut self, session: &Session, number: &str) {
        if let Some(plan) = session.plan() {
            let facts = Facts {
                shots: plan.shots.len(),
                figures: plan.figures.len(),
            };
            self.facts.insert(number.to_owned(), facts);
        }

        for line in session.script_lines() {
            if line.chips.is_empty() {
                continue;
            }
            self.chips
                .insert((number.to_owned(), line.index), line.chips);
        }
    }

    /// The chips of the element at this position of the scene.
    pub(crate) fn chips(&self, scene: &str, index: usize) -> &[Chip] {
        self.chips
            .get(&(scene.to_owned(), index))
            .map_or(&[], Vec::as_slice)
    }

    pub(crate) fn facts(&self, scene: &str) -> Option<Facts> {
        self.facts.get(scene).copied()
    }
}
