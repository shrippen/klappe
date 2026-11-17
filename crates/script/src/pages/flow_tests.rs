//! The flow as one model: the reviewer's cases and invariants over generated screenplays.

use super::*;
use crate::parse;

fn lay(text: &str, body_lines: usize) -> Pagination {
    let mut options = PageOptions::new(PageSize::A4, Words::ENGLISH);
    options.body_lines = body_lines;

    paginate(&parse(text).elements, &options)
}

/// Rows of text of a page without the names and `(MORE)`: what counts as "a line of a block".
fn body_rows(page: &Page) -> usize {
    let mut rows: Vec<usize> = page
        .lines
        .iter()
        .filter(|l| !matches!(l.kind, PageKind::Character | PageKind::More))
        .map(|l| l.row)
        .collect();
    rows.sort_unstable();
    rows.dedup();

    rows.len()
}

/// A thin page is right only when the block that starts the next page, in the rows from its
/// top to the first blank row, could not have started on it.
fn unavoidable(page: &Page, next: &Page, body_lines: usize) -> bool {
    let used = page.lines.iter().map(|l| l.row + 1).max().unwrap_or(0);
    let mut run = 0;
    while next.lines.iter().any(|l| l.row == run) {
        run += 1;
    }

    used + 1 + run > body_lines
}

fn last_row_kinds(page: &Page) -> Vec<PageKind> {
    let last = page.lines.iter().map(|l| l.row).max();

    page.lines
        .iter()
        .filter(|l| Some(l.row) == last)
        .map(|l| l.kind)
        .collect()
}

/// The reviewer's case: 40 rows of action, a name, 30 lines of `word n` and `end.`, then 39
/// rows of action. The dialogue has no sentence end before `end.`, so it is not cut; it
/// stands whole on the next page, and nothing is cut without (MORE) and (CONT'D).
#[test]
fn a_dialogue_without_a_cut_place_moves_whole() {
    let action: Vec<String> = (0..40).map(|i| format!("Action {i}")).collect();
    let words: Vec<String> = (0..30).map(|i| format!("word {i}")).collect();
    let after: Vec<String> = (0..39).map(|i| format!("After {i}")).collect();
    let text = format!(
        "{}\n\nANNA\n{}\nend.\n\n{}\n",
        action.join("\n"),
        words.join("\n"),
        after.join("\n")
    );
    let p = lay(&text, 59);

    for page in &p.pages {
        let more = page.lines.iter().any(|l| l.kind == PageKind::More);
        assert!(!more, "{page:?}");
    }
    assert_eq!(p.pages[0].lines.len(), 40);
    assert_eq!(p.pages[1].lines[0].text, "ANNA");
}

/// With sentence ends the same dialogue is cut after `end.`-like ends, with the markers.
#[test]
fn a_dialogue_with_sentence_ends_is_cut_with_markers() {
    let action: Vec<String> = (0..40).map(|i| format!("Action {i}")).collect();
    let words: Vec<String> = (0..30).map(|i| format!("Word {i}.")).collect();
    let text = format!("{}\n\nANNA\n{}\n", action.join("\n"), words.join("\n"));
    let p = lay(&text, 59);

    let first = last_row_kinds(&p.pages[0]);
    assert_eq!(first, [PageKind::More]);
    assert_eq!(p.pages[1].lines[0].text, "ANNA (CONT'D)");
}

#[test]
fn forty_headings_take_about_two_pages() {
    let text: String = (0..40)
        .map(|i| format!("INT. ROOM {i} - DAY\n\n"))
        .collect();
    let p = lay(&text, 59);

    assert!((2..=3).contains(&p.pages.len()), "{}", p.pages.len());
}

#[test]
fn two_thousand_headings_take_a_page_per_the_lines_not_per_heading() {
    let text: String = (0..2000)
        .map(|i| format!("INT. ROOM {i} - DAY\n\n"))
        .collect();
    let p = lay(&text, 59);

    // 3 rows each, 59 rows a page: about 102 pages.
    assert!((100..=110).contains(&p.pages.len()), "{}", p.pages.len());
}

/// A small deterministic random source, so a failure repeats.
struct Rng(u64);

impl Rng {
    fn next(&mut self, below: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);

