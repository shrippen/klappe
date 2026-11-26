//! The screenplay in the window: its text for the editor, and the import.

use super::{q, qobject};
use crate::fileurl;
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use klappe_services::{
    enter_step, error_text, tab_step, text_kinds, typed_caps, Align, EnterStep, Gap, HeaderLine,
    ImportReport, Laid, PrintFace, Revisions, SceneNumbers, ScriptPages, ScriptPrint, SheetPage,
    Slot, TextKind, SCRIPT_ASCENT_PT, SCRIPT_FONT_PT, TEXT_COLUMNS,
};
use serde_json::{json, Map, Value};

/// The letter of each line kind for `scriptKinds` and its name in `scriptLooks`.
const KINDS: [(TextKind, char, &str); 14] = [
    (TextKind::Blank, ' ', "blank"),
    (TextKind::Title, 't', "title"),
    (TextKind::Heading, 'h', "heading"),
    (TextKind::Action, 'a', "action"),
    (TextKind::Character, 'c', "character"),
    (TextKind::Parenthetical, 'p', "parenthetical"),
    (TextKind::Dialogue, 'd', "dialogue"),
    (TextKind::Transition, 'r', "transition"),
    (TextKind::Centered, 'm', "centered"),
    (TextKind::Lyrics, 'l', "lyrics"),
    (TextKind::Section, 's', "section"),
    (TextKind::Synopsis, 'y', "synopsis"),
    (TextKind::Note, 'n', "note"),
    (TextKind::PageBreak, 'b', "pageBreak"),
];

fn letter(kind: TextKind) -> char {
    KINDS
        .iter()
        .find(|(k, _, _)| *k == kind)
        .map_or(' ', |(_, letter, _)| *letter)
}

fn slot_of(letter: &str) -> Option<Slot> {
    let first = letter.chars().next()?;

    KINDS.iter().find(|(_, l, _)| *l == first)?.0.slot()
}

fn slot_letter(slot: Slot) -> String {
    letter(TextKind::of_slot(slot)).to_string()
}

/// What Enter does, as `enterStep` describes it.
pub(super) fn enter_json(line: &str, col: usize, kind: &str, below: Option<&str>) -> String {
    let below_blank = below.is_none_or(|b| b.trim().is_empty());

    match enter_step(line, col, slot_of(kind), below_blank) {
        EnterStep::Break => json!({ "step": "break" }),
        EnterStep::Retag => json!({ "step": "retag" }),
        EnterStep::GoBelow => json!({ "step": "below" }),
        EnterStep::Next(slot, gap) => json!({
            "step": "next",
            "kind": slot_letter(slot),
            "gap": match gap { Gap::Blank => "blank", Gap::None => "none" },
        }),
    }
    .to_string()
}

/// What Tab does, as `tabStep` describes it; empty for nothing.
pub(super) fn tab_json(line: &str, kind: &str) -> String {
    let Some(step) = tab_step(line, slot_of(kind)) else {
        return String::new();
    };

    json!({ "kind": slot_letter(step.slot), "text": step.text, "col": step.col }).to_string()
}

/// One letter per line of `text`, as `scriptLooks` names them.
pub(super) fn kinds_text(text: &str) -> String {
    text_kinds(text).into_iter().map(letter).collect()
}

/// Every letter of `kinds_text` with its kind and the columns the page sets it in, and the
/// letter of each kind by its name (`letters`).
pub(super) fn looks_json() -> String {
    let looks: Map<String, Value> = KINDS
        .iter()
        .map(|(kind, letter, name)| {
            let mut look = json!({ "kind": name });
            if let Some(margin) = kind.page_kind().map(|k| k.margin()) {
                look["x"] = json!(margin.x);
                look["width"] = json!(margin.width);
                look["align"] = json!(match margin.align {
                    Align::Left => "left",
                    Align::Right => "right",
                    Align::Center => "center",
                });
            }
            (letter.to_string(), look)
        })
        .collect();

    let letters: Map<String, Value> = KINDS
        .iter()
        .map(|(_, letter, name)| (name.to_string(), json!(letter.to_string())))
        .collect();

    json!({ "columns": TEXT_COLUMNS, "looks": looks, "letters": letters }).to_string()
}

/// The pages of the print view, as `scriptPrintJson` describes them.
pub(super) fn print_json(pages: &[SheetPage]) -> String {
    let pages: Vec<_> = pages
        .iter()
        .map(|page| {
            let texts: Vec<_> = page
                .texts
                .iter()
                .map(|t| {
                    json!({
                        "x": t.x_pt,
                        "y": t.baseline_pt,
                        "bold": t.face == PrintFace::Bold,
                        "text": t.text,
                    })
                })
                .collect();
            json!({ "width": page.width_pt, "height": page.height_pt, "texts": texts })
        })
        .collect();

    json!({ "fontPt": SCRIPT_FONT_PT, "ascentPt": SCRIPT_ASCENT_PT, "pages": pages }).to_string()
}

