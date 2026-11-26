//! What the screenplay outline shows beside each entry (DB-20, DB-21): its length from the
//! last page layout and the key a folded section is remembered by. The window and the
//! terminal take both from here and only show and fold.

use klappe_script::{length_text, Entry, EntryKind};

use crate::script_pages::{SceneSpan, ScriptPages};

/// What the outline shows for one entry, in the order of the entries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutlineInfo {
    /// A scene's own length, a section's total of its scenes; empty without a layout, for
    /// a scene the layout does not know yet and for a synopsis.
    pub length: String,
    /// What a folded section is remembered by: its level and title. Typing above it does
    /// not move it; renaming it opens it. Empty for scenes and synopses.
    pub key: String,
}

/// The page span of the scene of `entry` in the last layout: by position while the layout
/// has as many scenes as the text, else by the heading's line (typing since).
fn span_of<'a>(pages: &'a ScriptPages, scenes: usize, entry: &Entry) -> Option<&'a SceneSpan> {
    if pages.scenes.len() == scenes {
        return pages.scenes.get(entry.scene?);
    }

    pages.scenes.iter().find(|s| s.line == entry.line)
}

fn key_of(entry: &Entry) -> String {
    match entry.kind {
        EntryKind::Section(level) => format!("{level}{}", entry.text),
        _ => String::new(),
    }
}

/// The length and the fold key of every entry of `entries`, by index.
pub fn outline_info(entries: &[Entry], pages: Option<&ScriptPages>) -> Vec<OutlineInfo> {
    let scenes = entries.iter().filter(|e| e.scene.is_some()).count();
    let spans: Vec<Option<&SceneSpan>> = entries
        .iter()
        .map(|e| pages.and_then(|p| span_of(p, scenes, e)))
        .collect();

    entries
        .iter()
        .enumerate()
        .map(|(i, e)| {
            let length = match e.kind {
                EntryKind::Scene => spans[i].map(|s| s.length.clone()).unwrap_or_default(),
                EntryKind::Section(_) => section_length(entries, &spans, e),
                EntryKind::Synopsis => String::new(),
            };

            OutlineInfo {
                length,
                key: key_of(e),
            }
        })
        .collect()
}

/// The total of the scenes inside `section`, empty when none is laid out.
fn section_length(entries: &[Entry], spans: &[Option<&SceneSpan>], section: &Entry) -> String {
    let total: usize = entries
        .iter()
        .zip(spans)
        .filter(|(e, _)| e.scene.is_some() && e.line > section.line && e.line < section.end)
        .filter_map(|(_, span)| span.map(|s| s.eighths))
        .sum();

    match total {
        0 => String::new(),
        _ => length_text(total),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use klappe_script::outline;

    const BOOK: &str = "# Akt I\n\n= Es beginnt.\n\nINT. SAAL - TAG\n\nSie kommt an.\n\nEXT. HOF - NACHT\n\nEr geht.\n";

    fn pages(scenes: Vec<SceneSpan>) -> ScriptPages {
        ScriptPages {
            breaks: vec![],
            scenes,
            pages: 1,
            eighths: 9,
            minutes: 1.0,
            length: String::new(),
        }
    }

    fn span(line: usize, eighths: usize, length: &str) -> SceneSpan {
        SceneSpan {
            line,
            page: 1,
            eighths,
            length: length.into(),
        }
    }

    fn lengths(pages: Option<&ScriptPages>) -> Vec<String> {
        let info = outline_info(&outline(BOOK), pages);

        info.into_iter().map(|i| i.length).collect()
    }

    /// A section totals its scenes, a scene shows its own, a synopsis none.
    #[test]
    fn a_section_totals_its_scenes() {
        let laid = pages(vec![span(4, 4, "4/8"), span(8, 5, "5/8")]);

        assert_eq!(lengths(Some(&laid)), ["1 1/8", "", "4/8", "5/8"]);
    }

    /// Without a layout nothing has a length.
    #[test]
    fn no_layout_no_length() {
        assert_eq!(lengths(None), ["", "", "", ""]);
    }

    /// With another scene count than the text, a scene is found by its heading line.
    #[test]
    fn a_stale_layout_is_matched_by_line() {
        let laid = pages(vec![span(8, 5, "5/8")]);

        assert_eq!(lengths(Some(&laid)), ["5/8", "", "", "5/8"]);
    }

    /// Only sections have a key: their level and title.
    #[test]
    fn only_sections_have_a_key() {
        let keys: Vec<_> = outline_info(&outline(BOOK), None)
            .into_iter()
            .map(|i| i.key)
            .collect();

        assert_eq!(keys, ["1Akt I", "", "", ""]);
    }
}
