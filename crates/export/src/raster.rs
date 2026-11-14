//! The page as pixels: what `svg` and `pdf` draw, filled in by this module itself.
//!
//! ```text
//! Doc ─► Surface (paper) ─► plan prims (clipped to the set) ─► chrome prims
//!          fills, strokes, text: polygons ─► scan (coverage) ─► blend
//!          pictures: quad ─► scan (edge coverage) ─► bilinear colour ─► blend
//! ```

use crate::canvas::{dash_array, Anchor, Doc, Geom, Prim, Pt, Rect, Stroke, Weight};
use crate::error::ExportError;
use crate::font;
use crate::image::Picture;
use crate::palette::Rgb;
use crate::rgb8::Rgb8;
use crate::scan::{scan, Bounds};
use crate::stroke::{arc_steps, outline, Closure};

/// Print resolution: one page point is 150/72 pixels.
pub(crate) const RESOLUTION_DPI: f64 = 150.0;
const PT_PER_INCH: f64 = 72.0;
const CHANNELS: usize = 3;
const HALF: f32 = 0.5;

pub(crate) struct Surface {
    pub width: u32,
    pub height: u32,
    /// RGB, row by row.
    pub pixels: Vec<u8>,
}

/// The plan page of `doc` (further sheets are not drawn, as in SVG).
pub(crate) fn render(doc: &Doc) -> Result<Surface, ExportError> {
    let scale = RESOLUTION_DPI / PT_PER_INCH;
    let width = (doc.width_pt * scale).round() as u32;
    let height = (doc.height_pt * scale).round() as u32;

    let mut pixels = Vec::with_capacity(width as usize * height as usize * CHANNELS);
    for _ in 0..width as usize * height as usize {
        pixels.extend_from_slice(&[doc.paper.r, doc.paper.g, doc.paper.b]);
    }

    let mut painter = Painter {
        surface: Surface {
            width,
            height,
            pixels,
        },
        scale,
        paper: doc.paper,
        clip: Bounds {
            left: 0.0,
            top: 0.0,
            right: f64::from(width),
            bottom: f64::from(height),
        },
    };

    // The plan is clipped to the set; the axis runs past it.
    let page = painter.clip;
    painter.clip = page.intersect(scaled(doc.clip, scale));
    for prim in &doc.plan {
        painter.draw(prim)?;
    }

    painter.clip = page;
    for prim in &doc.chrome {
        painter.draw(prim)?;
    }

    Ok(painter.surface)
}

fn scaled(r: Rect, scale: f64) -> Bounds {
    Bounds {
        left: r.x * scale,
        top: r.y * scale,
        right: (r.x + r.w) * scale,
        bottom: (r.y + r.h) * scale,
    }
}

struct Painter {
    surface: Surface,
    /// Pixels per page point.
    scale: f64,
    paper: Rgb,
    clip: Bounds,
}

impl Painter {
    fn draw(&mut self, prim: &Prim) -> Result<(), ExportError> {
        match prim {
            Prim::Shape {
                geom, fill, stroke, ..
            } => {
                self.shape(geom, fill.map(|f| (f.rgb, f.opacity)), *stroke);
            }
            Prim::Text {
                at,
                anchor,
                size_pt,
                weight,
                rgb,
                text,
                ..
            } => self.text(*at, *anchor, *size_pt, *weight, *rgb, text),
            Prim::Image {
                at,
                width_pt,
                height_pt,
                rotation_deg,
                opacity,
                picture,
                ..
            } => self.image(
                *at,
                (*width_pt, *height_pt),
                *rotation_deg,
                *opacity,
                picture,
            )?,
        }

        Ok(())
    }

    fn px(&self, p: Pt) -> Pt {
        (p.0 * self.scale, p.1 * self.scale)
    }

