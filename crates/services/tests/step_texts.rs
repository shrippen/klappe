//! Every undo step has a text in both languages (LH-103): the name of a step is the variant of
//! its edit (`PlanEdit::Remove` is `Remove`) or a name `Session` gives it, and the window shows
//! `step.<name>` of the catalogue. A step without a text falls back to the plain "Undo".
#![allow(clippy::unwrap_used)]

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

fn read(path: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap()
}

/// The variants of `pub enum <name>` in the source `path`: the capitalised words at the
/// enum's first indent.
fn variants(path: &str, name: &str) -> Vec<String> {
    let source = read(path);
    let start = format!("pub enum {name} {{");
    let body: Vec<&str> = source
        .lines()
        .skip_while(|l| *l != start)
        .skip(1)
        .take_while(|l| *l != "}")
        .collect();

    body.iter()
        .filter(|l| l.starts_with("    ") && !l.starts_with("     "))
        .filter_map(|l| {
            let word: String = l
                .trim()
                .chars()
                .take_while(|c| c.is_alphanumeric())
                .collect();
            word.chars()
                .next()
                .filter(|c| c.is_uppercase())
                .map(|_| word)
        })
        .collect()
}

/// The names `Session` passes to `commit_step`.
fn session_steps() -> Vec<String> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/session");
    let mut names = Vec::new();
    for entry in fs::read_dir(dir).unwrap() {
        let text = fs::read_to_string(entry.unwrap().path()).unwrap();
        for part in text.split("commit_step(next, \"").skip(1) {
            // `commit` names nothing: an unnamed step.
            names.push(part.split('"').next().unwrap().to_owned());
        }
    }

    names
}

#[test]
fn every_step_has_a_text_in_both_languages() {
    // A scan that finds nothing, or misses a name known to be there, must not pass vacuously.
    let scans = [
        (
            variants("src/plan_edit.rs", "PlanEdit"),
            ["Remove", "AddObject", "SetLens"],
        ),
        (
            variants("src/project_edit.rs", "ProjectEdit"),
            ["AddScene", "EditCharacter", "RemoveTemplate"],
        ),
        (
            session_steps()
                .into_iter()
                .filter(|n| !n.is_empty())
                .collect(),
            ["AddShot", "RenamePlan", "Retime"],
        ),
    ];
    let mut names: BTreeSet<String> = BTreeSet::new();
    for (found, known) in scans {
        assert!(!found.is_empty(), "a scan found no step name");
        for name in known {
            assert!(
                found.iter().any(|f| f == name),
                "the scan misses {name}: {found:?}"
            );
        }
        names.extend(found);
    }

    for language in ["de", "en"] {
        let catalogue: serde_json::Value =
            serde_json::from_str(&read(&format!("i18n/{language}.json"))).unwrap();
        let missing: Vec<_> = names
            .iter()
            .filter(|n| catalogue.get(format!("step.{n}")).is_none())
            .collect();

        assert!(
            missing.is_empty(),
            "{language} lacks step.* for {missing:?}"
        );
    }
}
