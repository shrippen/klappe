//! Keys and one rendered frame on the example project, without a terminal.

use std::path::{Path, PathBuf};

use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use ratatui::Terminal;

use crate::app::{Focus, Mode};
use crate::{draw, App, Flow, Language};

fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe")
}

fn app() -> App {
    App::open(&example(), Language::En).unwrap_or_else(|e| unreachable!("{e}"))
}

fn press(app: &mut App, code: KeyCode) -> Flow {
    app.handle_key(KeyEvent::new(code, KeyModifiers::NONE))
}

fn ctrl(app: &mut App, c: char) -> Flow {
    app.handle_key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL))
}

fn typed(app: &mut App, text: &str) {
    for c in text.chars() {
        press(app, KeyCode::Char(c));
    }
}

fn screen(app: &mut App, width: u16, height: u16) -> String {
    let mut terminal =
        Terminal::new(TestBackend::new(width, height)).unwrap_or_else(|e| unreachable!("{e}"));
    terminal
        .draw(|f| draw(f, app))
        .unwrap_or_else(|e| unreachable!("{e}"));
    let buffer = terminal.backend().buffer();

    (0..height)
        .map(|y| {
            (0..width)
                .map(|x| buffer[(x, y)].symbol().to_owned())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn frame_shows_scene_list_text_and_status() {
    let mut app = app();
    let frame = screen(&mut app, 120, 20);

    assert!(frame.contains("Scenes"), "{frame}");
    assert!(frame.contains("Screenplay"), "{frame}");
    assert!(frame.contains("12  INT. SÄNGERSAAL – TAG"), "{frame}");
    assert!(frame.contains("Elisabeth tritt an den Tisch."), "{frame}");
    assert!(frame.contains("beispiel.klappe"), "{frame}");
    assert!(frame.contains("1:1"), "{frame}");
    assert!(!frame.contains("[+]"), "{frame}");
}

fn column(frame: &str, needle: &str) -> usize {
    let line = frame
        .lines()
        .find(|l| l.contains(needle))
        .unwrap_or_default();
    let byte = line.find(needle).unwrap_or(0);
    line[..byte].chars().count()
}

#[test]
fn dialogue_is_indented_in_display_only() {
    let mut app = app();
    let frame = screen(&mut app, 100, 20);

    let action = column(&frame, "Elisabeth tritt an den Tisch.");
    let cue = column(&frame, "ELISABETH");
    let speech = column(&frame, "Ihr kommt spät.");

    assert_eq!(cue - action, 20, "{frame}");
    assert_eq!(speech - action, 10, "{frame}");
    assert!(!app.text().contains("  ELISABETH"));
}

#[test]
fn help_overlay_opens_with_f1_and_closes_with_any_key() {
    let mut app = app();
    press(&mut app, KeyCode::F(1));
    assert!(screen(&mut app, 100, 24).contains("Ctrl+Z / Ctrl+Y  Undo / redo"));

    press(&mut app, KeyCode::Char('x'));
    assert_eq!(app.mode, Mode::Normal);
    assert!(!app.is_modified());
}

#[test]
fn german_frame_has_german_texts() {
    let mut app = App::open(&example(), Language::De).unwrap_or_else(|e| unreachable!("{e}"));
    let frame = screen(&mut app, 100, 20);

    assert!(frame.contains("Szenen"), "{frame}");
    assert!(frame.contains("Drehbuch"), "{frame}");
}

#[test]
fn typing_marks_modified_and_undo_keeps_the_mark() {
    let mut app = app();
    typed(&mut app, "Äh");
    assert!(app.is_modified());
    assert!(app.text().starts_with("ÄhINT."));
    assert!(screen(&mut app, 100, 20).contains("[+]"));

    ctrl(&mut app, 'z');
    assert!(app.text().starts_with("INT."));
    ctrl(&mut app, 'y');
    assert!(app.text().starts_with("ÄhINT."));
}

#[test]
fn quit_is_immediate_when_unmodified() {
    let mut app = app();
    assert_eq!(ctrl(&mut app, 'q'), Flow::Quit);
}

#[test]
fn quit_asks_when_modified() {
    let mut app = app();
    typed(&mut app, "a");

    assert_eq!(ctrl(&mut app, 'q'), Flow::Continue);
    assert!(screen(&mut app, 100, 20).contains("Quit anyway? (y/n)"));

    assert_eq!(press(&mut app, KeyCode::Char('n')), Flow::Continue);
    assert_eq!(app.mode, Mode::Normal);

    ctrl(&mut app, 'q');
    assert_eq!(press(&mut app, KeyCode::Char('y')), Flow::Quit);
}

/// A second scene in front of the example's one: headings at lines 0 and 2.
fn two_scenes() -> App {
    let mut app = app();
    typed(&mut app, "EXT. HOF");
    press(&mut app, KeyCode::Enter);
    press(&mut app, KeyCode::Enter);

    app
}

#[test]
fn list_follows_the_text() {
    let mut app = two_scenes();
    let frame = screen(&mut app, 120, 20);

    assert!(frame.contains("1  EXT. HOF"), "{frame}");
    assert!(frame.contains("12  INT. SÄNGERSAAL"), "{frame}");
}

#[test]
fn tab_and_enter_jump_to_the_selected_scene() {
    let mut app = two_scenes();
    press(&mut app, KeyCode::Tab);
    assert_eq!(app.focus, Focus::List);

    press(&mut app, KeyCode::Up);
    press(&mut app, KeyCode::Enter);
    assert_eq!((app.focus, app.cursor().line), (Focus::Editor, 0));

    press(&mut app, KeyCode::Tab);
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Enter);
    assert_eq!((app.focus, app.cursor().line), (Focus::Editor, 2));
}

#[test]
fn ctrl_g_jumps_from_the_list_selection() {
    let mut app = two_scenes();
    press(&mut app, KeyCode::Tab);
    press(&mut app, KeyCode::Home);
    press(&mut app, KeyCode::End);
    ctrl(&mut app, 'g');

    assert_eq!((app.focus, app.cursor().line), (Focus::Editor, 2));
}

#[test]
fn wheel_scrolls_without_moving_the_cursor() {
    let mut app = app();
    let _ = screen(&mut app, 100, 8);
    let before = app.cursor();
    let wheel = MouseEvent {
        kind: MouseEventKind::ScrollDown,
        column: 40,
        row: 3,
        modifiers: KeyModifiers::NONE,
    };
    app.handle_mouse(wheel);

    assert_eq!(app.cursor(), before);
    assert!(app.scroll > 0);
}

#[test]
fn cursor_stays_in_view_while_moving_down() {
    let mut app = app();
    let _ = screen(&mut app, 100, 8);
    for _ in 0..40 {
        press(&mut app, KeyCode::Down);
        let _ = screen(&mut app, 100, 8);
    }

    assert!(app.scroll > 0);
}
