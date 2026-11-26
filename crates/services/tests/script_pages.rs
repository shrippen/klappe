//! The screenplay on pages: laid out only when asked, page breaks and lengths for the
//! editor, the PDF (DB-30 to DB-32, DB-70, DN-02).
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_services::{
    HeaderLine, Language, Numbering, ProjectEdit, Revisions, SceneNumbering, SceneNumbers,
    ScriptLanguage, ScriptPaper, ScriptPrint, ScriptSetup, ScriptWords, ServiceError, Session,
};

const PRINT: ScriptPrint = ScriptPrint {
    scene_numbers: SceneNumbers::Shown,
    header: HeaderLine::Title,
    language: Language::English,
    revisions: Revisions::Off,
};

fn session(name: &str, text: &str) -> Session {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    let mut session = Session::create(&dir, "Nachtwache", Language::English).unwrap();
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
    s.set_script_setup(ScriptSetup::default());
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
fn the_pdf_reads_title_keys_like_the_form() {
    let text = format!("DraftDate: 1. Mai\nTitle: Nachtwache\n\n{}", scene(1, 3));
    let mut s = session("pages-title-key", &text);
    let path = out("pages-title-key.pdf");
    s.export_script_pdf(&path, &PRINT).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    let pdf = String::from_utf8_lossy(&bytes).into_owned();

    assert!(pdf.contains("(1. Mai) Tj"), "{pdf}");
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

#[test]
fn a_german_project_can_keep_the_international_words() {
    let dialogue: Vec<String> = (0..30).map(|i| format!("Sentence number {i}.")).collect();
    let text = format!("{}\nANNA\n{}\n", scene(1, 45), dialogue.join("\n"));
    let mut s = session("pages-international", &text);
    s.set_script_setup(ScriptSetup {
        language: Some(ScriptLanguage::De),
        words: ScriptWords::International,
        ..ScriptSetup::default()
    });
    let path = out("pages-international.pdf");
    s.export_script_pdf(&path, &PRINT).unwrap();
    let pdf = String::from_utf8_lossy(&std::fs::read(&path).unwrap()).into_owned();

    assert!(pdf.contains("(\\(MORE\\)) Tj"), "{pdf}");

    s.set_script_setup(ScriptSetup {
        language: Some(ScriptLanguage::De),
        ..ScriptSetup::default()
    });
    s.export_script_pdf(&path, &PRINT).unwrap();
    let pdf = String::from_utf8_lossy(&std::fs::read(&path).unwrap()).into_owned();
    assert!(pdf.contains("(\\(WEITER\\)) Tj"), "{pdf}");
}

#[test]
fn keeping_contd_leaves_the_rest_of_the_setup_on_disk() {
    let mut tui = session("pages-keep-contd", &scene(1, 3));
    let dir = tui.dir().to_path_buf();
    // Another program sets the language meanwhile.
    let mut window = Session::open(&dir).unwrap();
    window.set_script_setup(ScriptSetup {
        language: Some(ScriptLanguage::En),
        ..ScriptSetup::default()
    });
    window.save(klappe_services::OnOutside::Overwrite).unwrap();

    tui.keep_contd(true).unwrap();

    let again = Session::open(&dir).unwrap();
    assert!(again.project().script.contd_after_action);
    assert_eq!(again.project().script.language, Some(ScriptLanguage::En));
}

/// D-15: the print view gets the pages of the PDF, laid out from the text now: a second scene
/// typed after the first layout shows at once, with its number at both sides.
#[test]
fn the_print_pages_follow_the_text() {
    let mut s = session("print-pages", &scene(1, 2));
    s.paginate(Language::English);

    s.set_script_text(&(scene(1, 2) + &scene(2, 2)));
    let pages = s.print_pages(&PRINT);

    let numbers = pages[0].texts.iter().filter(|t| t.text == "2").count();
    assert_eq!(pages.len(), 1);
    assert_eq!(numbers, 2);
    assert!(pages[0].texts.iter().any(|t| t.text == "Nachtwache"));
}

/// The project form: the numbering reaches `projekt.toml` at once, the paper and the rest wait
/// for the next save, and English words are never international.
#[test]
fn the_form_chooses_the_whole_setup() {
    let mut s = session("pages-form", &scene(1, 3));
    let dir = s.dir().to_path_buf();

    s.choose_script_setup(ScriptSetup {
        language: Some(ScriptLanguage::En),
        words: ScriptWords::International,
        paper: ScriptPaper::Letter,
        contd_after_action: true,
        scene_numbering: SceneNumbering::Locked,
        first_scene: 7,
    })
    .unwrap();

    let chosen = s.project().script;
    assert_eq!(chosen.words, ScriptWords::Local);
    assert_eq!(chosen.paper, ScriptPaper::Letter);
    assert_eq!(chosen.first_scene, 7);

    // Only the numbering is on disk yet; the paper is written with the save.
    let on_disk = Session::open(&dir).unwrap().project().script;
    assert_eq!(on_disk.scene_numbering, SceneNumbering::Locked);
    assert_eq!(on_disk.first_scene, 7);
    assert_eq!(on_disk.paper, ScriptPaper::A4);

    // The project view shows the setup by names.
    let row = s.overview().script;
    assert_eq!(row.language, "en");
    assert_eq!(row.paper, "letter");
    assert_eq!(row.scene_numbering, "locked");
    assert_eq!(s.overview().script_papers, ["a4", "letter"]);
}

#[test]
fn german_words_may_be_international() {
    let mut s = session("pages-form-de", &scene(1, 3));

    s.choose_script_setup(ScriptSetup {
        language: Some(ScriptLanguage::De),
        words: ScriptWords::International,
        ..ScriptSetup::default()
    })
    .unwrap();

    assert_eq!(s.project().script.words, ScriptWords::International);
}

/// The setup is a setting like the plan's paper: undo and redo of an edit keep it, the
/// numbering that is on disk included.
#[test]
fn undo_and_redo_keep_the_script_setup() {
    let mut s = session("pages-undo", &scene(1, 3));
    let dir = s.dir().to_path_buf();

    s.edit_project(ProjectEdit::SetNumbering {
        numbering: Numbering::Camera,
    })
    .unwrap();
    s.choose_script_setup(ScriptSetup {
        paper: ScriptPaper::Letter,
        scene_numbering: SceneNumbering::Locked,
        first_scene: 7,
        ..ScriptSetup::default()
    })
    .unwrap();
    let chosen = s.project().script;

    s.undo();
    assert_eq!(s.project().script, chosen);
    s.redo();
    assert_eq!(s.project().script, chosen);

    // Nothing was written back from the old state: the file has the numbering chosen.
    s.undo();
    s.save(klappe_services::OnOutside::Overwrite).unwrap();
    let on_disk = Session::open(&dir).unwrap().project().script;
    assert_eq!(on_disk.first_scene, 7);
    assert_eq!(on_disk.scene_numbering, SceneNumbering::Locked);
}

/// A paper chosen for the output becomes the project's only when the file was written.
#[test]
fn a_refused_output_keeps_the_papers_of_the_project() {
    let mut s = session("pages-paper-refused", "");
    let path = out("pages-paper-refused.pdf");

    let none = s.export_script_on(&path, &PRINT, ScriptPaper::Letter);
    assert!(matches!(none, Err(ServiceError::NoScript)));
    assert_eq!(s.project().script.paper, ScriptPaper::A4);

    s.set_script_text(&scene(1, 3));
    s.export_script_on(&path, &PRINT, ScriptPaper::Letter)
        .unwrap();
    assert_eq!(s.project().script.paper, ScriptPaper::Letter);
}

/// A refused output leaves the session as it was: saved stays saved, nothing to back up.
#[test]
fn a_refused_output_leaves_a_saved_project_clean() {
    let mut s = session("pages-paper-clean", "");
    s.save(klappe_services::OnOutside::Overwrite).unwrap();
    assert!(!s.is_dirty());

    let none = s.export_script_on(&out("pages-paper-clean.pdf"), &PRINT, ScriptPaper::Letter);

    assert!(matches!(none, Err(ServiceError::NoScript)));
    assert!(!s.is_dirty());
    assert_eq!(s.autosave().unwrap(), klappe_services::Autosave::Clean);
}
