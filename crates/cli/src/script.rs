//! The screenplay's outputs on the command line: the PDF, the Final Draft file and the sides.

use std::path::Path;
use std::process::ExitCode;

use klappe_services::{
    error_text, fdx_loss_text, HeaderLine, Language, Revisions, SceneNumbers, ScriptPaper,
    ScriptPrint, Session,
};

use crate::{refuse, warn_load_notes};

/// The flags of `script-pdf` that switch something on.
pub(crate) struct PrintFlags {
    pub(crate) scene_numbers: bool,
    pub(crate) header: bool,
    pub(crate) revisions: bool,
    pub(crate) changed_pages: bool,
}

/// What `script-pdf` prints, from its flags.
pub(crate) fn print_of(language: Language, flags: PrintFlags) -> ScriptPrint {
    ScriptPrint {
        scene_numbers: if flags.scene_numbers {
            SceneNumbers::Shown
        } else {
            SceneNumbers::Hidden
        },
        header: if flags.header {
            HeaderLine::Title
        } else {
            HeaderLine::Off
        },
        language,
        revisions: if flags.changed_pages {
            Revisions::ChangedPages
        } else if flags.revisions {
            Revisions::On
        } else {
            Revisions::Off
        },
    }
}
/// Writes the screenplay; `paper` replaces the project's for this one PDF.
pub(crate) fn script_pdf(
    project: &Path,
    out: &Path,
    paper: Option<ScriptPaper>,
    print: &ScriptPrint,
) -> ExitCode {
    let written = write_script_pdf(project, out, paper, print);

    match written {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("klappe-cli: {message}");
            ExitCode::FAILURE
        }
    }
}

fn write_script_pdf(
    project: &Path,
    out: &Path,
    paper: Option<ScriptPaper>,
    print: &ScriptPrint,
) -> Result<(), String> {
    let mut session = Session::open(project).map_err(|e| error_text(print.language, &e))?;
    warn_load_notes(&session, print.language);
    if let Some(paper) = paper {
        // Only in memory: the CLI saves nothing.
        let mut setup = session.project().script;
        setup.paper = paper;
        session.set_script_setup(setup);
    }

    session
        .export_script_pdf(out, print)
        .map_err(|e| error_text(print.language, &e))
}

/// Writes the sides of `figure`; the scene numbers are always printed.
pub(crate) fn sides_pdf(project: &Path, figure: &str, out: &Path, language: Language) -> ExitCode {
    let print = ScriptPrint {
        scene_numbers: SceneNumbers::Shown,
        header: HeaderLine::Off,
        language,
        revisions: Revisions::Off,
    };
    let written = Session::open(project)
        .map_err(|e| error_text(language, &e))
        .and_then(|mut session| {
            warn_load_notes(&session, language);

            session
                .export_sides_pdf(out, figure, &print)
                .map_err(|e| error_text(language, &e))
        });

    match written {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => refuse(message),
    }
}

pub(crate) fn script_fdx(project: &Path, out: &Path, language: Language) -> ExitCode {
    match write_script_fdx(project, out, language) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => refuse(message),
    }
}

/// Writes the screenplay as FDX and notes on stderr what the file lacks.
fn write_script_fdx(project: &Path, out: &Path, language: Language) -> Result<(), String> {
    let session = Session::open(project).map_err(|e| error_text(language, &e))?;
    warn_load_notes(&session, language);

    let loss = session
        .export_script_fdx(out)
        .map_err(|e| error_text(language, &e))?;
    let note = fdx_loss_text(language, &loss);
    if !note.is_empty() {
        eprintln!("{note}");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_screenplay_goes_to_a_pdf() {
        let project = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe");
        let out =
            std::env::temp_dir().join(format!("klappe-cli-script-{}.pdf", std::process::id()));
        let print = ScriptPrint {
            scene_numbers: SceneNumbers::Shown,
            header: HeaderLine::Off,
            language: Language::English,
            revisions: Revisions::Off,
        };

        let written = write_script_pdf(&project, &out, Some(ScriptPaper::Letter), &print);

        assert_eq!(written, Ok(()));
        let bytes = std::fs::read(&out).unwrap_or_default();
        let _ = std::fs::remove_file(&out);
        assert!(bytes.starts_with(b"%PDF"));
        assert!(String::from_utf8_lossy(&bytes).contains("/MediaBox [0 0 612 792]"));
    }

    #[test]
    fn the_screenplay_goes_to_an_fdx_file() {
        let project = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe");
        let out =
            std::env::temp_dir().join(format!("klappe-cli-script-{}.fdx", std::process::id()));

        let written = write_script_fdx(&project, &out, Language::English);

        assert_eq!(written, Ok(()));
        let text = std::fs::read_to_string(&out).unwrap_or_default();
        let _ = std::fs::remove_file(&out);
        assert!(text.contains("<FinalDraft DocumentType=\"Script\""));
        assert!(text.contains("Type=\"Scene Heading\""));
    }
}
