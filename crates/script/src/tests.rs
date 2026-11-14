use super::*;

const SCENE: &str = "INT. SÄNGERSAAL – TAG #12#

Licht fällt durch die drei Fenster. ELISABETH tritt durch die Tür und bleibt stehen.

BURGVOGT
(ohne aufzusehen)
Ihr kommt spät.

ELISABETH
Der Weg war vereist.
";

fn els(text: &str) -> Vec<Element> {
    parse(text).elements
}

fn action(text: &str) -> Element {
    Element::Action(text.to_string())
}

fn cue(name: &str, extension: Option<&str>, dual: bool) -> Element {
    Element::Character {
        name: name.to_string(),
        extension: extension.map(str::to_string),
        dual,
    }
}

fn heading(text: &str, number: Option<&str>) -> Element {
    Element::SceneHeading {
        text: text.to_string(),
        number: number.map(str::to_string),
    }
}

#[test]
fn german_scene() {
    let expected = vec![
        heading("INT. SÄNGERSAAL – TAG", Some("12")),
        action(
            "Licht fällt durch die drei Fenster. ELISABETH tritt durch die Tür und bleibt stehen.",
        ),
        cue("BURGVOGT", None, false),
        Element::Parenthetical("ohne aufzusehen".into()),
        Element::Dialogue("Ihr kommt spät.".into()),
        cue("ELISABETH", None, false),
        Element::Dialogue("Der Weg war vereist.".into()),
    ];
    assert_eq!(els(SCENE), expected);
}

#[test]
fn german_scene_round_trip() {
    let script = parse(SCENE);
    let text = write(&script);

    assert_eq!(text, SCENE);
    assert_eq!(parse(&text), script);
}

#[test]
fn scene_heading_prefixes() {
    for line in [
        "INT. KÜCHE - NACHT",
        "EXT. HOF - TAG",
        "EST. BURG - TAG",
        "INT./EXT. AUTO - TAG",
        "INT/EXT AUTO - TAG",
        "I/E AUTO - TAG",
        "int. küche - nacht",
    ] {
        let found = els(&format!("{line}\n\nText."));
        assert_eq!(found[0], heading(line, None), "{line}");
    }
}

#[test]
fn heading_needs_blank_lines() {
    assert_eq!(els("INT. KÜCHE\nText."), vec![action("INT. KÜCHE\nText.")]);
    assert_eq!(
        els("INTERIEUR ist kein Kopf."),
        vec![action("INTERIEUR ist kein Kopf.")]
    );
}

#[test]
fn forced_heading_and_ellipsis() {
    assert_eq!(
        els(".ALTER SAAL\nText"),
        vec![heading("ALTER SAAL", None), action("Text")]
    );
    assert_eq!(els("...und weiter"), vec![action("...und weiter")]);
}

#[test]
fn scene_numbers() {
    assert_eq!(
        els("EXT. HOF - TAG #12A#\n\nx"),
        vec![heading("EXT. HOF - TAG", Some("12A")), action("x")]
    );
    assert_eq!(
        els("EXT. HOF #1-2#"),
        vec![heading("EXT. HOF", Some("1-2"))]
    );
    assert_eq!(els("EXT. HOF #a b#"), vec![heading("EXT. HOF #a b#", None)]);
}

#[test]
fn forced_action() {
    assert_eq!(
        els("!INT. KEIN KOPF\n\nx"),
        vec![action("INT. KEIN KOPF"), action("x")]
    );
    assert_eq!(els("!BOB\nsagt nichts"), vec![action("BOB\nsagt nichts")]);
}

#[test]
fn character_extension_and_dual() {
    let found = els("ELISABETH (V.O.)\nHallo.\n\nBURGVOGT ^\nNein.\n\nKIND (O.S.)^\nJa.");
    assert_eq!(found[0], cue("ELISABETH", Some("V.O."), false));
    assert_eq!(found[2], cue("BURGVOGT", None, true));
    assert_eq!(found[4], cue("KIND", Some("O.S."), true));
}

