//! Enter, Tab and suggestions while writing (DB-02, DB-03).

use klappe_services::{parse, Element, Slot};

use super::support::*;
use super::*;
use crate::buffer::Pos;
use crate::fountain::LineKind;

fn names(app: &App) -> Vec<String> {
    app.suggest
        .as_ref()
        .map(|s| s.items.options.clone())
        .unwrap_or_default()
}

#[test]
fn enter_at_the_end_of_each_element_follows_the_table() {
    // (text with the cursor at its end, what Enter adds, the slot of the new line)
    for (text, added, slot) in [
        ("INT. A - TAG", "\n\n", Slot::Action),
        ("Anna geht.", "\n\n", Slot::Action),
        ("ANNA\nHallo.", "\n\n", Slot::Action),
        ("CUT TO:", "\n\n", Slot::Heading),
        ("ANNA\n(leise)", "\n", Slot::Dialogue),
    ] {
        // A blank line before the text and one after it keep each line what it is.
        let (_s, mut app) = app_with(&format!("x\n\n{text}"));
        press(&mut app, KeyCode::Enter);

        assert_eq!(app.text(), format!("x\n\n{text}{added}"), "{text:?}");
        assert_eq!(app.pending.map(|p| p.1), Some(slot), "{text:?}");
        // The cursor stands on the new, empty last line.
        assert_eq!(
            app.cursor(),
            Pos {
                line: app.text().split('\n').count() - 1,
                col: 0
            },
            "{text:?}"
        );
    }
}

#[test]
fn a_conversation_costs_enter_tab_tab_enter_and_the_text() {
    let (_s, mut app) = app_with("");
    typed(&mut app, "INT. KÜCHE - TAG");
    press(&mut app, KeyCode::Enter);
    typed(&mut app, "Anna tritt ein.");
    press(&mut app, KeyCode::Enter);
    press(&mut app, KeyCode::Tab);
    typed(&mut app, "anna");
    press(&mut app, KeyCode::Enter);
    typed(&mut app, "Hallo.");
    press(&mut app, KeyCode::Enter);
    press(&mut app, KeyCode::Tab);
    typed(&mut app, "ben");
    press(&mut app, KeyCode::Enter);
    typed(&mut app, "Ja?");

    // Enter at the end of a dialogue, Tab, then the other character is proposed.
    press(&mut app, KeyCode::Enter);
    press(&mut app, KeyCode::Tab);
    assert_eq!(names(&app), ["ANNA"]);
    press(&mut app, KeyCode::Tab);
    typed(&mut app, "Nein.");

    assert_eq!(
        app.text(),
        "INT. KÜCHE - TAG\n\nAnna tritt ein.\n\nANNA\nHallo.\n\nBEN\nJa?\n\nANNA\nNein."
    );
    let els = parse(&app.text()).elements;
    assert_eq!(els.len(), 8);
    assert!(matches!(&els[6], Element::Character { name, .. } if name == "ANNA"));
}

#[test]
fn tab_on_an_empty_line_changes_the_element() {
    let (_s, mut app) = app_with("INT. A - TAG\n\nAnna geht.\n\n");

    // Action -> character -> action -> character.
    for slot in [Slot::Character, Slot::Action, Slot::Character] {
        press(&mut app, KeyCode::Tab);
        assert_eq!(app.pending.map(|p| p.1), Some(slot));
    }
    typed(&mut app, "bob");
    press(&mut app, KeyCode::Enter);

    // Dialogue -> parenthetical, typed in the brackets -> dialogue.
    press(&mut app, KeyCode::Tab);
    assert_eq!(app.pending.map(|p| p.1), Some(Slot::Parenthetical));
    assert_eq!(app.cursor().col, 1);
    typed(&mut app, "leise)");
    press(&mut app, KeyCode::Enter);
    typed(&mut app, "Hi.");

    assert_eq!(
        app.text(),
        "INT. A - TAG\n\nAnna geht.\n\nBOB\n(leise)\nHi."
    );
    let els = parse(&app.text()).elements;
    assert_eq!(els[3], Element::Parenthetical("leise".into()));
}

