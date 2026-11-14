//! Keys and one rendered frame on the example project, without a terminal.

use std::path::{Path, PathBuf};

use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use ratatui::style::Color;
use ratatui::Terminal;

use crate::app::{Focus, Mode};
use crate::{draw, theme, App, Flow, Language};

fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe")
}

fn app() -> App {
    App::open(&example(), Language::English).unwrap_or_else(|e| unreachable!("{e}"))
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

fn render(app: &mut App, width: u16, height: u16) -> Buffer {
    let mut terminal =
        Terminal::new(TestBackend::new(width, height)).unwrap_or_else(|e| unreachable!("{e}"));
    terminal
        .draw(|f| draw(f, app))
        .unwrap_or_else(|e| unreachable!("{e}"));

    terminal.backend().buffer().clone()
}

fn screen(app: &mut App, width: u16, height: u16) -> String {
    text_of(&render(app, width, height), width, height)
}

fn text_of(buffer: &Buffer, width: u16, height: u16) -> String {
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
    assert!(frame.contains("Scene 1/1"), "{frame}");
    assert!(frame.contains("12  INT. SÄNGERSAAL – TAG"), "{frame}");
    assert!(frame.contains("Elisabeth tritt an den Tisch."), "{frame}");
    assert!(frame.contains("beispiel.klappe"), "{frame}");
    assert!(frame.contains("1:1"), "{frame}");
    assert!(!frame.contains("modified"), "{frame}");
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
    let mut app = App::open(&example(), Language::German).unwrap_or_else(|e| unreachable!("{e}"));
    let frame = screen(&mut app, 100, 20);

    assert!(frame.contains("Szenen"), "{frame}");
    assert!(frame.contains("Szene 1/1"), "{frame}");
    assert!(frame.contains("Figuren: 2"), "{frame}");
    assert!(frame.contains("Speichern"), "{frame}");
    assert!(!frame.contains("Scene"), "{frame}");
}

#[test]
fn typing_marks_modified_and_undo_keeps_the_mark() {
    let mut app = app();
    typed(&mut app, "Äh");
    assert!(app.is_modified());
    assert!(app.text().starts_with("ÄhINT."));
    assert!(screen(&mut app, 100, 20).contains("modified"));

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

/// Terminal column of the cursor, relative to the editor's text area.
fn cursor_x(app: &mut App, text: &str) -> u16 {
    typed(app, text);
    let mut terminal =
        Terminal::new(TestBackend::new(120, 20)).unwrap_or_else(|e| unreachable!("{e}"));
    terminal
        .draw(|f| draw(f, app))
        .unwrap_or_else(|e| unreachable!("{e}"));
    let at = terminal
        .get_cursor_position()
        .unwrap_or_else(|e| unreachable!("{e}"));
    // Scene list is 30 columns wide at 120, plus the editor's border.
    at.x - 31
}

#[test]
fn cursor_follows_display_width() {
    assert_eq!(cursor_x(&mut app(), "\tab"), 4 + 2, "tab");
    assert_eq!(cursor_x(&mut app(), "日本語日本語"), 12, "wide");
    assert_eq!(cursor_x(&mut app(), "a😀b"), 4, "emoji");
}

/// Position of the first cell of `needle`, scanning rows below the header.
fn find(buffer: &Buffer, width: u16, height: u16, needle: &str) -> (u16, u16) {
    let text = text_of(buffer, width, height);
    for (y, line) in text.lines().enumerate().skip(1) {
        let Some(byte) = line.find(needle) else {
            continue;
        };
        let x = u16::try_from(line[..byte].chars().count()).unwrap_or(0);
        return (x, u16::try_from(y).unwrap_or(0));
    }
    unreachable!("{needle} not on screen:\n{text}")
}

/// Moves the cursor off the first line, the scene heading.
fn off_heading(app: &mut App) {
    press(app, KeyCode::Down);
    press(app, KeyCode::Down);
}

#[test]
fn active_scene_has_a_yellow_bar() {
    let mut app = two_scenes();
    let buffer = render(&mut app, 120, 20);
    // The cursor is in the second scene; the first one is not marked.
    let (x, y) = find(&buffer, 120, 20, "12  INT. SÄNGERSAAL");
    let bar = &buffer[(x - 2, y)];
    assert_eq!((bar.symbol(), bar.fg), ("▌", theme::ACCENT), "bar");
    assert_eq!(buffer[(x, y)].fg, theme::STRONG, "active text");

    let (x, y) = find(&buffer, 120, 20, "1  EXT. HOF");
    assert_eq!(buffer[(x - 2, y)].symbol(), " ", "no bar");
    assert_eq!(buffer[(x + 3, y)].fg, theme::SOFT, "other text");
}

#[test]
fn ground_and_status_mode_use_kante() {
    let mut app = app();
    let buffer = render(&mut app, 120, 20);

    assert_eq!(buffer[(60, 10)].bg, theme::GROUND);
    let (x, y) = find(&buffer, 120, 20, "TEXT");
    assert_eq!(y, 19, "status line");
    assert_eq!(
        (buffer[(x, y)].bg, buffer[(x, y)].fg),
        (theme::ACCENT, theme::GROUND)
    );
}

#[test]
fn heading_shows_no_number_marker_unless_edited() {
    let mut app = app();
    // The cursor is on the heading: the text as typed.
    assert!(screen(&mut app, 120, 20).contains("#12#"));

    off_heading(&mut app);
    let buffer = render(&mut app, 120, 20);
    let frame = text_of(&buffer, 120, 20);
    assert!(!frame.contains("#12#"), "{frame}");

    // Heading band in the editor (x >= 30): the number sits at its right end, quiet.
    let row: String = (30..120)
        .map(|x| buffer[(x, 1)].symbol().to_owned())
        .collect();
    assert!(row.contains("INT. SÄNGERSAAL – TAG"), "{row}");
    assert!(row.trim_end().ends_with("12"), "{row}");
    assert_eq!(buffer[(117, 1)].fg, theme::MUTED);
    assert_eq!(buffer[(31, 1)].fg, theme::ACCENT);
    assert_eq!(buffer[(119, 1)].bg, theme::SURFACE, "band spans the width");
}

#[test]
fn links_show_as_chips_in_the_figure_colour() {
    let mut app = app();
    off_heading(&mut app);
    let buffer = render(&mut app, 120, 20);

    // Element 0 is the action line with the mark E · M1, element 1 the cue with shot 12.2.
    let (x, y) = find(&buffer, 120, 20, "E · M1");
    assert_eq!(buffer[(x, y)].fg, theme::label("purple"));
    assert_eq!(buffer[(x + 5, y)].symbol(), "1");
    assert_eq!(buffer[(119 - 1, y)].symbol(), "1", "right edge");

    let (x, y) = find(&buffer, 120, 20, "12.2");
    assert_eq!(buffer[(x, y)].fg, theme::SOFT);
    let frame = text_of(&buffer, 120, 20);
    assert!(frame.contains("B · M2"), "{frame}");
    assert_ne!(theme::label("blue"), Color::Reset);
}

#[test]
fn header_counts_the_scene() {
    let mut app = app();
    let frame = screen(&mut app, 120, 20);
    let top = frame.lines().next().unwrap_or_default();

    assert!(top.contains("12  INT. SÄNGERSAAL – TAG"), "{top}");
    assert!(top.contains("Shots: 5"), "{top}");
    assert!(top.contains("Characters: 2"), "{top}");
}

#[test]
fn ctrl_p_filters_by_number_and_heading_and_switches() {
    let mut app = two_scenes();
    ctrl(&mut app, 'p');
    assert_eq!(app.mode, Mode::Goto);
    let frame = screen(&mut app, 120, 20);
    assert!(frame.contains("Go to scene"), "{frame}");
    assert_eq!(app.goto_matches(), [0, 1]);

    typed(&mut app, "sänger");
    assert_eq!(app.goto_matches(), [1], "heading, any case");
    assert!(screen(&mut app, 120, 20).contains("▌ 12  INT. SÄNGERSAAL"));

    press(&mut app, KeyCode::Backspace);
    for _ in 0..5 {
        press(&mut app, KeyCode::Backspace);
    }
    typed(&mut app, "12");
    assert_eq!(app.goto_matches(), [1], "number");

    press(&mut app, KeyCode::Enter);
    assert_eq!(
        (app.mode, app.focus, app.cursor().line),
        (Mode::Normal, Focus::Editor, 2)
    );
}

#[test]
fn ctrl_p_arrows_choose_and_esc_leaves() {
    let mut app = two_scenes();
    ctrl(&mut app, 'p');
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.cursor().line, 2, "second match");

    ctrl(&mut app, 'p');
    press(&mut app, KeyCode::Up);
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.cursor().line, 0, "first match");

    ctrl(&mut app, 'p');
    typed(&mut app, "zzz");
    assert!(screen(&mut app, 120, 20).contains("(no match)"));
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.mode, Mode::Goto, "nothing to go to");
    press(&mut app, KeyCode::Esc);
    assert_eq!((app.mode, app.cursor().line), (Mode::Normal, 0));
    assert!(!app.text().contains("zzz"), "typing went to the box");
}

