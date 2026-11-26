use super::*;
use crate::fdx::Style;
use crate::inline::visible_runs;

/// The script of a file, without the loss.
fn read(text: &str) -> Result<Script, FdxError> {
    read_fdx(text).map(|(script, _)| script)
}

fn cue(name: &str, extension: Option<&str>, dual: bool) -> Element {
    Element::Character {
        name: name.to_owned(),
        extension: extension.map(str::to_owned),
        dual,
    }
}

fn pairs(list: &[(&str, &str)]) -> Vec<(String, String)> {
    list.iter()
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect()
}

/// A script with every element kind FDX has.
fn full() -> Script {
    Script {
        title_page: pairs(&[
            ("Title", "Nachtwache"),
            ("Credit", "Written by"),
            ("Author", "Ada Müller"),
            ("Draft date", "26.11.2026"),
            ("Contact", "ada@example.org\nBerlin"),
            ("Notes", "Fassung 2"),
        ]),
        elements: vec![
            Element::SceneHeading {
                text: "INT. SÄNGERSAAL – TAG".to_owned(),
                number: Some("12A".to_owned()),
            },
            Element::Action("Licht fällt *durch* die **Fenster**.\nStille.".to_owned()),
            cue("ELISABETH", Some("V.O."), false),
            Element::Parenthetical("leise".to_owned()),
            Element::Dialogue("Der Weg war _vereist_ & lang <so>.".to_owned()),
            cue("BURGVOGT", None, true),
            Element::Dialogue("Ihr kommt ***spät***.".to_owned()),
            Element::Transition("CUT TO:".to_owned()),
            Element::PageBreak,
            Element::SceneHeading {
                text: "EXT. HOF - NACHT".to_owned(),
                number: None,
            },
            Element::Centered("ENDE".to_owned()),
            cue("MAX", None, false),
            Element::Dialogue("Hallo.".to_owned()),
        ],
    }
}

#[test]
fn script_round_trips_through_fdx() {
    let script = full();

    let (xml, loss) = write_fdx(&script);
    let back = read(&xml).unwrap();

    assert!(loss.is_empty(), "{loss:?}");
    assert_eq!(back, script);
}

#[test]
fn fdx_has_the_structure_final_draft_expects() {
    let (xml, _) = write_fdx(&full());

    assert!(xml.starts_with("<?xml"));
    assert!(xml.contains("<FinalDraft DocumentType=\"Script\""));
    assert!(xml.contains("<Paragraph Type=\"Scene Heading\" Number=\"12A\">"));
    assert!(xml.contains("<Text Style=\"Bold+Italic\">spät</Text>"));
    assert!(xml.contains("<DualDialogue>"));
    assert!(xml.contains("StartsNewPage=\"Yes\""));
    assert!(xml.contains("<Text>Der Weg war </Text>"));
    assert!(xml.contains("&amp; lang &lt;so&gt;."));
    assert!(xml.contains("<TitlePage>"));
}

#[test]
fn what_fdx_lacks_is_reported() {
    let script = Script {
        title_page: Vec::new(),
        elements: vec![
            Element::Section {
                depth: 1,
                text: "Akt".to_owned(),
            },
            Element::Synopsis("Kurz".to_owned()),
            Element::Note("merken".to_owned()),
            Element::Action("Text [[Regie]] bleibt.".to_owned()),
            Element::Lyrics("la la".to_owned()),
            Element::PageBreak,
            Element::PageBreak,
        ],
    };

    let (xml, loss) = write_fdx(&script);

    assert_eq!(
        loss,
        FdxLoss {
            sections: 1,
            synopses: 1,
            notes: 2,
            lyrics: 1,
            boneyards: 0,
            styles: 0,
            page_breaks: 2,
        }
    );
    let back = read(&xml).unwrap();
    assert_eq!(
        back.elements,
        vec![
            Element::Action("Text  bleibt.".to_owned()),
            Element::Action("la la".to_owned()),
        ]
    );
}

#[test]
fn a_page_break_starts_the_page_of_a_dual_dialogue() {
    let script = Script {
        title_page: Vec::new(),
        elements: vec![
            Element::PageBreak,
            cue("A", None, false),
            Element::Dialogue("x".to_owned()),
            cue("B", None, true),
            Element::Dialogue("y".to_owned()),
        ],
    };

    let (xml, loss) = write_fdx(&script);

    assert!(loss.is_empty());
    assert_eq!(read(&xml).unwrap(), script);
}

