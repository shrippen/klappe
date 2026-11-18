//! Search and replace, and the title page form (DB-07, DB-06).

use klappe_services::{parse, TITLE_KEYS};

use super::support::*;
use super::*;

const BOOK: &str = "INT. A - TAG\n\nAnna sieht Anna.\n\nANNA\nIch bin anna.\n\nBEN\nANNA ist da.";

fn find_app() -> (Scratch, App) {
    let (s, mut app) = app_with(BOOK);
    keys(&mut app, &[KeyCode::PageUp, KeyCode::PageUp]);
    press(&mut app, KeyCode::Home);
    ctrl(&mut app, 'f');

    (s, app)
}

fn at(app: &App) -> (usize, usize) {
    (app.cursor().line, app.cursor().col)
}

#[test]
fn ctrl_f_finds_as_you_type_and_steps_round() {
    let (_s, mut app) = find_app();
    assert_eq!(app.mode, Mode::Find);
    typed(&mut app, "anna");

    // Four lines hold five matches, case ignored.
    assert_eq!(app.find.hits.len(), 5);
    assert_eq!(at(&app), (2, 0));

    press(&mut app, KeyCode::Enter);
    assert_eq!(at(&app), (2, 11));
    for _ in 0..4 {
        press(&mut app, KeyCode::Enter);
    }
    assert_eq!(at(&app), (2, 0), "wraps round");

    ctrl(&mut app, 'p');
    assert_eq!(at(&app), (8, 0), "previous wraps too");
    press(&mut app, KeyCode::Down);
    assert_eq!(at(&app), (2, 0));
    press(&mut app, KeyCode::Up);
    assert_eq!(at(&app), (8, 0));
}

#[test]
fn the_case_option_narrows_the_matches() {
    let (_s, mut app) = find_app();
    typed(&mut app, "Anna");
    ctrl(&mut app, 't');

    assert_eq!(app.find.hits.len(), 2);
    ctrl(&mut app, 't');
    assert_eq!(app.find.hits.len(), 5);
}

#[test]
fn replace_one_goes_on_to_the_next() {
    let (_s, mut app) = find_app();
    typed(&mut app, "anna");
    press(&mut app, KeyCode::Tab);
    typed(&mut app, "Eva");
    press(&mut app, KeyCode::Enter);

    assert_eq!(app.text().lines().nth(2), Some("Eva sieht Anna."));
    assert_eq!(app.find.hits.len(), 4);
    assert_eq!(at(&app), (2, 10), "on the next match");
    ctrl(&mut app, 'r');
    assert_eq!(app.text().lines().nth(2), Some("Eva sieht Eva."));
}

#[test]
fn replace_all_is_one_undo_step() {
    let (_s, mut app) = find_app();
    typed(&mut app, "anna");
    press(&mut app, KeyCode::Tab);
    typed(&mut app, "Eva");
    ctrl(&mut app, 'a');

    assert_eq!(
        app.text(),
        "INT. A - TAG\n\nEva sieht Eva.\n\nEva\nIch bin Eva.\n\nBEN\nEva ist da."
    );
    assert!(screen(&mut app, 100, 24).contains("5 replaced"));
    assert_eq!(app.find.hits.len(), 0);

    press(&mut app, KeyCode::Esc);
    ctrl(&mut app, 'z');
    assert_eq!(app.text(), BOOK);
}

#[test]
fn a_character_limits_the_search_to_her_dialogue() {
    let (_s, mut app) = find_app();
    typed(&mut app, "anna");
    press(&mut app, KeyCode::Tab);
    press(&mut app, KeyCode::Tab);
    typed(&mut app, "Anna");

    assert_eq!(app.find.hits.len(), 1);
    assert_eq!(at(&app), (5, 8));

    press(&mut app, KeyCode::BackTab);
    press(&mut app, KeyCode::Char('X'));
    press(&mut app, KeyCode::BackTab);
    ctrl(&mut app, 'a');
    assert_eq!(app.text().lines().nth(5), Some("Ich bin X."));
    assert_eq!(app.text().lines().nth(2), Some("Anna sieht Anna."));
}

#[test]
fn f3_repeats_the_search_outside_the_box() {
    let (_s, mut app) = find_app();
    typed(&mut app, "anna");
    press(&mut app, KeyCode::Esc);
    assert_eq!(app.mode, Mode::Normal);

    press(&mut app, KeyCode::F(3));
    assert_eq!(at(&app), (2, 11));
    shift(&mut app, KeyCode::F(3));
    assert_eq!(at(&app), (2, 0));
}

#[test]
fn f3_finds_again_after_the_text_changed() {
    let (_s, mut app) = find_app();
    typed(&mut app, "anna");
    press(&mut app, KeyCode::Esc);
    assert_eq!(at(&app), (2, 0));

    // Text typed before the match moves it; F3 must look at the text as it is now.
    typed(&mut app, "ab ");
    assert_eq!(at(&app), (2, 3));
    press(&mut app, KeyCode::F(3));
    assert_eq!(at(&app), (2, 14));
}

#[test]
fn matches_are_marked_on_screen_and_the_box_shows_the_count() {
    let (_s, mut app) = find_app();
    typed(&mut app, "anna");
    let buffer = render(&mut app, 100, 24);
    let frame = text_of(&buffer, 100, 24);

    assert!(frame.contains("Find and replace"), "{frame}");
    assert!(frame.contains("1/5"), "{frame}");
    assert!(frame.contains("Ignore case"), "{frame}");
    let (x, y) = find(&buffer, 100, 24, "Ich bin anna.");
    let cell = &buffer[(x + 8, y)];
    assert!(
        cell.modifier.contains(ratatui::style::Modifier::UNDERLINED),
        "{cell:?}"
    );
}

