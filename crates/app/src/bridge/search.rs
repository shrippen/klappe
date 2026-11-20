//! Search and replace in the window's screenplay editor (DB-60): the hits of `script/` as
//! positions the text area counts in (UTF-16 units from the start of the text).
//!
//! ```text
//! "INT. SAAL\nAnna kommt."   find "anna"  ─►  [[10, 14]]
//! ```

use super::{q, qobject};
use cxx_qt_lib::QString;
use klappe_services::{find_all, parse_located, replace_hits, Case, Hit, Query, Vocabulary};
use serde_json::json;

/// How `caseMode` says case counts.
const CASE_MATCH: &str = "match";

fn case_of(mode: &str) -> Case {
    match mode == CASE_MATCH {
        true => Case::Match,
        false => Case::Ignore,
    }
}

/// The text split at line breaks, and where each line starts in UTF-16 units.
struct Lines {
    rows: Vec<String>,
    starts: Vec<usize>,
}

impl Lines {
    fn of(text: &str) -> Self {
        let rows: Vec<String> = text.split('\n').map(str::to_owned).collect();
        let mut starts = Vec::with_capacity(rows.len());
        let mut at = 0;
        for row in &rows {
            starts.push(at);
            at += row.encode_utf16().count() + 1;
        }

        Self { rows, starts }
    }

    /// Character `col` of line `line` (counted in characters, as `script/` does) in UTF-16 units.
    fn offset(&self, line: usize, col: usize) -> usize {
        let Some(row) = self.rows.get(line) else {
            return self.end();
        };
        let within: usize = row.chars().take(col).map(char::len_utf16).sum();

        self.starts[line] + within
    }

    /// Just after the last character of line `line`.
    fn line_end(&self, line: usize) -> usize {
        let Some(row) = self.rows.get(line) else {
            return self.end();
        };

        self.starts[line] + row.encode_utf16().count()
    }

    /// Just after the last character; `split` gives at least one line.
    fn end(&self) -> usize {
        let last = self.rows.len() - 1;

        self.starts[last] + self.rows[last].encode_utf16().count()
    }
}

/// The speaker filter; empty is everyone.
fn speaker_of(speaker: &str) -> Option<&str> {
    let speaker = speaker.trim();

    (!speaker.is_empty()).then_some(speaker)
}

fn hits(lines: &Lines, text: &str, find: &str, case: Case, speaker: &str) -> Vec<Hit> {
    let query = Query {
        find,
        case,
        speaker: speaker_of(speaker),
    };

    find_all(&lines.rows, &parse_located(text), &query)
}

/// The cue names of `text`, most frequent first, as `["ANNA", …]`.
pub(super) fn speakers_json(text: &str) -> String {
    json!(Vocabulary::from_located(&parse_located(text)).characters()).to_string()
}

/// Every hit of `find` in `text` (only the dialogue of `speaker`, when given) as
/// `[[start, end], …]`.
pub(super) fn hits_json(text: &str, find: &str, case_mode: &str, speaker: &str) -> String {
    let lines = Lines::of(text);
    let found: Vec<_> = hits(&lines, text, find, case_of(case_mode), speaker)
        .iter()
        .map(|h| {
            json!([
                lines.offset(h.line, h.start),
                lines.offset(h.end_line, h.end)
            ])
        })
        .collect();

    json!(found).to_string()
}

/// `replacement` in place of hit `index` (all hits when `index` is none) as `{ from, to,
/// text }`: the stretch of the text to put `text` in; empty when nothing is found.
pub(super) fn replace_json(
    text: &str,
    find: &str,
    case_mode: &str,
    speaker: &str,
    replacement: &str,
    index: Option<usize>,
) -> String {
    let lines = Lines::of(text);
    let all = hits(&lines, text, find, case_of(case_mode), speaker);
    let chosen = match index {
        Some(i) => all.get(i).map(|h| vec![*h]).unwrap_or_default(),
        None => all,
    };
    let Some(splice) = replace_hits(&lines.rows, &chosen, replacement) else {
        return String::new();
    };

    json!({
        "from": lines.offset(splice.from, 0),
        "to": lines.line_end(splice.to - 1),
        "text": splice.lines.join("\n"),
    })
    .to_string()
}

impl qobject::Controller {
    pub(super) fn find_in_script(
        &self,
        text: &QString,
        find: &QString,
        matching: &QString,
        speaker: &QString,
    ) -> QString {
        q(&hits_json(
            &text.to_string(),
            &find.to_string(),
            &matching.to_string(),
            &speaker.to_string(),
        ))
    }

    pub(super) fn script_speakers(&self, text: &QString) -> QString {
        q(&speakers_json(&text.to_string()))
    }

    pub(super) fn replace_in_script(
        &self,
        text: &QString,
        find: &QString,
        matching: &QString,
        speaker: &QString,
        replacement: &QString,
        index: i32,
    ) -> QString {
        let index = usize::try_from(index).ok();

        q(&replace_json(
            &text.to_string(),
            &find.to_string(),
            &matching.to_string(),
            &speaker.to_string(),
            &replacement.to_string(),
            index,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{hits_json, replace_json, speakers_json};

    fn parsed(json: &str) -> serde_json::Value {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn hits_are_text_positions_without_case() {
        let v = parsed(&hits_json(
            "INT. SAAL\n\nAnna sieht anna.",
            "anna",
            "ignore",
            "",
        ));

        assert_eq!(v, serde_json::json!([[11, 15], [22, 26]]));
        assert_eq!(
            parsed(&hits_json("Anna", "anna", "match", "")),
            serde_json::json!([])
        );
    }

    /// The text area counts UTF-16 units: a character outside the BMP is two of them.
    #[test]
    fn positions_count_as_the_text_area_does() {
        let v = parsed(&hits_json("🎬 Anna", "Anna", "ignore", ""));

        assert_eq!(v, serde_json::json!([[3, 7]]));
    }

    #[test]
    fn one_hit_or_all_are_replaced() {
        let text = "INT. SAAL\n\nAnna sieht Anna.";

        let one = parsed(&replace_json(text, "anna", "ignore", "", "Eva", Some(1)));
        assert_eq!(one["from"], 11);
        assert_eq!(one["to"], 27);
        assert_eq!(one["text"], "Anna sieht Eva.");

        let all = parsed(&replace_json(text, "anna", "ignore", "", "Eva", None));
        assert_eq!(all["text"], "Eva sieht Eva.");
        assert_eq!(replace_json(text, "Bert", "ignore", "", "Eva", None), "");
    }

    const DIALOGUE: &str = "ANNA\nJa, Eva.\n\nEVA\nEva ja.\n\nANNA\nNein.\n";

    /// The speaker limits the hits to that cue's dialogue, without case.
    #[test]
    fn the_speaker_limits_hits_and_replacement() {
        let all = parsed(&hits_json(DIALOGUE, "eva", "ignore", ""));
        let anna = parsed(&hits_json(DIALOGUE, "eva", "ignore", "anna"));

        // The cue line EVA is no dialogue: it counts without a speaker only.
        assert_eq!(all.as_array().unwrap().len(), 3);
        assert_eq!(anna, serde_json::json!([[9, 12]]));

        let done = parsed(&replace_json(DIALOGUE, "eva", "ignore", "Anna", "X", None));
        assert_eq!(done["text"], "Ja, X.");
    }

    #[test]
    fn speakers_are_the_cues_most_frequent_first() {
        assert_eq!(
            parsed(&speakers_json(DIALOGUE)),
            serde_json::json!(["ANNA", "EVA"])
        );
        assert_eq!(parsed(&speakers_json("")), serde_json::json!([]));
    }
}