const HAND: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="no" ?>
<!-- eine Datei von Hand -->
<FinalDraft DocumentType="Script" Template="No" Version="5">
  <Content>
    <Paragraph Number="3" Type="Scene Heading">
      <SceneProperties Length="1" Page="1"/>
      <Text>INT. K&#xDC;CHE &amp; FLUR &#8211; TAG</Text>
    </Paragraph>
    <Paragraph Type="Action"><Text>Sie </Text><Text Style="Bold">r&#228;umt</Text><Text Style="Bold"> auf</Text><Text>.</Text></Paragraph>
    <Paragraph Type="Action"><Text></Text></Paragraph>
    <Paragraph Type="Shot"><Text>CLOSE ON: Hände</Text></Paragraph>
    <DualDialogue>
      <Paragraph Type="Character"><Text>ANNA (O.S.)</Text></Paragraph>
      <Paragraph Type="Dialogue"><Text>Eins</Text></Paragraph>
      <Paragraph Type="Character"><Text>BEN</Text></Paragraph>
      <Paragraph Type="Parenthetical"><Text>(lacht)</Text></Paragraph>
      <Paragraph Type="Dialogue"><Text>Zwei &lt;3 &apos;x&apos; &quot;y&quot;</Text></Paragraph>
    </DualDialogue>
    <Paragraph Type="Action" Alignment="Center"><Text Style="Italic+Underline">Ende</Text></Paragraph>
  </Content>
  <TitlePage>
    <Content>
      <Paragraph Alignment="Center"><Text>Die Küche</Text></Paragraph>
      <Paragraph Alignment="Center"><Text>von</Text></Paragraph>
      <Paragraph Alignment="Center"><Text>Anna</Text></Paragraph>
      <Paragraph Alignment="Right"><Text>1.2.2026</Text></Paragraph>
      <Paragraph Alignment="Left"><Text>anna@example.org</Text></Paragraph>
      <Paragraph Alignment="Left"><Text>Hamburg</Text></Paragraph>
    </Content>
  </TitlePage>
</FinalDraft>
"#;

#[test]
fn a_hand_written_file_is_read() {
    let script = read(HAND).unwrap();

    assert_eq!(
        script.title_page,
        pairs(&[
            ("Title", "Die Küche"),
            ("Credit", "von"),
            ("Author", "Anna"),
            ("Draft date", "1.2.2026"),
            ("Contact", "anna@example.org\nHamburg"),
        ])
    );
    assert_eq!(
        script.elements,
        vec![
            Element::SceneHeading {
                text: "INT. KÜCHE & FLUR – TAG".to_owned(),
                number: Some("3".to_owned()),
            },
            Element::Action("Sie **räumt auf**.".to_owned()),
            Element::Action("CLOSE ON: Hände".to_owned()),
            cue("ANNA", Some("O.S."), false),
            Element::Dialogue("Eins".to_owned()),
            cue("BEN", None, true),
            Element::Parenthetical("lacht".to_owned()),
            Element::Dialogue("Zwei <3 'x' \"y\"".to_owned()),
            Element::Centered("_*Ende*_".to_owned()),
        ]
    );
}

#[test]
fn a_hand_written_file_survives_fountain() {
    let script = read(HAND).unwrap();

    let text = write(&script);

    assert_eq!(parse(&text).elements, script.elements);
}

#[test]
fn literal_marks_stay_text() {
    let xml = "<FinalDraft><Content><Paragraph Type=\"Action\"><Text>a_b *c</Text></Paragraph></Content></FinalDraft>";

    let script = read(xml).unwrap();

    assert_eq!(
        script.elements,
        vec![Element::Action("a\\_b \\*c".to_owned())]
    );
    assert_eq!(plain_text("a\\_b \\*c"), "a_b *c");
}

#[test]
fn blanks_stay_outside_the_marks() {
    let xml = "<FinalDraft><Content><Paragraph Type=\"Action\"><Text Style=\"Bold\"> fett </Text></Paragraph></Content></FinalDraft>";

    let script = read(xml).unwrap();

    assert_eq!(
        script.elements,
        vec![Element::Action("**fett**".to_owned())]
    );
}

