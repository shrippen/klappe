use super::{
    breaks_json, enter_json, kinds_text, looks_json, move_json, outline_json, print_json,
    report_json, suggest_json, tab_json, title_json, title_set, ImportReport,
};

/// Enter at the end of a heading opens an action after a blank line; at the end of a cue
/// the dialogue right below; in the middle of a line it is a plain break.
#[test]
fn enter_follows_the_element_flow() {
    let next = |line: &str, kind: &str| -> serde_json::Value {
        serde_json::from_str(&enter_json(line, line.chars().count(), kind, None)).unwrap()
    };

    assert_eq!(next("INT. SAAL", "h")["kind"], "a");
    assert_eq!(next("INT. SAAL", "h")["gap"], "blank");
    assert_eq!(next("ANNA", "c")["kind"], "d");
    assert_eq!(next("ANNA", "c")["gap"], "none");
    let middle: serde_json::Value =
        serde_json::from_str(&enter_json("ANNA", 2, "c", None)).unwrap();
    assert_eq!(middle["step"], "break");
}

/// Only the six elements of the flow take part: Enter leaves the others alone, Tab
/// counts them as action, and each flow element comes back as its own letter.
#[test]
fn the_flow_knows_six_letters() {
    let json = |s: String| -> serde_json::Value { serde_json::from_str(&s).unwrap() };
    let end = |line: &str, kind: &str| json(enter_json(line, line.chars().count(), kind, None));

    for letter in ["m", "t", " ", "s", "y", "n", "b", "l", "x", ""] {
        assert_eq!(end("Text", letter)["step"], "break", "Enter on {letter:?}");
        assert_eq!(json(tab_json("", letter))["kind"], "c", "Tab on {letter:?}");
    }
    assert_eq!(tab_json("", "h"), "");
    assert_eq!(tab_json("", "r"), "");
    assert_eq!(end("Text", "r")["kind"], "h");
    assert_eq!(end("Text", "d")["kind"], "a");
    assert_eq!(end("Text", "a")["kind"], "a");
    assert_eq!(end("(leise)", "p")["kind"], "d");
    assert_eq!(json(tab_json("", "p"))["kind"], "d");
    assert_eq!(json(tab_json("", "d"))["kind"], "p");
    assert_eq!(json(tab_json("", "a"))["kind"], "c");
    assert_eq!(json(tab_json("", "c"))["kind"], "a");
}

/// Tab turns an empty action line into a cue, an empty dialogue into `()`.
#[test]
fn tab_turns_an_empty_line() {
    let cue: serde_json::Value = serde_json::from_str(&tab_json("", "a")).unwrap();
    let paren: serde_json::Value = serde_json::from_str(&tab_json("", "d")).unwrap();

    assert_eq!(cue["kind"], "c");
    assert_eq!(paren["text"], "()");
    assert_eq!(paren["col"], 1);
    assert_eq!(tab_json("Text", "a"), "");
}
use klappe_services::{
    PageBreak, Placed, PrintFace, PrintUnderline, SceneSpan, ScriptPages, SheetPage,
};

/// Texts keep their place in points and their face.
#[test]
fn a_page_lists_its_texts_in_points() {
    let page = SheetPage {
        width_pt: 595.0,
        height_pt: 842.0,
        texts: vec![Placed {
            x_pt: 108.0,
            baseline_pt: 82.0,
            face: PrintFace::BoldItalic,
            underline: PrintUnderline::Yes,
            text: "INT. SAAL".into(),
        }],
    };

    let v: serde_json::Value = serde_json::from_str(&print_json(&[page])).unwrap();
    let text = &v["pages"][0]["texts"][0];
    assert_eq!(v["fontPt"], 12.0);
    assert_eq!(v["pages"][0]["height"], 842.0);
    assert_eq!(text["x"], 108.0);
    assert_eq!(text["y"], 82.0);
    assert_eq!(text["bold"], true);
    assert_eq!(text["italic"], true);
    assert_eq!(text["underline"], true);
}

/// The layout reaches the editor as the lines the pages start at and each scene's page;
/// without a layout it is empty.
#[test]
fn breaks_list_pages_and_scene_pages() {
    let pages = ScriptPages {
        breaks: vec![PageBreak {
            page: 2,
            label: "2".into(),
            line: 59,
        }],
        scenes: vec![
            SceneSpan {
                line: 0,
                page: 1,
                eighths: 4,
                length: "4/8".into(),
            },
            SceneSpan {
                line: 60,
                page: 2,
                eighths: 8,
                length: "1".into(),
            },
        ],
        pages: 2,
        eighths: 12,
        minutes: 1.5,
        length: "1 4/8".into(),
    };

    let v: serde_json::Value = serde_json::from_str(&breaks_json(Some(&pages))).unwrap();
    assert_eq!(v["pages"], 2);
    assert_eq!(v["breaks"][0]["page"], 2);
    assert_eq!(v["breaks"][0]["label"], "2");
    assert_eq!(v["breaks"][0]["line"], 59);
    assert_eq!(v["scenes"][1]["line"], 60);
    assert_eq!(v["scenes"][1]["page"], 2);

    let none: serde_json::Value = serde_json::from_str(&breaks_json(None)).unwrap();
    assert_eq!(none["pages"], 0);
    assert_eq!(none["breaks"].as_array().map(Vec::len), Some(0));
}

