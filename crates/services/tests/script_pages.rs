//! The screenplay on pages: laid out only when asked, page breaks and lengths for the
//! editor, the PDF (DB-30 to DB-32, DB-70, DN-02).
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_services::{
    HeaderLine, Language, SceneNumbers, ScriptLanguage, ScriptPaper, ScriptPrint, ScriptSetup,
    ServiceError, Session,
};

const PRINT: ScriptPrint = ScriptPrint {
    scene_numbers: SceneNumbers::Shown,
    header: HeaderLine::Title,
    language: Language::English,
};

fn session(name: &str, text: &str) -> Session {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    let mut session = Session::create(&dir, "Nachtwache").unwrap();
    session.set_script_text(text);

    session
}

fn out(name: &str) -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join(name)
}

/// A scene of `rows` rows of action, one per line, under a heading.
fn scene(number: u32, rows: usize) -> String {
    let lines: Vec<String> = (0..rows).map(|i| format!("Line {i}.")).collect();

    format!(
        "INT. ROOM {number} - DAY #{number}#\n\n{}\n\n",
        lines.join("\n")
    )
}

#[test]
fn nothing_is_laid_out_until_asked() {
    let mut s = session("pages-lazy", &scene(1, 3));
    assert!(s.script_pages().is_none());

    s.paginate(Language::English);
    assert_eq!(s.script_pages().map(|p| p.pages), Some(1));

    // Typing changes the text, not the pages shown, until the next call.
    s.set_script_text(&format!("{}{}", scene(1, 3), scene(2, 150)));
    assert_eq!(s.script_pages().map(|p| p.pages), Some(1));
    s.paginate(Language::English);
    assert_eq!(s.script_pages().map(|p| p.pages), Some(3));
}

#[test]
fn page_breaks_and_lengths_point_at_text_lines() {
    // A4 has 59 rows. Scene 1: heading + blank + 40 rows = 42 rows (6 eighths), text
    // lines 0 to 42; scene 2 follows after 2 blank rows and takes the rest.
    let text = format!("{}{}", scene(1, 40), scene(2, 40));
    let mut s = session("pages-breaks", &text);
    s.paginate(Language::English);
    let pages = s.script_pages().unwrap();

    assert_eq!(pages.pages, 2);
    assert_eq!(pages.scenes.len(), 2);
    assert_eq!((pages.scenes[0].line, pages.scenes[0].page), (0, 1));
    assert_eq!(pages.scenes[0].eighths, 6);
    assert_eq!(pages.scenes[0].length, "6/8");
    assert_eq!(pages.scenes[1].line, 43);

    // Scene 2: heading at row 44 (42 rows and 2 blanks), a blank, then "Line 0." to
    // "Line 12." on rows 46 to 58. "Line 13." starts page 2: text line 43 + 2 + 13.
    assert_eq!(pages.breaks.len(), 1);
    assert_eq!(pages.breaks[0].page, 2);
    assert_eq!(pages.breaks[0].line, 58);
}

#[test]
fn the_words_follow_the_projects_language() {
    let dialogue: Vec<String> = (0..30).map(|i| format!("Sentence number {i}.")).collect();
    let text = format!(
        "INT. A - DAY\n\n{}\n\nANNA\n{}\n",
        scene(1, 45),
        dialogue.join("\n")
    );
    let mut s = session("pages-words", &text);

    // No language chosen: the surface's decides.
    s.paginate(Language::German);
    assert_eq!(s.script_pages().map(|p| p.pages), Some(2));
    let german = out("pages-words-de.pdf");
    s.export_script_pdf(
        &german,
        &ScriptPrint {
            language: Language::German,
            ..PRINT
        },
    )
    .unwrap();
    let pdf = String::from_utf8_lossy(&std::fs::read(&german).unwrap()).into_owned();
    assert!(pdf.contains("(\\(WEITER\\)) Tj"), "{pdf}");
    assert!(pdf.contains("(ANNA \\(FORTS.\\)) Tj"), "{pdf}");

    // The project's language wins.
    s.set_script_setup(ScriptSetup {
        language: Some(ScriptLanguage::En),
        ..ScriptSetup::default()
    });
    let english = out("pages-words-en.pdf");
    s.export_script_pdf(
        &english,
        &ScriptPrint {
            language: Language::German,
            ..PRINT
        },
    )
    .unwrap();
    let pdf = String::from_utf8_lossy(&std::fs::read(&english).unwrap()).into_owned();
    assert!(pdf.contains("(\\(MORE\\)) Tj"), "{pdf}");
    assert!(pdf.contains("CONT'D"), "{pdf}");
}

#[test]
fn the_pdf_has_the_pages_of_the_text_and_the_title_page() {
    let text = format!(
        "Title: Nachtwache\nAuthor: Studio Weber\nDraft date: 2026-11-16\n\n{}",
        scene(12, 100)
    );
    let mut s = session("pages-pdf", &text);
    let path = out("pages-pdf.pdf");
    s.export_script_pdf(&path, &PRINT).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    let pdf = String::from_utf8_lossy(&bytes).into_owned();

    assert!(bytes.starts_with(b"%PDF"));
    // Title page and two pages of text.
    assert!(pdf.contains("/Count 3"), "{pdf}");
    assert!(pdf.contains("(NACHTWACHE) Tj"));
    assert!(pdf.contains("(Nachtwache - 2026-11-16) Tj"));
    // Scene number on both sides of the heading, page number 2 on the second page of text.
    assert_eq!(pdf.matches("(12) Tj").count(), 2);
    assert!(pdf.contains("(2.) Tj"));
    // Export laid the text out again, whatever was shown before.
    assert_eq!(s.script_pages().map(|p| p.pages), Some(2));
}

#[test]
fn letter_pages_hold_fewer_rows_than_a4() {
    let text = scene(1, 57);
    let mut s = session("pages-paper", &text);

    s.paginate(Language::English);
    assert_eq!(s.script_pages().map(|p| p.pages), Some(1));

    s.set_script_setup(ScriptSetup {
        paper: ScriptPaper::Letter,
        ..ScriptSetup::default()
    });
    s.paginate(Language::English);
    assert_eq!(s.script_pages().map(|p| p.pages), Some(2));
}

#[test]
fn a_project_without_screenplay_has_no_pdf() {
    let mut s = session("pages-none", "");
    let result = s.export_script_pdf(&out("pages-none.pdf"), &PRINT);

    assert!(matches!(result, Err(ServiceError::NoScript)));
}

#[test]
fn paginating_a_text_leaves_the_session_text_alone() {
    let mut s = session("pages-text", &scene(1, 3));
    s.paginate_text(&scene(2, 100), Language::English);

    assert_eq!(s.script_pages().map(|p| p.pages), Some(2));
    assert_eq!(s.script_text(), scene(1, 3));
}