#[test]
fn narrow_terminal_hides_the_list_until_asked() {
    let mut app = app();
    let narrow = screen(&mut app, 99, 20);
    assert!(!narrow.contains("Scenes"), "{narrow}");
    assert!(narrow.contains("Elisabeth tritt an den Tisch."), "{narrow}");

    let wide = screen(&mut app, 100, 20);
    assert!(wide.contains("Scenes"), "{wide}");

    ctrl(&mut app, 'b');
    assert!(
        !screen(&mut app, 100, 20).contains("Scenes"),
        "hidden at 100"
    );
    ctrl(&mut app, 'b');
    assert!(screen(&mut app, 100, 20).contains("Scenes"), "shown again");
}

#[test]
fn ctrl_b_shows_the_list_on_a_narrow_terminal_and_tab_too() {
    let mut app = app();
    let _ = screen(&mut app, 80, 20);
    ctrl(&mut app, 'b');
    assert!(screen(&mut app, 80, 20).contains("Scenes"));

    let mut app = self::app();
    let _ = screen(&mut app, 80, 20);
    press(&mut app, KeyCode::Tab);
    assert_eq!(app.focus, Focus::List);
    assert!(screen(&mut app, 80, 20).contains("Scenes"));

    ctrl(&mut app, 'b');
    assert_eq!(app.focus, Focus::Editor, "hiding the list leaves it");
}
