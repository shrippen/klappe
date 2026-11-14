//! What the header's status line says: translated messages, and when they go stale.

use klappe_services::{BackupState, ServiceError};

use crate::catalog::Language;

/// The service's error in the app's language; details (path, value) appended where they help.
pub fn error_text(language: Language, error: &ServiceError) -> String {
    let (key, detail) = match error {
        ServiceError::NoScene(id) => ("error.noScene", id.clone()),
        ServiceError::NoPlan(id) => ("error.noPlan", id.clone()),
        ServiceError::NoShot(id) => ("error.noShot", id.clone()),
        ServiceError::Gone => ("error.gone", String::new()),
        ServiceError::NoLocation(id) => ("error.noLocation", id.clone()),
        ServiceError::NoFreeCamera => ("error.noFreeCamera", String::new()),
        ServiceError::NoBackup => ("error.noBackup", String::new()),
        ServiceError::CameraInUse { camera, shot } => {
            return language
                .text("error.cameraInUse")
                .replace("{camera}", camera)
                .replace("{shot}", shot);
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
    fn every_error_kind_has_a_text_in_both_languages() {
        let errors = [
            ServiceError::NoScene("s".to_owned()),
            ServiceError::NoShot("x".to_owned()),
            ServiceError::Gone,
            ServiceError::NoLocation("l".to_owned()),
            ServiceError::NoFreeCamera,
            ServiceError::NoBackup,
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
