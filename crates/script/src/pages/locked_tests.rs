//! Locked pages (DB-34): pages keep their numbers when the text changes.

use super::*;
use crate::parse_located;

const BODY_LINES: usize = 12;

/// A scene of `count` action paragraphs, each two rows long, and a dialogue after every third.
fn story(count: usize) -> String {
    let mut text = String::from("INT. ROOM - DAY\n\n");

    for n in 1..=count {
        text.push_str(&format!(
            "Paragraph {n} is long enough to run over two rows of the page, no more. \
             It ends here.\n\n"
        ));
        if n % 3 == 0 {
            text.push_str(&format!(
                "ANNA\nWords number {n}. They go on and on over rows. Then they stop. \
                 And a last sentence, to be sure. Even a fourth one follows it up. \
                 Say a fifth to fill the rows. The sixth is the end of it.\n\n"
            ));
        }
    }

    text
}

fn options() -> PageOptions {
    let mut options = PageOptions::new(PageSize::A4, Words::ENGLISH);
    options.body_lines = BODY_LINES;

    options
}

fn lines(text: &str) -> usize {
    text.replace("\r\n", "\n").matches('\n').count() + 1
}

/// The pages of `text` as the first lock sees them and their starts.
fn lock(text: &str) -> (Pagination, Vec<PageStart>) {
    let located = parse_located(text);
    let elements: Vec<Element> = located.iter().map(|l| l.element.clone()).collect();
    let pages = paginate(&elements, &options());
    let starts = page_starts(&pages, &located, lines(text));

    (pages, starts)
}

/// `text` paged on the starts of a version locked on `old`.
fn locked(old: &str, starts: &[PageStart], text: &str) -> Pagination {
    let located = parse_located(text);
    let elements: Vec<Element> = located.iter().map(|l| l.element.clone()).collect();

    paginate_locked(&elements, &located, &options(), &carry(old, text, starts))
}

fn labels(pages: &Pagination) -> Vec<&str> {
    pages.pages.iter().map(|p| p.label.as_str()).collect()
}

/// The text of the first line of text on a page.
fn first_text(page: &Page) -> &str {
    page.lines
        .iter()
        .find(|l| !l.added)
        .map_or("", |l| l.text.as_str())
}

/// `text` with the lines `from..to` replaced by `with`.
fn replace_lines(text: &str, from: usize, to: usize, with: &str) -> String {
    let lines: Vec<&str> = text.split('\n').collect();
    let mut out: Vec<&str> = lines[..from].to_vec();
    out.extend(with.split('\n').filter(|_| !with.is_empty()));
    out.extend(&lines[to..]);

    out.join("\n")
}

/// The first line of the paragraph (name and words of a dialogue) that `line` is in.
fn paragraph(text: &str, line: usize) -> usize {
    let lines: Vec<&str> = text.split('\n').collect();

    (0..=line)
        .rev()
        .find(|&at| at == 0 || lines[at - 1].is_empty())
        .unwrap_or(0)
}

/// Three paragraphs of their own, six lines.
const EXTRA: &str = "Added one is long enough to run over two rows of the page, no more.\n\n\
                     Added two is long enough to run over two rows of the page, no more.\n\n\
                     Added three is long enough to run over two rows of the page, no more.\n";

fn rows(pages: &Pagination) -> Vec<Vec<(usize, usize, String)>> {
    let page = |p: &Page| {
        p.lines
            .iter()
            .map(|l| (l.row, l.x, l.text.clone()))
            .collect()
    };

    pages.pages.iter().map(page).collect()
}

#[test]
fn the_next_label_adds_a_letter() {
    for (from, to) in [
        ("12", "12A"),
        ("12A", "12B"),
        ("12Z", "12AA"),
        ("12AZ", "12BA"),
        ("1", "1A"),
    ] {
        assert_eq!(page_label_after(from), to);
    }
}

#[test]
fn text_that_did_not_change_pages_as_it_did() {
    for count in [8, 12, 20, 30] {
        let text = story(count);
        let (pages, starts) = lock(&text);
        assert!(pages.pages.len() > 2, "{count}");

        let again = locked(&text, &starts, &text);

        assert_eq!(rows(&again), rows(&pages), "{count}");
        assert_eq!(labels(&again), labels(&pages), "{count}");
    }
}

#[test]
fn a_dialogue_cut_by_a_page_keeps_its_cut() {
    // Over many lengths, some put a break in the middle of a dialogue: (MORE) / (CONT'D).
    let mut cuts = 0;

    for count in 8..40 {
        let text = story(count);
        let (pages, starts) = lock(&text);
        let cut = |p: &Page| p.lines.iter().any(|l| l.kind == PageKind::More);
        cuts += pages.pages.iter().filter(|p| cut(p)).count();

        assert_eq!(
            rows(&locked(&text, &starts, &text)),
            rows(&pages),
            "{count}"
        );
    }

    assert!(cuts > 0, "no dialogue was cut");
}

