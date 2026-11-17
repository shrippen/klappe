//! What each line of the text is, from the Fountain parser, for styling and the scene list.

use klappe_services::{
    inline_runs, outline_of, parse_located, title_page_lines, Element, Located, OutlineEntry, Slot,
    Vocabulary,
};

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
    /// A line of the title page (`Title: …`).
    Title,
}

impl LineKind {
    /// The element a line is for Enter and Tab; none for those that are not part of the flow.
    pub(crate) fn slot(self) -> Option<Slot> {
        match self {
            Self::Heading => Some(Slot::Heading),
            Self::Action => Some(Slot::Action),
            Self::Character => Some(Slot::Character),
            Self::Parenthetical => Some(Slot::Parenthetical),
            Self::Dialogue => Some(Slot::Dialogue),
            Self::Transition => Some(Slot::Transition),
            _ => None,
        }
    }

    pub(crate) fn of_slot(slot: Slot) -> Self {
        match slot {
            Slot::Heading => Self::Heading,
            Slot::Action => Self::Action,
            Slot::Character => Self::Character,
            Slot::Parenthetical => Self::Parenthetical,
            Slot::Dialogue => Self::Dialogue,
            Slot::Transition => Self::Transition,
        }
    }

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
    /// Sections, scenes and synopses as the outline lists them.
    pub outline: Vec<OutlineEntry>,
    /// In text order.
    pub elements: Vec<ElementPos>,
    /// What the screenplay names, for completion.
    pub vocab: Vocabulary,
    /// Per line: it belongs to a dual dialogue (its cue with `^`, parentheticals, dialogue).
    pub dual: Vec<bool>,
    /// Whether the element parsed last is part of a dual dialogue.
    dual_now: bool,
    /// Per line: it starts inside a `/* … */` an earlier line opened.
    pub boneyard: Vec<bool>,
    /// Scene number and element count of the scene the parsing is in.
    current: (Option<String>, usize),
}

impl Analysis {
    pub(crate) fn is_dual(&self, line: usize) -> bool {
        self.dual.get(line).copied().unwrap_or(false)
    }

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
/// stay its own. Typed text that is no element (a boneyard) shows as plain action, the
/// title page as `Title`; the inline marks of a boneyard are read per line.
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

    let text = lines.join("\n");
    let located = parse_located(&text);
    for item in &located {
        place(item, &mut seen);
    }
    seen.vocab = Vocabulary::from_located(&located);
    seen.outline = outline_of(&located, lines.len());

    let mut open = false;
    for line in lines {
        seen.boneyard.push(open);
        open = inline_runs(line, open).1;
    }
    for kind in seen.kinds.iter_mut().take(title_page_lines(&text)) {
        *kind = LineKind::Title;
    }

    seen
}

/// Characters that mark parts of a line the writer types as is.
const FORCE_CHARACTER: char = '@';
const NUMBER_MARK: char = '#';
const EXTENSION_OPEN: char = '(';
const SHARP_S: char = 'ß';

/// Uppercase as the parser counts it: `ß` stays, and a letter whose capital is longer
/// than one character stays, so the cursor column holds.
fn upper(text: &str) -> String {
    let one = |c: char| {
        let mut up = c.to_uppercase();
        match (up.next(), up.next()) {
            (Some(u), None) if c != SHARP_S => u,
            _ => c,
        }
    };

    text.chars().map(one).collect()
}

/// Uppercases the part of `line` before the first `cut`, `rest` stays as typed.
fn upper_before(line: &str, cut: usize) -> String {
    let (head, rest) = line.split_at(cut);

    upper(head) + rest
}

/// The cursor line in capitals, when it is a scene heading or a cue forced with `@` and has
/// lowercase letters left. A heading keeps its `#12a#` number as typed (also half typed), a cue its
/// extension, also while it is still open: `@anna (cont'd)` becomes `@ANNA (cont'd)`, Fountain
/// allows both cases. An unforced cue is no cue until it is in capitals, so it has nothing to fix.
pub(crate) fn typed_caps(lines: &[String], at: usize, pending: Option<LineKind>) -> Option<String> {
    let line = lines.get(at)?;
    let kind = match (analyse(lines).kind(at), pending) {
        (LineKind::Blank | LineKind::Action, Some(pending)) => pending,
        (kind, _) => kind,
    };
    let trimmed = line.trim_end();

    let cut = match kind {
        // The number from its opening `#` on, also while it is still open: `#3a` of `#3a#`.
        // A `#` followed by a space starts no number (as in the parser): `#2 flur` is text.
        LineKind::Heading => {
            let body = trimmed.strip_suffix(NUMBER_MARK).unwrap_or(trimmed);
            match body.rfind(NUMBER_MARK) {
                Some(at) if !body[at..].contains(char::is_whitespace) => at,
                _ => trimmed.len(),
            }
        }
        // A cue of a pending line and a forced one are capitals up to the extension.
        LineKind::Character
            if pending == Some(LineKind::Character)
                || line.trim_start().starts_with(FORCE_CHARACTER) =>
        {
            trimmed.find(EXTENSION_OPEN).unwrap_or(trimmed.len())
        }
        _ => return None,
    };

    let caps = upper_before(line, cut);

    (caps != *line).then_some(caps)
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

    out.dual_now = match &located.element {
        Element::Character { dual, .. } => *dual,
        Element::Parenthetical(_) | Element::Dialogue(_) => out.dual_now,
        _ => false,
    };
    if out.dual.len() < out.kinds.len() {
        out.dual.resize(out.kinds.len(), false);
    }

    let kind = kind_of(&located.element);
    for &at in &located.lines {
        if let Some(slot) = out.kinds.get_mut(at) {
            *slot = kind;
        }
        if let Some(slot) = out.dual.get_mut(at) {
            *slot = out.dual_now;
        }
    }
}

#[cfg(test)]
mod tests {
    use klappe_services::parse;

    use super::*;

    fn caps(text: &str, at: usize) -> Option<String> {
        typed_caps(&lines(text), at, None)
    }

    #[test]
    fn caps_for_headings_by_prefix_or_force() {
        for (typed, want) in [
            ("int. küche - tag", "INT. KÜCHE - TAG"),
            ("ext. straße", "EXT. STRAßE"),
            ("int./ext. auto", "INT./EXT. AUTO"),
            ("i/e auto", "I/E AUTO"),
            (".innen küche", ".INNEN KÜCHE"),
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
                LineKind::Title => 'K',
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
    fn no_title_page_no_title_kind() {
        assert_eq!(letters("INT. A\n\nx"), "H.A");
    }

    #[test]
    fn a_boneyard_is_followed_from_line_to_line() {
        let a = analyse(&lines("a /* b\nc\nd */ e\nf"));

        assert_eq!(a.boneyard, [false, true, true, false]);
    }

    #[test]
    fn the_vocabulary_comes_from_the_text() {
        let a = analyse(&lines(
            "INT. KÜCHE - TAG\n\nx\n\nANNA\nhi\n\nBEN\nho\n\nANNA\nna",
        ));

        assert_eq!(a.vocab.characters(), ["ANNA", "BEN"]);
        assert_eq!(a.vocab.locations(), ["KÜCHE"]);
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
        // The reader takes the first block as title page; the editor marks its lines.
        assert_eq!(letters("Title: Foo\nAuthor: Bar\n\nINT. A\n\nx"), "KK.H.A");
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
