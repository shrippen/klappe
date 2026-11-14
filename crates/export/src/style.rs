//! Display-list roles mapped to Kante palette roles (planung/KONZEPT.md, section 4).
//! Translucent areas use opacity, not extra colours.

use klappe_model::Dash as DrawDash;
use klappe_stellprobe::{look, Area, Hue, Role};

use crate::canvas::{Dash, Fill, Stroke};
use crate::error::ExportError;
use crate::palette::{Palette, Rgb};

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

fn class_of(role: Role) -> String {
    match role {
        Role::Drawing { color, .. } => format!("drawing {}", color.name()),
        Role::ArrowHead(c) => format!("arrow-head {}", c.name()),
        Role::FigurePath(c) => format!("figure-path {}", c.name()),
        Role::Mark(c) => format!("mark {}", c.name()),
        Role::Figure(c) => format!("figure {}", c.name()),
        _ => plain_class(role).into(),
    }
}

fn plain_class(role: Role) -> &'static str {
    match role {
        Role::Background => "background",
        Role::FigureHandle(_) | Role::CameraHandle => "handle",
        Role::Wall => "wall",
        Role::Object => "object",
        Role::Camera => "camera",
        Role::Frustum => "frustum",
        Role::CameraPath => "camera-path",
        Role::Light => "light",
        Role::LightBeam => "beam",
        Role::Note => "note",
        Role::Axis => "axis",
        _ => "group-dot",
    }
}

/// 0 for what the export does not stroke.
fn width_pt(role: Role) -> f64 {
    match role {
        Role::Background | Role::FigureHandle(_) | Role::CameraHandle => 0.0,
        Role::CameraPath | Role::FigurePath(_) => PATH_PT,
        Role::Axis => AXIS_PT,
        Role::Mark(_) => MARK_PT,
        Role::Drawing { .. } | Role::ArrowHead(_) => DRAWING_PT,
        _ => OUTLINE_PT,
    }
}

fn tint_opacity(role: Role) -> f64 {
    match role {
        Role::Mark(_) => MARK_OPACITY,
        Role::Drawing { .. } => DRAWING_OPACITY,
        _ => AREA_OPACITY,
    }
}

fn hue_rgb(hue: Hue, pal: &Palette) -> Result<Rgb, ExportError> {
    match hue {
        Hue::Ground => Ok(pal.paper),
        Hue::Text => Ok(pal.foreground),
        Hue::Muted => Ok(pal.muted),
        // Paper has no accent colour; a note takes the set-object ground.
        Hue::Accent => Ok(pal.surface),
        Hue::Color(c) => pal.semantic(c.name()),
    }
}

fn dash_of(dash: DrawDash) -> Dash {
    match dash {
        DrawDash::Solid => Dash::Solid,
        DrawDash::Dashed => Dash::Dashed,
        DrawDash::Dotted => Dash::Dotted,
    }
}

