//! The picture of a view as pixels (3D-KONZEPT.md, section 5): the scene's faces cut at the
//! near plane, projected, split into triangles and filled by `klappe_raster`. Covering is
//! right per pixel; colours stay roles.
//!
//! ```text
//!   scene ──► faces in the eye's frame ──► cut at NEAR_M ──► pixels ──► fan of triangles
//!                                                                         │ raster
//!   labels: anchor projected, kept where nothing nearer covers it    ◄── Frame
//! ```

use klappe_model::{FloorPlan, Project, SetPlan};
use klappe_raster::{rasterize, Corner, Frame, Paint, Triangle, NO_SOURCE};

use crate::display::Source;
use crate::eye::{clip_near, Eye, Pinhole, Vec3, NEAR_M};
use crate::scene::{scene, Solid};

/// Light falls from this direction (above, from the plan's top left), so tops are lighter
/// than sides and two sides differ.
const LIGHT: Vec3 = [-0.36, -0.48, 0.8];
/// Share of light a face gets even when it turns away.
const AMBIENT: f64 = 0.45;
const FULL_SHADE: f64 = 255.0;
/// A label is hidden when something lies this much nearer than its anchor.
const COVER_M: f32 = 0.05;

/// What a view shows: the pixels, what each source number means, and the names to write.
#[derive(Debug, Clone, PartialEq)]
pub struct Picture {
    pub frame: Frame,
    /// What `frame.source` numbers stand for: index into this list.
    pub sources: Vec<Source>,
    pub labels: Vec<PictureLabel>,
}

/// A name at a point of the picture, in pixels, e.g. a figure's above its head.
#[derive(Debug, Clone, PartialEq)]
pub struct PictureLabel {
    pub text: String,
    pub x: f64,
    pub y: f64,
}

/// What `eye` sees, `width_px` wide and as high as its aspect makes it; `None` when the eye
/// sees nothing (an unknown camera, one without marks) or the width is 0.
pub fn picture(
    project: &Project,
    set: &SetPlan,
    plan: &FloorPlan,
    eye: &Eye,
    width_px: usize,
) -> Option<Picture> {
    let pinhole = Pinhole::of(eye, plan)?;
    let Eye::Shot { t_s, .. } = eye;
    let scale = width_px as f64;
    let height_px = (pinhole.frame_height * scale).round() as usize;
    if width_px == 0 || height_px == 0 {
        return None;
    }

    let solids = scene(project, set, plan, *t_s);
    let mut sources = Vec::new();
    let mut triangles = Vec::new();
    let mut face = 0;
    for solid in &solids {
        let source = match &solid.source {
            Some(s) => {
                sources.push(s.clone());
                (sources.len() - 1) as u32
            }
            None => NO_SOURCE,
        };
        for corners in &solid.faces {
            let paint = Paint {
                role: solid.role.code(),
                shade: shade(corners),
                source,
                face,
                sparse: solid.role == crate::scene::SceneRole::Missing,
            };
            face += 1;
            fan(&pinhole, scale, corners, paint, &mut triangles);
        }
    }

    let frame = rasterize(width_px, height_px, &triangles);
    let labels = labels(&pinhole, scale, &solids, &frame);

    Some(Picture {
        frame,
        sources,
        labels,
    })
}

/// The face in pixels, cut at the near plane, as a fan of triangles from its first corner.
fn fan(eye: &Pinhole, scale: f64, corners: &[Vec3], paint: Paint, out: &mut Vec<Triangle>) {
    let local: Vec<Vec3> = corners.iter().map(|c| eye.local(*c)).collect();
    let near = clip_near(&local);
    if near.len() < 3 {
        return;
    }

    let pixel: Vec<Corner> = near
        .iter()
        .map(|p| {
            let [x, y] = eye.image(*p);
            Corner {
                x: (x * scale) as f32,
                y: (y * scale) as f32,
                depth: p[2] as f32,
            }
        })
        .collect();
    for i in 1..pixel.len() - 1 {
        out.push(Triangle {
            corners: [pixel[0], pixel[i], pixel[i + 1]],
            paint,
        });
    }
}

/// How much light the face gets: its normal against `LIGHT`, either side lit alike.
fn shade(corners: &[Vec3]) -> u8 {
    // Newell's method: the normal of a flat polygon, whatever its winding.
    let mut n = [0.0; 3];
    for (i, a) in corners.iter().enumerate() {
        let b = corners[(i + 1) % corners.len()];
        n[0] += (a[1] - b[1]) * (a[2] + b[2]);
        n[1] += (a[2] - b[2]) * (a[0] + b[0]);
        n[2] += (a[0] - b[0]) * (a[1] + b[1]);
    }
    let length = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    let light = (LIGHT[0] * LIGHT[0] + LIGHT[1] * LIGHT[1] + LIGHT[2] * LIGHT[2]).sqrt();
    if length == 0.0 {
        return (AMBIENT * FULL_SHADE) as u8;
    }

    let facing = ((n[0] * LIGHT[0] + n[1] * LIGHT[1] + n[2] * LIGHT[2]) / length / light).abs();

    ((AMBIENT + (1.0 - AMBIENT) * facing) * FULL_SHADE).round() as u8
}

/// The names whose anchor lies in the picture with nothing nearer in front of it.
fn labels(eye: &Pinhole, scale: f64, solids: &[Solid], frame: &Frame) -> Vec<PictureLabel> {
    let mut labels = Vec::new();

    for (text, anchor) in solids.iter().filter_map(|s| s.label.as_ref()) {
        let local = eye.local(*anchor);
        if local[2] < NEAR_M {
            continue;
        }
        let [x, y] = eye.image(local);
        let (x, y) = (x * scale, y * scale);
        let Some(i) = (x >= 0.0 && y >= 0.0)
            .then(|| frame.index(x as usize, y as usize))
            .flatten()
        else {
            continue;
        };
        if frame.depth[i] < local[2] as f32 - COVER_M {
            continue;
        }

        labels.push(PictureLabel {
            text: text.clone(),
            x,
            y,
        });
    }

    labels
}
