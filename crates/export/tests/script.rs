//! The screenplay PDF: Courier pages, title page, scene numbers.
#![allow(clippy::unwrap_used)]

use klappe_export::{
    script_pdf, Face, Paper, SceneNumbers, ScriptLine, ScriptPage, ScriptSheet, TitlePage,
};

fn line(row: usize, col: usize, text: &str) -> ScriptLine {
    ScriptLine {
        row,
        col,
        text: text.into(),
        face: Face::Regular,
        scene_number: None,
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
