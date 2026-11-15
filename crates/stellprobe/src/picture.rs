//! The picture of a view as pixels (3D-KONZEPT.md, section 5): the scene's faces cut at the
//! near plane, projected, split into triangles and filled by `klappe_raster`. Covering is
//! right per pixel; colours stay roles.
//!
//! ```text
//!   scene ──► faces in the eye's frame ──► cut at NEAR_M ──► pixels ──► triangles
//!                                                                         │ raster
//!   labels: anchor projected, kept where nothing nearer covers it    ◄── Frame
//! ```

use klappe_model::{FloorPlan, Point, Project, SetPlan};
use klappe_raster::{rasterize, Corner, Frame, Paint, Triangle, NO_SOURCE};

use crate::display::Source;
use crate::eye::{clip_near, Eye, Pinhole, Vec3, NEAR_M};
use crate::forms::Forms;
use crate::scene::{scene, Solid};
use crate::triangulate::triangulate;

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
    forms: &Forms,
    eye: &Eye,
    width_px: usize,
) -> Option<Picture> {
    let pinhole = Pinhole::of(eye, plan)?;
    let looking = match eye {
        Eye::Shot { camera, .. } => Some(camera.as_str()),
        Eye::Orbit { .. } => None,
    };
    let scale = width_px as f64;
    let height_px = (pinhole.frame_height * scale).round() as usize;
    if width_px == 0 || height_px == 0 {
        return None;
    }

    let solids = scene(project, set, plan, forms, eye.t_s(), looking);
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
        for f in &solid.faces {
            let paint = Paint {
                role: solid.role.code(),
                shade: shade(&f.corners),
                source,
                face: face + f.surface as u32,
                sparse: solid.role == crate::scene::SceneRole::Missing,
            };
            split(&pinhole, scale, &f.corners, paint, &mut triangles);
        }
        face += solid
            .faces
            .iter()
            .map(|f| f.surface as u32 + 1)
            .max()
            .unwrap_or(0);
    }

    let frame = rasterize(width_px, height_px, &triangles);
    let labels = labels(&pinhole, scale, &solids, &frame);

    Some(Picture {
        frame,
        sources,
        labels,
    })
}

/// The face in pixels, cut at the near plane, as triangles (a fan if convex, else ears).
fn split(eye: &Pinhole, scale: f64, corners: &[Vec3], paint: Paint, out: &mut Vec<Triangle>) {
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
    let spots: Vec<[f64; 2]> = pixel
        .iter()
        .map(|c| [f64::from(c.x), f64::from(c.y)])
        .collect();
    for [a, b, c] in triangulate(&spots) {
        out.push(Triangle {
            corners: [pixel[a], pixel[b], pixel[c]],
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

/// Where pixel (`x_px`, `y_px`) of a picture of `eye` `width_px` wide shows the floor, in plan
/// metres (LH-94: the tools of the plan work in 3D on this point); `None` above the horizon
/// or when the eye sees nothing.
pub fn floor_at(
    plan: &FloorPlan,
    eye: &Eye,
    width_px: f64,
    (x_px, y_px): (f64, f64),
) -> Option<Point> {
    let pinhole = Pinhole::of(eye, plan)?;

    pinhole.floor_at([x_px / width_px, y_px / width_px])
}

/// Where pixel (`x_px`, `y_px`) of a picture of `eye` `width_px` wide meets the level
/// `height_m` above the floor, in plan metres: a drag of something grabbed at that height
/// stays on its level, also where no floor lies under the pointer.
pub fn level_at(
    plan: &FloorPlan,
    eye: &Eye,
    width_px: f64,
    (x_px, y_px): (f64, f64),
    height_m: f64,
) -> Option<Point> {
    let pinhole = Pinhole::of(eye, plan)?;

    pinhole.level_at([x_px / width_px, y_px / width_px], height_m)
}

/// How much higher than `held_m` the pointer `to_px` has pulled what was grabbed at `from_px`
/// (both in a picture of `eye` `width_px` wide), in metres: the height drag in 3D. What was
/// grabbed lies at `held_m` above the floor; the pointer slides on the vertical plane through
/// it, across the view. `None` when `from_px` shows nothing at that height.
pub fn lift_at(
    plan: &FloorPlan,
    eye: &Eye,
    width_px: f64,
    from_px: (f64, f64),
    to_px: (f64, f64),
    held_m: f64,
) -> Option<f64> {
    let pinhole = Pinhole::of(eye, plan)?;
    let image = |(x_px, y_px): (f64, f64)| [x_px / width_px, y_px / width_px];

    pinhole.lift(image(from_px), image(to_px), held_m)
}

/// What pixel (`x`, `y`) of `picture` (made of `eye`) shows and how high above the floor that
/// point of it lies: where a drag grabs it. `None` for the floor, a gap or outside.
pub fn grab_at(
    plan: &FloorPlan,
    eye: &Eye,
    picture: &Picture,
    (x, y): (usize, usize),
) -> Option<(Source, f64)> {
    let source = source_at(picture, (x, y))?.clone();
    let i = picture.frame.index(x, y)?;
    let width = picture.frame.width as f64;
    let image = [(x as f64 + 0.5) / width, (y as f64 + 0.5) / width];
    let point = Pinhole::of(eye, plan)?.point_at(image, f64::from(picture.frame.depth[i]));

    Some((source, point[2]))
}

/// What pixel (`x`, `y`) of `picture` shows (LH-94: a click in 3D takes what one sees there);
/// `None` for the floor, a gap or outside the picture.
pub fn source_at(picture: &Picture, (x, y): (usize, usize)) -> Option<&Source> {
    let i = picture.frame.index(x, y)?;

    picture.sources.get(picture.frame.source[i] as usize)
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
        if x < 0.0 || y < 0.0 {
            continue;
        }
        // Two pixels side by side: a see-through body fills only one of them, so a name
        // behind it shows whichever pixel of the checkerboard its anchor hits.
        let (px, py) = (x as usize, y as usize);
        let Some(i) = frame.index(px, py) else {
            continue;
        };
        let beside = frame
            .index(px + 1, py)
            .or(px.checked_sub(1).and_then(|l| frame.index(l, py)));
        let nearest = beside.map_or(frame.depth[i], |j| frame.depth[i].max(frame.depth[j]));
        if nearest < local[2] as f32 - COVER_M {
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
