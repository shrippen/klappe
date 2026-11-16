//! The text of a service error in a language: the one place app and CLI take it from.

use crate::catalog::Language;
use crate::error::ServiceError;
use crate::field::Field;

/// The service's error in the given language; details (path, value) appended where they help.
pub fn error_text(language: Language, error: &ServiceError) -> String {
    // Wraps another error: its text first, then where the old example copy went.
    let ServiceError::ExampleAside { aside, source } = error else {
        return own_text(language, error);
    };

    language
        .text("error.exampleAside")
        .replace("{reason}", &error_text(language, source))
        .replace("{aside}", &aside.to_string_lossy())
}

fn own_text(language: Language, error: &ServiceError) -> String {
    let (key, detail) = match error {
        ServiceError::NoScene(id) => ("error.noScene", id.clone()),
        ServiceError::NoPlan(id) => ("error.noPlan", id.clone()),
        ServiceError::NoShot(id) => ("error.noShot", id.clone()),
        ServiceError::NoVersion(number) => ("error.noVersion", number.clone()),
        ServiceError::NoLocation(id) => ("error.noLocation", id.clone()),
        ServiceError::NoCharacter(id) => ("error.noCharacter", id.clone()),
        ServiceError::NotEmpty(path) => ("error.notEmpty", path.to_string_lossy().into_owned()),
        ServiceError::CameraInUse { camera, shot } => {
            return language
                .text("error.cameraInUse")
                .replace("{camera}", camera)
                .replace("{shot}", shot);
        }
        ServiceError::NumberTaken(number) => {
            return language
                .text("error.numberTaken")
                .replace("{number}", number);
        }
        ServiceError::LocationInUse { scenes, .. } => {
            return language
                .text("error.locationInUse")
                .replace("{count}", &scenes.to_string());
        }
        ServiceError::CharacterInUse { plans, .. } => {
            return language
                .text("error.characterInUse")
                .replace("{count}", &plans.to_string());
        }
        ServiceError::PlanHasShots { shots, .. } => {
            return language
                .text("error.planHasShots")
                .replace("{count}", &shots.to_string());
        }
        ServiceError::PlanNameTaken(name) => {
            return language.text("error.planNameTaken").replace("{name}", name);
        }
        ServiceError::Locked(layer) => {
            let name = language.text(&format!("layer.{}", layer.name()));
            return language.text("error.locked").replace("{layer}", &name);
        }
        ServiceError::BadValue { field, value } => return bad_value(language, field, value),
        ServiceError::Store(e) => ("error.store", e.to_string()),
        ServiceError::Export(e) => ("error.export", e.to_string()),
        // The path and the system's reason, e.g. "/a/b.pdf: No such file or directory".
        ServiceError::Write { path, source } => {
            ("error.write", format!("{}: {source}", path.display()))
        }
        other => (plain_key(other).unwrap_or_default(), String::new()),
    };

    language.text(key).replace("{detail}", &detail)
}

/// The refused field and its value; without a value, no colon with nothing after it.
fn bad_value(language: Language, field: &Field, value: &str) -> String {
    let name = language.text(&format!("field.{}", field.name()));
    let key = if value.is_empty() {
        "error.badField"
    } else {
        "error.badValue"
    };

    language
        .text(key)
        .replace("{field}", &name)
        .replace("{value}", value)
}