/// The last layout for the editor: where the pages start and on which page each scene begins.
/// Lines count from 0, pages from 1.
pub(super) fn breaks_json(pages: Option<&ScriptPages>) -> String {
    let Some(pages) = pages else {
        return json!({ "pages": 0, "breaks": [], "scenes": [] }).to_string();
    };
    let breaks: Vec<_> = pages
        .breaks
        .iter()
        .map(|b| json!({ "page": b.page, "line": b.line }))
        .collect();
    let scenes: Vec<_> = pages
        .scenes
        .iter()
        .map(|s| json!({ "line": s.line, "page": s.page }))
        .collect();

    json!({ "pages": pages.pages, "breaks": breaks, "scenes": scenes }).to_string()
}

/// The counts of an import for the report dialog.
fn report_json(report: &ImportReport) -> String {
    json!({
        "scenes": report.scenes,
        "created": report.created,
        "pending": report.pending,
        "notInBook": report.not_in_book,
        "stylesLost": report.styles_lost,
    })
    .to_string()
}

impl qobject::Controller {
    pub(super) fn script_kinds(&self, text: &QString) -> QString {
        q(&kinds_text(&text.to_string()))
    }

    pub(super) fn script_looks(&self) -> QString {
        q(&looks_json())
    }

    pub(super) fn script_print_json(mut self: Pin<&mut Self>) -> QString {
        let print = ScriptPrint {
            scene_numbers: SceneNumbers::Shown,
            header: HeaderLine::Off,
            language: self.rust().language,
            revisions: Revisions::Off,
        };
        let Some(s) = self.as_mut().session() else {
            return q(&print_json(&[]));
        };

        q(&print_json(&s.print_pages(&print)))
    }

    pub(super) fn script_breaks_json(mut self: Pin<&mut Self>) -> QString {
        let Some(s) = self.as_mut().session() else {
            return q(&breaks_json(None));
        };

        q(&breaks_json(s.script_pages()))
    }

    pub(super) fn enter_step(
        &self,
        line: &QString,
        col: i32,
        kind: &QString,
        below: &QString,
    ) -> QString {
        let below = below.to_string();
        let col = usize::try_from(col).unwrap_or_default();

        q(&enter_json(
            &line.to_string(),
            col,
            &kind.to_string(),
            Some(&below),
        ))
    }

    pub(super) fn tab_step(&self, line: &QString, kind: &QString) -> QString {
        q(&tab_json(&line.to_string(), &kind.to_string()))
    }

    pub(super) fn typed_caps(&self, text: &QString, at: i32, pending: &QString) -> QString {
        let at = usize::try_from(at).unwrap_or_default();
        let caps = typed_caps(&text.to_string(), at, slot_of(&pending.to_string()));

        q(&caps.unwrap_or_default())
    }

    pub(super) fn edit_script(mut self: Pin<&mut Self>, text: &QString) {
        let Some(s) = self.as_mut().session() else {
            return;
        };

        s.set_script_text(&text.to_string());
        self.as_mut().changed();
        self.refresh();
    }

    pub(super) fn paginate_script(mut self: Pin<&mut Self>) {
        let language = self.rust().language;
        let Some(s) = self.as_mut().session() else {
            return;
        };

        if s.paginate_if_changed(language) == Laid::Kept {
            return;
        }
        self.refresh();
    }

    pub(super) fn import_script(mut self: Pin<&mut Self>, path: &QString) -> bool {
        let language = self.rust().language;
        let path = fileurl::to_path(&path.to_string());
        let Some(s) = self.as_mut().session() else {
            return false;
        };

        let result = s.import_script(&path);
        let ok = result.is_ok();
        let message = match result {
            Ok(report) => {
                self.as_mut().set_import_report(q(&report_json(&report)));
                language.text("script.imported")
            }
            Err(e) => error_text(language, &e),
        };
        self.as_mut().changed();
        self.as_mut().refresh();
        self.as_mut().say(message);
        ok
    }
}

#[cfg(test)]
mod tests {
    use super::{breaks_json, enter_json, kinds_text, looks_json, print_json, tab_json};

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
    use klappe_services::{PageBreak, Placed, PrintFace, SceneSpan, ScriptPages, SheetPage};

    /// Texts keep their place in points and their face.
    #[test]
    fn a_page_lists_its_texts_in_points() {
        let page = SheetPage {
            width_pt: 595.0,
            height_pt: 842.0,
            texts: vec![Placed {
                x_pt: 108.0,
                baseline_pt: 82.0,
                face: PrintFace::Bold,
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
    }

    /// The layout reaches the editor as the lines the pages start at and each scene's page;
    /// without a layout it is empty.
    #[test]
    fn breaks_list_pages_and_scene_pages() {
        let pages = ScriptPages {
            breaks: vec![PageBreak { page: 2, line: 59 }],
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
}
