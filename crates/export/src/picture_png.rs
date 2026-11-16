//! A view of the set as PNG (LH-86): the frame of `klappe_stellprobe::picture` coloured with
//! the print palette, the names written over it.
//!
//! ```text
//! Picture { role, shade, edge per pixel } ─► role → SceneStyle ─► RGB ─► names ─► png_write
//! ```

use klappe_stellprobe::{Picture, SceneRole};

use crate::canvas::{Anchor, Prim, Weight};
use crate::error::ExportError;
use crate::palette::{palette, Rgb, Theme};
use crate::png_write;
use crate::raster::{draw_over, Surface};
use crate::style::{scene_style, shaded};

/// A screen picture has no sheet size to print at.
const PICTURE_DPI: f64 = 72.0;
/// Every role code of a pixel fits a byte.
const CODES: usize = 256;
/// A name is this share of the picture's width high.
const LABEL_SHARE: f64 = 1.0 / 80.0;
/// A name has a ring of the paper colour around it, this far (in pixels) from the letters.
const HALO_PX: f64 = 1.5;
const HALO_STEPS: [(f64, f64); 4] = [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)];

pub(crate) fn write(picture: &Picture, theme: Theme) -> Result<Vec<u8>, ExportError> {
    let pal = palette(theme)?;
    let frame = &picture.frame;

    // Face and line of each role code; a code without a role is the paper.
    let styles = (0..CODES)
        .map(|code| {
            let role = u8::try_from(code).ok().and_then(SceneRole::of_code);

            role.map(|role| scene_style(role, pal)).transpose()
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut pixels = Vec::with_capacity(frame.role.len() * 3);
    for i in 0..frame.role.len() {
        let rgb = match &styles[usize::from(frame.role[i])] {
            None => pal.paper,
            Some(style) if frame.edge[i] => style.line,
            Some(style) => shaded(style.face, frame.shade[i]),
        };
        pixels.extend_from_slice(&[rgb.r, rgb.g, rgb.b]);
    }

    let surface = Surface {
        width: frame.width as u32,
        height: frame.height as u32,
        pixels,
    };
    let names = names(picture, pal.foreground, pal.paper);
    let surface = draw_over(surface, pal.paper, &names)?;

    Ok(png_write::encode(
        surface.width,
        surface.height,
        &surface.pixels,
        PICTURE_DPI,
    ))
}

/// Each name centred at its anchor, over a ring of paper colour so it reads on any face.
fn names(picture: &Picture, ink: Rgb, paper: Rgb) -> Vec<Prim> {
    let size = picture.frame.width as f64 * LABEL_SHARE;
    let text = |dx: f64, dy: f64, rgb: Rgb, label: &klappe_stellprobe::PictureLabel| Prim::Text {
        class: "name".into(),
        at: (label.x + dx, label.y + dy),
        anchor: Anchor::Middle,
        size_pt: size,
        weight: Weight::Bold,
        rgb,
        text: label.text.clone(),
    };

    let mut prims = Vec::new();
    for label in &picture.labels {
        for (dx, dy) in HALO_STEPS {
            prims.push(text(dx * HALO_PX, dy * HALO_PX, paper, label));
        }
        prims.push(text(0.0, 0.0, ink, label));
    }

    prims
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use klappe_stellprobe::Frame;

    use super::*;
    use crate::png;

    /// One row of four pixels: a lit wall, a dim wall, a wall's edge, nothing.
    fn row() -> Picture {
        Picture {
            frame: Frame {
                width: 4,
                height: 1,
                role: [SceneRole::Wall.code(); 3].into_iter().chain([0]).collect(),
                shade: vec![255, 128, 255, 0],
                source: vec![u32::MAX; 4],
                face: vec![0; 4],
                depth: vec![1.0; 4],
                edge: vec![false, false, true, false],
            },
            sources: Vec::new(),
            labels: Vec::new(),
        }
    }

    /// LH-86: a wall pixel gets the wall's print colour (lit by its shade), its edge the
    /// outline colour, an empty pixel the paper.
    #[test]
    fn a_wall_pixel_has_the_walls_print_colour() {
        let pal = palette(Theme::Light).unwrap();
        let wall = scene_style(SceneRole::Wall, pal).unwrap();

        let file = write(&row(), Theme::Light).unwrap();
        let rgb = png::parse(&file).unwrap().rgb_over(pal.paper).unwrap();

        let pixel = |i: usize| Rgb {
            r: rgb[i * 3],
            g: rgb[i * 3 + 1],
            b: rgb[i * 3 + 2],
        };
        assert_eq!(pixel(0), wall.face);
        assert_eq!(pixel(1), shaded(wall.face, 128));
        assert_eq!(pixel(2), wall.line);
        assert_eq!(pixel(3), pal.paper);
        assert_ne!(wall.face, wall.line);
    }
}
