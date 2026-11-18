//! The status line: the mode word and the notice stay, the rest gives way.

use unicode_width::UnicodeWidthStr;

use super::support::*;
use super::*;
use crate::app::Notice;
use klappe_services::BookReport;

const GOLDEN_FOLDER: &str = "kt-status-film.klappe";
const MODE_CHIP: &str = " TEXT ";
const PDF_NOTICE: &str = "PDF written to out.pdf";

/// What the status line showed before notices got priority.
const UNCHANGED: [(u16, Language, &str); 4] = [
    (80, Language::German, " TEXT  kt-status-film.klappe  Szene 1/1  Szenen nicht im Buch: 1.  1:1  F1 Hilfe  ^P Sze"),
    (80, Language::English, " TEXT  kt-status-film.klappe  Scene 1/1  Scenes not in the script: 1.  1:1  F1 Help  ^P Scene  ^F Find"),
    (100, Language::German, " TEXT  kt-status-film.klappe  Szene 1/1  Szenen nicht im Buch: 1.  1:1  F1 Hilfe  ^P Szene  ^F Suchen"),
    (100, Language::English, " TEXT  kt-status-film.klappe  Scene 1/1  Scenes not in the script: 1.  1:1  F1 Help  ^P Scene  ^F Find"),
];

/// A project in a folder of its own, so tests do not remove each other's.
fn film(n: usize) -> Scratch {
    Scratch::named(
        "INT. HOF – TAG\n\nEin Hund.\n",
        &format!("kt-status-{n}-film.klappe"),
    )
}

/// The last row of the screen, without the trailing blanks.
fn status_row(app: &mut App, width: u16) -> String {
    let frame = screen(app, width, 24);

    frame
        .lines()
        .last()
        .unwrap_or_default()
        .trim_end()
        .to_owned()
}

fn with_notice(scratch: &Scratch, notice: Notice) -> App {
    let mut app = scratch.app();
    app.notice = Some(notice);

    app
}

#[test]
fn without_a_notice_the_status_line_is_unchanged() {
    let scratch = Scratch::named("INT. HOF – TAG\n\nEin Hund.\n", GOLDEN_FOLDER);
    // HOF is a play location: no question about it adds to the line.
    let places = "[[ort]]\nid = \"hofhofho\"\nname = \"HOF\"\n";
    std::fs::write(scratch.dir.join("orte.toml"), places).unwrap_or_else(|e| unreachable!("{e}"));
    for (width, language, expected) in UNCHANGED {
        let mut app = App::open(&scratch.dir, language).unwrap_or_else(|e| unreachable!("{e}"));
        let row = status_row(&mut app, width);
        let expected: String = expected.chars().take(usize::from(width)).collect();

        assert_eq!(row.trim_end(), expected.trim_end(), "{width} {language:?}");
    }
}

/// A book note of 1 scene to match, 1 gone from the book, or 1 location to match.
const BOOK_NOTES: [BookReport; 3] = [
    BookReport {
        pending: 1,
        not_in_book: 0,
        figures: 0,
        places: 0,
    },
    BookReport {
        pending: 0,
        not_in_book: 1,
        figures: 0,
        places: 0,
    },
    BookReport {
        pending: 0,
        not_in_book: 0,
        figures: 0,
        places: 1,
    },
];

#[test]
fn a_book_note_leaves_cursor_and_first_hint_at_80_columns() {
    // A folder name of 14 characters.
    let scratch = Scratch::named("INT. HOF – TAG\n\nEin Hund.\n", "kt-film5.klappe");

    for language in [Language::German, Language::English] {
        for report in BOOK_NOTES {
            let mut app = App::open(&scratch.dir, language).unwrap_or_else(|e| unreachable!("{e}"));
            app.book_note = klappe_services::book_report_text(language, &report);
            let row = status_row(&mut app, 80);

            assert!(
                row.contains(&app.book_note),
                "{language:?} {report:?}: {row}"
            );
            assert!(row.contains("  1:1  "), "{language:?} {report:?}: {row}");
            assert!(
                row.contains(app.texts.hints.split("  ").next().unwrap_or_default()),
                "{language:?} {report:?}: {row}"
            );
        }
    }
}

#[test]
fn the_mode_word_stays_next_to_a_long_notice() {
    let scratch = film(1);
    let long = Notice::SaveFailed("x".repeat(200));

    for width in [36u16, 80] {
        let mut app = with_notice(&scratch, long.clone());
        let row = status_row(&mut app, width);

        assert!(
            row.starts_with(&format!("{MODE_CHIP} Saving failed")),
            "{width}: {row}"
        );
        assert_eq!(row.chars().count(), usize::from(width), "{width}: {row}");
    }

    let mut app = with_notice(&scratch, Notice::SaveFirst);
    let row = status_row(&mut app, 36);
    let notice = app.texts.book_save_first.clone();

    assert!(
        row.starts_with(&format!("{MODE_CHIP} {}", &notice[..20])),
        "{row}"
    );
}

#[test]
fn a_gap_separates_the_notice_from_the_left_part() {
    let scratch = film(2);
    let notice = "Saving failed: ".to_owned() + &"x".repeat(30);

    for width in [60u16, 70, 100] {
        let mut app = with_notice(&scratch, Notice::SaveFailed("x".repeat(30)));
        let row = status_row(&mut app, width);

        assert!(row.ends_with(&format!(" {notice}")), "{width}: {row}");
    }
}

#[test]
fn the_book_note_gives_way_before_the_file_name() {
    let scratch = film(3);
    let mut app = with_notice(&scratch, Notice::Pdf(PDF_NOTICE.into()));
    app.book_note = "BOOKNOTE-XYZ".into();

    let wide = status_row(&mut app, 100);
    assert!(wide.contains("BOOKNOTE-XYZ"), "{wide}");

    let narrow = status_row(&mut app, 74);
    assert!(!narrow.contains("BOOKNOTE"), "{narrow}");
    assert!(narrow.contains(app.file_name()), "{narrow}");
    assert!(narrow.contains("Scene 1/1"), "{narrow}");
    assert!(narrow.ends_with(PDF_NOTICE), "{narrow}");
}

#[test]
fn the_file_name_is_cut_before_the_rest() {
    let scratch = film(4);
    let mut app = with_notice(&scratch, Notice::Pdf(PDF_NOTICE.into()));
    let row = status_row(&mut app, 60);

    assert!(row.contains('…'), "{row}");
    assert!(!row.contains("film.klappe"), "{row}");
    assert!(row.contains("Scene 1/1"), "{row}");
    assert!(row.contains("1:1"), "{row}");
    assert!(row.ends_with(PDF_NOTICE), "{row}");
}

#[test]
fn a_wide_file_name_is_cut_by_display_width() {
    let name = format!("kt-{}.klappe", "漢".repeat(30));
    let scratch = Scratch::named("INT. HOF – TAG\n\nEin Hund.\n", &name);
    let mut app = with_notice(&scratch, Notice::Pdf(PDF_NOTICE.into()));
    let row = status_row(&mut app, 80);

    assert!(row.ends_with(PDF_NOTICE), "{row}");
    assert!(row.contains("…  Scene 1/1"), "{row}");
    // One column may stay free when the next wide character does not fit.
    assert!(UnicodeWidthStr::width(row.as_str()) >= 78, "{row}");
}

#[test]
fn a_zero_wide_screen_with_a_notice_draws() {
    let scratch = film(9);
    let mut app = with_notice(&scratch, Notice::Pdf(PDF_NOTICE.into()));

    let _ = screen(&mut app, 0, 24);
}
