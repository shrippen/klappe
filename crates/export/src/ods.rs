//! The shot list as OpenDocument spreadsheet: a ZIP of `mimetype`, a manifest and `content.xml`.

use crate::sheet::{escape, grid, Cell};
use crate::shots::Table;
use crate::zip::Zip;

const MIMETYPE: &str = "application/vnd.oasis.opendocument.spreadsheet";
const MANIFEST: &str = concat!(
    r#"<?xml version="1.0" encoding="UTF-8"?>"#,
    r#"<manifest:manifest xmlns:manifest="urn:oasis:names:tc:opendocument:xmlns:manifest:1.0" manifest:version="1.2">"#,
    r#"<manifest:file-entry manifest:full-path="/" manifest:media-type="application/vnd.oasis.opendocument.spreadsheet"/>"#,
    r#"<manifest:file-entry manifest:full-path="content.xml" manifest:media-type="text/xml"/>"#,
    r#"</manifest:manifest>"#,
);
const CONTENT_HEAD: &str = concat!(
    r#"<?xml version="1.0" encoding="UTF-8"?>"#,
    r#"<office:document-content xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0" "#,
    r#"xmlns:table="urn:oasis:names:tc:opendocument:xmlns:table:1.0" "#,
    r#"xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0" office:version="1.2">"#,
    r#"<office:body><office:spreadsheet>"#,
);
const CONTENT_TAIL: &str = "</office:spreadsheet></office:body></office:document-content>";

pub(crate) fn write(table: &Table, sheet_name: &str) -> Vec<u8> {
    let mut zip = Zip::default();

    // The mimetype comes first and stays uncompressed: that is how readers tell the format.
    zip.add("mimetype", MIMETYPE.as_bytes());
    zip.add("META-INF/manifest.xml", MANIFEST.as_bytes());
    zip.add("content.xml", content(table, sheet_name).as_bytes());

    zip.finish()
}

fn content(table: &Table, sheet_name: &str) -> String {
    let mut out = String::from(CONTENT_HEAD);
    out.push_str(&format!(
        r#"<table:table table:name="{}">"#,
        escape(sheet_name)
    ));

    for row in grid(table) {
        out.push_str("<table:table-row>");
        for cell in row {
            out.push_str(&cell_xml(cell));
        }
        out.push_str("</table:table-row>");
    }

    out.push_str("</table:table>");
    out.push_str(CONTENT_TAIL);

    out
}

fn cell_xml(cell: Option<Cell>) -> String {
    match cell {
        None => "<table:table-cell/>".to_owned(),
        Some(Cell::Number(n)) => {
            format!(
                r#"<table:table-cell office:value-type="float" office:value="{n}"><text:p>{n}</text:p></table:table-cell>"#
            )
        }
        Some(Cell::Text(t)) => {
            // One paragraph per line.
            let paragraphs: String = t
                .lines()
                .map(|line| format!("<text:p>{}</text:p>", escape(line)))
                .collect();

            format!(
                r#"<table:table-cell office:value-type="string">{paragraphs}</table:table-cell>"#
            )
        }
    }
}