/// "BOB" typed as a cue, Enter pressed: the line below is the dialogue's.
fn after_cue() -> (Scratch, App) {
    let (s, mut app) = app_with("x\n\n");
    press(&mut app, KeyCode::Tab);
    typed(&mut app, "bob");
    press(&mut app, KeyCode::Enter);

    (s, app)
}

#[test]
fn tab_on_a_parenthetical_without_text_goes_to_the_dialogue() {
    let (_s, mut app) = after_cue();
    press(&mut app, KeyCode::Tab);
    assert_eq!(app.text(), "x\n\nBOB\n()");

    press(&mut app, KeyCode::Tab);
    assert_eq!(app.text(), "x\n\nBOB\n");
    assert_eq!(app.pending.map(|p| p.1), Some(Slot::Dialogue));
}

#[test]
fn heading_and_transition_ignore_tab() {
    let (_s, mut app) = app_with("x\n\n");
    typed(&mut app, "CUT TO:");
    let before = app.text();
    press(&mut app, KeyCode::Tab);
    assert_eq!(app.text(), before, "no Tab on a filled line");

    press(&mut app, KeyCode::Enter);
    assert_eq!(app.pending.map(|p| p.1), Some(Slot::Heading));
    press(&mut app, KeyCode::Tab);
    assert_eq!(
        app.pending.map(|p| p.1),
        Some(Slot::Heading),
        "no Tab on a heading"
    );
}

#[test]
fn tab_no_longer_switches_the_focus() {
    let mut app = app();
    press(&mut app, KeyCode::Tab);

    assert_eq!(app.focus, Focus::Editor);
    assert!(!app.is_modified());
}

#[test]
fn enter_inside_a_line_splits_it() {
    let (_s, mut app) = app_with("x\n\nAnna geht.");
    keys(&mut app, &[KeyCode::Left, KeyCode::Left, KeyCode::Left]);
    press(&mut app, KeyCode::Enter);

    assert_eq!(app.text(), "x\n\nAnna ge\nht.");
    assert_eq!(app.pending, None);
}

#[test]
fn enter_on_an_empty_line_is_a_plain_line_break() {
    let (_s, mut app) = app_with("x\n\n");
    press(&mut app, KeyCode::Enter);

    assert_eq!(app.text(), "x\n\n\n");
}

#[test]
fn enter_below_a_cue_goes_to_the_dialogue_that_stands_there() {
    let (_s, mut app) = app_with("x\n\nBOB\nHallo du.");
    keys(&mut app, &[KeyCode::Up, KeyCode::End]);
    press(&mut app, KeyCode::Enter);

    assert_eq!(app.text(), "x\n\nBOB\nHallo du.");
    assert_eq!(app.cursor(), Pos { line: 3, col: 9 });
}

#[test]
fn enter_in_the_middle_of_a_paragraph_does_not_start_an_element() {
    let (_s, mut app) = app_with("x\n\nEins.\nZwei.");
    keys(&mut app, &[KeyCode::Up, KeyCode::End]);
    press(&mut app, KeyCode::Enter);

    assert_eq!(app.text(), "x\n\nEins.\n\nZwei.");
    assert_eq!(app.pending, None);
}

#[test]
fn forced_elements_keep_working() {
    let (_s, mut app) = app_with("x\n\n");
    typed(&mut app, ".hinterhof");
    press(&mut app, KeyCode::Enter);
    typed(&mut app, "!INT. kein Kopf");
    press(&mut app, KeyCode::Enter);
    typed(&mut app, "@mcBride");
    press(&mut app, KeyCode::Down);
    typed(&mut app, ">raus");

    let els = parse(&app.text()).elements;
    assert_eq!(
        els[1],
        Element::SceneHeading {
            text: "HINTERHOF".into(),
            number: None
        }
    );
    assert_eq!(els[2], Element::Action("INT. kein Kopf".into()));
}

