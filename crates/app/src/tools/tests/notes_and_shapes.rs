//! Tests of the note and drawing tools and of typed heights.

use super::*;

#[test]
fn note_is_a_tool_and_a_click_edits_nothing() {
    assert_eq!(Tool::parse("note", ""), Some(Tool::Note));
    assert_eq!(click_edit(&Tool::Note, p(1.0, 1.0)), None);
}

#[test]
fn image_is_a_tool_and_a_click_edits_nothing() {
    assert_eq!(Tool::parse("image", ""), Some(Tool::Image));
    assert_eq!(click_edit(&Tool::Image, p(1.0, 1.0)), None);
    assert_eq!(
        drag_edit(&Tool::Image, None, p(1.0, 1.0), p(3.0, 1.0)),
        None
    );
}

/// The dialog hands a `file://` URL over, percent-escapes and all.
#[test]
fn the_picked_file_is_placed_where_the_click_was() {
    assert_eq!(
        image_edit("file:///home/a/Bilder/Skizze%20eins.png", p(2.0, 3.0)),
        PlanEdit::AddImage {
            source: "/home/a/Bilder/Skizze eins.png".into(),
            at: p(2.0, 3.0),
        }
    );
}

#[test]
fn a_click_on_a_note_edits_it_else_a_new_one_goes_there() {
    let at = p(2.0, 3.0);
    let note = Source::Note("n1".into());
    let wall = Source::Wall(0);

    assert_eq!(
        NoteDraft::click(at, Some(&note)),
        NoteDraft::Edit("n1".into())
    );
    assert_eq!(NoteDraft::click(at, Some(&wall)), NoteDraft::New(at));
    assert_eq!(NoteDraft::click(at, None), NoteDraft::New(at));
}

#[test]
fn the_typed_text_makes_the_note_edit() {
    let at = p(2.0, 3.0);

    assert_eq!(
        note_edit(&NoteDraft::New(at), " Licht an "),
        Some(PlanEdit::AddNote {
            at,
            text: "Licht an".into()
        })
    );
    assert_eq!(
        note_edit(&NoteDraft::Edit("n1".into()), "Licht aus"),
        Some(PlanEdit::EditNote {
            id: "n1".into(),
            text: "Licht aus".into()
        })
    );
    assert_eq!(note_edit(&NoteDraft::New(at), "  "), None);
    assert_eq!(note_edit(&NoteDraft::Idle, "x"), None);
}

fn blue_dotted_filled() -> DrawStyle {
    DrawStyle {
        color: FigureColor::Blue,
        dash: Dash::Dotted,
        fill: Fill::Filled,
    }
}

#[test]
fn the_drawing_tools_are_named_and_a_click_edits_nothing() {
    for name in ["line", "arrow", "rect", "ellipse", "area"] {
        let tool = Tool::parse(name, "");

        assert!(tool.is_some(), "{name}");
        assert_eq!(click_edit(&tool.unwrap_or(Tool::Select), p(1.0, 1.0)), None);
    }
}

#[test]
fn a_drag_draws_the_shape_of_the_tool_in_the_style_picked() {
    let (from, to) = (p(1.0, 1.0), p(4.0, 3.0));
    let style = blue_dotted_filled();
    let drawn = |shape| PlanEdit::AddDrawing {
        shape,
        color: FigureColor::Blue,
        dash: Dash::Dotted,
        fill: Fill::Filled,
    };

    let edit = |tool| draw_edit(&tool, style, from, to);

    assert_eq!(
        edit(Tool::Line),
        Some(drawn(DrawingShape::Line { from, to }))
    );
    assert_eq!(
        edit(Tool::Arrow),
        Some(drawn(DrawingShape::Arrow { from, to }))
    );
    assert_eq!(
        edit(Tool::Rect),
        Some(drawn(DrawingShape::Rect { from, to }))
    );
    assert_eq!(
        edit(Tool::Ellipse),
        Some(drawn(DrawingShape::Ellipse { from, to }))
    );
    assert_eq!(edit(Tool::Wall), None);
    assert_eq!(edit(Tool::Area), None);
}

/// A line needs a length; a rectangle and an ellipse a width and a height.
#[test]
fn a_drag_that_makes_nothing_draws_nothing() {
    let style = DrawStyle::default();
    let at = p(2.0, 2.0);

    assert_eq!(draw_edit(&Tool::Line, style, at, at), None);
    assert_eq!(draw_edit(&Tool::Arrow, style, at, at), None);
    assert_eq!(draw_edit(&Tool::Rect, style, at, p(5.0, 2.0)), None);
    assert_eq!(draw_edit(&Tool::Ellipse, style, at, p(2.0, 5.0)), None);
    assert!(draw_edit(&Tool::Line, style, at, p(5.0, 2.0)).is_some());
}

#[test]
fn the_style_is_read_from_the_names_of_the_tool_bar() {
    assert_eq!(
        DrawStyle::parse("blue", "dotted", "filled"),
        Some(blue_dotted_filled())
    );
    assert_eq!(DrawStyle::parse("pink", "solid", "outline"), None);
    assert_eq!(DrawStyle::parse("red", "wavy", "outline"), None);
    assert_eq!(DrawStyle::parse("red", "solid", "half"), None);
    assert_eq!(
        DrawStyle::parse("red", "solid", "outline"),
        Some(DrawStyle::default())
    );
}

