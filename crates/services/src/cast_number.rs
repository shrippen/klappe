//! The cast numbers (Besetzung): a number per character that the call sheet and schedule show
//! beside the name. The book gives them until they are locked.

use std::collections::{HashMap, HashSet};

/// Whether the numbers follow the book.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Numbers {
    /// New and unnumbered characters take the lowest numbers that are free.
    Moving,
    /// A new character takes the number after the highest one.
    Locked,
}

/// What the rule knows of a character.
pub(crate) struct Appearance<'a> {
    pub id: &'a str,
    pub name: &'a str,
    /// A group of extras has no number.
    pub extra: bool,
    /// In how many scenes of the book it is.
    pub scenes: usize,
    /// The place of its first scene in the book; `usize::MAX` for none.
    pub first: usize,
    /// The number set by hand or by locking.
    pub stored: Option<u32>,
}

/// The number of each character that has one. A stored number stays. The others, the extras
/// aside, are numbered by the number of scenes, most first; the first scene decides a tie, then
/// the name.
///
/// ```text
/// ANNA 3 scenes, WÄRTER 2 (first in 1), BOB 2 (first in 2), CHRIS 0
///   moving: ANNA 1, WÄRTER 2, BOB 3, CHRIS 4
///   BOB stored 2: ANNA 1, BOB 2, WÄRTER 3, CHRIS 4
///   locked, NEU is new: NEU 5
/// ```
pub(crate) fn cast_numbers(appearances: &[Appearance], numbers: Numbers) -> HashMap<String, u32> {
    let mut given: HashMap<String, u32> = appearances
        .iter()
        .filter_map(|a| Some((a.id.to_owned(), a.stored?)))
        .collect();

    let mut open: Vec<&Appearance> = appearances
        .iter()
        .filter(|a| a.stored.is_none() && !a.extra)
        .collect();
    open.sort_by_key(|a| (usize::MAX - a.scenes, a.first, a.name.to_lowercase()));

    let used: HashSet<u32> = given.values().copied().collect();
    let highest = used.iter().copied().max().unwrap_or(0);
    let mut free = (1..).filter(|n| !used.contains(n));
    let mut after = highest;

    for appearance in open {
        let number = match numbers {
            Numbers::Moving => free.next().unwrap_or(highest),
            Numbers::Locked => {
                after += 1;
                after
            }
        };
        given.insert(appearance.id.to_owned(), number);
    }

    given
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seen<'a>(id: &'a str, scenes: usize, first: usize) -> Appearance<'a> {
        Appearance {
            id,
            name: id,
            extra: false,
            scenes,
            first,
            stored: None,
        }
    }

    fn of(numbers: &HashMap<String, u32>, id: &str) -> Option<u32> {
        numbers.get(id).copied()
    }

    /// Most scenes first; a tie goes to the earlier first scene, then to the name.
    #[test]
    fn the_scenes_decide_the_order() {
        let all = [
            seen("bob", 2, 1),
            seen("anna", 3, 0),
            seen("waerter", 2, 0),
            seen("dora", 2, 1),
            seen("chris", 0, usize::MAX),
        ];
        let numbers = cast_numbers(&all, Numbers::Moving);

        assert_eq!(of(&numbers, "anna"), Some(1));
        assert_eq!(of(&numbers, "waerter"), Some(2));
        assert_eq!(of(&numbers, "bob"), Some(3));
        assert_eq!(of(&numbers, "dora"), Some(4));
        assert_eq!(of(&numbers, "chris"), Some(5));
    }

    #[test]
    fn extras_have_no_number() {
        let crowd = Appearance {
            extra: true,
            ..seen("crowd", 9, 0)
        };
        let numbers = cast_numbers(&[crowd, seen("anna", 1, 0)], Numbers::Moving);

        assert_eq!(of(&numbers, "crowd"), None);
        assert_eq!(of(&numbers, "anna"), Some(1));
    }

    /// A stored number stays, and the others take the lowest numbers left.
    #[test]
    fn a_stored_number_stays() {
        let bob = Appearance {
            stored: Some(2),
            ..seen("bob", 1, 2)
        };
        let numbers = cast_numbers(
            &[seen("anna", 3, 0), bob, seen("carl", 2, 1)],
            Numbers::Moving,
        );

        assert_eq!(of(&numbers, "anna"), Some(1));
        assert_eq!(of(&numbers, "bob"), Some(2));
        assert_eq!(of(&numbers, "carl"), Some(3));
    }

    /// An extra with a stored number keeps it.
    #[test]
    fn an_extra_may_hold_a_number() {
        let crowd = Appearance {
            extra: true,
            stored: Some(7),
            ..seen("crowd", 1, 0)
        };
        let numbers = cast_numbers(&[crowd], Numbers::Moving);

        assert_eq!(of(&numbers, "crowd"), Some(7));
    }

    /// Locked, a new character takes the number after the highest, however few its scenes.
    #[test]
    fn locked_numbers_count_on() {
        let stored = |id, n| Appearance {
            stored: Some(n),
            ..seen(id, 1, 0)
        };
        let all = [
            stored("anna", 1),
            stored("bob", 5),
            seen("new", 9, 0),
            seen("newer", 1, 4),
        ];
        let numbers = cast_numbers(&all, Numbers::Locked);

        assert_eq!(of(&numbers, "new"), Some(6));
        assert_eq!(of(&numbers, "newer"), Some(7));
    }
}