/// What stellprobe's look becomes on paper; the look says the colour, line and fill, this
/// adds widths, opacities and the outline of a solid shape.
pub(crate) fn role_style(role: Role, pal: &Palette) -> Result<RoleStyle, ExportError> {
    let class = class_of(role);
    let width_pt = width_pt(role);

    // Grips only show on screen; a list for export has none, and one that had draws nothing.
    // The image is drawn by its own code.
    if width_pt == 0.0 {
        return Ok(RoleStyle {
            class,
            fill: None,
            stroke: None,
        });
    }

    let look = look(role);
    let colour = hue_rgb(look.hue, pal)?;

    // Print deviations from the screen: a set object is filled with its ground tone, not
    // tinted; a solid shape (not an arrow head) gets a text-coloured outline.
    let fill = match (role, look.on_paper) {
        (Role::Object, _) => Some(Fill {
            rgb: pal.surface,
            opacity: 1.0,
        }),
        (_, Area::None) => None,
        (_, Area::Tint) => Some(Fill {
            rgb: colour,
            opacity: tint_opacity(role),
        }),
        (_, Area::Solid) => Some(Fill {
            rgb: colour,
            opacity: 1.0,
        }),
    };
    let outlined = look.area == Area::Solid && !matches!(role, Role::ArrowHead(_));
    let outline = if outlined { pal.foreground } else { colour };

    Ok(RoleStyle {
        class,
        fill,
        stroke: Some(Stroke {
            rgb: outline,
            width_pt,
            dash: dash_of(look.line),
        }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::{palette, Theme};
    use klappe_model::{FigureColor, Fill as DrawFill};

    type Expected = (Option<(Rgb, f64)>, Option<(Rgb, f64, Dash)>);
    type Case = (Role, String, Expected);

    fn line(rgb: Rgb, width: f64, dash: Dash) -> Option<(Rgb, f64, Dash)> {
        Some((rgb, width, dash))
    }

    fn fill(rgb: Rgb, opacity: f64) -> Option<(Rgb, f64)> {
        Some((rgb, opacity))
    }

    /// What is not stroked, and the outlines in the text, muted and path styles.
    fn plain_cases(pal: &Palette) -> Result<Vec<Case>, ExportError> {
        let fg = pal.foreground;
        let yellow = pal.semantic("yellow")?;
        let orange = pal.semantic("orange")?;
        let red = FigureColor::Red;

        Ok(vec![
            (Role::Background, "background".into(), (None, None)),
            (Role::FigureHandle(red), "handle".into(), (None, None)),
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
                Role::LightBeam,
                "beam".into(),
                (fill(orange, 0.15), line(orange, 0.75, Dash::Solid)),
            ),
            (
                Role::Axis,
                "axis".into(),
                (None, line(orange, 1.5, Dash::Dashed)),
            ),
        ])
    }

    /// The solid shapes with a text-coloured outline, and the coloured roles.
    fn solid_cases(pal: &Palette) -> Result<Vec<Case>, ExportError> {
        let fg = pal.foreground;
        let (red, c_red) = (pal.semantic("red")?, FigureColor::Red);
        let yellow = pal.semantic("yellow")?;
        let orange = pal.semantic("orange")?;
        let outline = line(fg, 0.75, Dash::Solid);

        Ok(vec![
            (Role::Camera, "camera".into(), (fill(yellow, 1.0), outline)),
            (Role::Light, "light".into(), (fill(orange, 1.0), outline)),
            (Role::Note, "note".into(), (fill(pal.surface, 1.0), outline)),
            (
                Role::GroupDot,
                "group-dot".into(),
                (fill(pal.paper, 1.0), outline),
            ),
            (
                Role::Figure(c_red),
                "figure red".into(),
                (fill(red, 1.0), outline),
            ),
            (
                Role::ArrowHead(c_red),
                "arrow-head red".into(),
                (fill(red, 1.0), line(red, 1.5, Dash::Solid)),
            ),
            (
                Role::FigurePath(c_red),
                "figure-path red".into(),
                (None, line(red, 1.25, Dash::Dotted)),
            ),
            (
                Role::Mark(c_red),
                "mark red".into(),
                (fill(red, 0.3), line(red, 1.25, Dash::Solid)),
            ),
        ])
    }

    fn drawing_cases(pal: &Palette) -> Result<Vec<Case>, ExportError> {
        let blue = pal.semantic("blue")?;
        let dashes = [
            (DrawDash::Solid, Dash::Solid),
            (DrawDash::Dashed, Dash::Dashed),
            (DrawDash::Dotted, Dash::Dotted),
        ];
        let fills = [
            (DrawFill::Outline, None),
            (DrawFill::Filled, fill(blue, 0.25)),
        ];
        let mut cases = Vec::new();

        for (draw_dash, dash) in dashes {
            for (draw_fill, tint) in fills {
                let role = Role::Drawing {
                    color: FigureColor::Blue,
                    dash: draw_dash,
                    fill: draw_fill,
                };

                cases.push((role, "drawing blue".into(), (tint, line(blue, 1.5, dash))));
            }
        }

        Ok(cases)
    }

    /// Class, fill (colour, opacity) and stroke (colour, width, dash) of every role, per theme.
    #[test]
    fn every_role_has_its_style() -> Result<(), ExportError> {
        for theme in [Theme::Light, Theme::Dark] {
            let pal = palette(theme)?;
            let mut cases = plain_cases(pal)?;
            cases.extend(solid_cases(pal)?);
            cases.extend(drawing_cases(pal)?);

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
