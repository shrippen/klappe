//! Display-list roles mapped to Kante palette roles (planung/KONZEPT.md, section 4).
//! Translucent areas use opacity, not extra colours.

use klappe_model::{Dash as DrawDash, Fill as DrawFill};
use klappe_stellprobe::Role;

use crate::canvas::{Dash, Fill, Stroke};
use crate::error::ExportError;
use crate::palette::Palette;

const OUTLINE_PT: f64 = 0.75;
const PATH_PT: f64 = 1.25;
const AXIS_PT: f64 = 1.5;
const MARK_PT: f64 = 1.25;
const DRAWING_PT: f64 = 1.5;
/// Frustum and beam: the outline carries the shape, the tint only hints.
const AREA_OPACITY: f64 = 0.15;
const MARK_OPACITY: f64 = 0.3;
const DRAWING_OPACITY: f64 = 0.25;

pub(crate) struct RoleStyle {
    pub class: String,
    pub fill: Option<Fill>,
    pub stroke: Option<Stroke>,
}

pub(crate) fn role_style(role: Role, pal: &Palette) -> Result<RoleStyle, ExportError> {
    let solid = |rgb| Fill { rgb, opacity: 1.0 };
    let tint = |rgb, opacity| Fill { rgb, opacity };
    let line = |rgb, width_pt, dash| Stroke {
        rgb,
        width_pt,
        dash,
    };

    let fg = pal.foreground;
    let style = match role {
        Role::Background => ("background".to_string(), None, None),
        // Grips only show on screen; a list for export has none, and one that had draws nothing.
        Role::FigureHandle(_) | Role::CameraHandle => ("handle".to_string(), None, None),
        Role::Wall => (
            "wall".to_string(),
            None,
            Some(line(fg, OUTLINE_PT, Dash::Solid)),
        ),
        Role::Object => (
            "object".to_string(),
            Some(solid(pal.surface)),
            Some(line(pal.muted, OUTLINE_PT, Dash::Solid)),
        ),
        Role::Camera => (
            "camera".to_string(),
            Some(solid(pal.semantic("yellow")?)),
            Some(line(fg, OUTLINE_PT, Dash::Solid)),
        ),
        Role::Frustum => {
            let c = pal.semantic("yellow")?;
            (
                "frustum".to_string(),
                Some(tint(c, AREA_OPACITY)),
                Some(line(c, OUTLINE_PT, Dash::Solid)),
            )
        }
        Role::CameraPath => (
            "camera-path".to_string(),
            None,
            Some(line(pal.semantic("yellow")?, PATH_PT, Dash::Dotted)),
        ),
        Role::Light => (
            "light".to_string(),
            Some(solid(pal.semantic("orange")?)),
            Some(line(fg, OUTLINE_PT, Dash::Solid)),
        ),
        Role::LightBeam => {
            let c = pal.semantic("orange")?;
            (
                "beam".to_string(),
                Some(tint(c, AREA_OPACITY)),
                Some(line(c, OUTLINE_PT, Dash::Solid)),
            )
        }
        Role::Note => (
            "note".to_string(),
            Some(solid(pal.surface)),
            Some(line(fg, OUTLINE_PT, Dash::Solid)),
        ),
        Role::Drawing { color, dash, fill } => {
            let c = pal.semantic(color.name())?;
            let tinted = (fill == DrawFill::Filled).then(|| tint(c, DRAWING_OPACITY));
            let dash = match dash {
                DrawDash::Solid => Dash::Solid,
                DrawDash::Dashed => Dash::Dashed,
                DrawDash::Dotted => Dash::Dotted,
            };
            (
                format!("drawing {}", color.name()),
                tinted,
                Some(line(c, DRAWING_PT, dash)),
            )
        }
        Role::ArrowHead(color) => {
            let c = pal.semantic(color.name())?;
            (
                format!("arrow-head {}", color.name()),
                Some(solid(c)),
                Some(line(c, DRAWING_PT, Dash::Solid)),
            )
        }
        Role::Axis => (
            "axis".to_string(),
            None,
            Some(line(pal.semantic("orange")?, AXIS_PT, Dash::Dashed)),
        ),
        Role::FigurePath(color) => (
            format!("figure-path {}", color.name()),
            None,
            Some(line(pal.semantic(color.name())?, PATH_PT, Dash::Dotted)),
        ),
        Role::Mark(color) => {
            let c = pal.semantic(color.name())?;
            (
                format!("mark {}", color.name()),
                Some(tint(c, MARK_OPACITY)),
                Some(line(c, MARK_PT, Dash::Solid)),
            )
        }
        Role::Figure(color) => (
            format!("figure {}", color.name()),
            Some(solid(pal.semantic(color.name())?)),
            Some(line(fg, OUTLINE_PT, Dash::Solid)),
        ),
        // Paper with an outline: stands out on every figure colour.
        Role::GroupDot => (
            "group-dot".to_string(),
            Some(solid(pal.paper)),
            Some(line(fg, OUTLINE_PT, Dash::Solid)),
        ),
    };

    Ok(RoleStyle {
        class: style.0,
        fill: style.1,
        stroke: style.2,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::{palette, Rgb, Theme};
    use klappe_model::FigureColor;

    type Expected = (Option<(Rgb, f64)>, Option<(Rgb, f64, Dash)>);

    /// Class, fill (colour, opacity) and stroke (colour, width, dash) of every role, per theme.
    #[test]
    fn every_role_has_its_style() -> Result<(), ExportError> {
        for theme in [Theme::Light, Theme::Dark] {
            let pal = palette(theme)?;
            let fg = pal.foreground;
            let c = |name: &str| pal.semantic(name);
            let red = c("red")?;
            let blue = c("blue")?;
            let yellow = c("yellow")?;
            let orange = c("orange")?;
            let line = |rgb, w, d| Some((rgb, w, d));
            let fill = |rgb, o| Some((rgb, o));

            let cases: Vec<(Role, String, Expected)> = vec![
                (Role::Background, "background".into(), (None, None)),
                (
                    Role::FigureHandle(FigureColor::Red),
                    "handle".into(),
                    (None, None),
                ),
                (Role::CameraHandle, "handle".into(), (None, None)),
                (
                    Role::Wall,
                    "wall".into(),
                    (None, line(fg, 0.75, Dash::Solid)),
                ),
                (
                    Role::Object,
                    "object".into(),
                    (fill(pal.surface, 1.0), line(pal.muted, 0.75, Dash::Solid)),
                ),
                (
                    Role::Camera,
                    "camera".into(),
                    (fill(yellow, 1.0), line(fg, 0.75, Dash::Solid)),
                ),
                (
                    Role::Frustum,
                    "frustum".into(),
                    (fill(yellow, 0.15), line(yellow, 0.75, Dash::Solid)),
                ),
                (
                    Role::CameraPath,
                    "camera-path".into(),
                    (None, line(yellow, 1.25, Dash::Dotted)),
                ),
                (
                    Role::Light,
                    "light".into(),
                    (fill(orange, 1.0), line(fg, 0.75, Dash::Solid)),
                ),
                (
                    Role::LightBeam,
                    "beam".into(),
                    (fill(orange, 0.15), line(orange, 0.75, Dash::Solid)),
                ),
                (
                    Role::Note,
                    "note".into(),
                    (fill(pal.surface, 1.0), line(fg, 0.75, Dash::Solid)),
                ),
                (
                    Role::ArrowHead(FigureColor::Red),
                    "arrow-head red".into(),
                    (fill(red, 1.0), line(red, 1.5, Dash::Solid)),
                ),
                (
                    Role::Axis,
                    "axis".into(),
                    (None, line(orange, 1.5, Dash::Dashed)),
                ),
                (
                    Role::FigurePath(FigureColor::Red),
                    "figure-path red".into(),
                    (None, line(red, 1.25, Dash::Dotted)),
                ),
                (
                    Role::Mark(FigureColor::Red),
                    "mark red".into(),
                    (fill(red, 0.3), line(red, 1.25, Dash::Solid)),
                ),
                (
                    Role::Figure(FigureColor::Red),
                    "figure red".into(),
                    (fill(red, 1.0), line(fg, 0.75, Dash::Solid)),
                ),
                (
                    Role::GroupDot,
                    "group-dot".into(),
                    (fill(pal.paper, 1.0), line(fg, 0.75, Dash::Solid)),
                ),
            ];
            let mut cases = cases;
            let drawings = [
                (DrawDash::Solid, Dash::Solid),
                (DrawDash::Dashed, Dash::Dashed),
                (DrawDash::Dotted, Dash::Dotted),
            ];
            for (draw_dash, dash) in drawings {
                for (draw_fill, tint) in [
                    (DrawFill::Outline, None),
                    (DrawFill::Filled, fill(blue, 0.25)),
                ] {
                    let role = Role::Drawing {
                        color: FigureColor::Blue,
                        dash: draw_dash,
                        fill: draw_fill,
                    };
                    cases.push((role, "drawing blue".into(), (tint, line(blue, 1.5, dash))));
                }
            }

            for (role, class, (want_fill, want_stroke)) in cases {
                let got = role_style(role, pal)?;

                assert_eq!(got.class, class, "{role:?}");
                assert_eq!(
                    got.fill.map(|f| (f.rgb, f.opacity)),
                    want_fill,
                    "{role:?} fill"
                );
                assert_eq!(
                    got.stroke.map(|s| (s.rgb, s.width_pt, s.dash)),
                    want_stroke,
                    "{role:?} stroke"
                );
            }
        }

        Ok(())
    }
}
