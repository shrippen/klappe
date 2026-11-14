//! What the header's status line says: translated messages, and when they go stale.

use klappe_services::{BackupState, Layer, ServiceError};

use crate::catalog::Language;

/// How QML and the catalogue (`layer.<name>`) call a layer.
pub fn layer_name(layer: Layer) -> &'static str {
    match layer {
        Layer::Set => "set",
        Layer::Figures => "figures",
        Layer::Cameras => "cameras",
        Layer::Lights => "lights",
        Layer::Notes => "notes",
    }
}

/// The service's error in the app's language; details (path, value) appended where they help.
pub fn error_text(language: Language, error: &ServiceError) -> String {
    let (key, detail) = match error {
        ServiceError::NoScene(id) => ("error.noScene", id.clone()),
        ServiceError::NoPlan(id) => ("error.noPlan", id.clone()),
        ServiceError::NoShot(id) => ("error.noShot", id.clone()),
        ServiceError::Gone => ("error.gone", String::new()),
        ServiceError::NoLocation(id) => ("error.noLocation", id.clone()),
        ServiceError::NoFreeCamera => ("error.noFreeCamera", String::new()),
        ServiceError::NoCharacter(id) => ("error.noCharacter", id.clone()),
        ServiceError::NotEmpty(path) => ("error.notEmpty", path.to_string_lossy().into_owned()),
        ServiceError::NoBackup => ("error.noBackup", String::new()),
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
        ServiceError::Locked(layer) => {
            let name = language.text(&format!("layer.{}", layer_name(*layer)));
            return language.text("error.locked").replace("{layer}", &name);
        }
        ServiceError::BadValue { field, value } => {
            let name = language.text(&format!("field.{field}"));
            return language
                .text("error.badValue")
                .replace("{field}", &name)
                .replace("{value}", value);
        }
        ServiceError::Store(e) => ("error.store", e.to_string()),
        ServiceError::Export(e) => ("error.export", e.to_string()),
        ServiceError::Write { path, .. } => ("error.write", path.to_string_lossy().into_owned()),
    };

    language.text(key).replace("{detail}", &detail)
}

/// How QML tells the backup states apart (`Controller.backupState`).
pub const BACKUP_NONE: &str = "none";
pub const BACKUP_FOUND: &str = "found";
pub const BACKUP_UNREADABLE: &str = "unreadable";

/// The backup `Session::open` found as QML gets it: the state's name, and for an unreadable
/// one the reason (empty otherwise).
pub fn backup_view(state: &BackupState) -> (&'static str, String) {
    match state {
        BackupState::None => (BACKUP_NONE, String::new()),
        BackupState::Found => (BACKUP_FOUND, String::new()),
        BackupState::Unreadable(reason) => (BACKUP_UNREADABLE, reason.clone()),
    }
}

/// One message in the header. A change to the project makes it stale: "saved" would
/// otherwise stay while unsaved edits pile up.
#[derive(Debug, Default)]
pub struct Status {
    text: String,
}

impl Status {
    pub fn say(&mut self, text: String) {
        self.text = text;
    }

    /// The project changed; true if there was a message to drop.
    pub fn project_changed(&mut self) -> bool {
        let stale = !self.text.is_empty();
        self.text.clear();
        stale
    }

    pub fn text(&self) -> &str {
        &self.text
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn the_backup_state_reaches_qml_with_its_reason() {
        assert_eq!(
            backup_view(&BackupState::None),
            (BACKUP_NONE, String::new())
        );
        assert_eq!(
            backup_view(&BackupState::Found),
            (BACKUP_FOUND, String::new())
        );
        assert_eq!(
            backup_view(&BackupState::Unreadable("kaputt".into())),
            (BACKUP_UNREADABLE, "kaputt".into())
        );
    }

    #[test]
    fn a_change_drops_the_saved_message() {
        let mut status = Status::default();
        status.say("Gespeichert".to_owned());

        assert!(status.project_changed());

        assert_eq!(status.text(), "");
        assert!(!status.project_changed());
    }

    #[test]
    fn errors_are_translated_with_details() {
        let write = ServiceError::Write {
            path: PathBuf::from("/a/b.pdf"),
            source: std::io::Error::other("kaputt"),
        };
        let bad = ServiceError::BadValue {
            field: "lens",
            value: "5000".to_owned(),
        };

        assert_eq!(
            error_text(Language::German, &write),
            "Kann nicht schreiben: /a/b.pdf"
        );
        assert_eq!(
            error_text(Language::English, &write),
            "Cannot write: /a/b.pdf"
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
        let layers = [
            Layer::Set,
            Layer::Figures,
            Layer::Cameras,
            Layer::Lights,
            Layer::Notes,
        ];

        for layer in layers {
            let key = format!("layer.{}", layer_name(layer));
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
            ServiceError::Gone,
            ServiceError::NoLocation("l".to_owned()),
            ServiceError::NoFreeCamera,
            ServiceError::NoBackup,
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
