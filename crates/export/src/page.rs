use std::collections::HashMap;

use klappe_model::{Angle, FigureColor, LightKind, Movement, Orientation, Paper, ShotSize};
use klappe_stellprobe::VectorView;

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
    /// How strong each item of the display list is drawn, 1 full and 0 not at all (LH-15,
    /// LH-87), by the item's place in the list; an item beyond the end, or none given, is full.
    pub fade: Vec<f64>,
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
    /// The legend's words for the signs of an appearing and a leaving mark (LH-26).
    pub arrival: String,
    pub departure: String,
    /// The legend's words for drawings (LH-19).
    pub drawing: DrawingTexts,
    /// Metre unit, e.g. "m".
    pub unit: String,
    /// Between whole and fraction of a length: ',' or '.'.
    pub decimal_separator: char,
    pub shots: ShotTexts,
    pub lamps: LampTexts,
}

/// The words of the lamp list under the legend (LH-51). Without a `line` the list is left out.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LampTexts {
    /// One lamp: `{label}`, `{kind}`, `{power}` (W), `{temp}` (K) and `{beam}` (degrees) are
    /// replaced, e.g. "{label}: {kind}, {power} W, {temp} K, {beam}°".
    pub line: String,
    /// A lamp that makes no light of its own (bounce, flag): `{label}`, `{kind}`, `{beam}`.
    pub passive: String,
    /// Closes a list that is cut short: `{n}` lamps are not shown, e.g. "+ {n} weitere".
    /// Without it the list is cut without a note.
    pub more: String,
    pub kinds: Vec<(LightKind, String)>,
    /// Name of the lamp sheet in ODS and XLSX, e.g. "Lampen".
    pub title: String,
    pub columns: LampColumns,
}

/// Column heads of the lamp sheet, in its order.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LampColumns {
    pub label: String,
    pub kind: String,
    pub power_w: String,
    pub color_temp_k: String,
    pub beam_deg: String,
}

impl LampTexts {
    /// What the lamp list calls `kind`; the built-in name without a word for it.
    pub(crate) fn kind_name(&self, kind: LightKind) -> &str {
        self.kinds
            .iter()
            .find(|(k, _)| *k == kind)
            .map_or(kind.name(), |(_, name)| name.as_str())
    }
}

/// What the legend calls a drawing: lines by their dash, a filled shape an area.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DrawingTexts {
    pub line: String,
    pub dashed: String,
    pub dotted: String,
    pub area: String,
    /// The colour names, appended to the word: "Zeichnung rot".
    pub colors: Vec<(FigureColor, String)>,
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
    /// e.g. "Kamerablicke"; follows the page title on the pages of the camera views.
    pub views: String,
}

/// A shot's camera view at the shot's start (`klappe_stellprobe::vector_view`), for the pages
/// of camera views in the PDF.
#[derive(Debug, Clone, PartialEq)]
pub struct ShotView {
    /// The id of the shot.
    pub shot: String,
    pub view: VectorView,
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
    /// Head of the column naming the shots the axis is crossed against (LH-41).
    pub axis: String,
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
            fade: Vec::new(),
        }
    }
}