#[test]
fn bad_files_are_refused() {
    let cases = [
        ("", "an element is missing"),
        ("<a><b></a>", "closes"),
        ("<a>", "never closed"),
        ("<a>&nope;</a>", "unknown entity"),
        ("<a x=1/>", "quotes"),
        ("<a/><b/>", "after the root"),
        ("<Script/>", "not <FinalDraft>"),
    ];

    for (text, expected) in cases {
        let error = read(text).unwrap_err().to_string();
        assert!(error.contains(expected), "{text}: {error}");
    }
}

#[test]
fn deep_nesting_is_refused() {
    let text = format!("{}{}", "<a>".repeat(100), "</a>".repeat(100));

    let error = read(&text).unwrap_err().to_string();

    assert!(error.contains("too deep"), "{error}");
}

#[test]
fn an_entity_cannot_be_declared() {
    let text = "<!DOCTYPE FinalDraft [<!ENTITY x \"y\">]><FinalDraft>&x;</FinalDraft>";

    assert!(read(text).is_err());
}

#[test]
fn a_file_without_content_is_an_empty_script() {
    let script = read("<FinalDraft DocumentType=\"Script\"/>").unwrap();

    assert_eq!(script, Script::default());
}

fn one_paragraph(kind: &str, runs: &str) -> String {
    format!(
        "<FinalDraft><Content><Paragraph Type=\"{kind}\">{runs}</Paragraph></Content></FinalDraft>"
    )
}

/// The text runs of the first paragraph of a written file.
fn runs_of(script: &Script) -> Vec<(String, String)> {
    let (xml, _) = write_fdx(script);
    let root = crate::xml::parse_xml(&xml).unwrap();
    let par = root.child("Content").unwrap().child("Paragraph").unwrap();

    par.nodes()
        .map(|run| (run.attr("Style").unwrap_or_default().to_owned(), run.text()))
        .collect()
}

#[test]
fn literal_marks_in_emphasis_are_kept() {
    let script = Script {
        title_page: Vec::new(),
        elements: vec![Element::Action("**E_MAIL** ist da.".to_owned())],
    };

    assert_eq!(
        runs_of(&script),
        vec![
            ("Bold".to_owned(), "E_MAIL".to_owned()),
            (String::new(), " ist da.".to_owned()),
        ]
    );
    // Read back, the literal mark carries its backslash; written again it is the same file.
    let (xml, _) = write_fdx(&script);
    assert_eq!(write_fdx(&read(&xml).unwrap()).0, xml);
}

#[test]
fn a_foreign_bold_run_with_a_mark_comes_back_whole() {
    let xml = one_paragraph("Action", "<Text Style=\"Bold\">a_b *c</Text>");

    let script = read(&xml).unwrap();

    assert_eq!(
        runs_of(&script),
        vec![("Bold".to_owned(), "a_b *c".to_owned())]
    );
}

#[test]
fn a_real_pair_of_marks_still_goes() {
    let script = Script {
        title_page: Vec::new(),
        elements: vec![Element::Action("_a_ *b* x_y".to_owned())],
    };

    assert_eq!(
        runs_of(&script),
        vec![
            ("Underline".to_owned(), "a".to_owned()),
            (String::new(), " ".to_owned()),
            ("Italic".to_owned(), "b".to_owned()),
            (String::new(), " x_y".to_owned()),
        ]
    );
}

/// Seconds `read_fdx` takes for a title page of `n` keyed paragraphs, the best of three.
fn title_time(n: usize) -> f64 {
    let mut xml = String::from("<FinalDraft><TitlePage><Content>");
    for i in 0..n {
        xml.push_str(&format!(
            "<Paragraph KlappeKey=\"K{i}\"><Text>v</Text></Paragraph>"
        ));
    }
    xml.push_str("</Content></TitlePage></FinalDraft>");

    (0..3)
        .map(|_| {
            let start = std::time::Instant::now();
            let script = read(&xml).unwrap();
            assert_eq!(script.title_page.len(), n);

            start.elapsed().as_secs_f64()
        })
        .fold(f64::MAX, f64::min)
}

/// Four times the paragraphs take about four times as long; a search through all keys per
/// paragraph takes sixteen times.
#[test]
fn many_title_paragraphs_read_in_linear_time() {
    let ratio = title_time(40_000) / title_time(10_000);

    assert!(ratio < 8.0, "ratio {ratio}");
}