    fn shape(&mut self, geom: &Geom, fill: Option<(Rgb, f64)>, stroke: Option<Stroke>) {
        let (points, closure) = match geom {
            Geom::Open(p) => (p.iter().map(|p| self.px(*p)).collect(), Closure::Open),
            Geom::Closed(p) => (p.iter().map(|p| self.px(*p)).collect(), Closure::Closed),
            Geom::Rect(r) => (
                vec![
                    self.px((r.x, r.y)),
                    self.px((r.x + r.w, r.y)),
                    self.px((r.x + r.w, r.y + r.h)),
                    self.px((r.x, r.y + r.h)),
                ],
                Closure::Closed,
            ),
            Geom::Circle { center, radius } => (
                circle(self.px(*center), radius * self.scale),
                Closure::Closed,
            ),
        };

        // Fill, then line over it, as in SVG and PDF.
        if let Some((rgb, opacity)) = fill {
            self.fill(std::slice::from_ref(&points), rgb, opacity);
        }

        let Some(line) = stroke else {
            return;
        };
        let dash_px: Vec<f64> = dash_array(line.dash, line.width_pt)
            .iter()
            .map(|d| d * self.scale)
            .collect();
        let polys = outline(
            &points,
            closure,
            line.width_pt * self.scale,
            &dash_px,
            self.clip,
        );
        self.fill(&polys, line.rgb, 1.0);
    }

    fn text(&mut self, at: Pt, anchor: Anchor, size_pt: f64, weight: Weight, rgb: Rgb, text: &str) {
        let x = match anchor {
            Anchor::Start => at.0,
            Anchor::Middle => at.0 - font::text_width_pt(text, weight, size_pt) / 2.0,
        };

        let contours: Vec<Vec<Pt>> = font::text_outline(text, weight, size_pt)
            .iter()
            .map(|c| c.iter().map(|p| self.px((p.0 + x, p.1 + at.1))).collect())
            .collect();
        self.fill(&contours, rgb, 1.0);
    }

    fn fill(&mut self, polys: &[Vec<Pt>], rgb: Rgb, opacity: f64) {
        let alpha = opacity.clamp(0.0, 1.0) as f32;
        if alpha <= 0.0 {
            return;
        }

        let width = self.surface.width as usize;
        let pixels = &mut self.surface.pixels;
        let source = [rgb.r, rgb.g, rgb.b].map(f32::from);
        scan(polys, self.clip, &mut |row, col, coverage| {
            blend(
                pixels,
                (row * width + col) * CHANNELS,
                source,
                coverage * alpha,
            );
        });
    }

    /// The picture's pixels where its (turned) rectangle is, sampled bilinearly.
    fn image(
        &mut self,
        at: Pt,
        (width_pt, height_pt): (f64, f64),
        rotation_deg: f64,
        opacity: f64,
        picture: &Picture,
    ) -> Result<(), ExportError> {
        let alpha = opacity.clamp(0.0, 1.0) as f32;
        if alpha <= 0.0 || width_pt <= 0.0 || height_pt <= 0.0 {
            return Ok(());
        }

        let texture = picture.rgb(self.paper)?;
        let (sin, cos) = rotation_deg.to_radians().sin_cos();
        let across = (width_pt * cos, width_pt * sin);
        let down = (-height_pt * sin, height_pt * cos);
        let quad = vec![
            self.px(at),
            self.px((at.0 + across.0, at.1 + across.1)),
            self.px((at.0 + across.0 + down.0, at.1 + across.1 + down.1)),
            self.px((at.0 + down.0, at.1 + down.1)),
        ];

        let width = self.surface.width as usize;
        let scale = self.scale;
        let pixels = &mut self.surface.pixels;
        let (tex_w, tex_h) = (f64::from(texture.width), f64::from(texture.height));
        scan(&[quad], self.clip, &mut |row, col, coverage| {
            // The pixel's centre in page points, turned back into the picture's own axes.
            let dx = (col as f64 + 0.5) / scale - at.0;
            let dy = (row as f64 + 0.5) / scale - at.1;
            let u = (dx * cos + dy * sin) / width_pt * tex_w;
            let v = (-dx * sin + dy * cos) / height_pt * tex_h;

            let source = bilinear(&texture, u, v);
            blend(
                pixels,
                (row * width + col) * CHANNELS,
                source,
                coverage * alpha,
            );
        });

        Ok(())
    }
}