/// The import report carries what an FDX import could not keep, for the report dialog.
#[test]
fn the_import_report_names_the_lost_styles() {
    let report = ImportReport {
        scenes: 2,
        styles_lost: 3,
        ..ImportReport::default()
    };

    let v: serde_json::Value = serde_json::from_str(&report_json(&report)).unwrap();

    assert_eq!(v["scenes"], 2);
    assert_eq!(v["stylesLost"], 3);
}

/// A cue and its line get their own letter, and the looks place the cue as the page does.
#[test]
fn a_cue_is_set_in_the_columns_of_the_page() {
    assert_eq!(kinds_text("INT. SAAL\n\nANNA\nHallo.\n\nCUT TO:"), "h cd r");

    let v: serde_json::Value = serde_json::from_str(&looks_json()).unwrap();
    assert_eq!(v["columns"], 60);
    assert_eq!(v["looks"]["c"]["kind"], "character");
    assert_eq!(v["looks"]["c"]["x"], 22);
    assert_eq!(v["looks"]["r"]["align"], "right");
    assert!(v["looks"]["s"].get("x").is_none());
    assert_eq!(v["letters"]["blank"], " ");
    assert_eq!(v["letters"]["action"], "a");
    assert_eq!(v["letters"]["heading"], "h");
}

const BOOK: &str = "# Akt I\n\n= Es beginnt.\n\nINT. SAAL - TAG\n\nSie kommt an.\n\nEXT. HOF - NACHT\n\nEr geht.\n";

fn span(line: usize, eighths: usize, length: &str) -> SceneSpan {
    SceneSpan {
        line,
        page: 1,
        eighths,
        length: length.into(),
    }
}

/// The outline lists section, synopsis and scenes in text order; a section's length is the
/// total of its scenes.
#[test]
fn outline_lists_the_tree_with_lengths() {
    let pages = ScriptPages {
        breaks: vec![],
        scenes: vec![span(4, 4, "4/8"), span(8, 5, "5/8")],
        pages: 1,
        eighths: 9,
        minutes: 1.0,
        length: "1 1/8".into(),
    };

    let v: serde_json::Value =
        serde_json::from_str(&outline_json(BOOK, Some(&pages), &[])).unwrap();
    let kinds: Vec<_> = v
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["kind"].clone())
        .collect();
    assert_eq!(kinds, ["section", "synopsis", "scene", "scene"]);
    assert_eq!(v[0]["level"], 1);
    assert_eq!(v[0]["text"], "Akt I");
    assert_eq!(v[0]["length"], "1 1/8");
    assert_eq!(v[2]["line"], 4);
    assert_eq!(v[2]["depth"], 1);
    assert_eq!(v[3]["length"], "5/8");

    let none: serde_json::Value = serde_json::from_str(&outline_json(BOOK, None, &[])).unwrap();
    assert_eq!(none[0]["length"], "");
    assert_eq!(none[2]["length"], "");
}

/// Typing since the layout changed the scene count: lengths are found by heading line.
#[test]
fn outline_matches_a_stale_layout_by_line() {
    let pages = ScriptPages {
        breaks: vec![],
        scenes: vec![span(8, 5, "5/8")],
        pages: 1,
        eighths: 5,
        minutes: 1.0,
        length: "5/8".into(),
    };

    let v: serde_json::Value =
        serde_json::from_str(&outline_json(BOOK, Some(&pages), &[])).unwrap();
    assert_eq!(v[2]["length"], "");
    assert_eq!(v[3]["length"], "5/8");
    assert_eq!(v[0]["length"], "5/8");
}

/// A scene moves down past its neighbour; the answer names the lines to replace and where
/// the block stands then. A last scene, a synopsis and a way that is none do not move.
#[test]
fn a_move_names_the_lines_to_replace() {
    let v: serde_json::Value = serde_json::from_str(&move_json(BOOK, 4, "down")).unwrap();
    assert_eq!(v["from"], 4);
    assert_eq!(v["to"], 11);
    assert_eq!(v["line"], 8);
    let lines: Vec<_> = v["lines"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| l.as_str().unwrap())
        .collect();
    assert_eq!(lines[0], "EXT. HOF - NACHT");
    assert_eq!(lines[4], "INT. SAAL - TAG");

    assert_eq!(move_json(BOOK, 8, "down"), "");
    assert_eq!(move_json(BOOK, 2, "up"), "");
    assert_eq!(move_json(BOOK, 4, "left"), "");
    assert_eq!(move_json(BOOK, 3, "up"), "");
}