#[test]
fn character_rules() {
    // lowercase, no blank before, nothing below, forced
    assert_eq!(els("Elisabeth\nHallo."), vec![action("Elisabeth\nHallo.")]);
    assert_eq!(
        els("Text.\nBURGVOGT\nHallo."),
        vec![action("Text.\nBURGVOGT\nHallo.")]
    );
    assert_eq!(els("BURGVOGT"), vec![action("BURGVOGT")]);
    assert_eq!(els("@McBride\nHi.")[0], cue("McBride", None, false));
    assert_eq!(els("ÄÖÜ\nHi.")[0], cue("ÄÖÜ", None, false));
}

#[test]
fn multi_line_dialogue() {
    let found = els("ELISABETH\nErste Zeile.\nZweite Zeile.\n(leise)\nDritte.");
    assert_eq!(found.len(), 4);
    assert_eq!(
        found[1],
        Element::Dialogue("Erste Zeile.\nZweite Zeile.".into())
    );
    assert_eq!(found[2], Element::Parenthetical("leise".into()));
}

#[test]
fn transitions() {
    assert_eq!(
        els("Text.\n\nDISSOLVE TO:\n\nText.")[1],
        Element::Transition("DISSOLVE TO:".into())
    );
    assert_eq!(
        els("CUT TO:\n\nx")[0],
        Element::Transition("CUT TO:".into())
    );
    assert_eq!(
        els("> ÜBERBLENDEN")[0],
        Element::Transition("ÜBERBLENDEN".into())
    );
    // Without blank line below it is a cue.
    assert_eq!(els("CUT TO:\nx")[0], cue("CUT TO:", None, false));
}

#[test]
fn centered() {
    assert_eq!(els("> ENDE <")[0], Element::Centered("ENDE".into()));
    assert_eq!(els(">ENDE<")[0], Element::Centered("ENDE".into()));
}

#[test]
fn sections_synopsis_note_break_lyrics() {
    let found = els("# Akt I\n\n## Kapitel 2\n\n= Sie kommt an.\n\n[[Dialekt prüfen]]\n\n===\n\n~Alle Vögel sind schon da");
    assert_eq!(
        found,
        vec![
            Element::Section {
                depth: 1,
                text: "Akt I".into()
            },
            Element::Section {
                depth: 2,
                text: "Kapitel 2".into()
            },
            Element::Synopsis("Sie kommt an.".into()),
            Element::Note("Dialekt prüfen".into()),
            Element::PageBreak,
            Element::Lyrics("Alle Vögel sind schon da".into()),
        ]
    );
}

#[test]
fn inline_note_stays_in_action() {
    assert_eq!(
        els("Sie geht [[langsam]] ab."),
        vec![action("Sie geht [[langsam]] ab.")]
    );
}

#[test]
fn title_page() {
    let script = parse("Title: Die Burg\nAuthor: A. Autor\nNotes:\n   Erste Zeile\n   Zweite Zeile\n\nINT. SAAL - TAG\n\nx");
    assert_eq!(
        script.title_page,
        vec![
            ("Title".to_string(), "Die Burg".to_string()),
            ("Author".to_string(), "A. Autor".to_string()),
            ("Notes".to_string(), "Erste Zeile\nZweite Zeile".to_string()),
        ]
    );
    assert_eq!(script.elements[0], heading("INT. SAAL - TAG", None));
}

#[test]
fn transition_is_no_title_page() {
    let script = parse("CUT TO:\n\nx");
    assert!(script.title_page.is_empty());
    assert_eq!(script.elements[0], Element::Transition("CUT TO:".into()));
}

#[test]
fn boneyard() {
    let found = els("Vorher /* weg\nüber Zeilen */ nachher.\n\n/* ganz weg */\n\nEnde. /* offen");
    assert_eq!(found, vec![action("Vorher  nachher."), action("Ende.")]);
}

#[test]
fn windows_line_endings() {
    assert_eq!(
        els("INT. A - TAG\r\n\r\nText.\r\n"),
        vec![heading("INT. A - TAG", None), action("Text.")]
    );
}

#[test]
fn empty_input() {
    assert_eq!(parse(""), Script::default());
    assert_eq!(write(&Script::default()), "");
}

