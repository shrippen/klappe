//! The locked versions of the screenplay on the command line (DB-40 to DB-42): lock one, list
//! them, compare two states of the book.

use std::path::Path;
use std::process::ExitCode;

use klappe_services::{
    color_text, error_text, Comparison, Draft, Edit, Language, OnOutside, Saved, SceneDiff,
    ScenePlace, SceneStatus, Session,
};

use clap::Subcommand;

use crate::{language_of, refuse, LanguageArg};
use std::path::PathBuf;

/// The commands on locked versions.
#[derive(Debug, Subcommand)]
pub(crate) enum VersionCommand {
    /// Locks the book as a version (DB-40): saves the project, locks the scene numbers and
    /// keeps the text for comparing.
    LockVersion {
        /// The project folder.
        #[arg(long)]
        project: PathBuf,
        /// The version's name, e.g. "Drehfassung"; a name no other version has.
        #[arg(long)]
        name: String,
        /// Language of the pages' words and of the output; the system's if left out.
        #[arg(long, value_enum)]
        language: Option<LanguageArg>,
    },
    /// Deletes a locked version with its text (DB-40); the others keep their colours.
    RemoveVersion {
        /// The project folder.
        #[arg(long)]
        project: PathBuf,
        /// The version's id, as `versions` lists it.
        #[arg(long)]
        id: String,
        /// Language of the output; the system's if left out.
        #[arg(long, value_enum)]
        language: Option<LanguageArg>,
    },
    /// The locked versions in the order of locking: id, name, date, colour.
    Versions {
        /// The project folder.
        #[arg(long)]
        project: PathBuf,
        /// Language of the colour names; the system's if left out.
        #[arg(long, value_enum)]
        language: Option<LanguageArg>,
    },
    /// What changed between two states of the book (DB-41, DB-42): per scene its number,
    /// heading and status (new, removed, changed, unchanged, moved), under a changed scene its
    /// lines with `-` and `+`. Without options the last version against the book now.
    Compare {
        /// The project folder.
        #[arg(long)]
        project: PathBuf,
        /// A version's id; the last version if left out.
        #[arg(long)]
        from: Option<String>,
        /// A version's id; the book now if left out.
        #[arg(long)]
        to: Option<String>,
        /// Language of the output; the system's if left out.
        #[arg(long, value_enum)]
        language: Option<LanguageArg>,
    },
}

impl VersionCommand {
    pub(crate) fn run(self) -> ExitCode {
        match self {
            VersionCommand::LockVersion {
                project,
                name,
                language,
            } => lock(&project, &name, language_of(language)),
            VersionCommand::RemoveVersion {
                project,
                id,
                language,
            } => remove(&project, &id, language_of(language)),
            VersionCommand::Versions { project, language } => list(&project, language_of(language)),
            VersionCommand::Compare {
                project,
                from,
                to,
                language,
            } => compare(&project, from, to, language_of(language)),
        }
    }
}

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

/// Deletes the version `id`.
pub(crate) fn remove(project: &Path, id: &str, language: Language) -> ExitCode {
    let removed = Session::open(project)
        .and_then(|mut s| s.remove_version(id))
        .map_err(|e| error_text(language, &e));

    match removed {
        Ok(()) => {
            println!(
                "{}",
                language.text("cli.versionRemoved").replace("{id}", id)
            );

            ExitCode::SUCCESS
        }
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