/// The catalogue key of an error that has no detail to add.
fn plain_key(error: &ServiceError) -> Option<&'static str> {
    match error {
        ServiceError::Gone => Some("error.gone"),
        ServiceError::NoFreeCamera => Some("error.noFreeCamera"),
        ServiceError::NoCamera => Some("error.noCamera"),
        ServiceError::NoBackup => Some("error.noBackup"),
        ServiceError::EmptyLibrary => Some("error.emptyLibrary"),
        ServiceError::EmptyTemplate => Some("error.emptyTemplate"),
        ServiceError::NoMarks => Some("error.noMarks"),
        ServiceError::NoPicture => Some("error.noPicture"),
        ServiceError::LastPlan(_) => Some("error.lastPlan"),
        ServiceError::FewPoints => Some("error.fewPoints"),
        ServiceError::NotCopyable => Some("error.notCopyable"),
        // Listed so a new variant must be placed: `error_text` words these itself.
        ServiceError::NoScene(_)
        | ServiceError::NoPlan(_)
        | ServiceError::NoShot(_)
        | ServiceError::NoVersion(_)
        | ServiceError::NoLocation(_)
        | ServiceError::NoCharacter(_)
        | ServiceError::NotEmpty(_)
        | ServiceError::CameraInUse { .. }
        | ServiceError::NumberTaken(_)
        | ServiceError::LocationInUse { .. }
        | ServiceError::CharacterInUse { .. }
        | ServiceError::PlanHasShots { .. }
        | ServiceError::PlanNameTaken(_)
        | ServiceError::Locked(_)
        | ServiceError::BadValue { .. }
        | ServiceError::ExampleAside { .. }
        | ServiceError::Store(_)
        | ServiceError::Export(_)
        | ServiceError::Write { .. } => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Layer;
    use std::path::PathBuf;

    /// Review of PR 37: a refused tilt said "field.tilt", the catalogue had no such key.
    #[test]
    fn every_refused_field_has_a_name() {
        for language in [Language::German, Language::English] {
            let raw: Vec<_> = Field::ALL
                .iter()
                .map(|f| f.name())
                .filter(|name| language.text(&format!("field.{name}")) == format!("field.{name}"))
                .collect();

            assert!(raw.is_empty(), "{language:?}: {raw:?}");
        }
    }

    /// Q2-04: a refusal with no value to show said "Ungültiger Wert für Element:".
    #[test]
    fn a_refusal_without_value_has_no_empty_tail() {
        for language in [Language::German, Language::English] {
            for field in Field::ALL {
                let error = ServiceError::BadValue {
                    field,
                    value: String::new(),
                };
                let text = error_text(language, &error);

                assert!(!text.trim_end().ends_with(':'), "{language:?}: {text}");
            }
        }
    }

    #[test]
    fn a_refused_copy_names_no_kind() {
        let error = ServiceError::NotCopyable;

        assert_eq!(
            error_text(Language::German, &error),
            "Die Auswahl lässt sich nicht duplizieren"
        );
        assert_eq!(
            error_text(Language::English, &error),
            "The selection cannot be duplicated"
        );
    }

    #[test]
    fn errors_are_translated_with_details() {
        let write = ServiceError::Write {
            path: PathBuf::from("/a/b.pdf"),
            source: std::io::Error::other("kaputt"),
        };
        let bad = ServiceError::BadValue {
            field: Field::Lens,
            value: "5000".to_owned(),
        };

        assert_eq!(
            error_text(Language::German, &write),
            "Kann nicht schreiben: /a/b.pdf: kaputt"
        );
        assert_eq!(
            error_text(Language::English, &write),
            "Cannot write: /a/b.pdf: kaputt"
        );
        assert_eq!(
            error_text(Language::German, &bad),
            "Ungültiger Wert für Objektiv (mm): 5000"
        );
        assert_eq!(
            error_text(Language::English, &bad),
            "Invalid value for Lens (mm): 5000"
        );
    }

    #[test]
    fn plan_refusals_are_translated() {
        let shots = ServiceError::PlanHasShots {
            id: "plan-a".to_owned(),
            shots: 5,
        };
        let taken = ServiceError::PlanNameTaken("Plan B".to_owned());
        let last = ServiceError::LastPlan("plan-a".to_owned());
        let few = ServiceError::FewPoints;

        assert_eq!(
            error_text(Language::German, &shots),
            "Der Plan enthält 5 Shot(s): lösche sie zuerst"
        );
        assert_eq!(
            error_text(Language::English, &taken),
            "This scene already has a plan called Plan B"
        );
        assert_eq!(
            error_text(Language::English, &last),
            "A scene keeps its last plan"
        );
        assert_eq!(
            error_text(Language::German, &few),
            "Eine Fläche braucht mindestens drei Punkte."
        );
        assert_eq!(
            error_text(Language::English, &few),
            "An area needs at least three points."
        );
    }

    /// Review of PR 10: label and shot number, no code text.
    #[test]
    fn a_camera_in_use_names_label_and_shot() {
        let error = ServiceError::CameraInUse {
            camera: "B".to_owned(),
            shot: "12.3".to_owned(),
        };

        assert_eq!(
            error_text(Language::German, &error),
            "Kamera B wird von Shot 12.3 genutzt"
        );
        assert_eq!(
            error_text(Language::English, &error),
            "Camera B is used by shot 12.3"
        );
    }

    #[test]
    fn a_locked_layer_is_named_in_the_message() {
        let error = ServiceError::Locked(Layer::Notes);

        assert_eq!(
            error_text(Language::German, &error),
            "Die Ebene Notizen ist gesperrt"
        );
        assert_eq!(
            error_text(Language::English, &error),
            "The layer Notes is locked"
        );
    }

    #[test]
    fn an_object_in_use_names_the_count() {
        let error = ServiceError::LocationInUse {
            id: "l".to_owned(),
            scenes: 3,
        };

        assert_eq!(
            error_text(Language::German, &error),
            "Der Drehort wird von 3 Szene(n) genutzt"
        );
        assert_eq!(
            error_text(Language::English, &error),
            "The location is used by 3 scene(s)"
        );
    }

    #[test]
    fn every_layer_has_a_name_in_both_languages() {
        for layer in Layer::ALL {
            let key = format!("layer.{}", layer.name());
            for language in [Language::German, Language::English] {
                assert_ne!(language.text(&key), key);
            }
        }
    }

    #[test]
    fn every_error_kind_has_a_text_in_both_languages() {
        let errors = [
            ServiceError::NoScene("s".to_owned()),
            ServiceError::NoShot("x".to_owned()),
            ServiceError::NoVersion("12.3ZZZ".to_owned()),
            ServiceError::Gone,
            ServiceError::NoLocation("l".to_owned()),
            ServiceError::NoFreeCamera,
            ServiceError::NoCamera,
            ServiceError::NoBackup,
            ServiceError::EmptyTemplate,
            ServiceError::Locked(Layer::Set),
            ServiceError::NoCharacter("c".to_owned()),
            ServiceError::NumberTaken("12".to_owned()),
            ServiceError::LocationInUse {
                id: "l".to_owned(),
                scenes: 2,
            },
            ServiceError::CharacterInUse {
                id: "c".to_owned(),
                plans: 1,
            },
            ServiceError::NotEmpty(PathBuf::from("/a")),
            ServiceError::ExampleAside {
                aside: PathBuf::from("/a.alt"),
                source: Box::new(ServiceError::NoMarks),
            },
            ServiceError::NoMarks,
            ServiceError::NoPicture,
            ServiceError::NotCopyable,
        ];

        for error in &errors {
            for language in [Language::German, Language::English] {
                let text = error_text(language, error);
                assert!(!text.starts_with("error."), "{text}");
                assert_ne!(text, error.to_string());
            }
        }
    }
}
