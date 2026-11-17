//! Pairs equal tokens of two sequences that most likely are the same token (patience
//! diff, extended).
//!
//! 1. A token that occurs exactly once in each sequence is paired, wherever it stands.
//! 2. Take two consecutive pairs of `old` (the start and the end of the sequences count
//!    as pairs) whose partners in `new` are consecutive pairs too, in the same direction.
//!    If the stretches between them are equally long and identical, pair them position by
//!    position. This is what pairs lines that occur more than once (a "Ja." or the cue
//!    ANNA): an equal line is paired only by being in the same place between two certain
//!    ones, never by being equal alone.
//!
//! Everything else stays unpaired.

use std::collections::HashMap;

/// For every token of `old` the index of its partner in `new`.
pub(crate) fn pair(old: &[u32], new: &[u32]) -> Vec<Option<usize>> {
    let mut out = vec![None; old.len()];
    let old_once = once(old);
    let new_once = once(new);

    for (token, &o) in &old_once {
        if let Some(&n) = new_once.get(token) {
            out[o] = Some(n);
        }
    }

    stretches(old, new, &mut out);
    out
}

/// The index of every token that occurs exactly once.
fn once(tokens: &[u32]) -> HashMap<u32, usize> {
    let mut seen: HashMap<u32, Option<usize>> = HashMap::new();
    for (i, &t) in tokens.iter().enumerate() {
        seen.entry(t).and_modify(|e| *e = None).or_insert(Some(i));
    }
    seen.into_iter()
        .filter_map(|(t, i)| Some((t, i?)))
        .collect()
}

/// Rule 2.
fn stretches(old: &[u32], new: &[u32], out: &mut [Option<usize>]) {
    // The sentinels stand just outside both sequences.
    let mut anchors: Vec<(isize, isize)> = vec![(-1, -1)];
    anchors.extend(
        out.iter()
            .enumerate()
            .filter_map(|(o, n)| n.map(|n| (o as isize, n as isize))),
    );
    anchors.push((old.len() as isize, new.len() as isize));

    let mut by_new: Vec<isize> = anchors.iter().map(|a| a.1).collect();
    by_new.sort_unstable();
    let rank = |n: isize| by_new.binary_search(&n).unwrap_or_default();

    for w in anchors.windows(2) {
        let ((o1, n1), (o2, n2)) = (w[0], w[1]);
        let consecutive = n1 < n2 && rank(n2) == rank(n1) + 1;
        if !consecutive || o2 - o1 != n2 - n1 {
            continue;
        }

        let len = (o2 - o1 - 1) as usize;
        let (o, n) = ((o1 + 1) as usize, (n1 + 1) as usize);
        if old[o..o + len] != new[n..n + len] {
            continue;
        }
        for k in 0..len {
            out[o + k] = Some(n + k);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::pair;

    #[test]
    fn a_token_unique_on_both_sides_is_paired_wherever_it_stands() {
        assert_eq!(
            pair(&[1, 2, 3], &[3, 1, 2]),
            vec![Some(1), Some(2), Some(0)]
        );
    }

    #[test]
    fn a_token_that_repeats_is_not_paired_alone() {
        assert_eq!(pair(&[7, 1], &[7, 7, 1]), vec![None, Some(2)]);
    }

    #[test]
    fn equal_stretches_between_pairs_are_paired_by_position() {
        // 9 and 8 are unique; the repeated 5 5 between them is in the same place.
        let paired = pair(&[9, 5, 5, 8], &[9, 5, 5, 8]);

        assert_eq!(paired, vec![Some(0), Some(1), Some(2), Some(3)]);
    }

    #[test]
    fn stretches_of_different_length_stay_unpaired() {
        assert_eq!(
            pair(&[9, 5, 8], &[9, 5, 5, 8]),
            vec![Some(0), None, Some(3)]
        );
    }

    #[test]
    fn the_sequence_ends_count_as_pairs() {
        assert_eq!(pair(&[5, 5], &[5, 5]), vec![Some(0), Some(1)]);
    }

    #[test]
    fn crossing_pairs_pair_nothing_between() {
        assert_eq!(
            pair(&[1, 5, 5, 2], &[2, 5, 5, 1]),
            vec![Some(3), None, None, Some(0)]
        );
    }
}
