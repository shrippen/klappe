//! What each line of the text is, from the Fountain parser, for styling and the scene list.

use klappe_services::{parse, Element};

/// Rows of text before the first paragraph, so a `Key: value` paragraph is no title page.
const PARAGRAPH_PREFIX: &str = "x\n\n";

const BONEYARD_OPEN: &str = "/*";
const BONEYARD_CLOSE: &str = "*/";

/// Display-only indents in columns (the stored text stays plain Fountain).
const INDENT_CHARACTER: usize = 20;
const INDENT_PARENTHETICAL: usize = 15;
const INDENT_DIALOGUE: usize = 10;
const INDENT_TRANSITION: usize = 30;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LineKind {
    Blank,
    Heading,
    Action,
    Character,
    Parenthetical,
    Dialogue,
    Transition,
    Centered,
    Section,
    Synopsis,
    Note,
    PageBreak,
    Lyrics,
}

impl LineKind {
    pub(crate) fn indent(self) -> usize {
        match self {
            Self::Character => INDENT_CHARACTER,
            Self::Parenthetical => INDENT_PARENTHETICAL,
            Self::Dialogue => INDENT_DIALOGUE,
            Self::Transition => INDENT_TRANSITION,
            _ => 0,
        }
    }
}

/// A scene heading in the text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Scene {
    pub line: usize,
    /// The `#12#` number; `None` when the heading has none.
    pub number: Option<String>,
    pub heading: String,
}

#[derive(Debug, Default)]
pub(crate) struct Analysis {
    pub kinds: Vec<LineKind>,
    pub scenes: Vec<Scene>,
}

impl Analysis {
    pub(crate) fn kind(&self, line: usize) -> LineKind {
        self.kinds.get(line).copied().unwrap_or(LineKind::Action)
    }

    /// Index of the scene the line belongs to: the last heading at or above it.
    pub(crate) fn scene_at(&self, line: usize) -> Option<usize> {
        self.scenes.iter().rposition(|s| s.line <= line)
    }
}

/// Classifies every line.
///
/// Each paragraph (lines up to a blank one) goes through the real parser alone, so the
/// rules stay the parser's; a paragraph is always surrounded by blank lines in the text.
pub(crate) fn analyse(lines: &[String]) -> Analysis {
    let mut out = Analysis {
        kinds: vec![LineKind::Blank; lines.len()],
        scenes: Vec::new(),
    };
    let mut start = 0;
    let mut in_boneyard = false;

    while start < lines.len() {
        if lines[start].trim().is_empty() {
            start += 1;
            continue;
        }

        let len = lines[start..]
            .iter()
            .take_while(|l| !l.trim().is_empty())
            .count();
        let para = &lines[start..start + len];
        let was_inside = in_boneyard;
        in_boneyard = boneyard_after(was_inside, para);

        // A paragraph that begins in a boneyard is not parsed: the parser would see
        // it as text. It shows as plain action and lists no scene.
        match was_inside {
            true => out.kinds[start..start + len].fill(LineKind::Action),
            false => paragraph(para, start, &mut out),
        }
        start += len;
    }

    out
}

/// Whether a boneyard `/* ... */` is still open after `lines`, given if one was open before.
fn boneyard_after(mut inside: bool, lines: &[String]) -> bool {
    let text = lines.join("\n");
    let mut rest = text.as_str();

    loop {
        let marker = if inside {
            BONEYARD_CLOSE
        } else {
            BONEYARD_OPEN
        };
        let Some(at) = rest.find(marker) else {
            return inside;
        };
        rest = &rest[at + marker.len()..];
        inside = !inside;
    }
}

