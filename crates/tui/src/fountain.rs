//! What each line of the text is, from the Fountain parser, for styling and the scene list.

use klappe_services::{parse_located, Element, Located};

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

/// An element (not a heading) and where the project's script links find it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ElementPos {
    /// The first text line the element stands on.
    pub line: usize,
    /// Number of the scene it is in, `None` before the first heading or without a number.
    pub scene: Option<String>,
    /// Position within the scene, as `ScriptLink::element` counts.
    pub index: usize,
}

impl Scene {
    /// The number in the script, else the position in the list (1-based).
    pub(crate) fn display_number(&self, index: usize) -> String {
        self.number
            .clone()
            .unwrap_or_else(|| (index + 1).to_string())
    }
}

#[derive(Debug, Default)]
pub(crate) struct Analysis {
    pub kinds: Vec<LineKind>,
    pub scenes: Vec<Scene>,
    /// In text order.
    pub elements: Vec<ElementPos>,
    /// Scene number and element count of the scene the parsing is in.
    current: (Option<String>, usize),
}

impl Analysis {
    pub(crate) fn kind(&self, line: usize) -> LineKind {
        self.kinds.get(line).copied().unwrap_or(LineKind::Action)
    }

    /// The element that starts on this line.
    pub(crate) fn element_at(&self, line: usize) -> Option<&ElementPos> {
        let at = self.elements.binary_search_by_key(&line, |e| e.line).ok()?;
        self.elements.get(at)
    }

    /// Index of the scene the line belongs to: the last heading at or above it.
    pub(crate) fn scene_at(&self, line: usize) -> Option<usize> {
        self.scenes.iter().rposition(|s| s.line <= line)
    }
}

/// Classifies every line.
///
/// The reader in `klappe_script` says which lines each element stands on, so the rules
/// stay its own. Typed text that is no element (a boneyard, a title page) shows as plain
/// action.
pub(crate) fn analyse(lines: &[String]) -> Analysis {
    let mut seen = Analysis {
        kinds: lines
            .iter()
            .map(|l| match l.trim().is_empty() {
                true => LineKind::Blank,
                false => LineKind::Action,
            })
            .collect(),
        ..Analysis::default()
    };

    for located in parse_located(&lines.join("\n")) {
        place(&located, &mut seen);
    }

    seen
}

fn kind_of(el: &Element) -> LineKind {
    match el {
        Element::SceneHeading { .. } => LineKind::Heading,
        Element::Action(_) => LineKind::Action,
        Element::Character { .. } => LineKind::Character,
        Element::Parenthetical(_) => LineKind::Parenthetical,
        Element::Dialogue(_) => LineKind::Dialogue,
        Element::Transition(_) => LineKind::Transition,
        Element::Centered(_) => LineKind::Centered,
        Element::Section { .. } => LineKind::Section,
        Element::Synopsis(_) => LineKind::Synopsis,
        Element::Note(_) => LineKind::Note,
        Element::PageBreak => LineKind::PageBreak,
        Element::Lyrics(_) => LineKind::Lyrics,
    }
}

fn place(located: &Located, out: &mut Analysis) {
    let Some(&line) = located.lines.first() else {
        return;
    };

    if let Element::SceneHeading { text, number } = &located.element {
        out.scenes.push(Scene {
            line,
            number: number.clone(),
            heading: text.clone(),
        });
        out.current = (number.clone(), 0);
    } else {
        out.elements.push(ElementPos {
            line,
            scene: out.current.0.clone(),
            index: out.current.1,
        });
        out.current.1 += 1;
    }

    let kind = kind_of(&located.element);
    for &at in &located.lines {
        if let Some(slot) = out.kinds.get_mut(at) {
            *slot = kind;
        }
    }
}

#[cfg(test)]
mod tests {
    use klappe_services::parse;

    use super::*;

    fn lines(text: &str) -> Vec<String> {
        text.split('\n').map(str::to_owned).collect()
    }

    fn scene_lines(a: &Analysis) -> Vec<usize> {
        a.scenes.iter().map(|s| s.line).collect()
    }

    /// Review of PR 57: a lone CR from another program breaks the line in the editor too,
    /// so the reader's line numbers match the editor's.
    #[test]
    fn a_lone_cr_breaks_the_line() {
        let buf = crate::buffer::Buffer::from_text("INT. A\n\na\rb\n\nINT. B\n\nc\n\nINT. C");
        let a = analyse(buf.lines());

        let headings: Vec<_> = a.scenes.iter().map(|s| buf.line(s.line)).collect();
        assert_eq!(headings, ["INT. A", "INT. B", "INT. C"]);
    }

