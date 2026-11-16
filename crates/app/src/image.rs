//! The card of a selected image (LH-18): its opacity typed or its file swapped, as plan edits.

use klappe_services::{read_number, Field, PlanEdit, Range, Source, Unit, MIN_IMAGE_OPACITY};

use crate::fileurl;
use crate::tools::Typed;

const PERCENT: f64 = 100.0;

/// The opacity in percent as typed, for the selected image; out of its range it is refused
/// as typed (`5`, not `0.05`). `None` when something else is selected.
pub fn opacity_edit(selected: Option<&Source>, text: &str) -> Typed {
    let Some(source @ Source::Image(_)) = selected else {
        return None;
    };
    let percent = read_number(
        Field::Opacity,
        text,
        Unit::Percent,
        Range::Within(MIN_IMAGE_OPACITY * PERCENT, PERCENT),
    );

    Some(percent.map(|percent| PlanEdit::SetImageOpacity {
        source: source.clone(),
        opacity: percent / PERCENT,
    }))
}

/// The image file `file` (a path or a `file://` URL) for the selected image. `None` when
/// something else is selected.
pub fn replace_edit(selected: Option<&Source>, file: &str) -> Option<PlanEdit> {
    let Some(source @ Source::Image(_)) = selected else {
        return None;
    };

    Some(PlanEdit::ReplaceImage {
        source: source.clone(),
        with: fileurl::to_path(file),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn image() -> Source {
        Source::Image("i1".into())
    }

    #[test]
    fn a_percent_is_a_share() {
        let edit = opacity_edit(Some(&image()), "40 %");

        assert_eq!(
            edit.unwrap().unwrap(),
            PlanEdit::SetImageOpacity {
                source: image(),
                opacity: 0.4
            }
        );
    }

    #[test]
    fn no_number_is_refused_and_no_image_is_none() {
        assert!(opacity_edit(Some(&image()), "viel").unwrap().is_err());
        assert!(opacity_edit(Some(&Source::Wall(0)), "50").is_none());
        assert!(replace_edit(None, "a.png").is_none());
    }

    #[test]
    fn a_percent_out_of_range_is_refused_as_typed() {
        // The status line names what was typed, not the share the session keeps.
        for typed in ["5", "150"] {
            assert!(
                opacity_edit(Some(&image()), typed).unwrap().is_err(),
                "{typed}"
            );
        }
    }

    #[test]
    fn a_url_is_a_path() {
        let edit = replace_edit(Some(&image()), "file:///tmp/a%20b.png");

        assert_eq!(
            edit,
            Some(PlanEdit::ReplaceImage {
                source: image(),
                with: "/tmp/a b.png".into()
            })
        );
    }
}
