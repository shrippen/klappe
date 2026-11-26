//! Longest common subsequence of lines and longest rising run, for `compare`.

/// The indices of `old` and `new` that no longest common subsequence pairs: what a line
/// diff shows as removed (`old`) and added (`new`).
pub(crate) fn unpaired(old: &[String], new: &[String]) -> (Vec<usize>, Vec<usize>) {
    // Equal ends need no table: most edits are in the middle.
    let head = old.iter().zip(new).take_while(|(a, b)| a == b).count();
    let tail = old[head..]
        .iter()
        .rev()
        .zip(new[head..].iter().rev())
        .take_while(|(a, b)| a == b)
        .count();
    let (mid_old, mid_new) = (&old[head..old.len() - tail], &new[head..new.len() - tail]);
    let (n, m) = (mid_old.len(), mid_new.len());

    // `len[i][j]`: the common length of `mid_old[i..]` and `mid_new[j..]`.
    let width = m + 1;
    let mut len = vec![0u32; (n + 1) * width];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            len[i * width + j] = if mid_old[i] == mid_new[j] {
                len[(i + 1) * width + j + 1] + 1
            } else {
                len[(i + 1) * width + j].max(len[i * width + j + 1])
            };
        }
    }

    let (mut removed, mut added) = (Vec::new(), Vec::new());
    let (mut i, mut j) = (0, 0);
    while i < n && j < m {
        if mid_old[i] == mid_new[j] {
            i += 1;
            j += 1;
        } else if len[(i + 1) * width + j] >= len[i * width + j + 1] {
            removed.push(head + i);
            i += 1;
        } else {
            added.push(head + j);
            j += 1;
        }
    }
    removed.extend((i..n).map(|k| head + k));
    added.extend((j..m).map(|k| head + k));

    (removed, added)
}

/// Per element of `seq`: whether it is in a longest strictly rising subsequence. The first
/// of several equally long ones.
pub(crate) fn in_rising(seq: &[usize]) -> Vec<bool> {
    let mut len = vec![1usize; seq.len()];
    let mut prev: Vec<Option<usize>> = vec![None; seq.len()];

    for i in 0..seq.len() {
        for j in 0..i {
            if seq[j] < seq[i] && len[j] + 1 > len[i] {
                len[i] = len[j] + 1;
                prev[i] = Some(j);
            }
        }
    }

    let mut inside = vec![false; seq.len()];
    let mut at = (0..seq.len()).reduce(|best, i| if len[i] > len[best] { i } else { best });
    while let Some(i) = at {
        inside[i] = true;
        at = prev[i];
    }

    inside
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(text: &str) -> Vec<String> {
        text.split_whitespace().map(str::to_owned).collect()
    }

    #[test]
    fn equal_lines_pair_up() {
        let a = lines("a b c");

        assert_eq!(unpaired(&a, &a), (vec![], vec![]));
    }

    #[test]
    fn an_insertion_and_a_deletion_are_found() {
        let (removed, added) = unpaired(&lines("a b c d"), &lines("a x c d e"));

        assert_eq!(removed, vec![1]);
        assert_eq!(added, vec![1, 4]);
    }

    #[test]
    fn either_side_may_be_empty() {
        assert_eq!(unpaired(&[], &lines("a b")), (vec![], vec![0, 1]));
        assert_eq!(unpaired(&lines("a b"), &[]), (vec![0, 1], vec![]));
    }

    #[test]
    fn the_longest_rising_run_stays() {
        assert_eq!(in_rising(&[1, 2, 3, 0]), vec![true, true, true, false]);
        assert_eq!(in_rising(&[3, 0, 1, 2]), vec![false, true, true, true]);
        assert_eq!(in_rising(&[]), Vec::<bool>::new());
    }
}