#[test]
fn scenes_group_elements() {
    let script = parse("Vorspann.\n\nINT. A - TAG #1#\n\nEins.\n\nEXT. B - TAG #2#\n\nZwei.\n\n= Neben.\n\nINT. C - TAG\n\n");
    let scenes = script.scenes();

    assert_eq!(scenes.len(), 4);
    assert_eq!(scenes[0].heading, None);
    assert_eq!(scenes[0].elements, &[action("Vorspann.")]);
    assert_eq!(scenes[1].number, Some("1"));
    assert_eq!(scenes[2].elements.len(), 2);
    assert_eq!(scenes[2].elements[1], Element::Synopsis("Neben.".into()));
    assert_eq!(scenes[3].heading, Some("INT. C - TAG"));
    assert!(scenes[3].elements.is_empty());

    let s2 = script.scene("2");
    assert_eq!(s2.map(|s| s.elements.len()), Some(2));
    assert!(script.scene("9").is_none());
}

#[test]
fn round_trip_all_kinds() {
    let text = "Title: Die Burg\nNotes:\n   a\n   b\n\n# Akt I\n\n= Kurz.\n\nINT. SÄNGERSAAL – TAG #12A#\n\nSie wartet.\nNoch immer.\n\nBURGVOGT (V.O.)\n(leise)\nEins.\nZwei.\n\nELISABETH^\nNein.\n\n[[Notiz]]\n\n===\n\n~Lied\n\nDISSOLVE TO:\n\n> ENDE <\n";
    let script = parse(text);

    assert_eq!(write(&script), text);
    assert_eq!(parse(&write(&script)), script);
}

#[test]
fn write_forces_when_needed() {
    let script = Script {
        title_page: Vec::new(),
        elements: vec![
            action("INT. KEIN KOPF"),
            action("BOB\nsagt nichts"),
            action("# kein Abschnitt"),
            action(""),
            heading("ALTER SAAL", None),
            cue("McBride", None, false),
            Element::Dialogue("Hi.".into()),
            Element::Transition("ÜBERBLENDEN".into()),
        ],
    };
    let text = write(&script);

    assert_eq!(parse(&text), script);
    assert!(text.contains("!INT. KEIN KOPF"));
    assert!(text.contains(".ALTER SAAL"));
    assert!(text.contains("@McBride"));
    assert!(text.contains(">ÜBERBLENDEN"));
}

fn round_trips(script: &Script) -> bool {
    parse(&write(script)) == *script
}

#[test]
fn cue_without_dialogue_round_trips() {
    let script = parse("@BURGVOGT\n\nEr schweigt.\n");
    assert_eq!(
        script.elements,
        vec![cue("BURGVOGT", None, false), action("Er schweigt.")]
    );
    assert_eq!(parse(&write(&script)), script);

    let script = parse("Text.\n\n@BURGVOGT\n");
    assert_eq!(parse(&write(&script)), script);
}

#[test]
fn key_like_first_element_round_trips() {
    let script = Script {
        title_page: Vec::new(),
        elements: vec![action("Ort: Burghof"), heading("INT. A - TAG", None)],
    };
    assert_eq!(parse(&write(&script)), script);
}

#[test]
fn crlf_title_page() {
    let script = parse("Title: Die Burg\r\nAuthor: A\r\n\r\nINT. A - TAG\r\n\r\nText.\r\n");
    assert_eq!(script.title_page.len(), 2);
    assert_eq!(
        script.elements,
        vec![heading("INT. A - TAG", None), action("Text.")]
    );

    let lone_cr = parse("Title: X\r\rText.");
    assert_eq!(lone_cr.title_page.len(), 1);
    assert_eq!(lone_cr.elements, vec![action("Text.")]);
}

#[test]
fn sharp_s_in_cue() {
    assert_eq!(els("GROßVATER\nHallo.")[0], cue("GROßVATER", None, false));
}

/// One sample per element kind, as a block that can stand anywhere.
fn samples() -> Vec<Vec<Element>> {
    vec![
        vec![heading("INT. A - TAG", Some("1"))],
        vec![heading("ALTER SAAL", None)],
        vec![action("Sie wartet.\nNoch immer.")],
        vec![action("INT. KEIN KOPF")],
        vec![action("Ort: Burghof")],
        vec![action("# kein Abschnitt")],
        vec![action("")],
        vec![cue("BURGVOGT", Some("V.O."), true)],
        vec![cue("McBride", None, false)],
        vec![
            cue("ELISABETH", None, false),
            Element::Parenthetical("leise".into()),
            Element::Dialogue("Eins.\nZwei.".into()),
        ],
        vec![
            cue("BOB: X", None, false),
            Element::Dialogue("Hi: ja".into()),
        ],
        vec![Element::Transition("CUT TO:".into())],
        vec![Element::Transition("ÜBERBLENDEN".into())],
        vec![Element::Centered("ENDE".into())],
        vec![Element::Section {
            depth: 2,
            text: "Akt".into(),
        }],
        vec![Element::Synopsis("Kurz.".into())],
        vec![Element::Note("Notiz".into())],
        vec![Element::PageBreak],
        vec![Element::Lyrics("Lied".into())],
    ]
}

