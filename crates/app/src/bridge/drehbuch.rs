//! The screenplay in the window: its text for the editor, and the import.

use super::{q, qobject};
use crate::fileurl;
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use klappe_services::{
    can_suggest, enter_step, error_text, heading_like, length_text, move_entry, outline,
    parse_located, sections_around, suggest, tab_step, text_kinds, typed_caps, visible_entries,
    Align, Context, EnterStep, Gap, HeaderLine, ImportReport, Laid, MoveWay, OutlineEntry,
    OutlineKind, PrintFace, Revisions, SceneNumbering, SceneNumbers, SceneSpan, ScriptLanguage,
    ScriptPages, ScriptPaper, ScriptPrint, ScriptSetup, ScriptWords, SheetPage, Slot, Stage,
    TextKind, Vocabulary, SCRIPT_ASCENT_PT, SCRIPT_FONT_PT, TEXT_COLUMNS,
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

/// The most choices the list under the line shows.
const MAX_SUGGESTIONS: usize = 8;

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

/// The suggestions for the end of line `line` of `text`, a line of kind `kind`, as `scriptSuggest`
/// describes them; empty for none. `pending` is the letter of the slot Enter or Tab made the line,
/// or empty: such a line is that element, a heading only while it still looks like one.
pub(super) fn suggest_json(text: &str, line: usize, kind: &str, pending: &str) -> String {
    let Some(row) = text.split('\n').nth(line) else {
        return String::new();
    };
    // The open element counts while the reader sees plain text there, as in the terminal.
    let plain = kind == letter(TextKind::Blank).to_string() || slot_of(kind) == Some(Slot::Action);
    let open = !pending.is_empty() && plain;
    let slot = slot_of(if open { pending } else { kind });
    let context = match slot {
        Some(Slot::Heading) if open && !heading_like(row) => Context::Line,
        Some(Slot::Heading) => Context::Heading,
        Some(Slot::Character) => Context::Cue,
        _ => Context::Line,
    };
    // Plain text gets none: no need to read the whole book for it.
    if !can_suggest(row, context) {
        return String::new();
    }
    let vocab = Vocabulary::from_located(&parse_located(text));
    let Some(found) = suggest(&vocab, row, line, context) else {
        return String::new();
    };

    // A time of day ends a heading, a name or extension a cue: the next element follows.
    let (next, gap) = match found.stage {
        Stage::Time => Slot::Heading.after_enter(),
        _ => Slot::Character.after_enter(),
    };
    let head_utf16: usize = row.chars().take(found.from).map(char::len_utf16).sum();
    let options: Vec<_> = found.options.iter().take(MAX_SUGGESTIONS).collect();

    json!({
        "from": head_utf16,
        "options": options,
        "finishes": found.stage.finishes(),
        "kind": slot_letter(next),
        "gap": match gap { Gap::Blank => "blank", Gap::None => "none" },
    })
    .to_string()
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
        .map(|b| json!({ "page": b.page, "label": b.label, "line": b.line }))
        .collect();
    let scenes: Vec<_> = pages
        .scenes
        .iter()
        .map(|s| json!({ "line": s.line, "page": s.page }))
        .collect();

    json!({ "pages": pages.pages, "breaks": breaks, "scenes": scenes }).to_string()
}

/// The page span of the scene `index` of `entries` in the last layout: by position while the
/// layout has as many scenes as the text, else by the heading's line (typing since).
fn span_of<'a>(
    pages: &'a ScriptPages,
    scenes: usize,
    entry: &OutlineEntry,
) -> Option<&'a SceneSpan> {
    if pages.scenes.len() == scenes {
        return pages.scenes.get(entry.scene?);
    }

    pages.scenes.iter().find(|s| s.line == entry.line)
}

/// What a folded section is remembered by: its level and title. Typing above it does not move
/// it; renaming it opens it.
fn fold_key(entry: &OutlineEntry) -> String {
    match entry.kind {
        OutlineKind::Section(level) => format!("{level}{}", entry.text),
        _ => String::new(),
    }
}