fn fill_title(app: &mut App, values: &[&str]) {
    ctrl(app, 't');
    for value in values {
        typed(app, value);
        press(app, KeyCode::Tab);
    }
}

#[test]
fn the_title_form_writes_the_title_page() {
    let (_s, mut app) = app_with(BOOK);
    let before = parse(BOOK).elements;
    fill_title(
        &mut app,
        &["Nachtwache", "von", "Eva", "Roman", "1.1.2027", "eva@x.de"],
    );
    assert!(screen(&mut app, 100, 24).contains("Draft date"));
    press(&mut app, KeyCode::Enter);

    let text = app.text();
    assert!(
        text.starts_with(
            "Title: Nachtwache\nCredit: von\nAuthor: Eva\nSource: Roman\nDraft date: 1.1.2027\nContact: eva@x.de\n\nINT. A - TAG"
        ),
        "{text}"
    );
    let script = parse(&text);
    assert_eq!(script.title_page.len(), TITLE_KEYS.len());
    assert_eq!(script.elements, before, "the body is as it was");
    assert!(app.is_modified());

    ctrl(&mut app, 'z');
    assert_eq!(app.text(), BOOK);
}

#[test]
fn the_form_shows_what_the_title_page_holds_and_keeps_other_keys() {
    let book = format!("Title: Alt\nCopyright: X\nContact:\n   Eva\n   Weg 1\n\n{BOOK}");
    let (_s, mut app) = app_with(&book);
    ctrl(&mut app, 't');
    let frame = screen(&mut app, 100, 24);
    assert!(frame.contains("Alt"), "{frame}");
    assert!(frame.contains("Eva"), "{frame}");
    assert!(frame.contains("Weg 1"), "{frame}");

    // Rename the title, leave the contact as it is.
    for _ in 0..3 {
        press(&mut app, KeyCode::Backspace);
    }
    typed(&mut app, "Neu");
    press(&mut app, KeyCode::Enter);

    assert_eq!(
        app.text(),
        format!("Title: Neu\nCopyright: X\nContact:\n   Eva\n   Weg 1\n\n{BOOK}")
    );
}

#[test]
fn esc_leaves_the_form_unchanged_and_emptied_fields_go() {
    let (_s, mut app) = app_with(&format!("Title: Alt\nAuthor: A\n\n{BOOK}"));
    ctrl(&mut app, 't');
    typed(&mut app, "xyz");
    press(&mut app, KeyCode::Esc);
    assert!(!app.is_modified());
    assert_eq!(app.mode, Mode::Normal);

    ctrl(&mut app, 't');
    ctrl(&mut app, 'u');
    keys(&mut app, &[KeyCode::Tab, KeyCode::Tab]);
    ctrl(&mut app, 'u');
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.text(), BOOK);
}

#[test]
fn the_title_page_lines_show_quiet() {
    let (_s, mut app) = app_with(&format!("Title: Alt\n\n{BOOK}"));
    let buffer = render(&mut app, 100, 24);
    let (x, y) = find(&buffer, 100, 24, "Title: Alt");

    assert_eq!(buffer[(x, y)].fg, theme::MUTED);
}

#[test]
fn alt_enter_adds_a_line_to_a_title_field() {
    let (_s, mut app) = app_with(BOOK);
    ctrl(&mut app, 't');
    keys(&mut app, &[KeyCode::Tab, KeyCode::Tab]);
    typed(&mut app, "Eva");
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::ALT));
    typed(&mut app, "Bert");
    // A line left empty is dropped.
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::ALT));
    let frame = screen(&mut app, 100, 24);
    assert!(frame.contains("Eva") && frame.contains("Bert"), "{frame}");

    press(&mut app, KeyCode::Enter);

    let text = app.text();
    assert!(
        text.starts_with("Author:\n   Eva\n   Bert\n\nINT. A - TAG"),
        "{text}"
    );
    assert_eq!(
        parse(&text).title_page,
        vec![("Author".into(), "Eva\nBert".into())]
    );

    // Reopened, the lines are there; Backspace takes the line break away.
    ctrl(&mut app, 't');
    keys(&mut app, &[KeyCode::Tab, KeyCode::Tab]);
    for _ in 0..5 {
        press(&mut app, KeyCode::Backspace);
    }
    press(&mut app, KeyCode::Enter);
    assert!(app.text().starts_with("Author: Eva\n\n"), "{}", app.text());

    ctrl(&mut app, 'z');
    ctrl(&mut app, 'z');
    assert_eq!(app.text(), BOOK);
}

#[test]
fn a_long_title_field_scrolls_and_keeps_frame_and_hint() {
    let (_s, mut app) = app_with(BOOK);
    ctrl(&mut app, 't');
    for _ in 0..5 {
        press(&mut app, KeyCode::Tab);
    }
    for i in 0..30 {
        typed(&mut app, &format!("Zeile{i}"));
        app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::ALT));
    }
    typed(&mut app, "Letzte");

    let frame = screen(&mut app, 100, 24);
    assert!(frame.contains("Letzte"), "{frame}");
    assert!(frame.contains(app.texts.title_hint.as_str()), "{frame}");
    assert!(frame.contains('└'), "{frame}");
}
