use std::collections::HashMap;

use klappe_model::{Angle, Movement, ShotSize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Paper {
    A4,
    A3,
    Letter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    Landscape,
    Portrait,
}

const DEFAULT_MARGIN_MM: f64 = 10.0;

/// The sheet around the plan.
#[derive(Debug, Clone, PartialEq)]
pub struct Page {
    pub paper: Paper,
    pub orientation: Orientation,
    pub margin_mm: f64,
    /// e.g. "Szene 12 · Sängersaal · Plan A".
    pub title: String,
    pub texts: Texts,
    /// The files of the images the display list names (`Shape::Image`), by the name in the
    /// list. The export reads no files: the caller hands the bytes in.
    pub images: HashMap<String, Vec<u8>>,
}

/// Every other visible text. The caller fills it from its translation catalogue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Texts {
    pub figure: String,
    pub camera: String,
    pub light: String,
    pub axis: String,
    pub path: String,
    pub mark: String,
    /// Metre unit, e.g. "m".
    pub unit: String,
    /// Between whole and fraction of a length: ',' or '.'.
    pub decimal_separator: char,
    pub shots: ShotTexts,
}

/// The shot list's words: heading, column heads and the names of sizes, angles and movements.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ShotTexts {
    /// e.g. "Shotliste"; follows the page title on the printed sheets.
    pub title: String,
    pub columns: ShotColumns,
    pub sizes: Vec<(ShotSize, String)>,
    pub angles: Vec<(Angle, String)>,
    pub movements: Vec<(Movement, String)>,
}

/// Column heads, in the order of the table.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ShotColumns {
    /// Head of the setup column in shooting order, and the word before a setup's number.
    pub setup: String,
    pub number: String,
    pub camera: String,
    pub lens_mm: String,
    pub size: String,
    pub angle: String,
    pub movement: String,
    pub from_s: String,
    pub to_s: String,
    pub duration_s: String,
    pub description: String,
    pub props: String,
    pub gear: String,
    pub crew: String,
    pub note: String,
}

impl Page {
    pub fn new(
        paper: Paper,
        orientation: Orientation,
        title: impl Into<String>,
        texts: Texts,
    ) -> Self {
        Self {
            paper,
            orientation,
            margin_mm: DEFAULT_MARGIN_MM,
            title: title.into(),
            texts,
            images: HashMap::new(),
        }
    }
}