/// Number of text lines an element took, and its kind.
fn kind_and_lines(el: &Element) -> (LineKind, usize) {
    match el {
        Element::SceneHeading { .. } => (LineKind::Heading, 1),
        Element::Action(t) => (LineKind::Action, t.matches('\n').count() + 1),
        Element::Character { .. } => (LineKind::Character, 1),
        Element::Parenthetical(_) => (LineKind::Parenthetical, 1),
        Element::Dialogue(t) => (LineKind::Dialogue, t.matches('\n').count() + 1),
        Element::Transition(_) => (LineKind::Transition, 1),
        Element::Centered(_) => (LineKind::Centered, 1),
        Element::Section { .. } => (LineKind::Section, 1),
        Element::Synopsis(_) => (LineKind::Synopsis, 1),
        Element::Note(_) => (LineKind::Note, 1),
        Element::PageBreak => (LineKind::PageBreak, 1),
        Element::Lyrics(_) => (LineKind::Lyrics, 1),
    }
}

fn paragraph(lines: &[String], first: usize, out: &mut Analysis) {
    let text = format!("{PARAGRAPH_PREFIX}{}", lines.join("\n"));
    let script = parse(&text);
    let mut at = first;
    let end = first + lines.len();

    for el in script.elements.iter().skip(1) {
        let (kind, count) = kind_and_lines(el);
        if let Element::SceneHeading { text, number } = el {
            out.scenes.push(Scene {
                line: at,
                number: number.clone(),
                heading: text.clone(),
            });
        }

        let stop = (at + count).min(end);
        for slot in out.kinds.iter_mut().take(stop).skip(at) {
            *slot = kind;
        }
        at = stop;
    }

    // Anything the parser did not account for (a boneyard) shows as plain action.
    for slot in out.kinds.iter_mut().take(end).skip(at) {
        *slot = LineKind::Action;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(text: &str) -> Vec<String> {
        text.split('\n').map(str::to_owned).collect()
    }

    #[test]
    fn kinds_of_a_dialogue_scene() {
        let a = analyse(&lines(
            "INT. SAAL – TAG #12#\n\nElisabeth tritt ein.\n\nELISABETH\n(leise)\nIhr kommt\nspät.\n\nCUT TO:\n",
        ));

        use LineKind::*;
        assert_eq!(
            a.kinds,
            [
                Heading,
                Blank,
                Action,
                Blank,
                Character,
                Parenthetical,
                Dialogue,
                Dialogue,
                Blank,
                Transition,
                Blank
            ]
        );
    }

    #[test]
    fn scenes_carry_number_heading_and_line() {
        let a = analyse(&lines("INT. SAAL – TAG #12#\n\ntext\n\nEXT. HOF\n\nmehr\n"));

        assert_eq!(
            a.scenes,
            [
                Scene {
                    line: 0,
                    number: Some("12".into()),
                    heading: "INT. SAAL – TAG".into()
                },
                Scene {
                    line: 4,
                    number: None,
                    heading: "EXT. HOF".into()
                },
            ]
        );
    }

    #[test]
    fn scene_at_is_the_last_heading_above() {
        let a = analyse(&lines("INT. A\n\nx\n\nINT. B\n\ny"));

        assert_eq!(a.scene_at(2), Some(0));
        assert_eq!(a.scene_at(5), Some(1));
    }

    #[test]
    fn boneyard_across_paragraphs_hides_its_scenes() {
        let a = analyse(&lines("INT. A\n\nx\n\n/*\n\nINT. B\n\n*/\n"));

        let headings: Vec<_> = a.scenes.iter().map(|s| s.heading.as_str()).collect();
        assert_eq!(headings, ["INT. A"]);
    }

    #[test]
    fn key_value_paragraph_is_no_title_page() {
        let a = analyse(&lines("INT. A\n\nNote: hallo\nzweite"));

        assert_eq!(a.kind(2), LineKind::Action);
        assert_eq!(a.kind(3), LineKind::Action);
    }

    #[test]
    fn indents_are_display_only_roles() {
        assert_eq!(LineKind::Character.indent(), 20);
        assert_eq!(LineKind::Dialogue.indent(), 10);
        assert_eq!(LineKind::Parenthetical.indent(), 15);
        assert_eq!(LineKind::Action.indent(), 0);
    }
}
