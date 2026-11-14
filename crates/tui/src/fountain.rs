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
/// The text is first cut as the parser cuts it (boneyards out, see [`strip`]); each
/// paragraph of what is left goes through the real parser alone, so the rules stay the
/// parser's. Lines that lie wholly in a boneyard show as plain action.
pub(crate) fn analyse(lines: &[String]) -> Analysis {
    let cut = strip(lines);
    let mut seen = Analysis {
        kinds: vec![LineKind::Blank; cut.lines.len()],
        ..Analysis::default()
    };
    let mut start = 0;

    while start < cut.lines.len() {
        if cut.lines[start].trim().is_empty() {
            start += 1;
            continue;
        }

        let len = cut.lines[start..]
            .iter()
            .take_while(|l| !l.trim().is_empty())
            .count();
        paragraph(&cut.lines[start..start + len], start, &mut seen);
        start += len;
    }

    back_to_lines(lines, &cut, seen)
}

/// The text as the parser sees it: line `i` of `lines` starts at line `origin[i]` of the
/// typed text. A boneyard swallows its line breaks, so `foo /*⏎⏎*/ bar` is one line.
struct Cut {
    lines: Vec<String>,
    origin: Vec<usize>,
}

/// Cuts `/* … */` out like `klappe_script`'s `strip_boneyard`; an unclosed one runs to the end.
fn strip(lines: &[String]) -> Cut {
    let mut cut = Cut {
        lines: Vec::new(),
        origin: Vec::new(),
    };
    let mut inside = false;

    for (i, line) in lines.iter().enumerate() {
        // Outside a boneyard the line break survives: a new line of the cut text.
        if !inside || cut.lines.is_empty() {
            cut.lines.push(String::new());
            cut.origin.push(i);
        }

        let mut rest = line.as_str();
        loop {
            let marker = if inside {
                BONEYARD_CLOSE
            } else {
                BONEYARD_OPEN
            };
            let Some(at) = rest.find(marker) else {
                if !inside {
                    push_last(&mut cut, rest);
                }
                break;
            };
            if !inside {
                push_last(&mut cut, &rest[..at]);
            }
            rest = &rest[at + marker.len()..];
            inside = !inside;
        }
    }

    cut
}

fn push_last(cut: &mut Cut, text: &str) {
    if let Some(last) = cut.lines.last_mut() {
        last.push_str(text);
    }
}

/// Kinds and scenes of the cut text, put back on the typed lines.
fn back_to_lines(lines: &[String], cut: &Cut, seen: Analysis) -> Analysis {
    // Typed text that is no line of its own for the parser lies in a boneyard.
    let mut kinds: Vec<LineKind> = lines
        .iter()
        .map(|l| match l.trim().is_empty() {
            true => LineKind::Blank,
            false => LineKind::Action,
        })
        .collect();

    for (kind, &at) in seen.kinds.iter().zip(&cut.origin) {
        if *kind != LineKind::Blank {
            kinds[at] = *kind;
        }
    }

    let scenes = seen
        .scenes
        .into_iter()
        .map(|s| Scene {
            line: cut.origin[s.line],
            ..s
        })
        .collect();

    let elements = seen
        .elements
        .into_iter()
        .map(|e| ElementPos {
            line: cut.origin[e.line],
            ..e
        })
        .collect();

    Analysis {
        kinds,
        scenes,
        elements,
        ..Analysis::default()
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
            out.current = (number.clone(), 0);
        } else {
            out.elements.push(ElementPos {
                line: at,
                scene: out.current.0.clone(),
                index: out.current.1,
            });
            out.current.1 += 1;
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