#[test]
fn a_blank_line_in_a_dialogue_keeps_it_a_dialogue() {
    let xml = format!(
        "<FinalDraft><Content>{}{}</Content></FinalDraft>",
        "<Paragraph Type=\"Character\"><Text>ANNA</Text></Paragraph>",
        "<Paragraph Type=\"Dialogue\"><Text>Eins.\n\nZwei.</Text></Paragraph>"
    );

    let script = read(&xml).unwrap();

    assert_eq!(parse(&write(&script)).elements, script.elements);
    assert_eq!(script.elements.len(), 2);
}

#[test]
fn foreign_text_with_fountain_triggers_stays_text() {
    let text = "siehe [[x]] und /* y */ und a_b *c";
    let xml = one_paragraph("Action", &format!("<Text>{text}</Text>"));

    let script = read(&xml).unwrap();

    assert_eq!(parse(&write(&script)).elements, script.elements);
    assert_eq!(runs_of(&script), vec![(String::new(), text.to_owned())]);
}

#[test]
fn boneyards_are_counted() {
    assert_eq!(boneyard_count("a /* b */ c\n\n/* d\n"), 2);
    assert_eq!(boneyard_count("a /\\* b"), 0);
}

/// Every combination of emphasis a text run can have.
fn all_styles() -> Vec<&'static str> {
    vec![
        "",
        "Bold",
        "Italic",
        "Bold+Italic",
        "Underline",
        "Bold+Underline",
        "Italic+Underline",
        "Bold+Italic+Underline",
    ]
}

fn run(style: &str, text: &str) -> String {
    format!("<Text Style=\"{style}\">{text}</Text>")
}

/// What the screenplay says of a one-paragraph file after Fountain: reading, writing,
/// parsing and taking the marks off.
fn through_fountain(runs: &str) -> String {
    let script = read(&one_paragraph("Action", runs)).unwrap();
    let parsed = parse(&write(&script));

    match parsed.elements.as_slice() {
        [Element::Action(text)] => plain_text(text),
        other => panic!("{other:?}"),
    }
}

#[test]
fn neighbouring_runs_of_any_two_styles_keep_their_text() {
    for first in all_styles() {
        for second in all_styles() {
            let runs = format!("{}{}", run(first, "Nie"), run(second, "mals"));

            assert_eq!(through_fountain(&runs), "Niemals", "{first} + {second}");
        }
    }
}

#[test]
fn three_runs_of_any_styles_keep_their_text() {
    for first in all_styles() {
        for second in all_styles() {
            for third in all_styles() {
                let runs = format!("{}{}{}", run(first, "a"), run(second, "b"), run(third, "c"));

                assert_eq!(
                    through_fountain(&runs),
                    "abc",
                    "{first} + {second} + {third}"
                );
            }
        }
    }
}

#[test]
fn runs_that_meet_with_blanks_between_keep_both_styles() {
    let runs = format!("{}{}", run("Bold", "Nie "), run("Bold+Italic", " mals"));

    let script = read(&one_paragraph("Action", &runs)).unwrap();

    assert_eq!(
        script.elements,
        vec![Element::Action("**Nie**  ***mals***".to_owned())]
    );
}

#[test]
fn an_emphasis_ending_in_a_slash_or_backslash_keeps_its_closing_mark() {
    for end in ["/", "\\", "a/", "//"] {
        let runs = format!("{}{}", run("Bold", &format!("ab{end}")), run("", " x"));
        let script = read(&one_paragraph("Action", &runs)).unwrap();
        let text = write(&script);

        assert_eq!(through_fountain(&runs), format!("ab{end} x"));
        // A slash tail stands plain behind the closing mark; a backslash is escaped inside.
        let slashes = end.trim_start_matches(|c| c != '/');
        let inside = end.trim_end_matches('/').replace('\\', "\\\\");
        let expected = format!("**ab{inside}**{slashes} x");
        assert!(text.contains(&expected), "{text}");
    }
}

#[test]
fn a_run_that_gives_up_its_style_is_counted() {
    let runs = format!("{}{}", run("Bold", "Nie"), run("Bold+Italic", "mals"));

    let (_, loss) = read_fdx(&one_paragraph("Action", &runs)).unwrap();

    assert_eq!(loss.styles, 1);
}

