//! The reports of DB-61: scenes, scripted locations, characters by scene, and the extract of
//! the breakdown. The words of every cell are worked out here; `export/` sets them.
//!
//! ```text
//! project + book + sheets ─► ReportTable ─┬─► PDF / CSV / ODS / XLSX
//!                          └► ExtractPage ─► PDF (a page per scene)
//! ```

use std::path::Path;

use klappe_export::{ExtractBox, ExtractPage, ReportTable, Theme};
use klappe_model::{BookState, Character, Element, ElementCategory, Kind, Scene};

use super::export::write_file;
use super::Session;
use crate::error::ServiceError;
use crate::sheet_time::{show_planned, show_prestop};
use crate::{Language, Report, ReportFormat};

/// The cell of a character in a scene of the cross table.
const IN_SCENE: &str = "X";
const LIST_SEPARATOR: &str = ", ";
const NUMBER_SEPARATOR: &str = "  ";
/// Between a character's name and its look in the extract.
const LOOK_SEPARATOR: &str = " – ";

impl Session {
    /// Writes `report` to `path` as `format`; the words are `language`'s. The scenes' lengths
    /// are those of the screenplay's layout, made first if there is none for this text.
    pub fn export_report(
        &mut self,
        report: Report,
        format: ReportFormat,
        path: &Path,
        language: Language,
    ) -> Result<(), ServiceError> {
        self.paginate_if_changed(language);
        if self.book_scenes().next().is_none() {
            return Err(ServiceError::NoBookScenes);
        }
        let bytes = self.report_bytes(report, format, language)?;

        write_file(path, bytes)
    }

    fn report_bytes(
        &self,
        report: Report,
        format: ReportFormat,
        language: Language,
    ) -> Result<Vec<u8>, ServiceError> {
        if report == Report::Extract && format == ReportFormat::Pdf {
            let pages = self.extract_pages(language);

            return Ok(klappe_export::extract_pdf(&pages, Theme::Light)?);
        }

        let table = self.report_table(report, language);
        let bytes = match format {
            ReportFormat::Pdf => klappe_export::report_pdf(&table, Theme::Light)?,
            ReportFormat::Csv => klappe_export::report_csv(&table).into_bytes(),
            ReportFormat::Ods => klappe_export::report_ods(&table)?,
            ReportFormat::Xlsx => klappe_export::report_xlsx(&table)?,
        };

        Ok(bytes)
    }

    fn report_table(&self, report: Report, language: Language) -> ReportTable {
        let t = |k: &str| language.text(k);
        let title = t(&format!("report.{}", report.name()));
        let (head, rows) = match report {
            Report::Scenes => (
                vec![
                    t("field.number"),
                    t("report.heading"),
                    t("report.length"),
                    t("report.figures"),
                ],
                self.report_scene_rows(),
            ),
            Report::Places => (
                vec![t("report.place"), t("report.location"), t("report.scenes")],
                self.place_report_rows(),
            ),
            Report::Cast => self.cast_table(&t("report.scene")),
            Report::Extract => (
                vec![
                    t("report.scene"),
                    t("report.category"),
                    t("report.element"),
                    t("report.status"),
                ],
                self.element_rows_of_scenes(language),
            ),
        };

        ReportTable { title, head, rows }
    }

    /// The scenes of the book with their index in `project.scenes`: the reports leave out
    /// what is struck from the book and what is no scene.
    fn book_scenes(&self) -> impl Iterator<Item = (usize, &Scene)> {
        let scenes = self.project.scenes.iter().enumerate();

        scenes.filter(|(_, s)| s.kind == Kind::Scene && s.state == BookState::InBook)
    }

    /// The characters of the scene at `entry`: those with a cue in the book, then those the
    /// sheet names that have none.
    pub(super) fn scene_figures(&self, entry: usize, scene: &Scene) -> Vec<&Character> {
        let speaking = self.speaking(entry);
        let silent = scene
            .sheet
            .characters
            .iter()
            .filter(|id| !speaking.contains(id));

        speaking
            .iter()
            .chain(silent)
            .filter_map(|id| self.project.character(id))
            .collect()
    }

    /// Number, heading, length (pages in eighths, from the layout) and characters.
    fn report_scene_rows(&self) -> Vec<Vec<String>> {
        self.book_scenes()
            .map(|(i, scene)| {
                let names: Vec<&str> = self
                    .scene_figures(i, scene)
                    .iter()
                    .map(|c| c.name.as_str())
                    .collect();

                vec![
                    scene.number.clone(),
                    scene.heading.clone(),
                    self.scene_length(&scene.id),
                    names.join(LIST_SEPARATOR),
                ]
            })
            .collect()
    }

