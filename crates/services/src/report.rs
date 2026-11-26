//! The reports of DB-61 and the ways to write them.

/// One report: what a surface offers in its menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Report {
    /// Number, heading, length and characters of every scene.
    Scenes,
    /// Every scripted location with its shooting location and its scenes.
    Places,
    /// Which character is in which scene, as a cross table.
    Cast,
    /// The breakdown of every scene: a page per scene, or a row per element.
    Extract,
    /// Every person of the team with phone, mail and address.
    Contacts,
    /// Replies and dialogue share per character, scenes and length per place and time of day
    /// (DB-53).
    Statistics,
    /// The replies of one character in book order (DB-52); the character is chosen first.
    Dialogue,
}

impl Report {
    /// Every report, in the order the menu offers them.
    pub const ALL: [Report; 7] = [
        Report::Scenes,
        Report::Places,
        Report::Cast,
        Report::Extract,
        Report::Contacts,
        Report::Statistics,
        Report::Dialogue,
    ];

    /// How surfaces, the command line and the catalogue (`report.<name>`) call the report.
    pub fn name(self) -> &'static str {
        match self {
            Report::Scenes => "szenen",
            Report::Places => "spielorte",
            Report::Cast => "figuren",
            Report::Extract => "auszug",
            Report::Contacts => "kontakte",
            Report::Statistics => "statistik",
            Report::Dialogue => "dialog",
        }
    }

    /// Whether the report lists the scenes of the book and so has nothing to write without them.
    pub(crate) fn needs_scenes(self) -> bool {
        self != Report::Contacts
    }

    /// The report `name` names; `None` for any other text.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|r| r.name() == name)
    }
}

/// The file a report is written as.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportFormat {
    Pdf,
    Csv,
    Ods,
    Xlsx,
}

impl ReportFormat {
    /// Every format, in the order the menu offers them.
    pub const ALL: [ReportFormat; 4] = [
        ReportFormat::Pdf,
        ReportFormat::Csv,
        ReportFormat::Ods,
        ReportFormat::Xlsx,
    ];

    /// How surfaces call the format; also the extension of the file.
    pub fn name(self) -> &'static str {
        match self {
            ReportFormat::Pdf => "pdf",
            ReportFormat::Csv => "csv",
            ReportFormat::Ods => "ods",
            ReportFormat::Xlsx => "xlsx",
        }
    }

    /// The format a file's extension asks for, in any case (`list.CSV`); `None` for any other.
    pub fn from_extension(extension: &str) -> Option<Self> {
        let lower = extension.to_lowercase();

        Self::ALL.into_iter().find(|f| f.name() == lower)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_round_trip() {
        for report in Report::ALL {
            assert_eq!(Report::from_name(report.name()), Some(report));
        }
        assert_eq!(Report::from_name("scenes"), None);
    }

    #[test]
    fn formats_follow_the_extension() {
        for format in ReportFormat::ALL {
            assert_eq!(ReportFormat::from_extension(format.name()), Some(format));
        }
        assert_eq!(
            ReportFormat::from_extension("XLSX"),
            Some(ReportFormat::Xlsx)
        );
        assert_eq!(ReportFormat::from_extension("png"), None);
    }

    /// Menus and files carry the report's title: `report.<name>` in both languages.
    #[test]
    fn every_report_has_its_title() {
        for report in Report::ALL {
            let key = format!("report.{}", report.name());

            for language in [crate::Language::German, crate::Language::English] {
                assert_ne!(language.text(&key), key);
            }
        }
    }
}
