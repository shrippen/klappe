//! The style card of a selected drawing (LH-19): its colour, line and fill as an edit.

use klappe_services::{PlanEdit, Source};

use crate::tools::DrawStyle;

/// `style` for the selected drawing; `None` when something else is selected.
pub fn style_edit(selected: Option<&Source>, style: DrawStyle) -> Option<PlanEdit> {
    let Some(Source::Drawing(id)) = selected else {
        return None;
    };

    Some(PlanEdit::SetDrawingStyle {
        source: Source::Drawing(id.clone()),
        color: style.color,
        dash: style.dash,
        fill: style.fill,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use klappe_services::{Dash, FigureColor, Fill};

    const STYLE: DrawStyle = DrawStyle {
        color: FigureColor::Cyan,
        dash: Dash::Dotted,
        fill: Fill::Filled,
    };

    #[test]
    fn a_drawing_takes_the_style() {
        let edit = style_edit(Some(&Source::Drawing("d1".into())), STYLE);

        assert_eq!(
            edit,
            Some(PlanEdit::SetDrawingStyle {
                source: Source::Drawing("d1".into()),
                color: FigureColor::Cyan,
                dash: Dash::Dotted,
                fill: Fill::Filled,
            })
        );
    }

    #[test]
    fn nothing_else_takes_a_style() {
        assert_eq!(style_edit(Some(&Source::Wall(0)), STYLE), None);
        assert_eq!(style_edit(None, STYLE), None);
    }
}