/// The entries of the scene list: `[{ kind: section | scene | synopsis, level (sections: 1 for
/// `#`), depth, line, end, text, number, length, key, folded }]` in text order, without what a
/// folded section holds (`folded`: the `key`s of the folded sections). A section's `length` is
/// the total of its scenes, a scene's its own, both from the last layout.
pub(super) fn outline_json(text: &str, pages: Option<&ScriptPages>, folded: &[String]) -> String {
    let entries = outline(text);
    let is_folded = |e: &OutlineEntry| folded.contains(&fold_key(e));
    let scenes = entries.iter().filter(|e| e.scene.is_some()).count();
    let eighths = |e: &OutlineEntry| -> usize {
        let Some(span) = pages.and_then(|p| span_of(p, scenes, e)) else {
            return 0;
        };
        span.eighths
    };

    let rows: Vec<_> = visible_entries(&entries, &is_folded, "")
        .into_iter()
        .map(|i| {
            let e = &entries[i];
            let (kind, level, length) = match e.kind {
                OutlineKind::Section(level) => {
                    let total: usize = entries
                        .iter()
                        .filter(|n| n.scene.is_some() && n.line > e.line && n.line < e.end)
                        .map(eighths)
                        .sum();
                    let length = if total > 0 {
                        length_text(total)
                    } else {
                        String::new()
                    };
                    ("section", level, length)
                }
                OutlineKind::Scene => {
                    let length = pages
                        .and_then(|p| span_of(p, scenes, e))
                        .map(|s| s.length.clone())
                        .unwrap_or_default();
                    ("scene", 0, length)
                }
                OutlineKind::Synopsis => ("synopsis", 0, String::new()),
            };
            json!({
                "kind": kind,
                "level": level,
                "depth": e.depth,
                "line": e.line,
                "end": e.end,
                "text": e.text,
                "number": e.number,
                "length": length,
                "key": fold_key(e),
                "folded": is_folded(e),
            })
        })
        .collect();

    Value::Array(rows).to_string()
}

/// Moving the scene or section at text line `line` one place (`up` or `down`), as `moveOutline`
/// describes it: `{ from, to, lines, line }`, the text lines `from..to` replaced by `lines`,
/// the block at `line` now, `unfold` the `key`s of the sections around it there; empty where it
/// cannot move.
pub(super) fn move_json(text: &str, line: usize, way: &str) -> String {
    let way = match way {
        "up" => MoveWay::Up,
        "down" => MoveWay::Down,
        _ => return String::new(),
    };
    let entries = outline(text);
    let lines: Vec<String> = text.lines().map(str::to_owned).collect();
    let Some(index) = entries
        .iter()
        .position(|e| e.line == line && e.kind != OutlineKind::Synopsis)
    else {
        return String::new();
    };
    let Some(moved) = move_entry(&lines, &entries, index, way) else {
        return String::new();
    };

    // The sections around the block after the move: it may have stepped into a folded one.
    let mut moved_lines = lines;
    moved_lines.splice(moved.from..moved.to, moved.lines.clone());
    let after = outline(&moved_lines.join("\n"));
    let unfold: Vec<String> = after
        .iter()
        .position(|e| e.line == moved.line)
        .map(|i| sections_around(&after, i))
        .unwrap_or_default()
        .into_iter()
        .map(|i| fold_key(&after[i]))
        .collect();

    json!({
        "from": moved.from,
        "to": moved.to,
        "lines": moved.lines,
        "line": moved.line,
        "unfold": unfold,
    })
    .to_string()
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

    pub(super) fn script_outline_json(mut self: Pin<&mut Self>, folded: &QString) -> QString {
        let folded: Vec<String> = serde_json::from_str(&folded.to_string()).unwrap_or_default();
        let Some(s) = self.as_mut().session() else {
            return q("[]");
        };

        q(&outline_json(s.script_text(), s.script_pages(), &folded))
    }

    pub(super) fn move_outline(mut self: Pin<&mut Self>, line: i32, way: &QString) -> QString {
        let Some(s) = self.as_mut().session() else {
            return QString::default();
        };
        let line = usize::try_from(line).unwrap_or_default();

        q(&move_json(s.script_text(), line, &way.to_string()))
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

    pub(super) fn script_suggest(
        &self,
        text: &QString,
        line: i32,
        kind: &QString,
        pending: &QString,
    ) -> QString {
        let line = usize::try_from(line).unwrap_or_default();

        q(&suggest_json(
            &text.to_string(),
            line,
            &kind.to_string(),
            &pending.to_string(),
        ))
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

    pub(super) fn set_script_setup(
        mut self: Pin<&mut Self>,
        language: &QString,
        words: &QString,
        paper: &QString,
        contd: &QString,
        numbering: &QString,
        first: i32,
    ) -> bool {
        let Some(s) = self.as_mut().session() else {
            return false;
        };

        // A first number that is no number stays as it was, as in the terminal.
        let current = s.project().script;
        let setup = ScriptSetup {
            language: ScriptLanguage::from_name(&language.to_string()),
            words: ScriptWords::from(words.to_string()),
            paper: ScriptPaper::from(paper.to_string()),
            contd_after_action: contd.to_string() == "on",
            scene_numbering: SceneNumbering::from(numbering.to_string()),
            first_scene: u32::try_from(first).unwrap_or(current.first_scene),
        };
        let result = s.choose_script_setup(setup);

        self.report(Some(result))
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
mod tests;
