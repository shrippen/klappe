//! The outline in the list pane (DB-20, DB-21): tree, collapse, filter, jump and moving.

use ratatui::crossterm::event::MouseButton;

use super::support::{app_with, keys};
use super::*;

const WIDTH: u16 = 140;
const HEIGHT: u16 = 24;

const TEXT: &str = "# Akt I

= Der Anfang.

## Ankunft

INT. SAAL - TAG #1#

Handlung eins.

INT. HOF - TAG #2#

Handlung zwei.

# Akt II

INT. KELLER - NACHT #3#

Handlung drei.
";

/// An app on `TEXT`, the outline on screen, the cursor at the end of the text.
fn outlined() -> (Scratch, App) {
    let (scratch, mut app) = app_with(TEXT);
    let _ = screen(&mut app, WIDTH, HEIGHT);

    (scratch, app)
}

fn alt(app: &mut App, code: KeyCode) {
    app.handle_key(KeyEvent::new(code, KeyModifiers::ALT));
    assert_backup_answered(app);
}

/// The outline rows on screen: the pane's left 34 columns, blank rows left out.
fn rows(app: &mut App) -> Vec<String> {
    screen(app, WIDTH, HEIGHT)
        .lines()
        .skip(1)
        .map(|l| {
            l.chars()
                .take(34)
                .collect::<String>()
                .trim_end()
                .to_string()
        })
        .filter(|l| !l.is_empty())
        .collect()
}

fn frame(app: &mut App) -> String {
    rows(app).join("\n")
}

/// Selects the entry with this text in the list.
fn select(app: &mut App, text: &str) {
    if app.focus != Focus::List {
        press(app, KeyCode::F(6));
    }
    press(app, KeyCode::Home);
    for _ in 0..app.analysis.outline.len() {
        if app.analysis.outline[app.list_sel].text == text {
            return;
        }
        press(app, KeyCode::Down);
    }
    unreachable!("{text} is not in the list");
}

fn selected(app: &App) -> &str {
    &app.analysis.outline[app.list_sel].text
}

fn click(app: &mut App, x: u16, y: u16) {
    app.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: x,
        row: y,
        modifiers: KeyModifiers::NONE,
    });
}

#[test]
fn the_list_shows_sections_scenes_and_synopses_as_a_tree_with_lengths() {
    let (_s, mut app) = outlined();
    let shown = frame(&mut app);

    for part in [
        "▾ Akt I",
        "= Der Anfang.",
        "▾ Ankunft",
        "1  INT. SAAL - TAG",
        "▾ Akt II",
    ] {
        assert!(shown.contains(part), "{part}:\n{shown}");
    }
    // A scene is one step deeper than its section; a section shows the total of its scenes.
    let line = |needle: &str| shown.lines().find(|l| l.contains(needle)).unwrap_or("");
    let indent = |needle: &str| line(needle).len() - line(needle).trim_start().len();
    assert!(indent("1  INT. SAAL") > indent("▾ Ankunft"), "{shown}");
    assert!(line("▾ Akt I").ends_with("2/8"), "{shown}");
    assert!(line("INT. HOF").ends_with("1/8"), "{shown}");
    assert!(line("▾ Akt II").ends_with("1/8"), "{shown}");
}

#[test]
fn left_collapses_a_section_and_right_expands_it() {
    let (_s, mut app) = outlined();
    select(&mut app, "Akt I");
    press(&mut app, KeyCode::Left);
    let shown = frame(&mut app);

    assert!(shown.contains("▸ Akt I"), "{shown}");
    assert!(
        !shown.contains("SAAL") && !shown.contains("Ankunft"),
        "{shown}"
    );
    assert!(shown.contains("KELLER"), "{shown}");
    // The total stays on the folded row.
    assert!(shown
        .lines()
        .any(|l| l.contains("▸ Akt I") && l.ends_with("2/8")));

    // Down skips what is hidden.
    press(&mut app, KeyCode::Down);
    assert_eq!(selected(&app), "Akt II");

    select(&mut app, "Akt I");
    press(&mut app, KeyCode::Right);
    assert!(frame(&mut app).contains("SAAL"));
}

#[test]
fn left_on_a_scene_goes_to_its_section() {
    let (_s, mut app) = outlined();
    select(&mut app, "INT. HOF - TAG");
    press(&mut app, KeyCode::Left);

    assert_eq!(selected(&app), "Ankunft");
    assert!(frame(&mut app).contains("HOF"), "a scene does not fold");
    press(&mut app, KeyCode::Left);
    assert!(!frame(&mut app).contains("HOF"));
}

#[test]
fn a_collapsed_section_stays_collapsed_while_typing_elsewhere() {
    let (_s, mut app) = outlined();
    select(&mut app, "Akt I");
    press(&mut app, KeyCode::Left);
    press(&mut app, KeyCode::F(6));
    typed(&mut app, " mehr");

    assert!(!frame(&mut app).contains("SAAL"));
}

