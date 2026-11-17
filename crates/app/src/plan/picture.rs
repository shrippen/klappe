//! The camera's picture in Kante's colours (LH-36): the roles of `scene_look` become the
//! colours the surface hands over, like the plan's roles become Kante names. No geometry
//! here: every pixel and label comes as the frame has it.

use std::collections::HashMap;

use klappe_services::{scene_look, Area, Frame, Picture, SceneRole, Source};
use serde_json::{json, Value};

use super::hue_name;

type Rgb = [u8; 3];

/// A tinted role is this share of its colour over the ground: the same as the plan's
/// `KanteShapeCanvas` tint.
const TINT: f64 = 0.2;
const FULL_SHADE: f64 = 255.0;
const FULL_STRENGTH: f64 = 1.0;
const MAX_CHANNEL: &str = "255";
/// The Qt name of the format `ppm` writes.
pub const IMAGE_FORMAT: &str = "PPM";
const TEXT_HUE: &str = "text";
const GROUND_HUE: &str = "ground";
const ACCENT_HUE: &str = "accent";
/// Every role code of a pixel fits a byte.
const CODES: usize = 256;

/// Kante's colours by name (`ground`, `text`, `muted`, `red` …), as the surface reads them
/// from the theme.
pub struct Palette(HashMap<String, Rgb>);

impl Palette {
    /// `{ name: [r, g, b], … }` with channels 0 to 255; `None` for anything else.
    pub fn parse(json: &str) -> Option<Self> {
        serde_json::from_str(json).ok().map(Self)
    }

    /// The colour of a name; an unknown one reads as the text colour, like the plan's canvas.
    fn color(&self, name: &str) -> Rgb {
        let text = self.0.get(TEXT_HUE).copied().unwrap_or_default();

        self.0.get(name).copied().unwrap_or(text)
    }
}

/// What a role's pixels are: the face (shade multiplies it) and the outline.
#[derive(Clone, Copy)]
struct Paint {
    face: Rgb,
    line: Rgb,
}

fn mix(from: Rgb, to: Rgb, share: f64) -> Rgb {
    let channel = |i: usize| {
        (f64::from(from[i]) + (f64::from(to[i]) - f64::from(from[i])) * share).round() as u8
    };

    [channel(0), channel(1), channel(2)]
}

fn shaded(color: Rgb, shade: u8) -> Rgb {
    let channel = |c: u8| (f64::from(c) * f64::from(shade) / FULL_SHADE).round() as u8;

    [channel(color[0]), channel(color[1]), channel(color[2])]
}

/// The paint of each role code; `None` for a code no role has (an empty pixel).
fn paints(palette: &Palette) -> Vec<Option<Paint>> {
    let ground = palette.color(GROUND_HUE);

    (0..CODES)
        .map(|code| {
            let look = scene_look(SceneRole::of_code(u8::try_from(code).ok()?)?);
            let hue = palette.color(hue_name(look.hue));
            let face = match look.area {
                Area::Tint => mix(ground, hue, TINT),
                Area::None | Area::Solid => hue,
            };

            Some(Paint { face, line: hue })
        })
        .collect()
}

/// The picture as a PPM image (the format `QImage` reads): the face of a pixel is its role's
/// colour times its shade, an outline is the role's colour as it is, empty is the ground.
/// `strength` says how much of its colour a role keeps (LH-15): a faded layer is mixed with the
/// ground. The outline of a source `selected` names is the accent (edge role "Auswahl") and
/// twice as wide, so the selection shows by form too, not only by colour.
pub fn ppm(
    picture: &Picture,
    palette: &Palette,
    strength: &dyn Fn(SceneRole) -> f64,
    selected: &dyn Fn(&Source) -> bool,
) -> Vec<u8> {
    let frame = &picture.frame;
    let paints = paints(palette);
    let ground = palette.color(GROUND_HUE);
    let accent = palette.color(ACCENT_HUE);
    let chosen: Vec<bool> = picture.sources.iter().map(selected).collect();
    let outlined = selection_outline(frame, &chosen);
    let strengths: Vec<f64> = (0..CODES)
        .map(|code| {
            u8::try_from(code)
                .ok()
                .and_then(SceneRole::of_code)
                .map_or(FULL_STRENGTH, strength)
        })
        .collect();

    let mut bytes = format!("P6\n{} {}\n{MAX_CHANNEL}\n", frame.width, frame.height).into_bytes();
    bytes.reserve(frame.role.len() * 3);

    for i in 0..frame.role.len() {
        let role = usize::from(frame.role[i]);
        let color = match paints[role] {
            None => ground,
            Some(_) if outlined[i] => accent,
            Some(paint) if frame.edge[i] => paint.line,
            Some(paint) => shaded(paint.face, frame.shade[i]),
        };
        bytes.extend_from_slice(&mix(ground, color, strengths[role]));
    }

    bytes
}

/// The pixels of the selection's outline: the edge of a chosen source and the pixel inside it
/// next to the edge (same source, above, below, left or right).
fn selection_outline(frame: &Frame, chosen: &[bool]) -> Vec<bool> {
    let (width, height) = (frame.width, frame.height);
    let is_chosen = |i: usize| chosen.get(frame.source[i] as usize) == Some(&true);
    let mut outlined = vec![false; frame.edge.len()];

    for i in (0..outlined.len()).filter(|&i| frame.edge[i] && is_chosen(i)) {
        let (x, y) = (i % width, i / width);
        let around = [
            Some(i),
            x.checked_sub(1).map(|_| i - 1),
            (x + 1 < width).then_some(i + 1),
            y.checked_sub(1).map(|_| i - width),
            (y + 1 < height).then_some(i + width),
        ];

        for j in around.into_iter().flatten() {
            outlined[j] |= frame.source[j] == frame.source[i];
        }
    }

    outlined
}

