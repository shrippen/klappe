//! DN-02: a screenplay of 120 pages is parsed and paged again within a second. Timings only
//! mean something in a release build: `make perf` runs this ignored test with `--release`.
#![allow(clippy::unwrap_used)]

use std::fmt::Write;
use std::time::{Duration, Instant};

use klappe_script::{
    carry, page_starts, paginate, paginate_locked, parse, parse_located, text_kinds, PageOptions,
    PageSize, Words,
};

const PAGES: usize = 120;
/// The target of DN-02.
const BUDGET: Duration = Duration::from_secs(1);
const RUNS: usize = 5;
/// The window's editor reads the kind of every line at each keystroke (D-15): one frame.
const KEYSTROKE_BUDGET: Duration = Duration::from_millis(16);

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

#[test]
#[ignore = "timing, run by `make perf` in release"]
fn the_kinds_of_120_pages_take_a_frame() {
    let text = screenplay();
    let mut slowest = Duration::ZERO;
    let mut lines = 0;

    for _ in 0..RUNS {
        let start = Instant::now();
        lines = text_kinds(&text).len();
        slowest = slowest.max(start.elapsed());
    }

    println!("{lines} lines, slowest of {RUNS} runs: {slowest:?} (budget {KEYSTROKE_BUDGET:?})");
    assert!(
        slowest <= KEYSTROKE_BUDGET,
        "{slowest:?} is over {KEYSTROKE_BUDGET:?}"
    );
}

/// DB-34: a locked script of 120 pages, edited in 30 places, is mapped onto its locked pages
/// and paged within the same budget.
#[test]
#[ignore = "timing, run by `make perf` in release"]
fn a_locked_script_of_120_pages_is_carried_and_paged_in_a_second() {
    let text = screenplay();
    let options = PageOptions::new(PageSize::A4, Words::ENGLISH);
    let located = parse_located(&text);
    let elements: Vec<_> = located.iter().map(|l| l.element.clone()).collect();
    let free = paginate(&elements, &options);
    let starts = page_starts(&free, &located, text.matches('\n').count() + 1);
    let edited = text.replacen("Rain taps on", "Rain taps hard on", 30);
    assert_ne!(edited, text);

    let mut slowest = Duration::ZERO;
    let mut pages = 0;
    for _ in 0..RUNS {
        let start = Instant::now();
        let located = parse_located(&edited);
        let elements: Vec<_> = located.iter().map(|l| l.element.clone()).collect();
        let carried = carry(&text, &edited, &starts);
        pages = paginate_locked(&elements, &located, &options, &carried)
            .pages
            .len();
        slowest = slowest.max(start.elapsed());
    }

    println!("{pages} pages, slowest of {RUNS} runs: {slowest:?} (budget {BUDGET:?})");
    assert!(pages >= PAGES);
    assert!(slowest <= BUDGET, "{slowest:?} is over {BUDGET:?}");
}
