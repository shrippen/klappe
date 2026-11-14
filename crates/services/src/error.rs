use thiserror::Error;

use crate::session::Layer;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error(transparent)]
    Store(#[from] klappe_store::StoreError),
    #[error("no scene {0}")]
    NoScene(String),
    #[error("no plan {0}")]
    NoPlan(String),
    #[error("there is no backup")]
    NoBackup,
    #[error("no shot {0}")]
    NoShot(String),
    /// What an edit names is no longer in the plan.
    #[error("the element is gone")]
    Gone,
    /// The scene's location is missing, so its set cannot change.
    #[error("no location {0}")]
    NoLocation(String),
    #[error("every camera letter is taken")]
    NoFreeCamera,
    /// By label and shot number, as the user knows them.
    #[error("camera {camera} is used by shot {shot}")]
    CameraInUse { camera: String, shot: String },
    /// The edit touches a layer the user has locked.
    #[error("the layer {0:?} is locked")]
    Locked(Layer),
    #[error("no character {0}")]
    NoCharacter(String),
    /// Scene numbers are unique within a project.
    #[error("scene number {0} is taken")]
    NumberTaken(String),
    /// A location cannot go while scenes are set there.
    #[error("location {id} is used by {scenes} scene(s)")]
    LocationInUse { id: String, scenes: usize },
    /// A character cannot go while a plan uses it.
    #[error("character {id} appears in {plans} plan(s)")]
    CharacterInUse { id: String, plans: usize },
    /// A new project needs an empty (or missing) folder.
    #[error("{} is not empty", .0.display())]
    NotEmpty(std::path::PathBuf),
    #[error("bad value for {field}: {value}")]
    BadValue { field: &'static str, value: String },
    #[error(transparent)]
    Export(#[from] klappe_export::ExportError),
    #[error("cannot write {path}: {source}")]
    Write {
        path: std::path::PathBuf,
        source: std::io::Error,
    },
}
