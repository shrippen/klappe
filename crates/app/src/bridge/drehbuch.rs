//! The screenplay in the window: its text and outline for the editor, and the import.

use super::{q, qobject};
use crate::fileurl;
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use klappe_services::{
    enter_step, error_text, outline, tab_step, text_kinds, typed_caps, Align, EnterStep, Gap,
    HeaderLine, ImportReport, Laid, OutlineKind, PrintFace, SceneNumbers, ScriptPrint, SheetPage,
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

/// The slots of the element flow and their letters, as `KINDS` names the kinds.
const SLOTS: [(Slot, TextKind); 6] = [
    (Slot::Heading, TextKind::Heading),
    (Slot::Action, TextKind::Action),
    (Slot::Character, TextKind::Character),
    (Slot::Parenthetical, TextKind::Parenthetical),
    (Slot::Dialogue, TextKind::Dialogue),
    (Slot::Transition, TextKind::Transition),
];

fn slot_of(letter: &str) -> Option<Slot> {
    let first = letter.chars().next()?;
    let kind = KINDS.iter().find(|(_, l, _)| *l == first)?.0;

    SLOTS
        .iter()
        .find(|(_, k)| *k == kind)
        .map(|(slot, _)| *slot)
}

fn slot_letter(slot: Slot) -> String {
    let kind = SLOTS
        .iter()
        .find(|(s, _)| *s == slot)
        .map_or(TextKind::Action, |(_, k)| *k);

    letter(kind).to_string()
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

/// Every letter of `kinds_text` with its kind and the columns the page sets it in.
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

    json!({ "columns": TEXT_COLUMNS, "looks": looks }).to_string()
}

/// The sections and scenes of `text` for the scene list; synopses are left out.
pub(super) fn outline_json(text: &str) -> String {
    let rows: Vec<_> = outline(text)
        .into_iter()
        .filter_map(|e| {
            let kind = match e.kind {
                OutlineKind::Section(_) => "section",
                OutlineKind::Scene => "scene",
                OutlineKind::Synopsis => return None,
            };

            Some(json!({
                "kind": kind,
                "depth": e.depth,
                "line": e.line,
                "number": e.number.unwrap_or_default(),
                "text": e.text,
            }))
        })
        .collect();

    json!(rows).to_string()
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

/// The counts of an import for the report dialog.
fn report_json(report: &ImportReport) -> String {
    json!({
        "scenes": report.scenes,
        "created": report.created,
        "pending": report.pending,
        "notInBook": report.not_in_book,
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
        };
        let Some(s) = self.as_mut().session() else {
            return q(&print_json(&[]));
        };

        q(&print_json(&s.print_pages(&print)))
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
    use super::{enter_json, kinds_text, looks_json, outline_json, print_json, tab_json};

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
    use klappe_services::{Placed, PrintFace, SheetPage};

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
    }

    /// Sections and scenes are listed with their line; a synopsis is not.
    #[test]
    fn the_outline_lists_sections_and_scenes() {
        let text = "# Akt I\n\nINT. SAAL - TAG #1#\n\n= Sie kommt an.\n\nEXT. HOF - NACHT\n";

        let v: serde_json::Value = serde_json::from_str(&outline_json(text)).unwrap();
        let rows = v.as_array().unwrap();

        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0]["kind"], "section");
        assert_eq!(rows[0]["text"], "Akt I");
        assert_eq!(rows[1]["kind"], "scene");
        assert_eq!(rows[1]["number"], "1");
        assert_eq!(rows[1]["line"], 2);
        assert_eq!(rows[2]["number"], "");
        assert_eq!(rows[2]["line"], 6);
    }

    #[test]
    fn no_text_is_an_empty_list() {
        assert_eq!(outline_json(""), "[]");
    }
}