    /// The editor reads a title page as the reader does (PR 57): its lines are no
    /// element, and a first block of `key: value` lines is one.
    #[test]
    fn a_title_page_is_no_element() {
        let a = analyse(&lines("Title: Foo\nAuthor: Bar\n\nINT. A\n\nx"));
        assert!(a.element_at(0).is_none());
        assert_eq!(scene_lines(&a), [3]);

        let a = analyse(&lines("EXT HOF: NACHT\n\nx\n\nINT. B"));
        assert_eq!(scene_lines(&a), [4]);
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
    fn elements_count_per_scene_from_the_heading() {
        let a = analyse(&lines("INT. A #1#\n\nx\n\nBOB\nhi\n\nINT. B #2#\n\ny\n"));
        let at = |line| a.element_at(line).map(|e| (e.scene.clone(), e.index));

        assert_eq!(at(2), Some((Some("1".into()), 0)));
        assert_eq!(at(4), Some((Some("1".into()), 1)), "cue");
        assert_eq!(at(5), Some((Some("1".into()), 2)), "dialogue");
        assert_eq!(at(9), Some((Some("2".into()), 0)), "counting restarts");
        assert_eq!(at(0), None, "a heading is no element");
        assert_eq!(at(3), None);
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
    fn text_after_a_closed_boneyard_is_parsed() {
        let text = "INT. A\n\nx\n\n/*\n\nINT. B\n\n*/\nINT. C\n\ny\n";

        let tui: Vec<_> = analyse(&lines(text))
            .scenes
            .iter()
            .map(|s| s.heading.clone())
            .collect();
        let parsed: Vec<_> = parse(text)
            .elements
            .iter()
            .filter_map(|el| match el {
                Element::SceneHeading { text, .. } => Some(text.clone()),
                _ => None,
            })
            .collect();

        assert_eq!(parsed, ["INT. A", "INT. C"]);
        assert_eq!(tui, parsed);
    }

    /// Review of PR 8: text behind `*/` joins the next line's paragraph, as in `parse`.
    #[test]
    fn text_behind_the_close_stays_in_its_paragraph() {
        for text in [
            "INT. A\n\n/*\n\nINT. B\n\nx */ more\nINT. C\n\ny\n",
            "INT. A\n\n/*\n\nINT. B\n\n*/ more\nINT. C\n\ny\n",
            // Review of PR 8, round 2: text before `/*` joins the text behind `*/`.
            "INT. A\n\nfoo /*\n\n*/ INT. C\n\ny\n",
            "INT. A\n\nfoo /*\n\nbar */\nINT. C\n\ny\n",
            "INT. A\n\nx */ INT. D\n\n/* a */ INT. E\n\nb /* c */\n\nINT. F\n",
        ] {
            assert_eq!(headings(text), scene_headings(text), "{text:?}");
        }
    }

    /// Every text of up to four lines from these pieces lists the parser's scenes.
    #[test]
    fn scenes_match_the_parser_around_boneyards() {
        const PIECES: [&str; 8] = [
            "",
            "INT. A",
            "x /*",
            "*/ INT. B",
            "/* y */",
            "z */",
            "/*",
            "a",
        ];

        let mut texts = vec![String::new()];
        for _ in 0..4 {
            texts = texts
                .iter()
                .flat_map(|t| PIECES.iter().map(move |p| format!("{t}{p}\n")))
                .collect();
            for text in &texts {
                assert_eq!(headings(text), scene_headings(text), "{text:?}");
            }
        }
    }

    fn headings(text: &str) -> Vec<String> {
        analyse(&lines(text))
            .scenes
            .iter()
            .map(|s| s.heading.clone())
            .collect()
    }

    fn scene_headings(text: &str) -> Vec<String> {
        parse(text)
            .elements
            .iter()
            .filter_map(|el| match el {
                Element::SceneHeading { text, .. } => Some(text.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn key_value_paragraph_is_no_title_page() {
        let a = analyse(&lines("INT. A\n\nNote: hallo\nzweite"));

        assert_eq!(a.kind(2), LineKind::Action);
        assert_eq!(a.kind(3), LineKind::Action);
    }

    /// Kinds of every line, written as one letter each for a compact table.
    fn letters(text: &str) -> String {
        analyse(&lines(text))
            .kinds
            .iter()
            .map(|k| match k {
                LineKind::Blank => '.',
                LineKind::Heading => 'H',
                LineKind::Action => 'A',
                LineKind::Character => 'C',
                LineKind::Parenthetical => 'P',
                LineKind::Dialogue => 'D',
                LineKind::Transition => 'T',
                LineKind::Centered => 'Z',
                LineKind::Section => 'S',
                LineKind::Synopsis => 'Y',
                LineKind::Note => 'N',
                LineKind::PageBreak => 'B',
                LineKind::Lyrics => 'L',
            })
            .collect()
    }

    #[test]
    fn kinds_with_blank_runs_and_trailing_space() {
        assert_eq!(letters("\n\nINT. A\n\n\n  \nx\ny\n\n"), "..H...AA..");
    }

    #[test]
    fn kinds_of_forced_elements() {
        let text = "!INT. A\nzwei\n\n.hof\n\n@bob\nhi\n\n>RAUS\n\n> mitte <\n\n~la la\n\n[[note]]\n\n===\n\n= kurz\n\n## Akt";
        assert_eq!(letters(text), "AA.H.CD.T.Z.L.N.B.Y.S");
    }

    #[test]
    fn kinds_of_dual_dialogue() {
        let text = "INT. A\n\nANNA\nhallo\n\nBOB ^\n(laut)\nhi\nnoch\n(leise)\nja";
        assert_eq!(letters(text), "H.CD.CPDDPD");
    }

    #[test]
    fn kinds_of_unforced_look_alikes() {
        // A lowercase name is action; a heading line under a cue is dialogue, and an
        // uppercase line `CUT TO:` with a line below is a cue, not a transition.
        let text = "INT. A\n\nanna\nhallo\n\nBOB\nINT. B\n\nCUT TO:\nx";
        assert_eq!(letters(text), "H.AA.CD.CD");
    }

    #[test]
    fn kinds_with_a_multiline_boneyard() {
        // Non-blank lines wholly inside the boneyard show as action, blank ones stay blank.
        let text = "INT. A\n\nx\n\n/*\nINT. B\n\nweg\n*/\n\nINT. C\n\ny";
        assert_eq!(letters(text), "H.A.AA.AA.H.A");
    }

    #[test]
    fn kinds_with_a_boneyard_inside_a_line() {
        assert_eq!(
            letters("INT. A\n\nfoo /* weg */ bar\n\nINT. B /* x */\n"),
            "H.A.H."
        );
    }

    #[test]
    fn kinds_with_a_boneyard_swallowing_breaks() {
        // `foo /*⏎⏎*/ bar` is one line for the parser, on the typed line 2; the
        // typed line 4 is no line of its own and shows as action.
        assert_eq!(letters("INT. A\n\nfoo /*\n\n*/ bar\n\nINT. B"), "H.A.A.H");
    }

    #[test]
    fn kinds_with_an_unclosed_boneyard() {
        assert_eq!(letters("INT. A\n\nx\n/*\nINT. B\n\ny"), "H.AAA.A");
    }

    #[test]
    fn kinds_with_an_opener_inside_dialogue() {
        let text = "INT. A\n\nBOB\nso /* nicht\nmehr */ doch\nende\n\nINT. B";
        assert_eq!(letters(text), "H.CDAD.H");
    }

    #[test]
    fn kinds_with_an_escaped_opener() {
        // `/\*` is the text `/*`, no boneyard: the next heading stays.
        assert_eq!(letters("INT. A\n\nx /\\* y\n\nINT. B"), "H.A.H");
    }

    #[test]
    fn kinds_of_a_title_page_block() {
        // The reader takes the first block as title page; the editor shows plain lines.
        assert_eq!(letters("Title: Foo\nAuthor: Bar\n\nINT. A\n\nx"), "AA.H.A");
    }

    #[test]
    fn elements_with_a_boneyard_count_on_typed_lines() {
        let a = analyse(&lines(
            "INT. A #1#\n\n/*\nweg\n*/\n\nBOB\n(leise)\nhi\n\n!INT. X\n\nINT. B\n\nz",
        ));
        let at = |n: usize| a.elements.iter().map(|e| e.line).nth(n);

        let found: Vec<_> = a
            .elements
            .iter()
            .map(|e| (e.line, e.scene.clone(), e.index))
            .collect();
        assert_eq!(
            found,
            [
                (6, Some("1".into()), 0),
                (7, Some("1".into()), 1),
                (8, Some("1".into()), 2),
                (10, Some("1".into()), 3),
                (14, None, 0),
            ]
        );
        assert_eq!(at(0), Some(6));
        let scenes: Vec<_> = a
            .scenes
            .iter()
            .map(|s| (s.line, s.number.clone()))
            .collect();
        assert_eq!(scenes, [(0, Some("1".into())), (12, None)]);
    }

    #[test]
    fn indents_are_display_only_roles() {
        assert_eq!(LineKind::Character.indent(), 20);
        assert_eq!(LineKind::Dialogue.indent(), 10);
        assert_eq!(LineKind::Parenthetical.indent(), 15);
        assert_eq!(LineKind::Action.indent(), 0);
    }
}
