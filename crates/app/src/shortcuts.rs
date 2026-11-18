//! Two Shortcuts with one sequence in the window fire neither (Qt calls it ambiguous). The
//! sequences of the window's QML sources must differ.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// Keys that Shortcuts of different surfaces (plan, dialog, field) take in turn, each enabled only
/// where it applies.
const CONTEXT_KEYS: &[&str] = &[
    "Escape",
    "Return",
    "Enter",
    "Left",
    "Right",
    "Up",
    "Down",
    "Menu",
    "Shift+F10",
];

/// Prefixes of the lines that name a sequence: a command's shortcut, a Shortcut, a tool's key.
const SEQUENCE_LINES: &[&str] = &["shortcut:", "sequence:", "sequences:"];

fn quoted(line: &str) -> Vec<String> {
    line.split('"')
        .skip(1)
        .step_by(2)
        .map(str::to_string)
        .collect()
}

/// The sequences of one file: `shortcut: "K"`, `sequence: "K"`, `sequences: ["A", "B"]`, and the
/// tool bar's `key: "Shift+B"` entries.
fn sequences(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    for line in text.lines() {
        let line = line.trim().trim_start_matches('{').trim();
        let is_sequence = SEQUENCE_LINES.iter().any(|p| line.starts_with(p));
        if is_sequence && line.contains('"') && !line.contains("=>") {
            found.extend(quoted(line));
        }
        if line.contains(" key: \"") && line.contains("name: \"") {
            found.extend(quoted(line.split("key:").nth(1).unwrap_or("")));
        }
        if line.contains("{ id: \"") && line.contains(" key: \"") {
            found.extend(quoted(
                line.split("key:")
                    .nth(1)
                    .unwrap_or("")
                    .split(", groups")
                    .next()
                    .unwrap_or(""),
            ));
        }
    }
    found.retain(|s| !CONTEXT_KEYS.contains(&s.as_str()));
    found
}

#[test]
fn no_sequence_twice() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("qml");
    let mut owners: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for entry in fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|e| e != "qml") {
            continue;
        }
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        for key in sequences(&fs::read_to_string(&path).unwrap()) {
            owners.entry(key).or_default().push(name.clone());
        }
    }

    // A key listed twice, in two files or in one, is the clash.
    let clashes: Vec<String> = owners
        .iter()
        .filter(|(_, files)| files.len() > 1)
        .map(|(key, files)| format!("{key}: {}", files.join(", ")))
        .collect();
    assert!(
        clashes.is_empty(),
        "sequences taken twice:\n{}",
        clashes.join("\n")
    );
}