/// A circle as a polygon, starting at 3 o'clock and running clockwise on screen, as SVG's
/// `<circle>` does (it matters for dashes).
fn circle(center: Pt, radius_px: f64) -> Vec<Pt> {
    let steps = arc_steps(radius_px);

    (0..steps)
        .map(|k| {
            let angle = std::f64::consts::TAU * k as f64 / steps as f64;
            (
                center.0 + radius_px * angle.cos(),
                center.1 + radius_px * angle.sin(),
            )
        })
        .collect()
}

/// `source` over the pixel at `at`, `alpha` of it.
fn blend(pixels: &mut [u8], at: usize, source: [f32; CHANNELS], alpha: f32) {
    for (dst, src) in pixels[at..at + CHANNELS].iter_mut().zip(source) {
        let below = f32::from(*dst);
        *dst = (below + (src - below) * alpha + HALF) as u8;
    }
}

/// The colour at (`u`, `v`) in texels, where texel (0, 0) is centred on (0.5, 0.5); outside
/// the picture the edge texels continue.
fn bilinear(image: &Rgb8, u: f64, v: f64) -> [f32; CHANNELS] {
    let (w, h) = (image.width as usize, image.height as usize);
    let (u, v) = (u - 0.5, v - 0.5);
    let (fx, fy) = (u.floor(), v.floor());
    let (tx, ty) = ((u - fx) as f32, (v - fy) as f32);

    let texel = |x: f64, y: f64| {
        let x = x.clamp(0.0, (w - 1) as f64) as usize;
        let y = y.clamp(0.0, (h - 1) as f64) as usize;
        let at = (y * w + x) * CHANNELS;

        [0, 1, 2].map(|c| f32::from(image.pixels[at + c]))
    };
    let (a, b) = (texel(fx, fy), texel(fx + 1.0, fy));
    let (c, d) = (texel(fx, fy + 1.0), texel(fx + 1.0, fy + 1.0));

    [0, 1, 2].map(|i| {
        let top = a[i] + (b[i] - a[i]) * tx;
        let bottom = c[i] + (d[i] - c[i]) * tx;

        top + (bottom - top) * ty
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::canvas::{Dash, Fill};

    const PAPER: Rgb = Rgb {
        r: 250,
        g: 250,
        b: 245,
    };
    const INK: Rgb = Rgb {
        r: 10,
        g: 20,
        b: 30,
    };

    /// A 96 x 48 pt page; at 150 dpi that is 200 x 100 pixels (2.0833 px per point).
    fn doc(plan: Vec<Prim>) -> Doc {
        Doc {
            width_pt: 96.0,
            height_pt: 48.0,
            paper: PAPER,
            clip: Rect {
                x: 0.0,
                y: 0.0,
                w: 96.0,
                h: 48.0,
            },
            plan,
            chrome: Vec::new(),
            sheets: Vec::new(),
        }
    }

    fn shape(geom: Geom, fill: Option<Fill>, stroke: Option<Stroke>) -> Prim {
        Prim::Shape {
            class: "t".into(),
            geom,
            fill,
            stroke,
        }
    }

    fn at(s: &Surface, x: u32, y: u32) -> [u8; 3] {
        let i = (y * s.width + x) as usize * CHANNELS;

        [s.pixels[i], s.pixels[i + 1], s.pixels[i + 2]]
    }

    fn ink() -> [u8; 3] {
        [INK.r, INK.g, INK.b]
    }

    #[test]
    fn the_page_is_150_dpi_over_paper() {
        let s = render(&doc(Vec::new())).unwrap();

        // 96 pt = 1.3333 in = 200 px; 48 pt = 100 px.
        assert_eq!((s.width, s.height), (200, 100));
        assert_eq!(at(&s, 0, 0), [250, 250, 245]);
        assert_eq!(at(&s, 199, 99), [250, 250, 245]);
    }

    #[test]
    fn a_rect_fills_its_pixels_and_blends_its_edge() {
        // 10..20 pt = 20.83..41.67 px across, 0..24 pt = 0..50 px down.
        let rect = Rect {
            x: 10.0,
            y: 0.0,
            w: 10.0,
            h: 24.0,
        };
        let fill = Fill {
            rgb: INK,
            opacity: 1.0,
        };
        let s = render(&doc(vec![shape(Geom::Rect(rect), Some(fill), None)])).unwrap();

        assert_eq!(at(&s, 30, 25), ink());
        assert_eq!(at(&s, 5, 25), [250, 250, 245]);
        assert_eq!(at(&s, 45, 25), [250, 250, 245]);
        assert_eq!(at(&s, 30, 60), [250, 250, 245]);

        // Pixel 20 is 0.1667 covered: 250 + (10 - 250) * 0.1667 = 210.
        let edge = at(&s, 20, 25);
        assert!((i32::from(edge[0]) - 210).abs() <= 1, "{edge:?}");
    }

    #[test]
    fn opacity_mixes_with_what_is_below() {
        let rect = Rect {
            x: 0.0,
            y: 0.0,
            w: 48.0,
            h: 48.0,
        };
        let fill = Fill {
            rgb: Rgb { r: 0, g: 0, b: 0 },
            opacity: 0.5,
        };
        let s = render(&doc(vec![shape(Geom::Rect(rect), Some(fill), None)])).unwrap();

        assert_eq!(at(&s, 10, 10), [125, 125, 123]);
    }

    #[test]
    fn a_line_has_its_width_and_butt_ends() {
        // 2 pt wide = 4.17 px, from x 10 pt to 30 pt at y 12 pt (25 px).
        let line = Stroke {
            rgb: INK,
            width_pt: 2.0,
            dash: Dash::Solid,
        };
        let geom = Geom::Open(vec![(10.0, 12.0), (30.0, 12.0)]);
        let s = render(&doc(vec![shape(geom, None, Some(line))])).unwrap();

        assert_eq!(at(&s, 30, 25), ink());
        assert_eq!(at(&s, 30, 21), [250, 250, 245]);
        assert_eq!(at(&s, 30, 29), [250, 250, 245]);
        // Butt ends: nothing before 20.83 px or after 62.5 px.
        assert_eq!(at(&s, 19, 25), [250, 250, 245]);
        assert_eq!(at(&s, 64, 25), [250, 250, 245]);
    }

    #[test]
    fn a_dotted_line_has_gaps_where_the_pattern_says() {
        // 1.25 pt wide, dotted: 1.25 on, 3.125 off = 4.375 pt = 9.11 px period; the first
        // dot is 0..2.6 px, the gap 2.6..9.11 px.
        let line = Stroke {
            rgb: INK,
            width_pt: 1.25,
            dash: Dash::Dotted,
        };
        let geom = Geom::Open(vec![(0.0, 12.0), (40.0, 12.0)]);
        let s = render(&doc(vec![shape(geom, None, Some(line))])).unwrap();

        assert_eq!(at(&s, 1, 25), ink());
        assert_eq!(at(&s, 6, 25), [250, 250, 245]);
        assert_eq!(at(&s, 10, 25), ink());
    }

    #[test]
    fn a_corner_is_round() {
        let line = Stroke {
            rgb: INK,
            width_pt: 12.0,
            dash: Dash::Solid,
        };
        let geom = Geom::Open(vec![(10.0, 30.0), (30.0, 30.0), (30.0, 10.0)]);
        let s = render(&doc(vec![shape(geom, None, Some(line))])).unwrap();

        // The outer corner of a mitre would be at (36, 36) pt = (75, 75) px; round stays in.
        assert_eq!(at(&s, 74, 74), [250, 250, 245]);
        assert_eq!(at(&s, 62, 62), ink());
    }

    #[test]
    fn a_circle_is_where_its_centre_and_radius_say() {
        let fill = Fill {
            rgb: INK,
            opacity: 1.0,
        };
        let geom = Geom::Circle {
            center: (24.0, 24.0),
            radius: 12.0,
        };
        let s = render(&doc(vec![shape(geom, Some(fill), None)])).unwrap();

        // Centre (50, 50) px, radius 25 px.
        assert_eq!(at(&s, 50, 50), ink());
        assert_eq!(at(&s, 70, 50), ink());
        assert_eq!(at(&s, 77, 50), [250, 250, 245]);
        assert_eq!(at(&s, 50, 23), [250, 250, 245]);
    }

    #[test]
    fn the_plan_is_clipped_but_the_chrome_is_not() {
        let fill = Fill {
            rgb: INK,
            opacity: 1.0,
        };
        let wide = Rect {
            x: 0.0,
            y: 0.0,
            w: 96.0,
            h: 48.0,
        };
        let mut d = doc(vec![shape(Geom::Rect(wide), Some(fill), None)]);
        d.clip = Rect {
            x: 24.0,
            y: 12.0,
            w: 24.0,
            h: 12.0,
        };
        d.chrome = vec![shape(
            Geom::Rect(Rect {
                x: 72.0,
                y: 36.0,
                w: 12.0,
                h: 6.0,
            }),
            Some(fill),
            None,
        )];
        let s = render(&d).unwrap();

        assert_eq!(at(&s, 75, 37), ink());
        assert_eq!(at(&s, 40, 20), [250, 250, 245]);
        assert_eq!(at(&s, 90, 40), ink());
        assert_eq!(at(&s, 160, 80), ink());
        assert_eq!(at(&s, 190, 10), [250, 250, 245]);
    }

    #[test]
    fn bilinear_blends_neighbours_and_holds_the_edge() {
        let image = Rgb8 {
            width: 2,
            height: 1,
            pixels: vec![0, 0, 0, 100, 200, 250],
        };

        assert_eq!(bilinear(&image, 0.5, 0.5), [0.0, 0.0, 0.0]);
        assert_eq!(bilinear(&image, 1.0, 0.5), [50.0, 100.0, 125.0]);
        assert_eq!(bilinear(&image, 1.5, 0.5), [100.0, 200.0, 250.0]);
        assert_eq!(bilinear(&image, -3.0, 9.0), [0.0, 0.0, 0.0]);
        assert_eq!(bilinear(&image, 7.0, 0.5), [100.0, 200.0, 250.0]);
    }

    /// Red left, blue right: 20 x 10 pt at (10, 10), turned 90° clockwise about its corner,
    /// so its width runs down the page and it lies at x 0..10 pt, y 10..30 pt.
    fn picture(opacity: f64) -> Prim {
        let bytes = crate::png_write::encode(2, 1, &[255, 0, 0, 0, 0, 255], RESOLUTION_DPI);

        Prim::Image {
            class: "t".into(),
            at: (10.0, 10.0),
            width_pt: 20.0,
            height_pt: 10.0,
            rotation_deg: 90.0,
            opacity,
            picture: std::sync::Arc::new(Picture::parse("t.png", &bytes).unwrap()),
        }
    }

    #[test]
    fn a_picture_is_turned_about_its_corner() {
        let s = render(&doc(vec![picture(1.0)])).unwrap();

        // Pixels at 5 pt across; 15 pt and 25 pt down: the red half, then the blue.
        let [r, g, b] = at(&s, 10, 31);
        assert!(r > 240 && g < 15 && b < 15, "{r} {g} {b}");
        let [r, g, b] = at(&s, 10, 52);
        assert!(r < 15 && g < 15 && b > 240, "{r} {g} {b}");
        assert_eq!(at(&s, 30, 31), [250, 250, 245]);
        assert_eq!(at(&s, 10, 70), [250, 250, 245]);
        assert_eq!(at(&s, 10, 15), [250, 250, 245]);
    }

    #[test]
    fn a_picture_with_opacity_shows_the_paper_through() {
        let s = render(&doc(vec![picture(0.5)])).unwrap();
        let [r, g, b] = at(&s, 10, 31);

        assert!(r >= 250 && (g as i32 - 125).abs() <= 4 && (b as i32 - 122).abs() <= 4);
    }

    #[test]
    fn a_picture_that_cannot_be_decoded_is_an_error() {
        let jpeg = [
            0xFF, 0xD8, 0xFF, 0xC0, 0x00, 0x11, 0x08, 0x00, 0x14, 0x00, 0x1E, 0x03, 0, 0, 0, 0, 0,
            0, 0, 0, 0,
        ];
        let broken = Prim::Image {
            class: "t".into(),
            at: (0.0, 0.0),
            width_pt: 10.0,
            height_pt: 10.0,
            rotation_deg: 0.0,
            opacity: 1.0,
            picture: std::sync::Arc::new(Picture::parse("t.jpg", &jpeg).unwrap()),
        };

        assert!(matches!(
            render(&doc(vec![broken])),
            Err(ExportError::Image { .. })
        ));
    }

    /// Coordinates a hand-edited project can carry: far away, and past what a double holds.
    const FAR_PT: [f64; 7] = [1e6, 1e12, 1e300, 1e307, f64::MAX, f64::INFINITY, f64::NAN];
    const FAR_FINITE_PT: [f64; 3] = [1e6, 1e12, 1e300];
    const FAR_BUDGET: std::time::Duration = std::time::Duration::from_secs(5);

    /// Whether the coordinate is still a number once scaled to pixels.
    fn fits(far: f64) -> bool {
        (far * (RESOLUTION_DPI / PT_PER_INCH)).is_finite()
    }

    fn timed(prims: Vec<Prim>) -> Surface {
        let start = std::time::Instant::now();
        let s = render(&doc(prims)).unwrap();

        assert!(start.elapsed() < FAR_BUDGET, "took {:?}", start.elapsed());

        s
    }

    fn line(dash: Dash) -> Stroke {
        Stroke {
            rgb: INK,
            width_pt: 2.0,
            dash,
        }
    }

    #[test]
    fn a_dashed_path_far_outside_renders_fast_and_in_phase() {
        for far in FAR_PT {
            let geom = Geom::Open(vec![(10.0, 12.0), (far, 12.0)]);
            let s = timed(vec![shape(geom, None, Some(line(Dash::Dashed)))]);

            // 12 pt on from 10 pt = 20.83..45.83 px, then 6 pt (12.5 px) off.
            let drawn = fits(far);
            assert_eq!(at(&s, 30, 25) == ink(), drawn, "{far}");
            assert_eq!(at(&s, 50, 25), [250, 250, 245], "{far}");
        }
    }

    #[test]
    fn a_dashed_path_across_the_page_from_far_away_shows() {
        for far in FAR_FINITE_PT {
            let geom = Geom::Open(vec![(-far, 12.0), (far, 12.0)]);
            let s = timed(vec![shape(geom, None, Some(line(Dash::Dashed)))]);

            let inked = (0..s.width).filter(|x| at(&s, *x, 25) == ink()).count();
            assert!(inked > 0 && inked < s.width as usize, "{far}: {inked}");
        }
    }

    #[test]
    fn a_solid_path_far_outside_renders_fast() {
        for far in FAR_PT {
            let geom = Geom::Open(vec![(-far, 12.0), (far, 12.0)]);
            let s = timed(vec![shape(geom, None, Some(line(Dash::Solid)))]);

            let drawn = fits(far);
            assert_eq!(at(&s, 30, 25) == ink(), drawn, "{far}");
        }
    }

    #[test]
    fn a_huge_fill_covers_the_page_or_nothing() {
        let fill = Fill {
            rgb: INK,
            opacity: 1.0,
        };
        for far in FAR_PT {
            let rect = Rect {
                x: -far,
                y: -far,
                w: 2.0 * far,
                h: 2.0 * far,
            };
            let s = timed(vec![shape(Geom::Rect(rect), Some(fill), None)]);
            let drawn = fits(far);
            assert_eq!(at(&s, 30, 25) == ink(), drawn, "{far}");

            let ring = Geom::Closed(vec![(-far, -far), (far, -far), (far, far), (-far, far)]);
            timed(vec![shape(ring, Some(fill), Some(line(Dash::Dotted)))]);
        }
    }

    #[test]
    fn a_huge_circle_renders_fast() {
        let fill = Fill {
            rgb: INK,
            opacity: 1.0,
        };
        for far in FAR_PT {
            let geom = Geom::Circle {
                center: (48.0, 24.0),
                radius: far,
            };
            timed(vec![shape(geom, Some(fill), Some(line(Dash::Dashed)))]);
        }
    }
}