#[test]
fn the_mouse_folds_at_the_marker_and_jumps_elsewhere() {
    let (_s, mut app) = outlined();
    let buffer = render(&mut app, WIDTH, HEIGHT);
    let (x, y) = find(&buffer, WIDTH, HEIGHT, "▾ Akt I");
    click(&mut app, x, y);

    let shown = frame(&mut app);
    assert!(
        shown.contains("▸ Akt I") && !shown.contains("SAAL"),
        "{shown}"
    );
    assert_eq!(app.focus, Focus::List);
    click(&mut app, x, y);
    assert!(frame(&mut app).contains("SAAL"), "second click expands");

    // A click on the heading of a scene moves the cursor there.
    let buffer = render(&mut app, WIDTH, HEIGHT);
    let (x, y) = find(&buffer, WIDTH, HEIGHT, "INT. HOF");
    click(&mut app, x + 2, y);
    assert_eq!((app.focus, app.cursor().line), (Focus::Editor, 10));
}

#[test]
fn typing_filters_by_heading_or_synopsis() {
    let (_s, mut app) = outlined();
    press(&mut app, KeyCode::F(6));
    typed(&mut app, "anfang");
    let shown = frame(&mut app);

    assert!(shown.contains("Filter: anfang"), "{shown}");
    assert!(
        shown.contains("= Der Anfang.") && shown.contains("Akt I"),
        "{shown}"
    );
    assert!(
        !shown.contains("SAAL") && !shown.contains("KELLER"),
        "{shown}"
    );
    assert_eq!(
        selected(&app),
        "Akt I",
        "the selection moves to the first row"
    );

    // Any case, and a heading.
    press(&mut app, KeyCode::Backspace);
    keys(&mut app, &[KeyCode::Backspace; 5]);
    typed(&mut app, "KeLLer");
    let shown = frame(&mut app);
    assert!(
        shown.contains("KELLER") && !shown.contains("SAAL"),
        "{shown}"
    );

    typed(&mut app, "zzz");
    assert!(frame(&mut app).contains("(no match)"));

    // Esc clears the filter, the next Esc leaves the list.
    press(&mut app, KeyCode::Esc);
    assert!(frame(&mut app).contains("SAAL"));
    assert_eq!(app.focus, Focus::List);
    press(&mut app, KeyCode::Esc);
    assert_eq!(app.focus, Focus::Editor);
}

#[test]
fn enter_jumps_to_the_entry_and_f6_comes_back_to_the_scene() {
    let (_s, mut app) = outlined();
    select(&mut app, "Ankunft");
    press(&mut app, KeyCode::Enter);
    assert_eq!(
        (app.focus, app.cursor().line),
        (Focus::Editor, 4),
        "section"
    );

    select(&mut app, "Der Anfang.");
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.cursor().line, 2, "synopsis");

    // Back: the cursor in a scene, F6 selects its row.
    app.editor.go_to_line(11);
    press(&mut app, KeyCode::F(6));
    assert_eq!(selected(&app), "INT. HOF - TAG");
}

#[test]
fn alt_down_moves_a_scene_with_its_text_as_one_undo_step() {
    let (_s, mut app) = outlined();
    select(&mut app, "INT. SAAL - TAG");
    alt(&mut app, KeyCode::Down);

    let moved = TEXT.replace(
        "INT. SAAL - TAG #1#\n\nHandlung eins.\n\nINT. HOF - TAG #2#\n\nHandlung zwei.\n",
        "INT. HOF - TAG #2#\n\nHandlung zwei.\n\nINT. SAAL - TAG #1#\n\nHandlung eins.\n",
    );
    assert_eq!(app.text(), moved);
    assert_eq!(
        selected(&app),
        "INT. SAAL - TAG",
        "the selection goes with it"
    );
    assert!(app.is_modified());

    ctrl(&mut app, 'z');
    assert_eq!(app.text(), TEXT, "one undo takes the whole move back");
    ctrl(&mut app, 'y');
    assert_eq!(app.text(), moved);
}

#[test]
fn alt_up_moves_a_scene_out_of_its_section_over_the_heading() {
    let (_s, mut app) = outlined();
    select(&mut app, "INT. KELLER - NACHT");
    alt(&mut app, KeyCode::Up);

    let entries = &app.analysis.outline;
    let at = |t: &str| entries.iter().position(|e| e.text == t).unwrap_or(99);
    assert!(at("INT. KELLER - NACHT") < at("Akt II"));
    assert_eq!(
        entries[at("INT. KELLER - NACHT")].number.as_deref(),
        Some("3")
    );
}

#[test]
fn a_scene_moved_into_a_folded_section_unfolds_it_and_stays_selected() {
    let (_s, mut app) = outlined();
    select(&mut app, "Akt I");
    press(&mut app, KeyCode::Left);
    select(&mut app, "INT. KELLER - NACHT");
    alt(&mut app, KeyCode::Up);
    assert_eq!(selected(&app), "INT. KELLER - NACHT");

    alt(&mut app, KeyCode::Down);
    assert_eq!(app.text(), TEXT);
}

