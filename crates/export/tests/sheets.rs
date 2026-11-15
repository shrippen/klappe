//! The shot list as ODS and XLSX (LH-85): the container, the XML and the cells.
#![allow(clippy::unwrap_used)]

use klappe_export::{shots_ods, shots_xlsx, ShotColumns, ShotOrder, ShotTexts};
use klappe_model::{Camera, CameraMark, FloorPlan, Movement, Point, Rig, Sensor, Shot, ShotSize};

const LOCAL_SIGNATURE: [u8; 4] = [0x50, 0x4B, 0x03, 0x04];
const END_SIGNATURE: [u8; 4] = [0x50, 0x4B, 0x05, 0x06];

fn texts() -> ShotTexts {
    ShotTexts {
        title: "Shotliste".into(),
        columns: ShotColumns {
            setup: "Setup".into(),
            number: "Nummer".into(),
            camera: "Kamera".into(),
            lens_mm: "Objektiv (mm)".into(),
            size: "Größe".into(),
            angle: "Perspektive".into(),
            movement: "Bewegung".into(),
            from_s: "Von (s)".into(),
            to_s: "Bis (s)".into(),
            duration_s: "Dauer (s)".into(),
            description: "Inhalt".into(),
            props: "Requisiten".into(),
            gear: "Technik".into(),
            crew: "Team".into(),
            note: "Notiz".into(),
        },
        sizes: vec![(ShotSize::Wide, "Totale".into())],
        angles: Vec::new(),
        movements: vec![(Movement::Pan, "Schwenk".into())],
    }
}

fn plan(shots: usize) -> FloorPlan {
    let mut plan = FloorPlan::new("a", "Plan A");
    plan.cameras = vec![Camera {
        id: "a".into(),
        label: "A".into(),
        sensor: Sensor::Super35,
        lens_mm: 35.0,
        aspect: 2.39,
        marks: vec![CameraMark {
            at_s: 0.0,
            at: Point::new(8.0, 12.0),
            facing_deg: -90.0,
            height_m: 1.6,
            tilt_deg: 0.0,
            hold_s: 0.0,
            via: None,
            rig: Rig::default(),
            lens_mm: None,
        }],
        follows: None,
    }];
    plan.shots = (1..=shots)
        .map(|n| Shot {
            id: format!("s{n}"),
            number: format!("12.{n}"),
            camera: "a".into(),
            size: ShotSize::Wide,
            angle: Default::default(),
            movement: Movement::Pan,
            lens_mm: None,
            from_s: n as f64,
            to_s: n as f64 + 2.5,
            description: format!("Brief {n} & <Kerze> \"Licht\""),
            props: String::new(),
            gear: "Dolly".into(),
            crew: "Kamera, Ton".into(),
            note: "Zeile 1\nZeile 2".into(),
        })
        .collect();

    plan
}

/// Name and data of every entry, read through the local headers; also checks the end record
/// counts them and the central directory sits where it says.
fn entries(zip: &[u8]) -> Vec<(String, Vec<u8>)> {
    let u16_at = |at: usize| usize::from(u16::from_le_bytes([zip[at], zip[at + 1]]));
    let u32_at = |at: usize| u32::from_le_bytes(zip[at..at + 4].try_into().unwrap()) as usize;

    let mut out = Vec::new();
    let mut at = 0;
    while zip[at..at + 4] == LOCAL_SIGNATURE {
        let size = u32_at(at + 18);
        let name_at = at + 30;
        let name_len = u16_at(at + 26);
        let data_at = name_at + name_len + u16_at(at + 28);
        let name = String::from_utf8(zip[name_at..name_at + name_len].to_vec()).unwrap();

        assert_eq!(u16_at(at + 8), 0, "{name} must be stored");
        assert_eq!(u32_at(at + 22), size);
        out.push((name, zip[data_at..data_at + size].to_vec()));
        at = data_at + size;
    }

    let end = zip.len() - 22;
    assert_eq!(zip[end..end + 4], END_SIGNATURE);
    assert_eq!(u16_at(end + 10), out.len());
    assert_eq!(u32_at(end + 16), at, "central directory offset");
    assert_eq!(u32_at(end + 12), end - at, "central directory size");
    assert_eq!(zip[at..at + 4], [0x50, 0x4B, 0x01, 0x02]);

    out
}