#[test]
fn undo_takes_back_one_element_change() {
    let (_s, mut app) = app_with("x\n\nAnna geht.");
    press(&mut app, KeyCode::Enter);
    press(&mut app, KeyCode::Tab);
    typed(&mut app, "bob");
    ctrl(&mut app, 'z');
    assert_eq!(app.text(), "x\n\nAnna geht.\n\n");
    assert_eq!(app.pending, None);

    ctrl(&mut app, 'z');
    assert_eq!(app.text(), "x\n\nAnna geht.");
}

#[test]
fn the_closing_bracket_is_stepped_over() {
    let (_s, mut app) = after_cue();
    press(&mut app, KeyCode::Tab);
    typed(&mut app, "(laut");
    typed(&mut app, ")");

    assert_eq!(app.text(), "x\n\nBOB\n((laut)");
    let (_s, mut app) = after_cue();
    press(&mut app, KeyCode::Tab);
    typed(&mut app, "laut)");
    assert_eq!(app.text(), "x\n\nBOB\n(laut)");
}

const BOOK: &str =
    "INT. KÜCHE - TAG\n\nx\n\nANTON\nHi.\n\nANNA\nHo.\n\nANTON\nNa.\n\nEXT. HOF - NACHT\n\ny\n\n";

#[test]
fn names_come_most_frequent_first_and_enter_takes_one() {
    let (_s, mut app) = app_with(BOOK);
    press(&mut app, KeyCode::Tab);
    typed(&mut app, "an");

    assert_eq!(names(&app), ["ANTON", "ANNA"]);
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Enter);

    assert!(app.text().ends_with("y\n\nANNA\n"), "{:?}", app.text());
    assert_eq!(app.pending.map(|p| p.1), Some(Slot::Dialogue));
    assert_eq!(app.suggest, None);
}

#[test]
fn tab_takes_a_suggestion_too_and_esc_dismisses_one() {
    let (_s, mut app) = app_with(BOOK);
    press(&mut app, KeyCode::Tab);
    typed(&mut app, "an");
    press(&mut app, KeyCode::Esc);
    assert_eq!(app.suggest, None);
    assert_eq!(app.mode, Mode::Normal);

    typed(&mut app, "t");
    press(&mut app, KeyCode::Tab);
    assert!(app.text().ends_with("y\n\nANTON\n"), "{:?}", app.text());
}

#[test]
fn a_heading_is_completed_prefix_location_time() {
    let (_s, mut app) = app_with(BOOK);
    // A transition ends with a heading to come.
    typed(&mut app, "CUT TO:");
    press(&mut app, KeyCode::Enter);
    typed(&mut app, "in");
    assert_eq!(names(&app), ["INT. ", "INT./EXT. "]);
    press(&mut app, KeyCode::Tab);
    assert!(app.text().ends_with("CUT TO:\n\nINT. "));

    typed(&mut app, "kü");
    assert_eq!(names(&app), ["KÜCHE - "]);
    press(&mut app, KeyCode::Tab);
    assert!(app.text().ends_with("INT. KÜCHE - "), "{:?}", app.text());

    typed(&mut app, "n");
    assert_eq!(names(&app), ["NACHT", "NIGHT"]);
    press(&mut app, KeyCode::Enter);

    // A time of day ends the heading: the action follows, no second Enter.
    assert!(
        app.text().ends_with("INT. KÜCHE - NACHT\n\n"),
        "{:?}",
        app.text()
    );
    assert_eq!(app.pending.map(|p| p.1), Some(Slot::Action));
    assert_eq!(app.cursor().line, app.text().lines().count());
}

#[test]
fn the_german_forced_prefix_and_the_times_of_day_are_offered() {
    let (_s, mut app) = app_with("x\n\n");
    typed(&mut app, ".inn");
    assert_eq!(names(&app), [".INNEN ", ".INNEN/AUSSEN "]);
    press(&mut app, KeyCode::Tab);
    typed(&mut app, "bad - ");
    assert_eq!(names(&app).first().map(String::as_str), Some("TAG"));
    assert!(names(&app).contains(&"CONTINUOUS".to_string()));
}

