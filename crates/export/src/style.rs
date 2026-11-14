//! Display-list roles mapped to Kante palette roles (planung/KONZEPT.md, section 4).
//! Translucent areas use opacity, not extra colours.

use klappe_model::{Dash as DrawDash, FigureColor, Fill as DrawFill};
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

pub(crate) fn colour_name(color: FigureColor) -> &'static str {
    match color {
        FigureColor::Purple => "purple",
        FigureColor::Blue => "blue",
        FigureColor::Green => "green",
        FigureColor::Yellow => "yellow",
        FigureColor::Orange => "orange",
        FigureColor::Red => "red",
        FigureColor::Cyan => "cyan",
    }
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
            let c = pal.semantic(colour_name(color))?;
            let tinted = (fill == DrawFill::Filled).then(|| tint(c, DRAWING_OPACITY));
            let dash = match dash {
                DrawDash::Solid => Dash::Solid,
                DrawDash::Dashed => Dash::Dashed,
                DrawDash::Dotted => Dash::Dotted,
            };
            (
                format!("drawing {}", colour_name(color)),
                tinted,
                Some(line(c, DRAWING_PT, dash)),
            )
        }
        Role::ArrowHead(color) => {
            let c = pal.semantic(colour_name(color))?;
            (
                format!("arrow-head {}", colour_name(color)),
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
            format!("figure-path {}", colour_name(color)),
            None,
            Some(line(
                pal.semantic(colour_name(color))?,
                PATH_PT,
                Dash::Dotted,
            )),
        ),
        Role::Mark(color) => {
            let c = pal.semantic(colour_name(color))?;
            (
                format!("mark {}", colour_name(color)),
                Some(tint(c, MARK_OPACITY)),
                Some(line(c, MARK_PT, Dash::Solid)),
            )
        }
        Role::Figure(color) => (
            format!("figure {}", colour_name(color)),
            Some(solid(pal.semantic(colour_name(color))?)),
            Some(line(fg, OUTLINE_PT, Dash::Solid)),
        ),
    };

    Ok(RoleStyle {
        class: style.0,
        fill: style.1,
        stroke: style.2,
    })
}
