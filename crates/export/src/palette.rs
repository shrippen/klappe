//! Kante's colours, parsed once from the embedded `palette.json`
//! (`scripts/sync-kante.sh` copies it from Kante unchanged).

use std::collections::HashMap;
use std::sync::OnceLock;

use serde_json::Value;

use crate::error::ExportError;

const PALETTE_JSON: &str = include_str!("../kante/palette.json");

/// Print uses `Light` (Kante's "Leinen"); `Dark` is the screen palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    /// `#rrggbb`, as SVG wants it.
    pub fn hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    /// 0..1 per channel, as PDF wants it.
    pub fn unit(self) -> [f64; 3] {
        [self.r, self.g, self.b].map(|c| f64::from(c) / f64::from(u8::MAX))
    }

    fn parse(text: &str) -> Option<Rgb> {
        let digits = text.strip_prefix('#').filter(|d| d.len() == 6)?;
        let channel = |i: usize| u8::from_str_radix(digits.get(i..i + 2)?, 16).ok();

        Some(Rgb {
            r: channel(0)?,
            g: channel(2)?,
            b: channel(4)?,
        })
    }
}

#[derive(Debug)]
pub(crate) struct Palette {
    /// The page.
    pub paper: Rgb,
    /// Walls, text, outlines.
    pub foreground: Rgb,
    /// Quiet outlines (set objects, frame).
    pub muted: Rgb,
    /// Set object fill.
    pub surface: Rgb,
    semantic: HashMap<String, Rgb>,
}

impl Palette {
    pub fn semantic(&self, name: &str) -> Result<Rgb, ExportError> {
        self.semantic
            .get(name)
            .copied()
            .ok_or_else(|| ExportError::Palette(format!("no semantic colour {name}")))
    }
}

#[derive(Debug)]
struct Palettes {
    light: Palette,
    dark: Palette,
}

static PALETTES: OnceLock<Result<Palettes, String>> = OnceLock::new();

pub(crate) fn palette(theme: Theme) -> Result<&'static Palette, ExportError> {
    let parsed = PALETTES.get_or_init(|| parse_all(PALETTE_JSON));
    let palettes = parsed
        .as_ref()
        .map_err(|e| ExportError::Palette(e.clone()))?;

    Ok(match theme {
        Theme::Light => &palettes.light,
        Theme::Dark => &palettes.dark,
    })
}

fn parse_all(json: &str) -> Result<Palettes, String> {
    let root: Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
    let light = root.get("light").ok_or("no light palette")?;

    Ok(Palettes {
        light: parse_one(light, ToneKind::Flat)?,
        dark: parse_one(&root, ToneKind::Bright)?,
    })
}

/// Light colours are plain strings; dark ones come as bright/neutral/faded.
#[derive(Clone, Copy)]
enum ToneKind {
    Flat,
    Bright,
}

fn parse_one(v: &Value, kind: ToneKind) -> Result<Palette, String> {
    let colour = |group: &str, key: &str| -> Result<Rgb, String> {
        v.get(group)
            .and_then(|g| g.get(key))
            .and_then(Value::as_str)
            .and_then(Rgb::parse)
            .ok_or_else(|| format!("bad colour {group}.{key}"))
    };

    let mut semantic = HashMap::new();
    let table = v
        .get("semantic")
        .and_then(Value::as_object)
        .ok_or("no semantic colours")?;
    for (name, tones) in table {
        let text = match kind {
            ToneKind::Flat => tones.as_str(),
            ToneKind::Bright => tones.get("bright").and_then(Value::as_str),
        };
        // Entries like "blue-hover" are not hues; skip what does not parse.
        if let Some(rgb) = text.and_then(Rgb::parse) {
            semantic.insert(name.clone(), rgb);
        }
    }

    Ok(Palette {
        paper: colour("backgrounds", "bg-panel")?,
        foreground: colour("foregrounds", "fg0")?,
        muted: colour("foregrounds", "fg3")?,
        surface: colour("backgrounds", "bg2")?,
        semantic,
    })
}
