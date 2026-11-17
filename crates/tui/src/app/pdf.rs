//! The script as PDF (`Ctrl+E`): scene numbers, header and the (CONT'D) setting of the
//! project, then the file next to the screenplay (DB-70, DB-31).

use klappe_services::{HeaderLine, SceneNumbers, ScriptPrint};
use ratatui::crossterm::event::{KeyCode, KeyEvent};

use super::{App, Flow, Mode, Notice};

/// The PDF's name in the project folder.
const PDF_FILE: &str = "drehbuch.pdf";

/// What the rows of the form set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PdfRow {
    Numbers,
    Header,
    Contd,
}

pub(crate) const PDF_ROWS: [PdfRow; 3] = [PdfRow::Numbers, PdfRow::Header, PdfRow::Contd];

#[derive(Debug)]
pub(crate) struct PdfForm {
    pub sel: usize,
    pub numbers: SceneNumbers,
    pub header: HeaderLine,
    /// A project setting: written to the project when the PDF is.
    pub contd: bool,
}

impl PdfForm {
    pub(crate) fn is_on(&self, row: PdfRow) -> bool {
        match row {
            PdfRow::Numbers => self.numbers == SceneNumbers::Shown,
            PdfRow::Header => self.header == HeaderLine::Title,
            PdfRow::Contd => self.contd,
        }
    }

    fn toggle(&mut self) {
        match PDF_ROWS[self.sel] {
            PdfRow::Numbers => {
                self.numbers = match self.numbers {
                    SceneNumbers::Hidden => SceneNumbers::Shown,
                    SceneNumbers::Shown => SceneNumbers::Hidden,
                }
            }
            PdfRow::Header => {
                self.header = match self.header {
                    HeaderLine::Off => HeaderLine::Title,
                    HeaderLine::Title => HeaderLine::Off,
                }
            }
            PdfRow::Contd => self.contd = !self.contd,
        }
    }
}

impl App {
    pub(super) fn open_pdf(&mut self) -> Flow {
        // The PDF is made from the saved screenplay, as the project setting is saved with it.
        if self.is_modified() {
            self.notice = Some(Notice::Pdf(self.texts.pdf_save_first.clone()));
            return Flow::Continue;
        }

        self.pdf = Some(PdfForm {
            sel: 0,
            numbers: SceneNumbers::Shown,
            header: HeaderLine::Off,
            contd: self.session.project().script.contd_after_action,
        });
        self.mode = Mode::Pdf;
        self.suggest = None;

        Flow::Continue
    }

    pub(super) fn pdf_key(&mut self, key: KeyEvent) -> Flow {
        let Some(form) = self.pdf.as_mut() else {
            self.mode = Mode::Normal;
            return Flow::Continue;
        };
        let count = PDF_ROWS.len();

        match key.code {
            KeyCode::Esc => self.close_pdf(),
            KeyCode::Enter => self.write_pdf(),
            KeyCode::Tab | KeyCode::Down => form.sel = (form.sel + 1) % count,
            KeyCode::BackTab | KeyCode::Up => form.sel = (form.sel + count - 1) % count,
            KeyCode::Char(' ') | KeyCode::Left | KeyCode::Right => form.toggle(),
            _ => {}
        }

        Flow::Continue
    }

    fn close_pdf(&mut self) {
        self.pdf = None;
        self.mode = Mode::Normal;
    }

    /// Keeps a changed (CONT'D) setting in the project, then writes the PDF.
    fn write_pdf(&mut self) {
        let Some(form) = self.pdf.take() else {
            return;
        };
        self.mode = Mode::Normal;

        let changed = form.contd != self.session.project().script.contd_after_action;

        let path = self.session.dir().join(PDF_FILE);
        let print = ScriptPrint {
            scene_numbers: form.numbers,
            header: form.header,
            language: self.lang,
        };
        let done = match changed {
            true => self.session.keep_contd(form.contd),
            false => Ok(()),
        }
        .and_then(|()| self.session.export_script_pdf(&path, &print));

        self.notice = Some(Notice::Pdf(match done {
            Ok(()) => self.texts.pdf_done(PDF_FILE),
            Err(e) => e.to_string(),
        }));
    }
}