#[test]
fn text_added_to_a_page_runs_over_on_a_page_with_a_letter() {
    let text = story(12);
    let (pages, starts) = lock(&text);
    assert!(pages.pages.len() >= 5, "{}", pages.pages.len());
    let next = first_text(&pages.pages[3]).to_owned();

    // Three paragraphs before the one page 4 starts in.
    let at = paragraph(&text, starts[3].line);
    let grown = replace_lines(&text, at, at, EXTRA);

    let now = locked(&text, &starts, &grown);

    assert_eq!(&labels(&now)[..5], ["1", "2", "3", "3A", "4"]);
    assert_eq!(first_text(&now.pages[4]), next);
    assert_eq!(now.pages.len(), pages.pages.len() + 1);
}

#[test]
fn a_page_taken_out_is_a_sheet_that_says_so() {
    let text = story(12);
    let (pages, starts) = lock(&text);
    assert!(pages.pages.len() >= 6);
    let to = first_text(&pages.pages[5]).to_owned();

    // Everything from the first line of page 5 up to the first line of page 6.
    let cut = replace_lines(&text, starts[4].line, starts[5].line, "");

    let now = locked(&text, &starts, &cut);

    assert_eq!(labels(&now)[3..6], ["4", "5", "6"]);
    assert_eq!(now.pages[4].omitted, ["5"]);
    assert_eq!(first_text(&now.pages[5]), to);
    let line = &now.pages[4].lines[0];
    assert_eq!(line.text, "5 OMITTED");
    assert!(line.added);
}

#[test]
fn several_pages_taken_out_make_one_sheet() {
    let text = story(24);
    let (pages, starts) = lock(&text);
    assert!(pages.pages.len() >= 8, "{}", pages.pages.len());

    let cut = replace_lines(&text, starts[3].line, starts[6].line + 1, "");
    let now = locked(&text, &starts, &cut);

    assert_eq!(now.pages[3].omitted, ["4", "5", "6"]);
    assert_eq!(now.pages[3].lines[0].text, "4-6 OMITTED");
    assert_eq!(now.pages[4].label, "7");
}

#[test]
fn pages_taken_out_at_the_end_are_a_sheet_at_the_end() {
    let text = story(12);
    let (pages, starts) = lock(&text);
    let last = starts.last().unwrap().line;
    let cut = replace_lines(&text, last, lines(&text), "");

    let now = locked(&text, &starts, &cut);

    let end = now.pages.last().unwrap();
    assert_eq!(end.omitted.len(), 1);
    assert_eq!(end.label, pages.pages.last().unwrap().label);
}

#[test]
fn a_page_with_less_text_stays_short() {
    let text = story(12);
    let (pages, starts) = lock(&text);
    let next = first_text(&pages.pages[2]).to_owned();

    // The first paragraph of page 2 and its blank line go.
    let at = starts[1].line;
    let cut = replace_lines(&text, at, at + 2, "");

    let now = locked(&text, &starts, &cut);

    assert_eq!(labels(&now), labels(&pages));
    assert_eq!(first_text(&now.pages[2]), next);
    let used = |p: &Page| p.lines.iter().map(|l| l.row).max().unwrap_or(0);
    assert!(used(&now.pages[1]) < used(&pages.pages[1]));
}

#[test]
fn a_second_lock_keeps_the_added_pages_and_the_struck_ones() {
    let text = story(24);
    let (pages, starts) = lock(&text);
    assert!(pages.pages.len() >= 8);

    // Page 3 grows, page 6 goes.
    let at = paragraph(&text, starts[3].line);
    let grown = replace_lines(&text, at, at, EXTRA);
    let cut = replace_lines(&grown, starts[5].line + 6, starts[6].line + 7, "");
    let located = parse_located(&cut);
    let elements: Vec<Element> = located.iter().map(|l| l.element.clone()).collect();
    let carried = carry(&text, &cut, &starts);
    let second = paginate_locked(&elements, &located, &options(), &carried);
    assert!(labels(&second).contains(&"3A"), "{:?}", labels(&second));
    assert!(second.pages.iter().any(|p| p.omitted == ["6"]));

    // Locked again, the same text pages the same.
    let starts2 = page_starts(&second, &located, lines(&cut));
    let again = locked(&cut, &starts2, &cut);

    assert_eq!(rows(&again), rows(&second));
    assert_eq!(labels(&again), labels(&second));
}

#[test]
fn a_page_start_within_a_line_follows_an_unchanged_line_only() {
    let start = |line, offset| PageStart {
        label: "2".into(),
        line,
        offset,
    };
    let old = "one\ntwo\nthree\nfour";
    let first = PageStart {
        label: "1".into(),
        line: 0,
        offset: 0,
    };

    // A line before is added: the start moves down with its line, offset kept.
    let moved = carry(
        old,
        "new\none\ntwo\nthree\nfour",
        &[first.clone(), start(2, 5)],
    );
    assert_eq!((moved[1].line, moved[1].offset), (3, 5));

    // The line is gone: the next kept line, from its start.
    let gone = carry(old, "one\ntwo\nfour", &[first.clone(), start(2, 5)]);
    assert_eq!((gone[1].line, gone[1].offset), (2, 0));

    // The line is changed: it counts as gone.
    let changed = carry(old, "one\ntwo\nthree!\nfour", &[first.clone(), start(2, 5)]);
    assert_eq!((changed[1].line, changed[1].offset), (3, 0));

    // Nothing kept from the line on: the end.
    let end = carry(old, "one\ntwo", &[first, start(2, 5)]);
    assert_eq!(end[1].line, 2);
}
