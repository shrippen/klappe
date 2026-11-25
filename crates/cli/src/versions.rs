//! The locked versions of the screenplay on the command line (DB-40 to DB-42): lock one, list
//! them, compare two states of the book.

use std::path::Path;
use std::process::ExitCode;

use klappe_services::{
    color_text, error_text, Comparison, Draft, Edit, Language, OnOutside, Saved, SceneDiff,
    ScenePlace, SceneStatus, Session,
};

use crate::refuse;

/// Locks the book as the version `name`; saves the project as Ctrl+S does.
pub(crate) fn lock(project: &Path, name: &str, language: Language) -> ExitCode {
    let locked = Session::open(project)
        .and_then(|mut s| {
            let saved = s.lock_version(name, language, OnOutside::Ask)?;

            Ok((saved, s.versions().last().cloned()))
        })
        .map_err(|e| error_text(language, &e));

    match locked {
        Ok((Saved::Done, Some(version))) => {
            let color = color_text(language, version.color);
            let text = language
                .text("cli.versionLocked")
                .replace("{name}", &version.name)
                .replace("{color}", &color)
                .replace("{date}", &version.date);
            println!("{text}");

            ExitCode::SUCCESS
        }
        Ok(_) => refuse(language.text("cli.lockConflict")),
        Err(message) => refuse(message),
    }
}

/// One line per version in the order of locking: id, name, date, colour.
pub(crate) fn list(project: &Path, language: Language) -> ExitCode {
    let session = match Session::open(project) {
        Ok(session) => session,
        Err(e) => return refuse(error_text(language, &e)),
    };

    if session.versions().is_empty() {
        println!("{}", language.text("versions.none"));
    }
    for v in session.versions() {
        println!(
            "{}\t{}\t{}\t{}",
            v.id,
            v.name,
            v.date,
            color_text(language, v.color)
        );
    }

    ExitCode::SUCCESS
}

/// Compares `from` (the last version if left out) with `to` (the book now if left out).
pub(crate) fn compare(
    project: &Path,
    from: Option<String>,
    to: Option<String>,
    language: Language,
) -> ExitCode {
    let compared = Session::open(project)
        .and_then(|s| match (from, to) {
            (None, None) => s.revision(),
            (from, to) => {
                let last = s.versions().last().map(|v| Draft::Version(v.id.clone()));
                let from = from.map(Draft::Version).or(last);
                let to = to.map_or(Draft::Current, Draft::Version);

                s.compare_drafts(&from.ok_or(klappe_services::ServiceError::NoVersions)?, &to)
            }
        })
        .map_err(|e| error_text(language, &e));

    match compared {
        Ok(comparison) => {
            print!("{}", comparison_text(&comparison, language));

            ExitCode::SUCCESS
        }
        Err(message) => refuse(message),
    }
}

/// Per scene its number, heading and what became of it; under a changed scene its lines, `-`
/// for the old text, `+` for the new.
fn comparison_text(comparison: &Comparison, language: Language) -> String {
    let mut out = String::new();

    for scene in &comparison.scenes {
        out.push_str(&scene_line(scene, language));
        out.push('\n');
        if scene.status != SceneStatus::Changed {
            continue;
        }

        for line in &scene.lines {
            let sign = match line.change {
                Edit::Added => '+',
                Edit::Removed => '-',
            };
            out.push_str(&format!("  {sign} {}\n", line.text));
        }
    }

    out
}

/// `12<TAB>INT. SAAL - TAG<TAB>geändert, verschoben`.
fn scene_line(scene: &SceneDiff, language: Language) -> String {
    let status = match scene.status {
        SceneStatus::New => "compare.new",
        SceneStatus::Removed => "compare.removed",
        SceneStatus::Changed => "compare.changed",
        SceneStatus::Same => "compare.same",
    };
    let mut words = vec![language.text(status)];
    if scene.place == ScenePlace::Moved {
        words.push(language.text("compare.moved"));
    }

    format!(
        "{}\t{}\t{}",
        scene.number.as_deref().unwrap_or("-"),
        scene.heading.as_deref().unwrap_or(""),
        words.join(", ")
    )
}
