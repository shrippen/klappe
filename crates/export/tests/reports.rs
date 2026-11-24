//! The reports of DB-61 as CSV, ODS, XLSX and PDF.
#![allow(clippy::unwrap_used)]

use klappe_export::{
    extract_pdf, report_csv, report_ods, report_pdf, report_xlsx, ExportError, ExtractBox,
    ExtractPage, ReportTable, Theme,
};

const LOCAL_SIGNATURE: [u8; 4] = [0x50, 0x4B, 0x03, 0x04];
const A4_LANDSCAPE: &str = "/MediaBox [0 0 841.89 595.28]";
const A4_PORTRAIT: &str = "/MediaBox [0 0 595.28 841.89]";

fn table(rows: usize) -> ReportTable {
    ReportTable {
        title: "Szenen".into(),
        head: vec!["Nr.".into(), "Kopf".into(), "Figuren".into()],
        rows: (1..=rows)
            .map(|n| {
                vec![
                    format!("{n}A"),
                    format!("INT. KÜCHE & <Flur> {n}"),
                    "Anna, \"Bo\"".into(),
                ]
            })
            .collect(),
    }
}

fn page(number: &str, boxes: Vec<ExtractBox>) -> ExtractPage {
    ExtractPage {
        title: format!("{number}  INT. KÜCHE - TAG"),
        facts: vec!["Spieltag: Tag 3".into(), "Vorstopp: 1:30".into()],
        boxes,
    }
}

fn boxed(title: &str, lines: usize) -> ExtractBox {
    ExtractBox {
        title: title.into(),
        lines: (1..=lines).map(|n| format!("Ding {n} (offen)")).collect(),
    }
}

fn sheets_in(bytes: &[u8]) -> usize {
    String::from_utf8_lossy(bytes)
        .matches("/Type /Page /")
        .count()
}

/// Data of the zip entry `name` (stored, not compressed); empty when there is none.
fn entry(zip: &[u8], name: &str) -> String {
    let u16_at = |at: usize| usize::from(u16::from_le_bytes([zip[at], zip[at + 1]]));
    let u32_at = |at: usize| u32::from_le_bytes(zip[at..at + 4].try_into().unwrap()) as usize;

    let mut at = 0;
    while zip[at..at + 4] == LOCAL_SIGNATURE {
        let size = u32_at(at + 18);
        let name_len = u16_at(at + 26);
        let name_at = at + 30;
        let data_at = name_at + name_len + u16_at(at + 28);
        if zip[name_at..name_at + name_len] == *name.as_bytes() {
            return String::from_utf8(zip[data_at..data_at + size].to_vec()).unwrap();
        }
        at = data_at + size;
    }

    String::new()
}

#[test]
fn csv_has_the_head_and_a_row_per_line() {
    let csv = report_csv(&table(2));

    assert_eq!(
        csv,
        "Nr.,Kopf,Figuren\r\n\
         1A,INT. KÜCHE & <Flur> 1,\"Anna, \"\"Bo\"\"\"\r\n\
         2A,INT. KÜCHE & <Flur> 2,\"Anna, \"\"Bo\"\"\"\r\n"
    );
}

#[test]
fn ods_holds_the_cells_as_text() {
    let zip = report_ods(&table(2)).unwrap();

    assert_eq!(&zip[30..38], b"mimetype");
    let content = entry(&zip, "content.xml");
    assert!(content.contains(r#"table:name="Szenen""#));
    assert!(content.contains("INT. KÜCHE &amp; &lt;Flur&gt; 2"));
    // A number of a scene is a name: 1A, never a number cell.
    assert!(content.contains("<text:p>1A</text:p>"));
    assert!(!content.contains("office:value-type=\"float\""));
}

#[test]
fn xlsx_holds_the_cells() {
    let zip = report_xlsx(&table(2)).unwrap();

    assert!(entry(&zip, "xl/workbook.xml").contains(r#"name="Szenen""#));
    let sheet = entry(&zip, "xl/worksheets/sheet1.xml");
    assert!(sheet.contains("Nr."));
    assert!(sheet.contains("INT. KÜCHE &amp; &lt;Flur&gt; 1"));
    assert!(sheet.contains("1A"));
}

#[test]
fn a_table_is_a_pdf_on_a4_landscape() {
    let pdf = report_pdf(&table(3), Theme::Light).unwrap();

    assert!(pdf.starts_with(b"%PDF"));
    assert_eq!(sheets_in(&pdf), 1);
    assert!(String::from_utf8_lossy(&pdf).contains(A4_LANDSCAPE));
}

/// The head row stands on every sheet: the head's text is in the file once per sheet.
#[test]
fn a_long_table_runs_over_sheets_with_its_head() {
    let pdf = report_pdf(&table(120), Theme::Light).unwrap();

    let sheets = sheets_in(&pdf);
    assert!(sheets > 1, "{sheets}");
    assert_eq!(
        String::from_utf8_lossy(&pdf).matches("(Kopf)").count(),
        sheets
    );
}

#[test]
fn an_empty_table_still_prints_its_head() {
    let pdf = report_pdf(&table(0), Theme::Light).unwrap();

    assert_eq!(sheets_in(&pdf), 1);
    assert!(String::from_utf8_lossy(&pdf).contains("(Kopf)"));
}

#[test]
fn the_extract_has_a_portrait_page_per_scene() {
    let pages = [
        page("1", vec![boxed("Requisiten", 2), boxed("Kostüm", 1)]),
        page("2", vec![boxed("Figuren", 1)]),
    ];

    let pdf = extract_pdf(&pages, Theme::Light).unwrap();

    assert!(pdf.starts_with(b"%PDF"));
    assert_eq!(sheets_in(&pdf), 2);
    let text = String::from_utf8_lossy(&pdf);
    assert!(text.contains(A4_PORTRAIT));
    assert!(text.contains("(Requisiten)"));
    assert!(text.contains("Spieltag: Tag 3"));
}

#[test]
fn a_box_longer_than_a_page_goes_on_to_the_next() {
    let pdf = extract_pdf(&[page("1", vec![boxed("Requisiten", 150)])], Theme::Light).unwrap();

    assert!(sheets_in(&pdf) > 1);
    assert!(String::from_utf8_lossy(&pdf).contains("(Ding 150 \\(offen\\))"));
}

#[test]
fn an_extract_without_scenes_is_refused() {
    assert!(matches!(
        extract_pdf(&[], Theme::Light),
        Err(ExportError::EmptyPlan)
    ));
}
