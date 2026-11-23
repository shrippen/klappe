//! The text of a scene as the scene sheet shows it for marking (D-23): its lines, where a
//! mark stands in them and whether a name occurs.
//!
//! A mark is a quote: the words and which of their occurrences in the scene it is, counted
//! line by line in reading order. A mark never spans two lines.
//!
//!   line 0  "Ein Stapel Seiten. Nebel."          "Nebel" #0 → line 0, byte 19
//!   line 1  "Nebel kriecht herein."              "Nebel" #1 → line 1, byte 0

use klappe_model::TextMark;
use klappe_script::Element as Line;

/// A line of the scene's text: its kind (`action`, `cue` …) and what it says.
pub(crate) struct Shown {
    pub kind: &'static str,
    pub text: String,
}

/// The lines of a scene worth marking in: no notes, sections, synopses or page breaks. An
/// element of several lines gives one line each.
pub(crate) fn shown(scene: &[Line]) -> Vec<Shown> {
    let mut lines = Vec::new();

    for element in scene {
        let (kind, text) = match element {
            Line::Action(t) => ("action", t.clone()),
            Line::Character {
                name, extension, ..
            } => match extension {
                Some(x) => ("cue", format!("{name} ({x})")),
                None => ("cue", name.clone()),
            },
            Line::Parenthetical(t) => ("parenthetical", format!("({t})")),
            Line::Dialogue(t) => ("dialogue", t.clone()),
            Line::Transition(t) => ("transition", t.clone()),
            Line::Centered(t) => ("centered", t.clone()),
            Line::Lyrics(t) => ("lyrics", t.clone()),
            _ => continue,
        };
        lines.extend(text.split('\n').map(|t| Shown {
            kind,
            text: t.to_owned(),
        }));
    }

    lines
}

/// Where the mark stands: the line and the byte its words start at. `None`: the text no
/// longer has that occurrence.
pub(crate) fn locate(lines: &[Shown], mark: &TextMark) -> Option<(usize, usize)> {
    if mark.text.is_empty() {
        return None;
    }

    lines
        .iter()
        .enumerate()
        .flat_map(|(i, l)| l.text.match_indices(&mark.text).map(move |(at, _)| (i, at)))
        .nth(mark.occurrence)
}

/// Which occurrence the words `text` are that stand in line `line` right after `prefix`.
/// `None`: they do not stand there, or they are blank.
pub(crate) fn occurrence(lines: &[Shown], line: usize, prefix: &str, text: &str) -> Option<usize> {
    let shown = &lines.get(line)?.text;
    let at = prefix.len();
    let fits = shown.starts_with(prefix) && shown[at..].starts_with(text);
    if !fits || text.trim().is_empty() {
        return None;
    }

    let before: usize = lines[..line]
        .iter()
        .map(|l| l.text.matches(text).count())
        .sum();
    let here = shown
        .match_indices(text)
        .take_while(|(i, _)| *i < at)
        .count();

    Some(before + here)
}

/// Whether `name` stands in the lines as a word or words of its own, the case aside:
/// `Nebel` in „Nebel kriecht“, not in „Nebelmaschine“.
pub(crate) fn names(lines: &[Shown], name: &str) -> bool {
    let name = name.trim().to_lowercase();
    if name.is_empty() {
        return false;
    }

    lines.iter().any(|l| {
        let text = l.text.to_lowercase();
        text.match_indices(&name).any(|(at, _)| {
            let before = text[..at].chars().next_back();
            let after = text[at + name.len()..].chars().next();
            !before.is_some_and(char::is_alphanumeric) && !after.is_some_and(char::is_alphanumeric)
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(texts: &[&str]) -> Vec<Shown> {
        texts
            .iter()
            .map(|t| Shown {
                kind: "action",
                text: (*t).to_owned(),
            })
            .collect()
    }

    fn mark(text: &str, occurrence: usize) -> TextMark {
        TextMark {
            element: "e".into(),
            text: text.into(),
            occurrence,
        }
    }

    #[test]
    fn the_shown_lines_skip_notes_and_split_actions() {
        let scene = vec![
            Line::Action("Nebel.\nEin Stapel Seiten.".into()),
            Line::Note("Licht prüfen".into()),
            Line::Character {
                name: "ANNA".into(),
                extension: Some("V.O.".into()),
                dual: false,
            },
            Line::Parenthetical("leise".into()),
            Line::Dialogue("Wer da?".into()),
        ];

        let shown = shown(&scene);
        let texts: Vec<(&str, &str)> = shown.iter().map(|l| (l.kind, l.text.as_str())).collect();

        assert_eq!(
            texts,
            [
                ("action", "Nebel."),
                ("action", "Ein Stapel Seiten."),
                ("cue", "ANNA (V.O.)"),
                ("parenthetical", "(leise)"),
                ("dialogue", "Wer da?"),
            ]
        );
    }

    /// Occurrences count over the lines in reading order; one beyond the last is lost.
    #[test]
    fn a_mark_is_its_occurrence_in_reading_order() {
        let lines = lines(&["Ein Nebel. Nebel.", "Nebel kriecht."]);

        assert_eq!(locate(&lines, &mark("Nebel", 0)), Some((0, 4)));
        assert_eq!(locate(&lines, &mark("Nebel", 1)), Some((0, 11)));
        assert_eq!(locate(&lines, &mark("Nebel", 2)), Some((1, 0)));
        assert_eq!(locate(&lines, &mark("Nebel", 3)), None);
        assert_eq!(locate(&lines, &mark("", 0)), None);
    }

    /// The occurrence of a selection is the one `locate` finds again.
    #[test]
    fn a_selection_gives_the_occurrence_locate_finds() {
        let lines = lines(&["Ein Nebel. Nebel.", "Nebel kriecht."]);

        assert_eq!(occurrence(&lines, 0, "Ein ", "Nebel"), Some(0));
        assert_eq!(occurrence(&lines, 0, "Ein Nebel. ", "Nebel"), Some(1));
        assert_eq!(occurrence(&lines, 1, "", "Nebel"), Some(2));
        assert_eq!(locate(&lines, &mark("Nebel", 2)), Some((1, 0)));
    }

    #[test]
    fn a_selection_that_is_not_in_the_line_is_refused() {
        let lines = lines(&["Ein Nebel."]);

        assert_eq!(occurrence(&lines, 0, "Ein ", "Stapel"), None);
        assert_eq!(occurrence(&lines, 0, "Zwei ", "Nebel"), None);
        assert_eq!(occurrence(&lines, 0, "Ein", " "), None);
        assert_eq!(occurrence(&lines, 1, "", "Nebel"), None);
    }

    #[test]
    fn a_name_counts_as_whole_words_in_any_case() {
        let lines = lines(&["Die Nebelmaschine läuft.", "Ein STAPEL SEITEN fällt."]);

        assert!(!names(&lines, "Nebel"));
        assert!(names(&lines, "Nebelmaschine"));
        assert!(names(&lines, "Stapel Seiten"));
        assert!(!names(&lines, "Seite"));
        assert!(!names(&lines, " "));
    }
}
