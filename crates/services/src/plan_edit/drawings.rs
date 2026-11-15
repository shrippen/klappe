//! Changes to a drawing that is on the plan already (LH-19): its style.

use klappe_model::{Dash, FigureColor, Fill};
use klappe_stellprobe::Source;

use super::Scope;
use crate::error::ServiceError;
use crate::field::Field;

/// Gives the drawing `source` names a new look; any other source is refused.
pub(super) fn set_drawing_style(
    scope: &mut Scope<'_>,
    source: &Source,
    (color, dash, fill): (FigureColor, Dash, Fill),
) -> Result<(), ServiceError> {
    let Source::Drawing(id) = source else {
        return Err(ServiceError::BadValue {
            field: Field::Source,
            value: String::new(),
        });
    };
    let drawing = scope
        .plan
        .drawings
        .iter_mut()
        .find(|d| d.id == *id)
        .ok_or(ServiceError::Gone)?;

    drawing.color = color;
    drawing.dash = dash;
    drawing.fill = fill;

    Ok(())
}
