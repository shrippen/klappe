//! The selection after a copy (LH-08).

use klappe_services::Source;

use super::{added, Selection};

/// The selection after a copy: what the plan shows anew, except marks. A copied mark stands
/// right behind its original, which the indices do not show: the marks selected before, each
/// moved on behind the copies in front of it. A copied figure's marks are not selected.
pub fn copied(selected: &Selection, before: &[Source], after: &[Source]) -> Selection {
    let mut found = added(before, after);
    found.0.retain(|s| !matches!(s, Source::Mark { .. }));

    let mut marks: Vec<(&str, usize)> = selected
        .0
        .iter()
        .filter_map(|s| match s {
            Source::Mark { character, index } => Some((character.as_str(), *index)),
            _ => None,
        })
        .collect();
    marks.sort();

    let mut rank = 0;
    let mut last = "";
    for (character, index) in marks {
        rank = if character == last { rank + 1 } else { 1 };
        last = character;
        found.0.push(Source::Mark {
            character: character.to_owned(),
            index: index + rank,
        });
    }

    found
}

#[cfg(test)]
mod tests {
    use super::super::Pick;
    use super::*;

    fn mark(character: &str, index: usize) -> Source {
        Source::Mark {
            character: character.into(),
            index,
        }
    }

    /// A copied mark stands right behind its original: the marks that are selected move on by
    /// the copies before them; marks of a copied figure are not selected.
    #[test]
    fn a_copied_mark_is_selected_behind_its_original() {
        let marks = |n: usize| (0..n).map(|i| mark("e", i)).collect::<Vec<_>>();
        let selected = Selection::default().boxed(vec![mark("e", 0), mark("e", 2)], Pick::Replace);

        let copies = copied(&selected, &marks(4), &marks(6));

        assert_eq!(copies.sources(), [mark("e", 1), mark("e", 4)]);

        let mut after = marks(4);
        after.extend([Source::Figure("e-2".into()), mark("e-2", 0)]);
        let figure = Selection::of(Some(Source::Figure("e".into())));

        assert_eq!(
            copied(&figure, &marks(4), &after).sources(),
            [Source::Figure("e-2".into())]
        );
    }
}