/// `{ width, height, labels: [{ text, x, y }] }`: the frame's size and the names to write, in
/// its pixels.
pub fn frame_json(picture: &Picture) -> String {
    let labels: Vec<Value> = picture
        .labels
        .iter()
        .map(|l| json!({ "text": l.text, "x": l.x, "y": l.y }))
        .collect();

    json!({
        "width": picture.frame.width,
        "height": picture.frame.height,
        "labels": labels,
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use klappe_services::{FigureColor, PictureLabel, Source};

    use super::*;

    const GROUND: Rgb = [250, 250, 250];
    const TEXT: Rgb = [10, 10, 10];
    const RED: Rgb = [200, 0, 0];

    fn palette() -> Palette {
        Palette::parse(
            r#"{"ground":[250,250,250],"text":[10,10,10],"muted":[100,100,100],"red":[200,0,0]}"#,
        )
        .expect("palette")
    }

    /// Four pixels: empty, floor, a lit red figure, an edge of that figure.
    fn picture() -> Picture {
        let frame = klappe_services::Frame {
            width: 4,
            height: 1,
            role: vec![
                0,
                SceneRole::Floor.code(),
                SceneRole::Figure(FigureColor::Red).code(),
                SceneRole::Figure(FigureColor::Red).code(),
            ],
            shade: vec![0, 255, 128, 128],
            source: vec![0; 4],
            face: vec![0; 4],
            depth: vec![0.0; 4],
            edge: vec![false, false, false, true],
        };

        Picture {
            frame,
            sources: Vec::new(),
            labels: vec![PictureLabel {
                text: "Elisabeth".into(),
                x: 2.0,
                y: 0.5,
            }],
        }
    }

    fn pixels(bytes: &[u8]) -> Vec<Rgb> {
        let body = &bytes[bytes.len() - 12..];

        body.chunks(3).map(|c| [c[0], c[1], c[2]]).collect()
    }

    #[test]
    fn it_is_a_ppm_of_the_frames_size() {
        let bytes = ppm(&picture(), &palette(), &|_| 1.0, &|_| false);

        assert!(bytes.starts_with(b"P6\n4 1\n255\n"));
        assert_eq!(bytes.len(), b"P6\n4 1\n255\n".len() + 12);
    }

    #[test]
    fn qt_reads_it() {
        let image = cxx_qt_lib::QImage::from_data(
            &ppm(&picture(), &palette(), &|_| 1.0, &|_| false),
            Some(IMAGE_FORMAT),
        )
        .expect("image");

        assert_eq!((image.width(), image.height()), (4, 1));
    }

    #[test]
    fn empty_is_the_ground_and_the_floor_a_tint_of_muted() {
        let pixels = pixels(&ppm(&picture(), &palette(), &|_| 1.0, &|_| false));

        assert_eq!(pixels[0], GROUND);
        // 20 % of muted (100) over the ground (250), fully lit.
        assert_eq!(pixels[1], [220, 220, 220]);
    }

    #[test]
    fn shade_multiplies_a_figure_and_an_edge_keeps_its_colour() {
        let pixels = pixels(&ppm(&picture(), &palette(), &|_| 1.0, &|_| false));

        assert_eq!(pixels[2], [100, 0, 0]);
        assert_eq!(pixels[3], RED);
    }

    /// LH-15: a faded layer's pixels are mixed with the ground, the others stay.
    #[test]
    fn a_faded_role_is_mixed_with_the_ground() {
        let figures_half = |role: SceneRole| match role {
            SceneRole::Figure(_) => 0.5,
            _ => 1.0,
        };

        let pixels = pixels(&ppm(&picture(), &palette(), &figures_half, &|_| false));

        assert_eq!(pixels[0], GROUND);
        assert_eq!(pixels[1], [220, 220, 220]);
        assert_eq!(pixels[2], [175, 125, 125]);
        assert_eq!(pixels[3], [225, 125, 125]);
    }

    /// LH-94: the outline of a selected source is the accent, and a pixel wider than the plain
    /// outline: the selection is not told by colour alone. A palette whose accent is the line
    /// colour of the figure (the dark theme's yellow) changes the picture all the same.
    #[test]
    fn a_selected_source_has_a_double_accent_outline() {
        let accent: Rgb = RED;
        let palette = Palette::parse(
            r#"{"ground":[250,250,250],"text":[10,10,10],"muted":[100,100,100],"red":[200,0,0],"accent":[200,0,0]}"#,
        )
        .expect("palette");
        let mut picture = picture();
        picture.sources = vec![Source::Figure("e".into())];

        let chosen = |source: &Source| *source == Source::Figure("e".into());
        let marked = pixels(&ppm(&picture, &palette, &|_| 1.0, &chosen));
        let plain = pixels(&ppm(&picture, &palette, &|_| 1.0, &|_| false));

        assert_ne!(marked, plain);
        assert_eq!(marked[3], accent);
        assert_eq!(marked[2], accent, "the pixel beside the outline");
        assert_eq!(marked[1], plain[1], "other sources stay");
        assert_eq!(marked[0], plain[0]);
    }

    #[test]
    fn an_unknown_name_reads_as_text() {
        let palette = Palette::parse(r#"{"text":[10,10,10]}"#).expect("palette");

        assert_eq!(palette.color("orange"), TEXT);
    }

    #[test]
    fn frame_json_has_the_size_and_the_labels() {
        let json: Value = serde_json::from_str(&frame_json(&picture())).expect("json");

        assert_eq!(json["width"], 4);
        assert_eq!(json["height"], 1);
        assert_eq!(json["labels"][0]["text"], "Elisabeth");
        assert_eq!(json["labels"][0]["x"], 2.0);
    }
}