fn text_of(entries: &[(String, Vec<u8>)], name: &str) -> String {
    let (_, data) = entries.iter().find(|(n, _)| n == name).unwrap();

    String::from_utf8(data.clone()).unwrap()
}

/// Tags open and close in order, attributes are quoted, `&` starts an entity.
fn assert_well_formed(xml: &str) {
    let mut stack: Vec<&str> = Vec::new();
    let mut rest = xml;
    if let Some(declaration) = rest.strip_prefix("<?xml") {
        rest = &declaration[declaration.find("?>").unwrap() + 2..];
    }

    while let Some(open) = rest.find('<') {
        let text = &rest[..open];
        for (i, _) in text.match_indices('&') {
            let entity = &text[i..];
            assert!(
                ["&amp;", "&lt;", "&gt;", "&quot;", "&apos;"]
                    .iter()
                    .any(|e| entity.starts_with(e)),
                "bare & in {text:?}"
            );
        }
        assert!(!text.contains('>'), "stray > in {text:?}");

        let close = rest[open..].find('>').unwrap() + open;
        let tag = &rest[open + 1..close];
        assert_eq!(tag.matches('"').count() % 2, 0, "quotes in <{tag}>");
        let name = tag.split([' ', '/']).find(|s| !s.is_empty()).unwrap();
        match tag {
            t if t.starts_with('/') => assert_eq!(stack.pop(), Some(name), "closing {t}"),
            t if t.ends_with('/') => {}
            _ => stack.push(name),
        }
        rest = &rest[close + 1..];
    }

    assert!(stack.is_empty(), "unclosed {stack:?}");
}

#[test]
fn ods_holds_the_required_files_mimetype_first() {
    let zip = shots_ods(&plan(2), &texts(), ShotOrder::Cut);
    let entries = entries(&zip);

    let names: Vec<&str> = entries.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(names, ["mimetype", "META-INF/manifest.xml", "content.xml"]);
    assert_eq!(
        text_of(&entries, "mimetype"),
        "application/vnd.oasis.opendocument.spreadsheet"
    );
    // The mimetype sits at a fixed place: readers sniff it at byte 38.
    assert_eq!(&zip[30..38], b"mimetype");
    assert_well_formed(&text_of(&entries, "META-INF/manifest.xml"));
    assert_well_formed(&text_of(&entries, "content.xml"));
}

