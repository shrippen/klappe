//! What each role looks like, once: colour, line style and fill. The GUI and the export
//! translate it into their own format (Kante colour names, print colours, dash patterns).

use klappe_model::{Dash, FigureColor, Fill};

use crate::display::Role;

/// The colour of a role, as a role of the palette, not a value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hue {
    /// Stands out from every figure colour.
    Ground,
    Text,
    Muted,
    Accent,
    /// One of the palette's hues; also the figure colours.
    Color(FigureColor),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Area {
    None,
    /// Translucent: what lies below stays readable.
    Tint,
    Solid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Look {
    pub hue: Hue,
    pub line: Dash,
    /// The fill on screen.
    pub area: Area,
    /// The fill on paper; differs from `area` only where noted in `look`.
    pub on_paper: Area,
}

const ORANGE: Hue = Hue::Color(FigureColor::Orange);
const YELLOW: Hue = Hue::Color(FigureColor::Yellow);

fn plain(hue: Hue, line: Dash, area: Area) -> Look {
    Look {
        hue,
        line,
        area,
        on_paper: area,
    }
}

pub fn look(role: Role) -> Look {
    match role {
        Role::Drawing {
            color,
            dash,
            fill: model_fill,
        } => {
            // A filled drawing is tinted, like the view cone: the plan below stays readable.
            let area = match model_fill {
                Fill::Outline => Area::None,
                Fill::Filled => Area::Tint,
            };

            plain(Hue::Color(color), dash, area)
        }
        Role::ArrowHead(c) | Role::FigureHandle(c) | Role::Figure(c) => {
            plain(Hue::Color(c), Dash::Solid, Area::Solid)
        }
        Role::FigurePath(c) => plain(Hue::Color(c), Dash::Dotted, Area::None),
        // The one wanted difference: on screen a tint would hide the ground below the mark,
        // on paper the tint reads better than a bare ring.
        Role::Mark(c) => Look {
            on_paper: Area::Tint,
            ..plain(Hue::Color(c), Dash::Solid, Area::None)
        },
        _ => fixed(role),
    }
}

/// The roles that carry no colour of their own; the coloured ones are told apart in `look`.
fn fixed(role: Role) -> Look {
    match role {
        // The image takes no colour; the canvas draws it as it is.
        Role::Background => plain(Hue::Muted, Dash::Solid, Area::None),
        Role::Wall => plain(Hue::Text, Dash::Solid, Area::None),
        Role::Object => plain(Hue::Muted, Dash::Solid, Area::Tint),
        Role::LightBeam => plain(ORANGE, Dash::Solid, Area::Tint),
        Role::Light => plain(ORANGE, Dash::Solid, Area::Solid),
        Role::Note => plain(Hue::Accent, Dash::Solid, Area::Solid),
        Role::Axis => plain(ORANGE, Dash::Dashed, Area::None),
        Role::CameraPath => plain(YELLOW, Dash::Dotted, Area::None),
        Role::Frustum => plain(YELLOW, Dash::Solid, Area::Tint),
        Role::Camera | Role::CameraHandle => plain(YELLOW, Dash::Solid, Area::Solid),
        // The group dot. The ground colour stands out from every figure colour (the text
        // colour did not).
        _ => plain(Hue::Ground, Dash::Solid, Area::Solid),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_mark_differs_on_paper() {
        let red = FigureColor::Red;
        let roles = [
            Role::Background,
            Role::Wall,
            Role::Object,
            Role::LightBeam,
            Role::Light,
            Role::Note,
            Role::ArrowHead(red),
            Role::Axis,
            Role::CameraPath,
            Role::Frustum,
            Role::Camera,
            Role::FigurePath(red),
            Role::FigureHandle(red),
            Role::CameraHandle,
            Role::Figure(red),
            Role::GroupDot,
        ];

        for role in roles {
            assert_eq!(look(role).area, look(role).on_paper, "{role:?}");
        }
        assert_eq!(look(Role::Mark(red)).area, Area::None);
        assert_eq!(look(Role::Mark(red)).on_paper, Area::Tint);
    }
}