    fn place_report_rows(&self) -> Vec<Vec<String>> {
        let (mut places, _) = self.shot_scenes();
        let in_book: Vec<usize> = self.book_scenes().map(|(i, _)| i).collect();
        for (i, place) in places.iter_mut().enumerate() {
            if !in_book.contains(&i) {
                *place = None;
            }
        }

        self.place_rows(&places)
            .into_iter()
            .map(|p| vec![p.name, p.location_name, p.scenes.join(LIST_SEPARATOR)])
            .collect()
    }

    /// A row per scene, a column per character that is in any scene, `X` where it is in it,
    /// or the name of its look there.
    fn cast_table(&self, scene_head: &str) -> (Vec<String>, Vec<Vec<String>>) {
        let in_scenes: Vec<Vec<&str>> = self
            .book_scenes()
            .map(|(i, s)| {
                let figures = self.scene_figures(i, s);

                figures.iter().map(|c| c.id.as_str()).collect()
            })
            .collect();
        let cast: Vec<&Character> = self
            .project
            .characters
            .iter()
            .filter(|c| in_scenes.iter().any(|ids| ids.contains(&c.id.as_str())))
            .collect();

        let mut head = vec![scene_head.to_owned()];
        head.extend(cast.iter().map(|c| c.name.clone()));
        let rows = self
            .book_scenes()
            .zip(&in_scenes)
            .map(|((_, scene), ids)| {
                let mut row = vec![scene.number.clone()];
                for c in &cast {
                    if !ids.contains(&c.id.as_str()) {
                        row.push(String::new());
                        continue;
                    }

                    let look = self.scene_look(scene, &c.id);
                    row.push(look.map_or(IN_SCENE, |l| l.name.as_str()).to_owned());
                }

                row
            })
            .collect();

        (head, rows)
    }

    /// Scene, category, element and status: a row per element of a scene.
    fn element_rows_of_scenes(&self, language: Language) -> Vec<Vec<String>> {
        let mut rows = Vec::new();

        for (i, scene) in self.book_scenes() {
            for (category, elements) in self.extract_categories(i, scene) {
                for element in elements {
                    rows.push(vec![
                        scene.number.clone(),
                        language.text(&format!("elementCategory.{}", category.name())),
                        element.name.clone(),
                        language.text(&format!("elementStatus.{}", element.status.name())),
                    ]);
                }
            }
        }

        rows
    }

    /// The elements of the scene's sheet and those of the looks its characters wear, by
    /// category; an element in both is there once.
    fn extract_categories(
        &self,
        entry: usize,
        scene: &Scene,
    ) -> Vec<(ElementCategory, Vec<&Element>)> {
        let mut ids = scene.sheet.elements.clone();

        for (_, look) in self.worn_looks(entry, scene) {
            for id in &look.elements {
                if !ids.contains(id) {
                    ids.push(id.clone());
                }
            }
        }

        self.group_by_category(&ids)
    }

    /// A page per scene: head, the sheet's facts, a box per category that has elements and one
    /// for the characters.
    fn extract_pages(&self, language: Language) -> Vec<ExtractPage> {
        self.book_scenes()
            .map(|(i, scene)| {
                let sheet = &scene.sheet;
                let facts = [
                    ("report.storyDay", sheet.story_day.clone()),
                    ("report.prestop", show_prestop(sheet.prestop_s)),
                    ("report.planned", show_planned(sheet.planned_min)),
                ];
                let mut boxes: Vec<ExtractBox> = self
                    .extract_categories(i, scene)
                    .into_iter()
                    .map(|(category, elements)| ExtractBox {
                        title: language.text(&format!("elementCategory.{}", category.name())),
                        lines: elements
                            .iter()
                            .map(|e| {
                                let status = format!("elementStatus.{}", e.status.name());

                                format!("{} ({})", e.name, language.text(&status))
                            })
                            .collect(),
                    })
                    .collect();
                let figures = self.scene_figures(i, scene);
                if !figures.is_empty() {
                    boxes.push(ExtractBox {
                        title: language.text("report.figures"),
                        lines: figures
                            .iter()
                            .map(|c| match self.scene_look(scene, &c.id) {
                                Some(look) => format!("{}{LOOK_SEPARATOR}{}", c.name, look.name),
                                None => c.name.clone(),
                            })
                            .collect(),
                    });
                }

                ExtractPage {
                    title: format!("{}{NUMBER_SEPARATOR}{}", scene.number, scene.heading)
                        .trim()
                        .to_owned(),
                    facts: facts
                        .iter()
                        .filter(|(_, value)| !value.is_empty())
                        .map(|(key, value)| format!("{}: {value}", language.text(key)))
                        .collect(),
                    boxes,
                }
            })
            .collect()
    }
}