#[test]
fn ods_cells_follow_the_csv() {
    let zip = shots_ods(&plan(2), &texts(), ShotOrder::Cut);
    let content = text_of(&entries(&zip), "content.xml");

    assert_eq!(content.matches("<table:table-row>").count(), 3);
    assert!(content.contains(r#"table:name="Shotliste""#));
    // Head, in the CSV's order.
    let head = ["Nummer", "Kamera", "Objektiv (mm)", "Größe", "Notiz"];
    let positions: Vec<usize> = head
        .iter()
        .map(|h| content.find(&format!("<text:p>{h}</text:p>")).unwrap())
        .collect();
    assert!(positions.windows(2).all(|w| w[0] < w[1]));
    // Numbers are numbers, the number column stays text.
    assert!(content.contains(r#"office:value-type="float" office:value="35""#));
    assert!(content.contains(r#"office:value="3.5""#));
    assert!(content.contains(r#"office:value-type="string"><text:p>12.1</text:p>"#));
    // Escapes and line breaks.
    assert!(content.contains("Brief 1 &amp; &lt;Kerze&gt; &quot;Licht&quot;"));
    assert!(content.contains("<text:p>Zeile 1</text:p><text:p>Zeile 2</text:p>"));
}

#[test]
fn xlsx_holds_the_required_files() {
    let zip = shots_xlsx(&plan(2), &texts(), ShotOrder::Cut);
    let entries = entries(&zip);

    let names: Vec<&str> = entries.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(
        names,
        [
            "[Content_Types].xml",
            "_rels/.rels",
            "xl/workbook.xml",
            "xl/_rels/workbook.xml.rels",
            "xl/worksheets/sheet1.xml"
        ]
    );
    for (name, data) in &entries {
        assert_well_formed(std::str::from_utf8(data).unwrap_or_else(|_| panic!("{name}")));
    }
    assert!(text_of(&entries, "xl/workbook.xml").contains(r#"<sheet name="Shotliste""#));
}

#[test]
fn xlsx_cells_follow_the_csv() {
    let zip = shots_xlsx(&plan(2), &texts(), ShotOrder::Cut);
    let sheet = text_of(&entries(&zip), "xl/worksheets/sheet1.xml");

    assert_eq!(sheet.matches("<row ").count(), 3);
    assert!(sheet.contains(r#"<c r="A1" t="inlineStr"><is><t xml:space="preserve">Nummer</t>"#));
    assert!(sheet.contains(r#"<c r="N1" t="inlineStr">"#));
    assert!(sheet.contains(r#"<c r="A2" t="inlineStr"><is><t xml:space="preserve">12.1</t>"#));
    assert!(sheet.contains(r#"<c r="C2"><v>35</v></c>"#));
    assert!(sheet.contains(r#"<c r="H3"><v>4.5</v></c>"#));
    assert!(sheet.contains("Brief 1 &amp; &lt;Kerze&gt; &quot;Licht&quot;"));
}

#[test]
fn shooting_order_adds_the_setup_column_as_number() {
    let zip = shots_xlsx(&plan(2), &texts(), ShotOrder::Shooting);
    let sheet = text_of(&entries(&zip), "xl/worksheets/sheet1.xml");

    assert!(sheet.contains(r#"<c r="A1" t="inlineStr"><is><t xml:space="preserve">Setup</t>"#));
    assert!(sheet.contains(r#"<c r="A2"><v>1</v></c>"#));
    assert!(sheet.contains(r#"<c r="B2" t="inlineStr"><is><t xml:space="preserve">12.1</t>"#));
    assert!(sheet.contains(r#"<c r="O1""#));
}

#[test]
fn without_shots_only_the_head_row_remains() {
    let ods = text_of(
        &entries(&shots_ods(&plan(0), &texts(), ShotOrder::Cut)),
        "content.xml",
    );
    let xlsx = text_of(
        &entries(&shots_xlsx(&plan(0), &texts(), ShotOrder::Cut)),
        "xl/worksheets/sheet1.xml",
    );

    assert_eq!(ods.matches("<table:table-row>").count(), 1);
    assert_eq!(xlsx.matches("<row ").count(), 1);
}

#[test]
fn a_sheet_name_excel_would_refuse_is_cleaned() {
    let mut texts = texts();
    texts.title = "Szene 1/2: [Plan] mit einem sehr langen Namen".into();

    let zip = shots_xlsx(&plan(0), &texts, ShotOrder::Cut);
    let workbook = text_of(&entries(&zip), "xl/workbook.xml");

    assert!(
        workbook.contains(r#"<sheet name="Szene 12 Plan mit einem sehr la""#),
        "{workbook}"
    );
}

/// Needs LibreOffice: it reads both files and writes the same CSV rows.
/// `cargo test -p klappe-export -- --ignored`
#[test]
#[ignore = "needs soffice"]
fn libreoffice_reads_both_files() {
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("sheets-soffice");
    std::fs::create_dir_all(&dir).unwrap();
    let plan = plan(2);
    std::fs::write(
        dir.join("ods.ods"),
        shots_ods(&plan, &texts(), ShotOrder::Cut),
    )
    .unwrap();
    std::fs::write(
        dir.join("xlsx.xlsx"),
        shots_xlsx(&plan, &texts(), ShotOrder::Cut),
    )
    .unwrap();

    for name in ["ods.ods", "xlsx.xlsx"] {
        let status = std::process::Command::new("soffice")
            .args(["--headless", "--convert-to", "csv", "--outdir"])
            .arg(&dir)
            .arg(dir.join(name))
            .env("HOME", &dir)
            .status()
            .unwrap();
        assert!(status.success());
    }

    for name in ["ods.csv", "xlsx.csv"] {
        let csv = std::fs::read_to_string(dir.join(name)).unwrap();
        assert!(
            csv.starts_with("Nummer,Kamera,Objektiv (mm),Größe,"),
            "{name}: {csv}"
        );
        assert!(csv.contains("12.1,A,35,Totale,"), "{name}: {csv}");
        assert!(csv.contains("\"Zeile 1\nZeile 2\""), "{name}: {csv}");
    }
}
