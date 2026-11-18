//! The Sperrstand against the book (DB-33): which heading holds a number several carry, and
//! what the Sperrstand of the book as it stands is.

use std::collections::HashMap;

use klappe_model::{LockedNumber, Project};
use klappe_script::is_omitted;

use super::{Fate, Plan};

/// What the Sperrstand says about the numbers the book carries more than once.
#[derive(Debug, Default)]
pub(crate) struct Holding {
    /// The headings (counted among the headings of the text) that keep their number.
    pub holders: Vec<usize>,
    /// The numbers carried twice that stay as they are: the Sperrstand knows no holder among
    /// the scenes, or a scene with the number is a candidate of an open question.
    pub twins: Vec<String>,
}

impl Plan {
    /// The holder of every number carried by several scenes: the heading `plan` matches to
    /// the Sperrstand's entry of that number. None when there is no such heading, or when a
    /// heading with the number is a candidate of an open question (it may be the holder).
    pub(crate) fn holding(&self, project: &Project, lock: &[LockedNumber]) -> Holding {
        let mut groups: HashMap<&str, Vec<usize>> = HashMap::new();
        for (n, state) in self.book.iter().enumerate() {
            let scene = state.heading.as_deref().is_some_and(|h| !is_omitted(h));
            let Some(number) = state.number.as_deref().filter(|_| scene) else {
                continue;
            };
            groups.entry(number).or_default().push(n);
        }

        let mut out = Holding::default();
        let mut twins: Vec<(usize, &str)> = Vec::new();
        for (number, members) in groups.iter().filter(|(_, m)| m.len() > 1) {
            match self.holder_of(project, lock, number, members) {
                Some(n) => out.holders.push(self.heading_index(n)),
                None => twins.push((members[0], number)),
            }
        }

        twins.sort();
        out.holders.sort_unstable();
        out.twins = twins.into_iter().map(|(_, n)| n.to_owned()).collect();

        out
    }

    /// The scene among `members`, all with `number`, that the Sperrstand's entry of the
    /// number is matched to; `None` if it is not one of them or one may be an open entry's.
    fn holder_of(
        &self,
        project: &Project,
        lock: &[LockedNumber],
        number: &str,
        members: &[usize],
    ) -> Option<usize> {
        let open = self.fates.iter().any(|(_, fate)| match fate {
            Fate::Pending(found) => found.iter().any(|n| members.contains(n)),
            _ => false,
        });
        if open {
            return None;
        }

        let known = lock
            .iter()
            .find(|l| l.number == number && !l.id.is_empty())?;
        let matched = self.fates.iter().find_map(|(entry, fate)| match fate {
            Fate::Matched(n) if project.scenes[*entry].id == known.id => Some(*n),
            _ => None,
        });

        matched.filter(|n| members.contains(n))
    }

    /// The Sperrstand of the book as it stands: every numbered heading in text order with
    /// the entry that holds it, none for a struck heading.
    pub(crate) fn numbers(&self, project: &Project) -> Vec<LockedNumber> {
        let held = |n: usize| {
            self.fates.iter().find_map(|(entry, fate)| match fate {
                Fate::Matched(scene) if *scene == n => Some(project.scenes[*entry].id.clone()),
                _ => None,
            })
        };
        let found = self.book.iter().enumerate().filter_map(|(n, state)| {
            let number = state.number.clone()?;
            let heading = state.heading.as_deref()?;
            let id = held(n).filter(|_| !is_omitted(heading)).unwrap_or_default();

            Some(LockedNumber { number, id })
        });

        found.collect()
    }
}