#[test]
fn every_kind_in_every_position_round_trips() {
    let titles = [
        Vec::new(),
        vec![("Title".to_string(), "Die Burg".to_string())],
    ];
    let filler = action("Füller.");

    for title_page in titles {
        for block in samples() {
            let variants = [
                block.clone(),
                [block.clone(), vec![filler.clone()]].concat(),
                [vec![filler.clone()], block.clone()].concat(),
                [vec![filler.clone()], block.clone(), vec![filler.clone()]].concat(),
                [block.clone(), block.clone()].concat(),
            ];
            for elements in variants {
                let script = Script {
                    title_page: title_page.clone(),
                    elements,
                };
                assert!(round_trips(&script), "{script:?}\n{:?}", write(&script));
            }
        }
    }
}

/// `/*` in an action starts a boneyard on reading; the writer must protect it.
#[test]
fn boneyard_opener_in_text_round_trips() {
    let texts = [
        "*/*/",
        "a /* b",
        "/*",
        "\\/*",
        "/*/*",
        "x */ y",
        "a /\\* b",
        "/\\*",
        "C:\\/\\*",
        "/\\\\*",
        "/\\*/*/\\\\*",
    ];

    for text in texts {
        let script = Script {
            title_page: Vec::new(),
            elements: vec![action(text)],
        };

        assert!(round_trips(&script), "{text:?} -> {:?}", write(&script));
    }
}

#[test]
fn boneyard_opener_in_title_round_trips() {
    let script = Script {
        title_page: vec![("Title".to_string(), "a /* b".to_string())],
        elements: vec![action("Text.")],
    };

    assert!(round_trips(&script), "{:?}", write(&script));
}

fn at(text: &str) -> Vec<(Element, Vec<usize>)> {
    parse_located(text)
        .into_iter()
        .map(|l| (l.element, l.lines))
        .collect()
}

#[test]
fn located_elements_carry_their_lines() {
    let found = at("INT. A\n\nx\ny\n\nBOB\n(leise)\nhi\nda\n\n===\n");

    assert_eq!(
        found,
        [
            (heading("INT. A", None), vec![0]),
            (action("x\ny"), vec![2, 3]),
            (cue("BOB", None, false), vec![5]),
            (Element::Parenthetical("leise".into()), vec![6]),
            (Element::Dialogue("hi\nda".into()), vec![7, 8]),
            (Element::PageBreak, vec![10]),
        ]
    );
}

#[test]
fn located_elements_skip_the_title_page_and_blank_runs() {
    let found = at("\n\nTitle: Foo\nAuthor: Bar\n\n\n\nINT. A\n\nx");

    assert_eq!(
        found,
        [(heading("INT. A", None), vec![7]), (action("x"), vec![9])]
    );
}

#[test]
fn located_lines_count_through_boneyards() {
    let found = at("INT. A\n\n/*\nINT. B\n*/\n\nfoo /*\n\n*/ bar\n\nINT. C\n");

    assert_eq!(
        found,
        [
            (heading("INT. A", None), vec![0]),
            (action("foo  bar"), vec![6]),
            (heading("INT. C", None), vec![10]),
        ]
    );
}

#[test]
fn located_lines_read_cr_as_a_line_break() {
    assert_eq!(
        at("INT. A\r\n\r\nx\r\ny"),
        [
            (heading("INT. A", None), vec![0]),
            (action("x\ny"), vec![2, 3])
        ]
    );
}

#[test]
fn located_elements_are_those_of_parse() {
    for text in [
        SCENE,
        "a /* b\n\nc */ d\n\nINT. X\n\nA\nB\n(c)\nd",
        "!x\n\n@bob\nhi",
    ] {
        let located: Vec<_> = parse_located(text).into_iter().map(|l| l.element).collect();

        assert_eq!(located, parse(text).elements, "{text:?}");
    }
}
