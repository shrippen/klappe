//! What the header's status line says: translated messages, and when they go stale.

use klappe_services::BackupState;

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
}