#[test]
fn enter_and_ctrl_g_do_nothing_when_the_filter_matches_nothing() {
    let (_s, mut app) = outlined();
    select(&mut app, "INT. KELLER - NACHT");
    let before = app.cursor();
    for c in "xyzq".chars() {
        press(&mut app, KeyCode::Char(c));
    }

    press(&mut app, KeyCode::Enter);
    assert_eq!((app.focus, app.cursor()), (Focus::List, before));
    ctrl(&mut app, 'g');
    assert_eq!((app.focus, app.cursor()), (Focus::List, before));
}

#[test]
fn alt_moves_a_whole_section() {
    let (_s, mut app) = outlined();
    select(&mut app, "Akt I");
    alt(&mut app, KeyCode::Down);

    assert!(app
        .text()
        .starts_with("# Akt II\n\nINT. KELLER - NACHT #3#"));
    assert!(app
        .text()
        .contains("# Akt I\n\n= Der Anfang.\n\n## Ankunft"));
    assert_eq!(selected(&app), "Akt I");
    assert_eq!(app.cursor().line, 6, "the cursor stands on the moved block");

    ctrl(&mut app, 'z');
    assert_eq!(app.text(), TEXT);
}

#[test]
fn nothing_moves_at_the_end_in_a_synopsis_or_under_a_filter() {
    let (_s, mut app) = outlined();
    select(&mut app, "Akt I");
    alt(&mut app, KeyCode::Up);
    select(&mut app, "Der Anfang.");
    alt(&mut app, KeyCode::Down);
    assert_eq!(app.text(), TEXT);
    assert!(!app.is_modified());

    select(&mut app, "Akt I");
    typed(&mut app, "akt");
    alt(&mut app, KeyCode::Down);
    assert_eq!(app.text(), TEXT, "no move while filtered");
}

#[test]
fn a_move_saves_as_text_that_reads_back_the_same() {
    let (scratch, mut app) = outlined();
    select(&mut app, "Ankunft");
    alt(&mut app, KeyCode::Down);
    ctrl(&mut app, 's');

    // "Ankunft" is the last section of its parent: nothing moved, the file is as it was.
    assert_eq!(scratch.script(), TEXT);

    select(&mut app, "INT. SAAL - TAG");
    alt(&mut app, KeyCode::Down);
    ctrl(&mut app, 's');
    let saved = scratch.script();
    assert_eq!(saved, app.text());
    assert_eq!(
        klappe_services::parse(&saved).elements.len(),
        klappe_services::parse(TEXT).elements.len()
    );
}

#[test]
fn the_help_lists_the_outline_keys() {
    let (_s, mut app) = outlined();
    press(&mut app, KeyCode::F(1));
    let help = app.texts.help.join("\n");

    for key in ["Alt+Up / Down", "Left / Right", "Type"] {
        assert!(help.contains(key), "{key}:\n{help}");
    }
}

#[test]
fn the_selection_is_a_visible_row_after_folding_and_going_back() {
    let (_s, mut app) = outlined();
    select(&mut app, "Akt I");
    press(&mut app, KeyCode::Left);
    press(&mut app, KeyCode::F(6));
    // The cursor goes into a hidden scene, then back to the list.
    app.editor.go_to_line(6);
    press(&mut app, KeyCode::F(6));
    assert_eq!(
        selected(&app),
        "Akt I",
        "the folded section, not the hidden scene"
    );

    alt(&mut app, KeyCode::Down);
    assert!(app.text().starts_with("# Akt II"), "the section moved");
    ctrl(&mut app, 'z');
    app.editor.go_to_line(6);
    press(&mut app, KeyCode::F(6));

    press(&mut app, KeyCode::Enter);
    assert_eq!(app.cursor().line, 0, "Enter jumps to the visible row");
}

#[test]
fn leaving_a_filter_selects_a_visible_row() {
    let (_s, mut app) = outlined();
    select(&mut app, "Akt I");
    press(&mut app, KeyCode::Left);
    typed(&mut app, "saal");
    assert_eq!(selected(&app), "Akt I");
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Down);
    assert_eq!(selected(&app), "INT. SAAL - TAG");
    press(&mut app, KeyCode::Esc);

    assert_eq!(selected(&app), "Akt I", "SAAL is hidden again");
    alt(&mut app, KeyCode::Down);
    assert!(app.text().starts_with("# Akt II"));
}

#[test]
fn the_filter_matches_the_number_the_list_shows() {
    let (_s, mut app) = app_with("INT. EINS - TAG\n\nText.\n\nINT. ZWEI - TAG\n\nText.\n");
    let _ = screen(&mut app, WIDTH, HEIGHT);
    press(&mut app, KeyCode::F(6));
    typed(&mut app, "2");

    let shown = frame(&mut app);
    assert!(shown.contains("ZWEI") && !shown.contains("EINS"), "{shown}");
}
