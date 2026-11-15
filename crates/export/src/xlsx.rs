//! The shot list as Excel workbook (SpreadsheetML): a ZIP with one worksheet, text inline.

use crate::sheet::{escape, grid, Cell};
use crate::shots::Table;
use crate::zip::Zip;

const CONTENT_TYPES: &str = concat!(
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
    r#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">"#,
    r#"<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>"#,
    r#"<Default Extension="xml" ContentType="application/xml"/>"#,
    r#"<Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>"#,
    r#"<Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>"#,
    r#"</Types>"#,
);
const ROOT_RELS: &str = concat!(
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
    r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">"#,
    r#"<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>"#,
    r#"</Relationships>"#,
);
const WORKBOOK_RELS: &str = concat!(
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
    r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">"#,
    r#"<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>"#,
    r#"</Relationships>"#,
);
const SHEET_HEAD: &str = concat!(
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
    r#"<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData>"#,
);
const SHEET_TAIL: &str = "</sheetData></worksheet>";

/// Excel refuses longer sheet names and these characters.
const MAX_NAME_CHARS: usize = 31;
const FORBIDDEN_IN_NAME: [char; 7] = ['[', ']', ':', '*', '?', '/', '\\'];
const FALLBACK_NAME: &str = "Sheet1";
const LETTERS: usize = 26;

pub(crate) fn write(table: &Table, sheet_name: &str) -> Vec<u8> {
    let mut zip = Zip::default();

    zip.add("[Content_Types].xml", CONTENT_TYPES.as_bytes());
    zip.add("_rels/.rels", ROOT_RELS.as_bytes());
    zip.add("xl/workbook.xml", workbook(sheet_name).as_bytes());
    zip.add("xl/_rels/workbook.xml.rels", WORKBOOK_RELS.as_bytes());
    zip.add("xl/worksheets/sheet1.xml", sheet(table).as_bytes());

    zip.finish()
}

fn workbook(sheet_name: &str) -> String {
    format!(
        concat!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
            r#"<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" "#,
            r#"xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">"#,
            r#"<sheets><sheet name="{}" sheetId="1" r:id="rId1"/></sheets></workbook>"#,
        ),
        escape(&valid_name(sheet_name))
    )
}

fn valid_name(name: &str) -> String {
    let clean: String = name
        .chars()
        .filter(|c| !FORBIDDEN_IN_NAME.contains(c))
        .take(MAX_NAME_CHARS)
        .collect();
    if clean.trim().is_empty() {
        return FALLBACK_NAME.to_owned();
    }

    clean
}

fn sheet(table: &Table) -> String {
    let mut out = String::from(SHEET_HEAD);

    for (r, row) in grid(table).into_iter().enumerate() {
        out.push_str(&format!(r#"<row r="{}">"#, r + 1));
        for (c, cell) in row.into_iter().enumerate() {
            let Some(cell) = cell else {
                continue;
            };
            out.push_str(&cell_xml(&format!("{}{}", column(c), r + 1), &cell));
        }
        out.push_str("</row>");
    }

    out.push_str(SHEET_TAIL);

    out
}

fn cell_xml(at: &str, cell: &Cell) -> String {
    match cell {
        Cell::Number(n) => format!(r#"<c r="{at}"><v>{n}</v></c>"#),
        Cell::Text(t) => format!(
            r#"<c r="{at}" t="inlineStr"><is><t xml:space="preserve">{}</t></is></c>"#,
            escape(t)
        ),
    }
}

/// Column letters: 0 is A, 25 is Z, 26 is AA.
fn column(index: usize) -> String {
    let mut letters = Vec::new();
    let mut rest = index + 1;
    while rest > 0 {
        letters.push(b'A' + ((rest - 1) % LETTERS) as u8);
        rest = (rest - 1) / LETTERS;
    }
    letters.reverse();

    String::from_utf8_lossy(&letters).into_owned()
}
