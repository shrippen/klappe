use super::*;

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
    let back = read_fdx(&xml).unwrap();

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
            page_breaks: 2,
        }
    );
    let back = read_fdx(&xml).unwrap();
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
    assert_eq!(read_fdx(&xml).unwrap(), script);
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
    let script = read_fdx(HAND).unwrap();

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
    let script = read_fdx(HAND).unwrap();

    let text = write(&script);

    assert_eq!(parse(&text).elements, script.elements);
}

#[test]
fn literal_marks_stay_text() {
    let xml = "<FinalDraft><Content><Paragraph Type=\"Action\"><Text>a_b *c</Text></Paragraph></Content></FinalDraft>";

    let script = read_fdx(xml).unwrap();

    assert_eq!(
        script.elements,
        vec![Element::Action("a\\_b \\*c".to_owned())]
    );
    assert_eq!(plain_text("a\\_b \\*c"), "a_b *c");
}

#[test]
fn blanks_stay_outside_the_marks() {
    let xml = "<FinalDraft><Content><Paragraph Type=\"Action\"><Text Style=\"Bold\"> fett </Text></Paragraph></Content></FinalDraft>";

    let script = read_fdx(xml).unwrap();

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
        let error = read_fdx(text).unwrap_err().to_string();
        assert!(error.contains(expected), "{text}: {error}");
    }
}

#[test]
fn deep_nesting_is_refused() {
    let text = format!("{}{}", "<a>".repeat(100), "</a>".repeat(100));

    let error = read_fdx(&text).unwrap_err().to_string();

    assert!(error.contains("too deep"), "{error}");
}

#[test]
fn an_entity_cannot_be_declared() {
    let text = "<!DOCTYPE FinalDraft [<!ENTITY x \"y\">]><FinalDraft>&x;</FinalDraft>";

    assert!(read_fdx(text).is_err());
}

#[test]
fn a_file_without_content_is_an_empty_script() {
    let script = read_fdx("<FinalDraft DocumentType=\"Script\"/>").unwrap();

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
    assert_eq!(write_fdx(&read_fdx(&xml).unwrap()).0, xml);
}

#[test]
fn a_foreign_bold_run_with_a_mark_comes_back_whole() {
    let xml = one_paragraph("Action", "<Text Style=\"Bold\">a_b *c</Text>");

    let script = read_fdx(&xml).unwrap();

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
            let script = read_fdx(&xml).unwrap();
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

    let script = read_fdx(&xml).unwrap();

    assert_eq!(parse(&write(&script)).elements, script.elements);
    assert_eq!(script.elements.len(), 2);
}

#[test]
fn foreign_text_with_fountain_triggers_stays_text() {
    let text = "siehe [[x]] und /* y */ und a_b *c";
    let xml = one_paragraph("Action", &format!("<Text>{text}</Text>"));

    let script = read_fdx(&xml).unwrap();

    assert_eq!(parse(&write(&script)).elements, script.elements);
    assert_eq!(runs_of(&script), vec![(String::new(), text.to_owned())]);
}

#[test]
fn boneyards_are_counted() {
    assert_eq!(boneyard_count("a /* b */ c\n\n/* d\n"), 2);
    assert_eq!(boneyard_count("a /\\* b"), 0);
}
