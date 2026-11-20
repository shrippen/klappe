//! Capitals as the writer types (DB-01): a scene heading and a cue forced with `@` turn to
//! capitals on the line being typed, in every editor.

use crate::flow::upper;
use crate::parse::{FORCE_CHARACTER, NUMBER_MARK};
use crate::{text_kinds, Slot, TextKind};

/// Characters that mark parts of a line the writer types as is.
const EXTENSION_OPEN: char = '(';

/// Uppercases the part of `line` before the first `cut`, `rest` stays as typed.
fn upper_before(line: &str, cut: usize) -> String {
    let (head, rest) = line.split_at(cut);

    upper(head) + rest
}

/// Line `at` of `text` in capitals, when it is a scene heading or a cue forced with `@` and
/// has lowercase letters left. A heading keeps its `#12a#` number as typed (also half typed),
/// a cue its extension, also while it is still open: `@anna (cont'd)` becomes
/// `@ANNA (cont'd)`, Fountain allows both cases. An unforced cue is no cue until it is in
/// capitals, so it has nothing to fix; one that Enter or Tab made (`pending`, the element an
/// empty or plain line is meant to be) is.
pub fn typed_caps(text: &str, at: usize, pending: Option<Slot>) -> Option<String> {
    let line = text.split('\n').nth(at)?;
    let read = text_kinds(text)
        .get(at)
        .copied()
        .unwrap_or(TextKind::Action);
    let kind = match (read, pending) {
        (TextKind::Blank | TextKind::Action, Some(Slot::Heading)) => TextKind::Heading,
        (TextKind::Blank | TextKind::Action, Some(Slot::Character)) => TextKind::Character,
        (kind, _) => kind,
    };
    let trimmed = line.trim_end();

    let cut = match kind {
        // The number from its opening `#` on, also while it is still open: `#3a` of `#3a#`.
        // A `#` followed by a space starts no number (as in the parser): `#2 flur` is text.
        TextKind::Heading => {
            let body = trimmed.strip_suffix(NUMBER_MARK).unwrap_or(trimmed);
            match body.rfind(NUMBER_MARK) {
                Some(at) if !body[at..].contains(char::is_whitespace) => at,
                _ => trimmed.len(),
            }
        }
        // A cue of a pending line and a forced one are capitals up to the extension.
        TextKind::Character
            if pending == Some(Slot::Character)
                || line.trim_start().starts_with(FORCE_CHARACTER) =>
        {
            trimmed.find(EXTENSION_OPEN).unwrap_or(trimmed.len())
        }
        _ => return None,
    };

    let caps = upper_before(line, cut);

    (caps != line).then_some(caps)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn caps(text: &str, at: usize) -> Option<String> {
        typed_caps(text, at, None)
    }

    #[test]
    fn caps_for_headings_by_prefix_or_force() {
        for (typed, want) in [
            ("int. küche - tag", "INT. KÜCHE - TAG"),
            ("ext. straße", "EXT. STRAßE"),
            ("int./ext. auto", "INT./EXT. AUTO"),
            ("i/e auto", "I/E AUTO"),
            (".innen küche", ".INNEN KÜCHE"),
            ("innen. küche - tag", "INNEN. KÜCHE - TAG"),
            ("außen. hof", "AUßEN. HOF"),
            ("int. küche #3a#", "INT. KÜCHE #3a#"),
        ] {
            assert_eq!(caps(typed, 0).as_deref(), Some(want), "{typed}");
        }
    }

    #[test]
    fn caps_for_forced_cues_keep_the_extension() {
        assert_eq!(caps("@anna", 0).as_deref(), Some("@ANNA"));
        assert_eq!(caps("@anna (cont'd)", 0).as_deref(), Some("@ANNA (cont'd)"));
        assert_eq!(caps("@elisabeth ^", 0).as_deref(), Some("@ELISABETH ^"));
    }

    #[test]
    fn no_caps_for_the_rest() {
        for text in [
            "Anna geht.",
            "anna\nHallo",
            "...und weiter",
            "@ANNA",
            "INT. KÜCHE",
            "ste. etwas",
        ] {
            assert_eq!(caps(text, 0), None, "{text}");
        }
    }

    #[test]
    fn caps_need_the_blank_lines_of_a_heading() {
        assert_eq!(caps("a\n\nint. x\n\nb", 2).as_deref(), Some("INT. X"));
        assert_eq!(caps("a\nint. x\nb", 1), None);
        assert_eq!(caps("a\n\nint. x\nund mehr", 2), None);
    }

    /// A line Tab made a cue takes capitals before its speech is typed.
    #[test]
    fn a_pending_cue_takes_capitals() {
        assert_eq!(
            typed_caps("x\n\nanna (o.s.)", 2, Some(Slot::Character)).as_deref(),
            Some("ANNA (o.s.)")
        );
        assert_eq!(typed_caps("x\n\nanna", 2, Some(Slot::Action)), None);
    }
}
