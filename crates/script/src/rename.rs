//! Renaming the cues of one character in the text (DB-04). Only the name of a cue changes:
//! an extension (`(V.O.)`), the dual mark and the force mark stay, and so does everything that
//! is no cue, a name in an action line or a dialogue included.

use crate::{parse_located, Element, FORCE_CHARACTER};

/// `text` with every cue `is_called` accepts renamed to `to`, and how many cues changed. A cue
/// in capitals stays in capitals (`ANNA` → `EVA`, `@Anna` → `@Eva`). A cue that already reads
/// as `to` is no change.
pub fn rename_cues(text: &str, is_called: impl Fn(&str) -> bool, to: &str) -> (String, usize) {
    let to = to.trim();
    let mut lines: Vec<String> = text.split_inclusive('\n').map(str::to_owned).collect();
    let mut count = 0;

    for located in parse_located(text) {
        let Element::Character { name, .. } = &located.element else {
            continue;
        };
        let Some(&at) = located.lines.first() else {
            continue;
        };
        if !is_called(name) {
            continue;
        }

        // `ANNA` becomes `EVA`, `@Anna` becomes `@Eva`.
        let new = match *name == name.to_uppercase() {
            true => to.to_uppercase(),
            false => to.to_owned(),
        };
        let Some(line) = lines.get_mut(at) else {
            continue;
        };
        let Some(start) = name_start(line, name) else {
            continue;
        };
        if new == *name {
            continue;
        }

        line.replace_range(start..start + name.len(), &new);
        count += 1;
    }

    (lines.concat(), count)
}

/// Where the cue's name begins in its line: after the indent and the force mark.
fn name_start(line: &str, name: &str) -> Option<usize> {
    let indent = line.len() - line.trim_start().len();
    let rest = &line[indent..];
    let marked = rest.strip_prefix(FORCE_CHARACTER).unwrap_or(rest);
    let skipped = marked.len() - marked.trim_start().len();
    let start = indent + (rest.len() - marked.len()) + skipped;

    line[start..].starts_with(name).then_some(start)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn anna(name: &str) -> bool {
        name.eq_ignore_ascii_case("anna")
    }

    #[test]
    fn extensions_stay() {
        let text = "ANNA (V.O.)\nHallo.\n\nANNA (CONT'D)\nNoch eins.\n";
        let (out, count) = rename_cues(text, anna, "Eva");

        assert_eq!(out, "EVA (V.O.)\nHallo.\n\nEVA (CONT'D)\nNoch eins.\n");
        assert_eq!(count, 2);
    }

    #[test]
    fn dual_dialogue_keeps_its_mark() {
        let text = "BOB\nHi.\n\nANNA ^\nHo.\n";
        let (out, count) = rename_cues(text, anna, "Eva");

        assert_eq!(out, "BOB\nHi.\n\nEVA ^\nHo.\n");
        assert_eq!(count, 1);
    }

    #[test]
    fn a_forced_cue_keeps_its_force_mark_and_case() {
        let (out, count) = rename_cues("@Anna\nHallo.\n", anna, "Eva");

        assert_eq!(out, "@Eva\nHallo.\n");
        assert_eq!(count, 1);
    }

    #[test]
    fn every_name_the_figure_answers_to_is_renamed() {
        let text = "ANNA\nA.\n\nANNI\nB.\n\nBOB\nC.\n";
        let (out, count) = rename_cues(text, |n| anna(n) || n == "ANNI", "Eva");

        assert_eq!(out, "EVA\nA.\n\nEVA\nB.\n\nBOB\nC.\n");
        assert_eq!(count, 2);
    }

    #[test]
    fn prose_notes_and_the_title_page_stay() {
        let text = "Title: Anna und Bob\n\nINT. ANNA - TAG\n\nANNA geht. [[ANNA fragen]]\n\n\
                    ANNA\n(zu ANNA)\nIch bin ANNA.\n";
        let (out, count) = rename_cues(text, anna, "Eva");

        assert_eq!(out, text.replacen("\nANNA\n(zu", "\nEVA\n(zu", 1));
        assert_eq!(count, 1);
    }

    #[test]
    fn a_text_without_the_figure_is_unchanged() {
        let (out, count) = rename_cues("BOB\nHi.\n", anna, "Eva");

        assert_eq!(out, "BOB\nHi.\n");
        assert_eq!(count, 0);
    }
}
