//! The screenplay PDF: Courier pages, title page, scene numbers.
#![allow(clippy::unwrap_used)]

use klappe_export::{
    script_pdf, script_sheet_pages, Face, Paper, Revised, SceneNumbers, ScriptLine, ScriptPage,
    ScriptSheet, TitlePage,
};

fn line(row: usize, col: usize, text: &str) -> ScriptLine {
    ScriptLine {
        row,
        col,
        text: text.into(),
        face: Face::Regular,
        scene_number: None,
        revised: Revised::No,
    }
}

fn sheet() -> ScriptSheet {
    let mut heading = line(0, 0, "INT. ROOM - DAY");
    heading.face = Face::Bold;
    heading.scene_number = Some("12".into());

    ScriptSheet {
        paper: Paper::A4,
        columns: 60,
        title_page: TitlePage::default(),
        pages: vec![
            ScriptPage {
                label: None,
                lines: vec![heading, line(2, 22, "ANNA")],
            },
            ScriptPage {
                label: Some("2.".into()),
                lines: vec![line(0, 10, "Hello (again).")],
            },
        ],
        scene_numbers: SceneNumbers::Hidden,
        header: None,
        revision: None,
    }
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[test]
fn a_pdf_with_one_page_per_page() {
    let bytes = script_pdf(&sheet());
    let pdf = text(&bytes);

    assert!(bytes.starts_with(b"%PDF-1.4"));
    assert!(pdf.contains("/Count 2"), "{pdf}");
    assert!(pdf.contains("/BaseFont /Courier "));
    assert!(pdf.contains("/BaseFont /Courier-Bold "));
    assert!(pdf.contains("(Hello \\(again\\).) Tj"));
    assert!(pdf.trim_end().ends_with("%%EOF"));
    // Standard fonts only: nothing is embedded.
    assert!(!pdf.contains("FontFile"));
}

#[test]
fn the_page_number_stands_top_right_from_page_two() {
    let pdf = text(&script_pdf(&sheet()));

    // 540 pt (right edge of the text) less two columns of 7.2 pt, 46 pt from the top.
    assert_eq!(pdf.matches("(2.) Tj").count(), 1);
    assert!(pdf.contains("525.6 795.89 Td (2.) Tj"), "{pdf}");
}

#[test]
fn scene_numbers_stand_left_and_right_when_asked_for() {
    let mut s = sheet();
    assert_eq!(text(&script_pdf(&s)).matches("(12) Tj").count(), 0);

    s.scene_numbers = SceneNumbers::Shown;
    let pdf = text(&script_pdf(&s));

    assert_eq!(pdf.matches("(12) Tj").count(), 2);
    // Left of the text (108 pt less a gap and two columns) and right of it (540 pt + gap).
    assert!(pdf.contains("86.4 759.89 Td (12) Tj"), "{pdf}");
    assert!(pdf.contains("547.2 759.89 Td (12) Tj"), "{pdf}");
}

#[test]
fn the_title_page_comes_first_and_the_header_is_optional() {
    let mut s = sheet();
    s.title_page = TitlePage {
        title: "Nachtwache".into(),
        author: "Studio Weber".into(),
        draft_date: "2026-11-16".into(),
        contact: "Studio Weber\nHamburg".into(),
        ..TitlePage::default()
    };
    s.header = Some("Nachtwache - Draft 2".into());
    let pdf = text(&script_pdf(&s));

    assert!(pdf.contains("/Count 3"), "{pdf}");
    assert!(pdf.contains("(NACHTWACHE) Tj"));
    assert!(pdf.contains("(Studio Weber) Tj"));
    assert!(pdf.contains("(Hamburg) Tj"));
    assert!(pdf.contains("(2026-11-16) Tj"));
    // The header is on the two script pages, not on the title page.
    assert_eq!(pdf.matches("(Nachtwache - Draft 2) Tj").count(), 2);
}

#[test]
fn an_empty_script_is_one_blank_page() {
    let mut s = sheet();
    s.pages.clear();

    assert!(text(&script_pdf(&s)).contains("/Count 1"));
}

#[test]
fn every_paper_has_its_size() {
    let sizes = [
        (Paper::A4, "595.28 841.89"),
        (Paper::A3, "841.89 1190.55"),
        (Paper::Letter, "612 792"),
    ];

    for (paper, size) in sizes {
        let mut s = sheet();
        s.paper = paper;

        let pdf = text(&script_pdf(&s));

        assert!(
            pdf.contains(&format!("/MediaBox [0 0 {size}]")),
            "{paper:?}: {pdf}"
        );
    }
}

#[test]
fn the_text_width_moves_the_right_edge() {
    let mut s = sheet();
    s.columns = 50;
    s.scene_numbers = SceneNumbers::Shown;

    let pdf = text(&script_pdf(&s));

    // 108 pt + 50 columns = 468 pt; page number two columns left, scene number a gap right.
    assert!(pdf.contains("453.6 795.89 Td (2.) Tj"), "{pdf}");
    assert!(pdf.contains("475.2 759.89 Td (12) Tj"), "{pdf}");
}

/// D-15: the print view gets the PDF's places: the cue 22 columns right of the 1.5 inch
/// margin, on the baseline of row 2 below the 1 inch top; the title page first.
#[test]
fn the_pages_place_each_text_as_the_pdf_does() {
    let mut with_title = sheet();
    with_title.title_page.title = "Nachtwache".into();

    let pages = script_sheet_pages(&with_title);
    let anna = pages[1].texts.iter().find(|t| t.text == "ANNA").unwrap();

    assert_eq!(pages.len(), 3);
    assert_eq!(pages[0].texts[0].text, "NACHTWACHE");
    assert!((anna.x_pt - (108.0 + 22.0 * 7.2)).abs() < 1e-9);
    assert!((anna.baseline_pt - (72.0 + 2.0 * 12.0 + 10.0)).abs() < 1e-9);
    assert!(pdf_has(&with_title, "266.4 735.89 Td (ANNA)"));
}

fn pdf_has(sheet: &ScriptSheet, ops: &str) -> bool {
    text(&script_pdf(sheet)).contains(ops)
}

#[test]
fn a_revised_line_has_a_mark_in_the_right_margin_and_the_page_a_revision_line() {
    let mut s = sheet();
    s.pages[1].lines[0].revised = Revised::Yes;
    s.revision = Some("Revision Blau · 2026-11-25".into());
    let pages = script_sheet_pages(&s);

    let marks = |page: usize| {
        pages[page]
            .texts
            .iter()
            .filter(|t| t.text == "*")
            .collect::<Vec<_>>()
    };
    assert!(marks(0).is_empty());
    let [mark] = marks(1)[..] else {
        panic!("one mark expected");
    };
    // Right of the text area (108 pt + 60 columns), on the row of the line.
    assert!(mark.x_pt > 108.0 + 60.0 * 7.2);
    let hello = pages[1]
        .texts
        .iter()
        .find(|t| t.text.starts_with("Hello"))
        .unwrap();
    assert_eq!(mark.baseline_pt, hello.baseline_pt);

    // The revision line is on both pages, above the page number's line.
    for page in &pages {
        let revision = page
            .texts
            .iter()
            .find(|t| t.text.starts_with("Revision"))
            .unwrap();
        assert!(revision.baseline_pt < 46.0);
    }
}

/// DB-41: the mark ends 10 mm before the paper's edge (a printer does not reach it) and never
/// lies on a scene number.
#[test]
fn the_revision_mark_stays_on_the_paper_and_off_the_numbers() {
    const MM_PT: f64 = 72.0 / 25.4;
    const CHAR_PT: f64 = 7.2;

    for paper in [Paper::A4, Paper::Letter] {
        let mut s = sheet();
        s.paper = paper;
        s.scene_numbers = SceneNumbers::Shown;
        s.pages[0].lines[0].revised = Revised::Yes;
        s.pages[0].lines[0].scene_number = Some("12A".into());
        s.pages[0].lines[1].revised = Revised::Yes;
        let pages = script_sheet_pages(&s);
        let page = &pages[0];

        let marks: Vec<_> = page.texts.iter().filter(|t| t.text == "*").collect();
        assert_eq!(marks.len(), 2, "{paper:?}");
        for mark in marks {
            assert!(
                mark.x_pt + CHAR_PT <= page.width_pt - 10.0 * MM_PT,
                "{paper:?}: ends at {}",
                mark.x_pt + CHAR_PT
            );
            let numbers = page
                .texts
                .iter()
                .filter(|t| t.text == "12A" && t.baseline_pt == mark.baseline_pt);
            for number in numbers {
                let apart =
                    mark.x_pt + CHAR_PT <= number.x_pt || mark.x_pt >= number.x_pt + 3.0 * CHAR_PT;
                assert!(apart, "{paper:?}: mark at {} on number", mark.x_pt);
            }
        }
    }
}