        (self.0 >> 33) as usize % below
    }
}

fn sentences(rng: &mut Rng, count: usize, joiner: &str) -> String {
    (0..count)
        .map(|_| {
            let words = 2 + rng.next(14);
            let text = (0..words).map(|_| "word").collect::<Vec<_>>().join(" ");
            format!(
                "{}{text}.",
                if rng.next(2) == 0 { "Some " } else { "More " }
            )
        })
        .collect::<Vec<_>>()
        .join(joiner)
}

/// A random screenplay of headings, action, dialogue (also dual and with a parenthetical),
/// transitions, centred text and lyrics.
fn script(rng: &mut Rng, blocks: usize) -> String {
    let mut text = String::new();
    let mut headings_in_a_row = 0;

    for _ in 0..blocks {
        let kind = rng.next(10);
        if kind == 0 && headings_in_a_row < 3 {
            headings_in_a_row += 1;
            text.push_str(&format!("INT. PLACE {} - DAY\n\n", rng.next(100)));
            continue;
        }
        headings_in_a_row = 0;

        let joiner = if rng.next(2) == 0 { " " } else { "\n" };
        let count = 1 + rng.next(8);
        let body = sentences(rng, count, joiner);
        let block = match kind {
            1..=3 => body,
            4 | 5 => format!("ANNA\n{body}"),
            6 => format!("ANNA\n(softly)\n{}", body),
            7 => format!("ANNA\n{body}\n\nBEN ^\n{}", sentences(rng, 2, " ")),
            8 => "CUT TO:".to_string(),
            _ => format!("~{}", sentences(rng, 2, "\n~")),
        };
        text.push_str(&block);
        text.push_str("\n\n");
    }

    text
}

fn ends_a_sentence(text: &str) -> bool {
    let text = text
        .trim_end()
        .trim_end_matches(['"', '\'', ')', '”', '»', '’']);

    text.ends_with(['.', '!', '?', '…'])
}

#[test]
fn every_cut_and_page_keeps_the_rules() {
    let mut rng = Rng(7);

    for round in 0..300 {
        let body_lines = 8 + rng.next(30);
        let blocks = 5 + rng.next(60);
        let text = script(&mut rng, blocks);
        let p = lay(&text, body_lines);
        let ctx = format!("round {round}, {body_lines} rows\n{text}");
        let last = p.pages.len().saturating_sub(1);

        for (n, page) in p.pages.iter().enumerate() {
            // Every page but the last has at least two lines of body.
            if n != last && body_rows(page) < 2 {
                assert!(
                    unavoidable(page, &p.pages[n + 1], body_lines),
                    "page {} thin: {ctx}",
                    n + 1
                );
            }
            assert!(page.lines.iter().all(|l| l.row < body_lines), "{ctx}");

            let ends = last_row_kinds(page);
            // No heading last on a page unless the script ends there.
            if n != last {
                assert!(
                    !ends.contains(&PageKind::Heading),
                    "heading last, page {}: {ctx}",
                    n + 1
                );
            }

            // (MORE) ends a page and the next page opens with the name again, and back.
            let has_more = ends == [PageKind::More];
            let next_contd = p.pages.get(n + 1).is_some_and(|q| q.lines[0].added);
            assert_eq!(has_more, next_contd, "page {}: {ctx}", n + 1);
            assert!(
                page.lines
                    .iter()
                    .filter(|l| l.kind == PageKind::More)
                    .count()
                    <= 1,
                "{ctx}"
            );
        }

        // An action cut falls at a sentence end, unless the action is taller than a page.
        for (n, page) in p.pages.iter().enumerate().take(last) {
            let (Some(tail), Some(head)) = (
                page.lines.iter().max_by_key(|l| l.row),
                p.pages[n + 1].lines.first(),
            ) else {
                continue;
            };
            if tail.kind != PageKind::Action || head.element != tail.element {
                continue;
            }

            let rows = p
                .pages
                .iter()
                .flat_map(|q| &q.lines)
                .filter(|l| l.element == tail.element)
                .count();
            assert!(
                ends_a_sentence(&tail.text) || rows > body_lines,
                "{}: {ctx}",
                tail.text
            );
        }
    }
}