#[test]
fn runs_that_keep_their_style_are_not_counted() {
    let runs = format!("{}{}", run("Bold", "Nie"), run("Italic", " mals"));

    let (script, loss) = read_fdx(&one_paragraph("Action", &runs)).unwrap();

    assert_eq!(loss.styles, 0);
    assert_eq!(
        script.elements,
        vec![Element::Action("**Nie** *mals*".to_owned())]
    );
}

#[test]
fn literal_backslashes_stay_text() {
    let cases = [
        (run("", "C:\\_tmp\\_x"), "C:\\_tmp\\_x"),
        (run("", "\\*a\\*"), "\\*a\\*"),
        (run("Bold", "a\\*"), "a\\*"),
        (run("", "a\\\\b \\[[x]]"), "a\\\\b \\[[x]]"),
        (run("Italic", "\\\\*x"), "\\\\*x"),
    ];

    for (runs, expected) in cases {
        assert_eq!(through_fountain(&runs), expected, "{runs}");
    }
}

#[test]
fn a_style_given_up_is_judged_again_against_the_run_before() {
    let runs = format!(
        "{}{}{}",
        run("Bold", "aa"),
        run("Italic", "bbb"),
        run("Bold", "cccc")
    );

    let (script, loss) = read_fdx(&one_paragraph("Action", &runs)).unwrap();

    assert_eq!(
        script.elements,
        vec![Element::Action("**aa**bbb**cccc**".to_owned())]
    );
    assert_eq!(loss.styles, 1);
}

#[test]
fn runs_meeting_at_a_backslash_keep_it_text() {
    let cases = [
        (
            format!("{}{}", run("Underline", "C:\\"), run("", "_tmp_")),
            "C:\\_tmp_",
        ),
        (
            format!("{}{}", run("Bold", "a\\"), run("", "_b_")),
            "a\\_b_",
        ),
        (
            format!(
                "{}{}{}{}",
                run("", "a\\"),
                run("Bold", "*x"),
                run("", " "),
                run("Italic", "y")
            ),
            "a\\*x y",
        ),
    ];

    for (runs, expected) in cases {
        assert_eq!(through_fountain(&runs), expected, "{runs}");
    }
}

#[test]
fn brackets_over_runs_do_not_open_a_note() {
    let runs = format!(
        "{}{}{}",
        run("", "a["),
        run("Bold", "["),
        run("Italic", "[bbbb]] c")
    );

    assert_eq!(through_fountain(&runs), "a[[[bbbb]] c");
}

#[test]
fn blanks_that_lose_their_style_are_counted() {
    let runs = format!("{}{}{}", run("", "a"), run("Underline", " "), run("", "b"));

    let (_, loss) = read_fdx(&one_paragraph("Action", &runs)).unwrap();

    assert_eq!(loss.styles, 1);
}

/// Small deterministic generator, so the sequences are the same on every run.
fn next_random(state: &mut u64) -> usize {
    *state = state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);

    (*state >> 33) as usize
}

/// A random paragraph of runs: their style, their text.
fn random_runs(state: &mut u64) -> Vec<(Style, String)> {
    const PIECES: [&str; 8] = ["\\", "*", "_", "[", "]", "/", "a", " "];
    let styles = all_styles();
    let mut runs: Vec<(Style, String)> = Vec::new();

    for _ in 0..=next_random(state) % 4 {
        let style = Style::parse(styles[next_random(state) % styles.len()]);
        let text: String = (0..=next_random(state) % 4)
            .map(|_| PIECES[next_random(state) % PIECES.len()])
            .collect();

        match runs.last_mut() {
            Some((last, joined)) if *last == style => joined.push_str(&text),
            _ => runs.push((style, text)),
        }
    }

    runs
}

/// The Fountain line of `runs` read back: each visible character with its style.
fn read_back(runs: &[(Style, String)]) -> (Vec<(char, Style)>, usize) {
    let xml: String = runs
        .iter()
        .map(|(style, text)| run(&style.attr().unwrap_or_default(), text))
        .collect();
    let (script, loss) = read_fdx(&one_paragraph("Action", &xml)).unwrap();
    let parsed = parse(&write(&script));
    let [Element::Action(line)] = parsed.elements.as_slice() else {
        panic!("{xml}: {:?}", parsed.elements);
    };
    let seen = visible_runs(line)
        .iter()
        .flat_map(|(m, t)| {
            assert!(!m.note && !m.boneyard, "{xml} -> {line}");
            let style = Style {
                bold: m.bold,
                italic: m.italic,
                underline: m.underline,
            };
            t.chars().map(move |c| (c, style))
        })
        .collect();

    (seen, loss.styles)
}

