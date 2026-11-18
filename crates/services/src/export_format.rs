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
    /// One PDF page per shot, its camera strong, the rest faded (LH-87).
    ShotPages,
    /// One PDF page per camera (LH-87).
    CameraPages,
    /// One PNG per shot in a folder (LH-87).
    ShotFiles,
    /// One PNG per camera in a folder (LH-87).
    CameraFiles,
    /// The playback as an MP4 video, written by the installed ffmpeg (LH-84).
    Video,
}

impl ExportFormat {
    /// Every format, in the order the menu offers them.
    pub const ALL: [ExportFormat; 14] = [
        ExportFormat::Pdf,
        ExportFormat::Png,
        ExportFormat::Svg,
        ExportFormat::Csv,
        ExportFormat::Ods,
        ExportFormat::Xlsx,
        ExportFormat::Frames,
        ExportFormat::View3d,
        ExportFormat::Camera,
        ExportFormat::ShotPages,
        ExportFormat::CameraPages,
        ExportFormat::ShotFiles,
        ExportFormat::CameraFiles,
        ExportFormat::Video,
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
            ExportFormat::ShotPages => "shotpages",
            ExportFormat::CameraPages => "camerapages",
            ExportFormat::ShotFiles => "shotfiles",
            ExportFormat::CameraFiles => "camerafiles",
            ExportFormat::Video => "mp4",
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
            ExportFormat::Pdf
                | ExportFormat::Png
                | ExportFormat::Svg
                | ExportFormat::Frames
                | ExportFormat::ShotPages
                | ExportFormat::CameraPages
                | ExportFormat::ShotFiles
                | ExportFormat::CameraFiles
        )
    }

    /// The extension of the file written; `None` where it is a folder.
    pub fn extension(self) -> Option<&'static str> {
        match self {
            ExportFormat::Frames | ExportFormat::ShotFiles | ExportFormat::CameraFiles => None,
            ExportFormat::ShotPages | ExportFormat::CameraPages => Some("pdf"),
            ExportFormat::View3d | ExportFormat::Camera => Some("png"),
            other => Some(other.name()),
        }
    }

    /// The file or folder name offered for a plan called `title`: the PNGs of a view sit
    /// beside the plan's under their own name (`<title>-3d.png`), a folder has no extension.
    pub fn file_name(self, title: &str) -> String {
        let part = match self {
            ExportFormat::View3d => "-3d",
            ExportFormat::Camera => "-camera",
            ExportFormat::ShotPages | ExportFormat::ShotFiles => "-shots",
            ExportFormat::CameraPages | ExportFormat::CameraFiles => "-cameras",
            _ => "",
        };

        match self.extension() {
            Some(extension) => format!("{title}{part}.{extension}"),
            None => format!("{title}{part}"),
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

    #[test]
    fn names_of_the_files_offered() {
        let names: Vec<String> = ExportFormat::ALL
            .iter()
            .map(|f| f.file_name("Plan-A"))
            .collect();

        assert_eq!(
            names,
            [
                "Plan-A.pdf",
                "Plan-A.png",
                "Plan-A.svg",
                "Plan-A.csv",
                "Plan-A.ods",
                "Plan-A.xlsx",
                "Plan-A",
                "Plan-A-3d.png",
                "Plan-A-camera.png",
                "Plan-A-shots.pdf",
                "Plan-A-cameras.pdf",
                "Plan-A-shots",
                "Plan-A-cameras",
                "Plan-A.mp4"
            ]
        );
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