#[test]
fn an_extension_finishes_the_cue() {
    let (_s, mut app) = app_with(BOOK);
    press(&mut app, KeyCode::Tab);
    typed(&mut app, "anna (v");
    assert_eq!(names(&app), ["(V.O.)"]);
    press(&mut app, KeyCode::Enter);

    assert!(
        app.text().ends_with("y\n\nANNA (V.O.)\n"),
        "{:?}",
        app.text()
    );
    assert_eq!(app.pending.map(|p| p.1), Some(Slot::Dialogue));
}

#[test]
fn the_extension_list_offers_the_english_and_german_ones() {
    let (_s, mut app) = app_with(BOOK);
    press(&mut app, KeyCode::Tab);
    typed(&mut app, "anna (");
    let all = names(&app);

    for want in ["(OFF)", "(V.O.)", "(O.S.)", "(CONT'D)", "(WEITER)"] {
        assert!(all.iter().any(|a| a == want), "{want} in {all:?}");
    }
}

#[test]
fn up_and_down_choose_instead_of_moving_the_cursor() {
    let (_s, mut app) = app_with(BOOK);
    press(&mut app, KeyCode::Tab);
    typed(&mut app, "an");
    let at = app.cursor();

    press(&mut app, KeyCode::Down);
    assert_eq!(app.suggest.as_ref().map(|s| s.sel), Some(1));
    press(&mut app, KeyCode::Up);
    assert_eq!(app.suggest.as_ref().map(|s| s.sel), Some(0));
    assert_eq!(app.cursor(), at);
}

#[test]
fn the_suggestion_box_is_drawn_under_the_cursor() {
    let (_s, mut app) = app_with(BOOK);
    press(&mut app, KeyCode::Tab);
    typed(&mut app, "an");
    let frame = screen(&mut app, 100, 24);

    assert!(frame.contains("ANTON"), "{frame}");
    assert!(frame.contains("ANNA"), "{frame}");
}

#[test]
fn no_suggestion_for_plain_action_text() {
    let (_s, mut app) = app_with(BOOK);
    typed(&mut app, "Anton geht in die Küche");

    assert_eq!(app.suggest, None);
}

#[test]
fn the_line_after_a_transition_is_no_heading_until_it_is_one() {
    let (_s, mut app) = app_with("x\n\n");
    typed(&mut app, "CUT TO:");
    press(&mut app, KeyCode::Enter);
    assert_eq!(
        app.analysis.kind(app.cursor().line),
        LineKind::Heading,
        "empty: the heading to come"
    );

    // Prose typed there is an action, in its case, and shows as one.
    typed(&mut app, "Anna geht.");
    assert_eq!(app.text(), "x\n\nCUT TO:\n\nAnna geht.");
    assert_eq!(app.analysis.kind(app.cursor().line), LineKind::Action);
    assert_eq!(app.suggest, None);
}

#[test]
fn a_typed_prefix_keeps_the_line_a_heading_and_is_capitalised_when_complete() {
    let (_s, mut app) = app_with("x\n\n");
    typed(&mut app, "CUT TO:");
    press(&mut app, KeyCode::Enter);
    typed(&mut app, "in");

    // Only possibly a prefix: shown as the heading to come, typed as it is.
    assert_eq!(app.analysis.kind(app.cursor().line), LineKind::Heading);
    assert_eq!(app.text(), "x\n\nCUT TO:\n\nin");

    typed(&mut app, "t. küche");
    assert_eq!(app.text(), "x\n\nCUT TO:\n\nINT. KÜCHE");
}

#[test]
fn prose_that_starts_like_a_prefix_stays_as_typed() {
    let (_s, mut app) = app_with("x\n\n");
    typed(&mut app, "CUT TO:");
    press(&mut app, KeyCode::Enter);
    typed(&mut app, "In der Küche");

    assert_eq!(app.text(), "x\n\nCUT TO:\n\nIn der Küche");
    assert_eq!(app.analysis.kind(app.cursor().line), LineKind::Action);
}

