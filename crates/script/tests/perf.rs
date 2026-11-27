//! DN-02: a screenplay of 120 pages is parsed and paged again within a second. Timings only
//! mean something in a release build: `make perf` runs this ignored test with `--release`.
#![allow(clippy::unwrap_used)]

use std::fmt::Write;
use std::time::{Duration, Instant};

use klappe_script::{
    carry, edit_shown, page_starts, paginate, paginate_locked, parse, parse_located, shown_lines,
    text_kinds, toggle_mark, Change, Hide, Mark, PageOptions, PageSize, Spot, Words,
};

const PAGES: usize = 120;
/// The target of DN-02.
const BUDGET: Duration = Duration::from_secs(1);
const RUNS: usize = 5;
/// Loading the editor's text of 120 pages (D-36).
const LOAD_BUDGET: Duration = Duration::from_millis(300);
/// The window's editor reads the kind of every line at each keystroke (D-15): one frame.
const KEYSTROKE_BUDGET: Duration = Duration::from_millis(16);
/// Ctrl+A, Ctrl+B runs in the window's thread: it must not feel like a hang.
const TOGGLE_BUDGET: Duration = Duration::from_millis(300);

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

/// D-36: the window shows a script of 120 pages without the marks of emphasis: loading
/// projects every line once, a keystroke changes one line of it and reads the kinds.
#[test]
#[ignore = "timing, run by `make perf` in release"]
fn the_shown_text_of_120_pages_loads_and_a_keystroke_is_a_frame() {
    // Emphasis in some lines, so the projection has marks to hide.
    let text = screenplay().replace("Nobody moves.", "Nobody **moves** *at all*.");
    let hide = [Hide::Emphasis];
    let mut loading = Duration::ZERO;
    let mut lines = 0;

    for _ in 0..RUNS {
        let start = Instant::now();
        lines = shown_lines(&text, &hide).len();
        loading = loading.max(start.elapsed());
    }
    println!("{lines} lines shown, slowest of {RUNS} runs: {loading:?} (budget {LOAD_BUDGET:?})");
    assert!(
        loading <= LOAD_BUDGET,
        "{loading:?} is over {LOAD_BUDGET:?}"
    );

    // One character typed in the middle of the script, into a bold word.
    let line = text
        .lines()
        .position(|l| l.contains("**moves**"))
        .unwrap_or(0);
    let at = Spot { line, col: 12 };
    let key = Change {
        from: at,
        to: at,
        typed: "x".to_owned(),
        runs: Vec::new(),
    };
    let mut slowest = Duration::ZERO;
    for _ in 0..RUNS {
        let start = Instant::now();
        let edited = edit_shown(&text, &hide, &key);
        slowest = slowest.max(start.elapsed());
        assert_eq!(edited.shown.len(), 1);
    }
    println!("a keystroke, slowest of {RUNS} runs: {slowest:?} (budget {KEYSTROKE_BUDGET:?})");
    assert!(
        slowest <= KEYSTROKE_BUDGET,
        "{slowest:?} is over {KEYSTROKE_BUDGET:?}"
    );
}

/// Ctrl+A, Ctrl+B over 120 pages. Lines that cannot take bold (a parenthetical, `CUT TO:`, a cue
/// with an extension) stay as they are; finding that out must not read the book once per line.
#[test]
#[ignore = "timing, run by `make perf` in release"]
fn bold_over_the_whole_script_of_120_pages_is_quick() {
    let text = screenplay().replace("Nobody moves.", "Nobody *moves*.");
    let lines = text.split('\n').count();
    let last = Spot {
        line: lines - 1,
        col: 0,
    };
    let all = (Spot { line: 0, col: 0 }, last);
    let mut slowest = Duration::ZERO;

    for _ in 0..RUNS {
        let start = Instant::now();
        let done = toggle_mark(&text, &[Hide::Emphasis], all, Mark::Bold);
        slowest = slowest.max(start.elapsed());
        assert!(done.source.contains("**Rain taps"));
    }

    println!("{lines} lines, bold over all, slowest of {RUNS} runs: {slowest:?} (budget {TOGGLE_BUDGET:?})");
    assert!(
        slowest <= TOGGLE_BUDGET,
        "{slowest:?} is over {TOGGLE_BUDGET:?}"
    );
}
