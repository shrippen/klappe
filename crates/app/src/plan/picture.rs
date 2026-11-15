//! The camera's picture in Kante's colours (LH-36): the roles of `scene_look` become the
//! colours the surface hands over, like the plan's roles become Kante names. No geometry
//! here: every pixel and label comes as the frame has it.

use std::collections::HashMap;

use klappe_services::{scene_look, Area, Picture, SceneRole};
use serde_json::{json, Value};

use super::hue_name;

type Rgb = [u8; 3];

/// A tinted role is this share of its colour over the ground: the same as the plan's
/// `KanteShapeCanvas` tint.
const TINT: f64 = 0.2;
const FULL_SHADE: f64 = 255.0;
const MAX_CHANNEL: &str = "255";
/// The Qt name of the format `ppm` writes.
pub const IMAGE_FORMAT: &str = "PPM";
const TEXT_HUE: &str = "text";
const GROUND_HUE: &str = "ground";
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
pub fn ppm(picture: &Picture, palette: &Palette) -> Vec<u8> {
    let frame = &picture.frame;
    let paints = paints(palette);
    let ground = palette.color(GROUND_HUE);

    let mut bytes = format!("P6\n{} {}\n{MAX_CHANNEL}\n", frame.width, frame.height).into_bytes();
    bytes.reserve(frame.role.len() * 3);

    for i in 0..frame.role.len() {
        let color = match paints[usize::from(frame.role[i])] {
            None => ground,
            Some(paint) if frame.edge[i] => paint.line,
            Some(paint) => shaded(paint.face, frame.shade[i]),
        };
        bytes.extend_from_slice(&color);
    }

    bytes
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
    use klappe_services::{FigureColor, PictureLabel};

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
        let bytes = ppm(&picture(), &palette());

        assert!(bytes.starts_with(b"P6\n4 1\n255\n"));
        assert_eq!(bytes.len(), b"P6\n4 1\n255\n".len() + 12);
    }

    #[test]
    fn qt_reads_it() {
        let image = cxx_qt_lib::QImage::from_data(&ppm(&picture(), &palette()), Some(IMAGE_FORMAT))
            .expect("image");

        assert_eq!((image.width(), image.height()), (4, 1));
    }

    #[test]
    fn empty_is_the_ground_and_the_floor_a_tint_of_muted() {
        let pixels = pixels(&ppm(&picture(), &palette()));

        assert_eq!(pixels[0], GROUND);
        // 20 % of muted (100) over the ground (250), fully lit.
        assert_eq!(pixels[1], [220, 220, 220]);
    }

    #[test]
    fn shade_multiplies_a_figure_and_an_edge_keeps_its_colour() {
        let pixels = pixels(&ppm(&picture(), &palette()));

        assert_eq!(pixels[2], [100, 0, 0]);
        assert_eq!(pixels[3], RED);
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