/// The tool bar translates each name with `color.<name>`, `dash.<name>`, `fill.<name>`.
#[test]
fn every_style_name_has_a_text_in_both_languages() {
    let keys = FigureColor::ALL
        .iter()
        .map(|c| format!("color.{}", c.name()))
        .chain(Dash::ALL.iter().map(|d| format!("dash.{}", d.name())))
        .chain(Fill::ALL.iter().map(|f| format!("fill.{}", f.name())));

    for key in keys {
        for language in [Language::German, Language::English] {
            assert_ne!(language.text(&key), key);
        }
    }
}

/// Review of PR 24 (clicks not snapped): the diagonal neighbour of the first point is
/// 0.354 m away and added a corner instead of closing the area.
#[test]
fn a_click_on_a_diagonal_grid_neighbour_closes() {
    let draft = AreaDraft::default()
        .click(Point::new(0.0, 0.0))
        .click(Point::new(2.0, 0.0))
        .click(Point::new(2.0, 2.0));

    assert!(draft.closes(Point::new(0.25, 0.25), Closing::Free));
    assert!(!draft.closes(Point::new(0.5, 0.5), Closing::Free));
}

/// Review of PR 86: the closing radius was fixed at 0.4 m; on a 5 cm grid a click 0.3 m from
/// the first point closed a narrow area instead of adding its corner.
#[test]
fn a_fine_grid_draws_a_narrow_area() {
    let draft = AreaDraft::default()
        .click(Point::new(0.0, 0.0))
        .click(Point::new(4.0, 0.0))
        .click(Point::new(4.0, 0.3));

    assert!(!draft.closes(Point::new(0.0, 0.3), Closing::OnGrid(0.05)));
}

/// Review of PR 86: on a 1 m grid a strip 1 m wide closed as a triangle (1.6 steps radius).
#[test]
fn a_metre_wide_strip_does_not_close() {
    let draft = AreaDraft::default()
        .click(Point::new(0.0, 0.0))
        .click(Point::new(4.0, 0.0))
        .click(Point::new(4.0, 1.0));

    assert!(!draft.closes(Point::new(0.0, 1.0), Closing::OnGrid(1.0)));
    assert!(!draft.closes(Point::new(0.0, 1.0), Closing::Free));
}

#[test]
fn clicks_collect_the_points_of_an_area() {
    let (a, b, c) = (p(0.0, 0.0), p(4.0, 0.0), p(4.0, 3.0));

    let draft = AreaDraft::default().click(a).click(b).click(b).click(c);

    assert_eq!(draft.points(), [a, b, c]);
    assert!(!draft.is_empty());
}

/// Two points make no area: a click on the first point adds a point. With three, a click
/// within 0.4 m of the first (a grid neighbour) closes; further away it adds.
#[test]
fn a_click_on_the_first_point_closes_from_three_points_on() {
    let (a, b, c) = (p(0.0, 0.0), p(4.0, 0.0), p(4.0, 3.0));
    let two = AreaDraft::default().click(a).click(b);
    let three = two.clone().click(c);

    assert!(!AreaDraft::default().closes(a, Closing::Free));
    assert!(!two.closes(a, Closing::Free));
    assert!(three.closes(a, Closing::Free));
    assert!(three.closes(p(0.25, 0.0), Closing::Free));
    assert!(!three.closes(p(0.5, 0.0), Closing::Free));

    // Snapped, only a click on the first point closes; a grid neighbour adds a corner.
    assert!(three.closes(a, Closing::OnGrid(0.25)));
    assert!(!three.closes(p(0.25, 0.0), Closing::OnGrid(0.25)));
}

#[test]
fn an_area_needs_three_points_and_takes_the_style() {
    let (a, b, c) = (p(0.0, 0.0), p(4.0, 0.0), p(4.0, 3.0));
    let two = AreaDraft::default().click(a).click(b);
    let three = two.clone().click(c);

    assert_eq!(area_edit(&two, DrawStyle::default()), None);
    assert_eq!(
        area_edit(&three, blue_dotted_filled()),
        Some(PlanEdit::AddDrawing {
            shape: DrawingShape::Area {
                points: vec![a, b, c]
            },
            color: FigureColor::Blue,
            dash: Dash::Dotted,
            fill: Fill::Filled,
        })
    );
}

/// Like the notes: with the drawings layer locked the tool took points and dropped the
/// area on closing.
#[test]
fn a_locked_drawings_layer_takes_no_points() {
    assert!(matches!(
        layer_open(Layer::Drawings, Lock::Locked),
        Err(ServiceError::Locked(Layer::Drawings))
    ));
    assert!(layer_open(Layer::Drawings, Lock::Unlocked).is_ok());
}

/// LH-92: the free form is a tool; its clicks set points, and the closed draft is one new
/// object, from three points on.
#[test]
fn a_free_form_is_made_from_three_points() {
    let (a, b, c) = (p(0.0, 0.0), p(4.0, 0.0), p(0.0, 3.0));
    let two = AreaDraft::default().click(a).click(b);
    let three = two.clone().click(c);

    assert_eq!(Tool::parse("freeform", ""), Some(Tool::FreeForm));
    assert_eq!(click_edit(&Tool::FreeForm, a), None);
    assert_eq!(form_edit(&two), None);
    assert_eq!(
        form_edit(&three),
        Some(PlanEdit::AddShape {
            points: vec![a, b, c]
        })
    );
    assert!(three.closes(a, Closing::Free));
}

/// LH-92: the points of a free form are an object of the set, those of an area a drawing.
#[test]
fn a_free_form_is_held_by_the_set_layer() {
    assert_eq!(draft_layer(&Tool::FreeForm), Layer::Set);
    assert_eq!(draft_layer(&Tool::Area), Layer::Drawings);
    assert!(matches!(
        layer_open(Layer::Set, Lock::Locked),
        Err(ServiceError::Locked(Layer::Set))
    ));
}
