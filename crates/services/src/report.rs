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
}

impl Report {
    /// Every report, in the order the menu offers them.
    pub const ALL: [Report; 4] = [
        Report::Scenes,
        Report::Places,
        Report::Cast,
        Report::Extract,
    ];

    /// How surfaces, the command line and the catalogue (`report.<name>`) call the report.
    pub fn name(self) -> &'static str {
        match self {
            Report::Scenes => "szenen",
            Report::Places => "spielorte",
            Report::Cast => "figuren",
            Report::Extract => "auszug",
        }
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

    /// The file name offered for `report` of a project called `title`, e.g. `Film-szenen.pdf`.
    pub fn file_name(self, title: &str, report: Report) -> String {
        format!("{title}-{}.{}", report.name(), self.name())
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

    #[test]
    fn names_of_the_files_offered() {
        let names: Vec<String> = ReportFormat::ALL
            .iter()
            .map(|f| f.file_name("Film", Report::Cast))
            .collect();

        assert_eq!(
            names,
            [
                "Film-figuren.pdf",
                "Film-figuren.csv",
                "Film-figuren.ods",
                "Film-figuren.xlsx"
            ]
        );
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
