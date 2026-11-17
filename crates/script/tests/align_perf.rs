//! Matching a 120-page script against its previous state stays well under 100 ms. Only
//! meaningful in a release build: `make perf` runs it with `--release`.
#![allow(clippy::unwrap_used)]

use std::time::{Duration, Instant};

use klappe_script::{align, parse, scene_states, Outcome};

const SCENES: usize = 150;
const LINES_PER_SCENE: usize = 12;
const BUDGET: Duration = Duration::from_millis(100);
/// CI machines vary; only a real regression crosses this.
const LIMIT_FACTOR: u32 = 4;
const RUNS: usize = 11;

/// About 120 pages: 150 scenes of 12 elements; every tenth scene is renumbered, every
/// seventh heading reworded, every thirteenth scene rewritten.
fn script(edited: bool) -> String {
    let mut s = String::new();
    for n in 0..SCENES {
        let number = if edited && n % 10 == 0 {
            n + 1000
        } else {
            n + 1
        };
        let place = if edited && n % 7 == 0 { "NEU" } else { "ORT" };
        let tag = if edited && n % 13 == 0 { "x" } else { "" };
        s.push_str(&format!("INT. {place} {n} – TAG #{number}#\n\n"));
        for l in 0..LINES_PER_SCENE / 2 {
            s.push_str(&format!(
                "Handlung {n} Zeile {l} {tag} mit Wörtern, die sich unterscheiden.\n\n"
            ));
            s.push_str(&format!("FIGUR\nSie sagt {n} und {l} {tag}.\n\n"));
        }
    }
    s
}

#[test]
#[ignore = "timing, run by `make perf` in release"]
fn a_120_page_script_aligns_fast() {
    let old = scene_states(&parse(&script(false)));
    let new_text = script(true);

    let mut times = Vec::new();
    let mut last = None;
    for _ in 0..RUNS {
        let t = Instant::now();
        let new = parse(&new_text);
        last = Some(align(&old, &new));
        times.push(t.elapsed());
    }
    times.sort();
    let median = times[RUNS / 2];
    println!("perf align: {median:?} (parse included, budget {BUDGET:?})");

    let found = last
        .unwrap()
        .old
        .iter()
        .filter(|o| matches!(o, Outcome::Matched { .. }))
        .count();
    assert!(found > SCENES / 2, "only {found} of {SCENES} matched");
    assert!(median < BUDGET * LIMIT_FACTOR, "{median:?}");
}
