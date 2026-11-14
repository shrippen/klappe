//! The committed examples (`demo/beispiel-*.klappe`) against the generator and the format.
//! The generator's files go to cargo's per-crate test directory, not `/tmp`.
#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

const DEMO: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../demo");
const LANGS: [&str; 2] = ["de", "en"];

fn example(lang: &str) -> PathBuf {
    Path::new(DEMO).join(format!("beispiel-{lang}.klappe"))
}

/// Every file under `dir` by its path relative to `dir`.
fn tree(dir: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut files = BTreeMap::new();
    let mut open = vec![dir.to_owned()];

    while let Some(folder) = open.pop() {
        for entry in std::fs::read_dir(&folder).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                open.push(path);
                continue;
            }
            let name = path
                .strip_prefix(dir)
                .unwrap()
                .to_string_lossy()
                .into_owned();
            files.insert(name, std::fs::read(&path).unwrap());
        }
    }

    files
}

/// `make check` runs this: a changed world or generator that was not generated again fails.
#[test]
fn the_committed_examples_are_up_to_date() {
    let out = Path::new(env!("CARGO_TARGET_TMPDIR")).join("demo-out");
    let _ = std::fs::remove_dir_all(&out);

    let status = Command::new(env!("CARGO_BIN_EXE_klappe-demo"))
        .arg("--out")
        .arg(&out)
        .output()
        .unwrap();

    assert!(status.status.success(), "{status:?}");
    for lang in LANGS {
        let fresh = tree(&out.join(format!("beispiel-{lang}.klappe")));

        assert_eq!(
            tree(&example(lang)).keys().collect::<Vec<_>>(),
            fresh.keys().collect::<Vec<_>>(),
            "files of beispiel-{lang}.klappe differ: run `cargo run -p klappe-demo`"
        );
        for (name, bytes) in &fresh {
            assert!(
                tree(&example(lang))[name] == *bytes,
                "demo/beispiel-{lang}.klappe/{name} is out of date: run `cargo run -p klappe-demo`"
            );
        }
    }
}

/// What a plan needs to open: its cameras, characters and script elements exist, in both
/// languages.
#[test]
fn the_examples_hold_together() {
    for lang in LANGS {
        let loaded = klappe_store::load(&example(lang)).unwrap();
        let project = loaded.project;
        let script = klappe_script::parse(&loaded.script.unwrap());

        assert_eq!(project.scenes.len(), 1, "{lang}");
        let scene = &project.scenes[0];
        let plan = &scene.plans[0];
        let elements = script.scene(&scene.number).unwrap().elements;

        assert!(project.location(&scene.location).is_some(), "{lang}");
        assert_eq!(plan.shots.len(), 5, "{lang}");
        for shot in &plan.shots {
            assert!(plan.camera(&shot.camera).is_some(), "{lang} {}", shot.id);
        }
        for figure in &plan.figures {
            assert!(project.character(&figure.character).is_some(), "{lang}");
        }
        for link in &plan.links {
            assert!(
                link.element < elements.len(),
                "{lang} link {}",
                link.element
            );
        }
        assert_eq!(plan.duration_s(), 40.0, "{lang}");
    }
}
