//! `klappe-tui [--language de|en] <folder>.klappe`

use std::path::PathBuf;
use std::process::ExitCode;

use klappe_tui::{run, App, Language};

const LANGUAGE_FLAG: &str = "--language";
const CODE_GERMAN: &str = "de";
const CODE_ENGLISH: &str = "en";

/// What the command line asked for.
struct Args {
    dir: PathBuf,
    lang: Language,
}

/// `de` or `en`; the catalogue itself takes anything and falls back to English.
fn language_of(code: &str) -> Option<Language> {
    match code.to_ascii_lowercase().as_str() {
        CODE_GERMAN => Some(Language::German),
        CODE_ENGLISH => Some(Language::English),
        _ => None,
    }
}

/// `Err` carries the message to print.
fn parse_args() -> Result<Args, String> {
    let mut lang = Language::pick(None);
    let mut dir = None;
    let mut args = std::env::args().skip(1);

    while let Some(arg) = args.next() {
        if arg != LANGUAGE_FLAG {
            dir = Some(PathBuf::from(arg));
            continue;
        }
        let Some(code) = args.next().and_then(|c| language_of(&c)) else {
            return Err(klappe_tui::unknown_language(lang));
        };
        lang = code;
    }

    match dir {
        Some(dir) => Ok(Args { dir, lang }),
        None => Err(klappe_tui::usage(lang)),
    }
}

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(args) => args,
        Err(message) => {
            eprintln!("klappe-tui: {message}");
            return ExitCode::FAILURE;
        }
    };

    let mut app = match App::open(&args.dir, args.lang) {
        Ok(app) => app,
        Err(e) => {
            eprintln!("klappe-tui: {}: {e}", klappe_tui::open_failed(args.lang));
            return ExitCode::FAILURE;
        }
    };

    match run(&mut app) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("klappe-tui: {e}");
            ExitCode::FAILURE
        }
    }
}
