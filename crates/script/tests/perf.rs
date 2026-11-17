//! DN-02: a screenplay of 120 pages is parsed and paged again within a second. Timings only
//! mean something in a release build: `make perf` runs this ignored test with `--release`.
#![allow(clippy::unwrap_used)]

use std::fmt::Write;
use std::time::{Duration, Instant};

use klappe_script::{paginate, parse, PageOptions, PageSize, Words};

const PAGES: usize = 120;
/// The target of DN-02.
const BUDGET: Duration = Duration::from_secs(1);
const RUNS: usize = 5;

/// Scenes of action and dialogue, until the layout reaches `PAGES` pages.
fn screenplay() -> String {
    let options = PageOptions::new(PageSize::A4, Words::ENGLISH);
    let mut text = String::new();
    let mut scene = 0;

    while paginate(&parse(&text).elements, &options).pages.len() < PAGES {
        scene += 1;
        let _ = write!(text, "INT. ROOM {scene} - DAY\n\n");
        text.push_str(
            "Rain taps on the window. A clock ticks. Somebody coughs in the next room, \
             and the light flickers twice before it settles. Nobody moves.\n\n",
        );
        for line in 0..6 {
            let _ = write!(
                text,
                "ANNA\n(quietly)\nThis is line {line}. It is long enough to wrap over \
                 two or three rows of the page. Then it ends.\n\nBEN\nI know. We both do.\n\n"
            );
        }
        text.push_str("CUT TO:\n\n");
    }

    text
}

#[test]
#[ignore = "timing, run by `make perf` in release"]
fn a_script_of_120_pages_is_paged_in_a_second() {
    let text = screenplay();
    let options = PageOptions::new(PageSize::A4, Words::ENGLISH);
    let mut slowest = Duration::ZERO;
    let mut pages = 0;

    for _ in 0..RUNS {
        let start = Instant::now();
        pages = paginate(&parse(&text).elements, &options).pages.len();
        slowest = slowest.max(start.elapsed());
    }

    println!("{pages} pages, slowest of {RUNS} runs: {slowest:?} (budget {BUDGET:?})");
    assert!(pages >= PAGES);
    assert!(slowest <= BUDGET, "{slowest:?} is over {BUDGET:?}");
}