/// The characters whose style is checked: not the blanks at the edge of a run (they stand
/// outside its marks), and none at all in a run of blanks.
fn checked(runs: &[(Style, String)]) -> Vec<(char, Option<Style>)> {
    let mut out = Vec::new();

    for (style, text) in runs {
        let lead = text.chars().count() - text.trim_start().chars().count();
        let end = text.trim_end().chars().count();
        let blank = text.trim().is_empty();

        for (i, c) in text.chars().enumerate() {
            let outer = c.is_whitespace() && (i < lead || i >= end || blank);
            out.push((c, if outer { None } else { Some(*style) }));
        }
    }

    out
}

#[test]
fn random_runs_keep_their_text_and_count_lost_style() {
    let mut state = 7;

    for _ in 0..20_000 {
        let runs = random_runs(&mut state);
        let all: String = runs.iter().map(|(_, t)| t.as_str()).collect();
        if all.trim().is_empty() {
            continue;
        }

        let (seen, lost) = read_back(&runs);
        let wanted = checked(&runs);
        let text: String = seen.iter().map(|(c, _)| *c).collect();
        assert_eq!(text, all.trim(), "{runs:?}");

        // The paragraph's outer blanks are not kept; skip them on the wanted side.
        let skipped = all.len() - all.trim_start().len();
        let wanted = &wanted[skipped..skipped + seen.len()];
        let same = seen
            .iter()
            .zip(wanted)
            .all(|((_, s), (_, w))| w.is_none_or(|w| *s == w));
        let blank_run_lost = runs
            .iter()
            .any(|(s, t)| *s != Style::default() && t.trim().is_empty());
        assert!(same || lost > 0, "{runs:?} -> {seen:?}");
        assert!(!blank_run_lost || lost > 0, "{runs:?}");
    }
}

/// Kinds whose text Klappe shows as it stands, with the paragraph type of each.
const RAW_KINDS: [(&str, &str); 3] = [
    ("Scene Heading", "INT. HAUS [1987] B_RO *2* \\ 3"),
    ("Character", "MAX_M *[V.O.]* \\ X"),
    ("Transition", "CUT_TO *[1]* \\"),
];

#[test]
fn heading_cue_and_transition_keep_their_text() {
    for (kind, text) in RAW_KINDS {
        let xml = one_paragraph(kind, &format!("<Text>{text}</Text>"));
        let script = read(&xml).unwrap();

        let shown = match &script.elements[0] {
            Element::SceneHeading { text, .. } | Element::Transition(text) => text.clone(),
            Element::Character { name, .. } => name.clone(),
            other => panic!("{other:?}"),
        };
        assert_eq!(shown, text, "{kind}");
    }
}

#[test]
fn a_cue_with_an_underscore_reads_through_fountain() {
    for (kind, text) in RAW_KINDS {
        let xml = one_paragraph(kind, &format!("<Text>{text}</Text>"));
        let script = read(&xml).unwrap();

        let again = parse(&write(&script));

        assert_eq!(again.elements, script.elements, "{kind}");
    }
}

#[test]
fn raw_kinds_keep_their_text_through_fdx() {
    for (kind, text) in RAW_KINDS {
        let xml = one_paragraph(kind, &format!("<Text>{text}</Text>"));
        let script = read(&xml).unwrap();

        let (written, loss) = write_fdx(&script);

        assert_eq!(read(&written).unwrap().elements, script.elements, "{kind}");
        assert_eq!(loss, FdxLoss::default(), "{kind}");
    }
}

#[test]
fn a_name_that_fountain_would_force_stays_a_cue() {
    for name in ["@MAX", ".MAX", "!MAX", ">MAX", "~MAX"] {
        let xml = one_paragraph("Character", &format!("<Text>{name}</Text>"));
        let script = read(&xml).unwrap();

        assert_eq!(parse(&write(&script)).elements, script.elements, "{name}");
    }
}

#[test]
fn a_heading_that_fountain_would_misread_stays_a_heading() {
    for text in ["@INT. HAUS", "= INT. HAUS"] {
        let xml = one_paragraph("Scene Heading", &format!("<Text>{text}</Text>"));
        let script = read(&xml).unwrap();

        assert_eq!(parse(&write(&script)).elements, script.elements, "{text}");
    }
}
