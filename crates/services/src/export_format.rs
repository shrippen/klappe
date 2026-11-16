//! The ways to write an export, and what each needs from a surface.

/// One export: what a surface offers in its menu and asks for before it writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Pdf,
    Png,
    Svg,
    Csv,
    Ods,
    Xlsx,
    /// The plan at every mark as PNGs, in a folder.
    Frames,
    /// The 3D view as it is on screen, as PNG (LH-86).
    View3d,
    /// What a camera sees, as PNG (LH-86).
    Camera,
}

impl ExportFormat {
    /// Every format, in the order the menu offers them.
    pub const ALL: [ExportFormat; 9] = [
        ExportFormat::Pdf,
        ExportFormat::Png,
        ExportFormat::Svg,
        ExportFormat::Csv,
        ExportFormat::Ods,
        ExportFormat::Xlsx,
        ExportFormat::Frames,
        ExportFormat::View3d,
        ExportFormat::Camera,
    ];

    /// How surfaces and the catalogue (`menu.<name>Saved`) call the format.
    pub fn name(self) -> &'static str {
        match self {
            ExportFormat::Pdf => "pdf",
            ExportFormat::Png => "png",
            ExportFormat::Svg => "svg",
            ExportFormat::Csv => "csv",
            ExportFormat::Ods => "ods",
            ExportFormat::Xlsx => "xlsx",
            ExportFormat::Frames => "frames",
            ExportFormat::View3d => "view3d",
            ExportFormat::Camera => "camera",
        }
    }

    /// The format `name` names; `None` for any other text.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|f| f.name() == name)
    }

    /// Whether it prints the plan on a sheet. Such an export asks for the paper, and the
    /// sheet it was printed on is the project's afterwards.
    pub fn needs_paper(self) -> bool {
        matches!(
            self,
            ExportFormat::Pdf | ExportFormat::Png | ExportFormat::Svg | ExportFormat::Frames
        )
    }

    /// The extension of the file written; `None` where it is a folder.
    pub fn extension(self) -> Option<&'static str> {
        match self {
            ExportFormat::Frames => None,
            ExportFormat::View3d | ExportFormat::Camera => Some("png"),
            other => Some(other.name()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_round_trip() {
        for format in ExportFormat::ALL {
            assert_eq!(ExportFormat::from_name(format.name()), Some(format));
        }
        assert_eq!(ExportFormat::from_name("zip"), None);
    }

    /// The window writes the message `menu.<name>Saved` after each export.
    #[test]
    fn every_format_has_its_message() {
        for format in ExportFormat::ALL {
            let key = format!("menu.{}Saved", format.name());

            for language in [crate::Language::German, crate::Language::English] {
                assert_ne!(language.text(&key), key);
            }
        }
    }
}
