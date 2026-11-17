//! The script as PDF (DB-70) and the (CONT'D) setting (DB-31).

use super::support::*;
use super::*;

const BOOK: &str = "INT. A - TAG\n\nAnna sieht Anna.\n\nANNA\nIch bin anna.";

fn contd_saved(s: &Scratch) -> bool {
    let project = std::fs::read_to_string(s.dir.join("projekt.toml")).unwrap_or_default();

    project.contains("contd_after_action = true")
}

#[test]
fn ctrl_e_asks_for_a_saved_text_first() {
    let (_s, mut app) = app_with(BOOK);
    typed(&mut app, "x");
    ctrl(&mut app, 'e');

    assert_eq!(app.mode, Mode::Normal);
    assert!(screen(&mut app, 160, 24).contains("PDF: save first (Ctrl+S)"));
}

#[test]
fn enter_writes_the_pdf_next_to_the_script() {
    let (s, mut app) = app_with(BOOK);
    ctrl(&mut app, 'e');
    assert_eq!(app.mode, Mode::Pdf);
    assert!(screen(&mut app, 100, 24).contains("Scene numbers"));
    press(&mut app, KeyCode::Enter);

    assert_eq!(app.mode, Mode::Normal);
    let pdf = std::fs::read(s.dir.join("drehbuch.pdf")).unwrap_or_default();
    assert!(pdf.starts_with(b"%PDF"));
    assert!(screen(&mut app, 100, 24).contains("PDF written: drehbuch.pdf"));
}

#[test]
fn escape_writes_nothing() {
    let (s, mut app) = app_with(BOOK);
    ctrl(&mut app, 'e');
    press(&mut app, KeyCode::Esc);

    assert_eq!(app.mode, Mode::Normal);
    assert!(!s.dir.join("drehbuch.pdf").exists());
}

#[test]
fn contd_after_action_is_kept_in_the_project() {
    let (s, mut app) = app_with(BOOK);
    ctrl(&mut app, 'e');
    keys(&mut app, &[KeyCode::Down, KeyCode::Down]);
    press(&mut app, KeyCode::Char(' '));
    assert!(screen(&mut app, 100, 24).contains("(CONT'D) after action"));
    press(&mut app, KeyCode::Enter);

    assert!(contd_saved(&s));
    ctrl(&mut app, 'e');
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Char(' '));
    press(&mut app, KeyCode::Enter);
    assert!(!contd_saved(&s));
}

#[test]
fn the_contd_setting_writes_no_other_file() {
    let (s, mut app) = app_with(BOOK);
    // The window saved its plans and characters meanwhile.
    let characters = s.dir.join("figuren.toml");
    let theirs = std::fs::read_to_string(&characters).unwrap_or_default() + "# extern\n";
    std::fs::write(&characters, &theirs).unwrap();

    ctrl(&mut app, 'e');
    keys(&mut app, &[KeyCode::Down, KeyCode::Down]);
    press(&mut app, KeyCode::Char(' '));
    press(&mut app, KeyCode::Enter);

    assert!(contd_saved(&s));
    assert_eq!(std::fs::read_to_string(&characters).unwrap(), theirs);
}