/// A folded section hides what it holds, synopsis and scenes; its entry says it is folded.
#[test]
fn folded_sections_leave_out_their_content() {
    let folded = ["1Akt I".to_string()];
    let v: serde_json::Value = serde_json::from_str(&outline_json(BOOK, None, &folded)).unwrap();

    assert_eq!(v.as_array().map(Vec::len), Some(1));
    assert_eq!(v[0]["key"], "1Akt I");
    assert_eq!(v[0]["folded"], true);
}

/// A move names the sections around the block where it lands, so the window can open them.
#[test]
fn a_move_names_the_sections_to_open() {
    let v: serde_json::Value = serde_json::from_str(&move_json(BOOK, 4, "down")).unwrap();

    assert_eq!(v["unfold"], serde_json::json!(["1Akt I"]));

    let two = "# A\n\nINT. X\n\n# B\n\nINT. Y\n";
    let up: serde_json::Value = serde_json::from_str(&move_json(two, 6, "up")).unwrap();
    assert_eq!(up["unfold"], serde_json::json!(["1A"]));
}

/// A cue prefix lists the names most frequent first, at most eight; a finished name goes on
/// to the dialogue, a time of day to the action; an empty cue after two speakers proposes
/// the other one.
#[test]
fn suggestions_list_names_and_say_what_follows() {
    let json = |s: String| -> serde_json::Value { serde_json::from_str(&s).unwrap() };
    let book = "A\n\nANNA\nx\n\nBEN\ny\n\nANNA\nz\n\n";

    let names = json(suggest_json(&format!("{book}A"), 11, "c", ""));
    assert_eq!(names["options"], serde_json::json!(["ANNA"]));
    assert_eq!(names["from"], 0);
    assert_eq!(names["finishes"], true);
    assert_eq!(names["kind"], "d");
    assert_eq!(names["gap"], "none");

    let other = json(suggest_json(book, 11, "c", ""));
    assert_eq!(other["options"], serde_json::json!(["BEN"]));

    let time = json(suggest_json("INT. SAAL - TA", 0, "h", ""));
    assert_eq!(time["options"][0], "TAG");
    assert_eq!(time["kind"], "a");
    assert_eq!(time["gap"], "blank");
    assert_eq!(time["from"], 12);

    assert_eq!(suggest_json("Sie kommt.", 0, "a", ""), "");
    assert_eq!(suggest_json("INT. SAAL", 5, "h", ""), "");

    // An opened heading line with prose is an action: no location offered.
    let book = "INT. GARTEN - TAG\n\nAnna liest.\n\nCUT TO:\n\nSie g";
    assert_eq!(suggest_json(book, 6, "a", "h"), "");
    assert!(!suggest_json("INT. GARTEN - TAG\n\nINT. G", 2, "h", "h").is_empty());

    // The open element counts only while the reader sees plain text: a line that became a
    // heading by what was typed is one, whatever Tab made it before.
    let book = "INT. GARTEN - TAG\n\nAnna liest.\n\nINT. G";
    assert!(!suggest_json(book, 4, "h", "c").is_empty());
}

#[test]
fn at_most_eight_suggestions_show() {
    let names: String = ('A'..='L').map(|c| format!("FIG{c}\nx\n\n")).collect();
    let found: serde_json::Value =
        serde_json::from_str(&suggest_json(&format!("{names}FIG"), 36, "c", "")).unwrap();

    assert_eq!(found["options"].as_array().map(Vec::len), Some(8));
}

/// The form reads the title page by key and writes it back; an emptied field leaves the
/// text, the body stays, and JSON that is no object changes nothing.
#[test]
fn the_title_page_goes_through_json() {
    let text = "Title: Heimat\nContact: A\n  B\n\nINT. SAAL\n";

    let v: serde_json::Value = serde_json::from_str(&title_json(text)).unwrap();
    assert_eq!(v["Title"], "Heimat");
    assert_eq!(v["Contact"], "A\nB");
    assert_eq!(v["Author"], "");

    let set = title_set(text, r#"{"Title":"Neu","Contact":"","Author":"Anna"}"#);
    let v: serde_json::Value = serde_json::from_str(&title_json(&set)).unwrap();
    assert_eq!(v["Title"], "Neu");
    assert_eq!(v["Author"], "Anna");
    assert_eq!(v["Contact"], "");
    assert!(set.ends_with("INT. SAAL\n"));
    assert!(!set.contains("Contact"));

    assert_eq!(title_set(text, "nonsense"), text);
}

/// An empty line in a field (Contact: "Eva", "", "Weg 1") does not reach the text, as in
/// the terminal's form.
#[test]
fn blank_lines_of_a_field_are_dropped() {
    let set = title_set("INT. SAAL\n", r#"{"Contact":"Eva\n\nWeg 1"}"#);

    let v: serde_json::Value = serde_json::from_str(&title_json(&set)).unwrap();
    assert_eq!(v["Contact"], "Eva\nWeg 1");
    assert_eq!(set, "Contact:\n   Eva\n   Weg 1\n\nINT. SAAL\n");
}