#[test]
fn alt_h_and_alt_t_make_a_heading_and_a_transition() {
    // (text, key, what the line becomes, the element read)
    for (text, key, want) in [
        ("Anna geht.", 'h', ".ANNA GEHT."),
        ("int. küche - tag", 'h', "INT. KÜCHE - TAG"),
        (".alt", 'h', ".ALT"),
        ("cut to:", 't', "CUT TO:"),
        ("fade out.", 't', ">FADE OUT."),
    ] {
        let (_s, mut app) = app_with(&format!("x\n\n{text}\n\ny"));
        keys(&mut app, &[KeyCode::Up, KeyCode::Up]);
        press(&mut app, KeyCode::End);
        alt(&mut app, key);

        assert_eq!(app.text(), format!("x\n\n{want}\n\ny"), "{text:?}");
        let kind = parse(&app.text()).elements[1].clone();
        match key {
            'h' => assert!(matches!(kind, Element::SceneHeading { .. }), "{kind:?}"),
            _ => assert!(matches!(kind, Element::Transition(_)), "{kind:?}"),
        }
        // One undo step.
        ctrl(&mut app, 'z');
        assert_eq!(app.text(), format!("x\n\n{text}\n\ny"), "{text:?}");
    }
}

#[test]
fn alt_h_on_an_empty_line_opens_a_heading_line() {
    let (_s, mut app) = app_with("x\n\n");
    alt(&mut app, 'h');

    assert_eq!(app.text(), "x\n\n");
    assert_eq!(app.pending.map(|p| p.1), Some(Slot::Heading));
    typed(&mut app, "int. a - tag");
    assert!(app.text().ends_with("INT. A - TAG"), "{}", app.text());

    let (_s, mut app) = app_with("x\n\n");
    alt(&mut app, 't');
    assert_eq!(app.pending.map(|p| p.1), Some(Slot::Transition));
}

/// The text with the cursor on `line` (from 0) at its end, `key` pressed.
fn alt_on(text: &str, line: usize, key: char) -> App {
    let (_s, mut app) = app_with(text);
    app.editor.go_to(Pos {
        line,
        col: usize::MAX,
    });
    alt(&mut app, key);

    app
}

#[test]
fn alt_h_and_alt_t_leave_the_title_page_alone() {
    let text = "Title: Film\nAuthor: Eva\n\nINT. A - TAG\n\nx";
    for key in ['h', 't'] {
        let app = alt_on(text, 0, key);

        assert_eq!(app.text(), text, "{key}");
    }
}

#[test]
fn alt_h_and_alt_t_set_the_line_apart_in_a_paragraph() {
    // (text, line, key, result)
    for (text, line, key, want) in [
        (
            "x\n\nAnna geht.\nint. küche - tag\n\ny",
            3,
            'h',
            "x\n\nAnna geht.\n\nINT. KÜCHE - TAG\n\ny",
        ),
        (
            "x\n\nANNA\nhallo du.\n\ny",
            3,
            't',
            "x\n\nANNA\n\n>HALLO DU.\n\ny",
        ),
        (
            "x\n\nint. küche - tag\nAnna geht.\n\ny",
            2,
            'h',
            "x\n\nINT. KÜCHE - TAG\n\nAnna geht.\n\ny",
        ),
        ("x\n\ncut to:\ny", 2, 't', "x\n\nCUT TO:\n\ny"),
    ] {
        let mut app = alt_on(text, line, key);

        assert_eq!(app.text(), want, "{text:?}");
        let found = parse(&app.text()).elements;
        let slot = |e| Slot::of(e);
        let wanted = if key == 'h' {
            Slot::Heading
        } else {
            Slot::Transition
        };
        assert!(found.iter().any(|e| slot(e) == Some(wanted)), "{found:?}");
        // One undo step.
        ctrl(&mut app, 'z');
        assert_eq!(app.text(), text, "{text:?}");
    }
}

#[test]
fn alt_h_keeps_the_scene_number_as_typed() {
    let app = alt_on("x\n\nint. a #12a#\n\ny", 2, 'h');

    assert_eq!(app.text(), "x\n\nINT. A #12a#\n\ny");
}

#[test]
fn alt_h_does_not_force_a_leading_ellipsis() {
    let text = "x\n\n...und dann\n\ny";
    let app = alt_on(text, 2, 'h');

    assert_eq!(app.text(), text);
}
