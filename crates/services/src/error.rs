use klappe_store::StoreError;
use thiserror::Error;

use crate::field::Field;
use crate::session::Layer;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error(transparent)]
    Store(#[from] klappe_store::StoreError),
    #[error("no scene {0}")]
    NoScene(String),
    /// The entry has no open question, or the scene is none of its candidates.
    #[error("no open decision for scene {0}")]
    NoDecision(String),
    /// The dialog line of a place question is not there.
    #[error("no answer {0} to the place question")]
    NoPlaceAnswer(usize),
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
    /// The scene is shot nowhere yet (DB-51): neither it nor its scripted location has a
    /// shooting location, so a plan has no set.
    #[error("the scene has no shooting location")]
    NotShot,
    /// A shot needs a camera, and the plan has none.
    #[error("the plan has no camera")]
    NoCamera,
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
    #[error("no element {0}")]
    NoElement(String),
    #[error("no person {0}")]
    NoPerson(String),
    /// Another character has the cast number.
    #[error("cast number {0} is taken")]
    CastNumberTaken(u32),
    #[error("no look {0}")]
    NoLook(String),
    /// Only costume and make-up elements belong to a look; holds the element's name.
    #[error("element {0} is neither costume nor make-up")]
    NotLookElement(String),
    /// Scene numbers are unique within a project.
    #[error("scene number {0} is taken")]
    NumberTaken(String),
    /// A location cannot go while scenes are set there.
    #[error("location {id} is used by {scenes} scene(s)")]
    LocationInUse { id: String, scenes: usize },
    /// A scripted location cannot go while scene headings name it (D-21).
    #[error("{headings} heading(s) name the scripted location")]
    PlaceInUse { headings: usize },
    /// A character cannot go while a plan uses it.
    #[error("character {id} appears in {plans} plan(s)")]
    CharacterInUse { id: String, plans: usize },
    /// A scene keeps at least one plan.
    #[error("{0} is the scene's last plan")]
    LastPlan(String),
    /// An area keeps at least three points.
    #[error("an area needs at least 3 points")]
    FewPoints,
    /// A plan cannot go while it holds shots: they would go with it.
    #[error("plan {id} holds {shots} shot(s)")]
    PlanHasShots { id: String, shots: usize },
    /// Plan names are unique within a scene, so the variants can be told apart.
    #[error("plan name {0} is taken")]
    PlanNameTaken(String),
    /// A version has the camera of its base: only the base's camera is changed.
    #[error("shot {0} is a version: it has the camera of its base")]
    VersionCamera(String),
    /// A new project needs an empty (or missing) folder.
    #[error("{} is not empty", .0.display())]
    NotEmpty(std::path::PathBuf),
    /// There is nothing to pass on: the project has no own objects (or templates, for a
    /// template file).
    #[error("the project has nothing of its own to pass on")]
    EmptyLibrary,
    /// A plan without marks has no frames to write (LH-83).
    #[error("the plan has no marks")]
    NoMarks,
    /// The video was stopped before it was whole (LH-84).
    #[error("the video was stopped")]
    Cancelled,
    /// A plan of no duration has no video (LH-84).
    #[error("the plan has no duration")]
    NoDuration,
    /// The plan has no shot (or camera) to write a page for (LH-87).
    #[error("the plan has no shots or cameras")]
    NoPages,
    /// A view has nothing to show: 3D not open yet, or no camera with marks (LH-86).
    #[error("there is no picture")]
    NoPicture,
    /// The project has no screenplay to page or print (DB-70).
    #[error("the project has no screenplay")]
    NoScript,
    /// A file to import is no Final Draft file; holds what is wrong with it (DB-72).
    #[error("not a Final Draft file: {0}")]
    BadFdx(String),
    /// A report has no scene to list: none is in the book.
    #[error("no scenes in the book")]
    NoBookScenes,
    /// A plan with no figure and no camera has nothing to file as a template.
    #[error("the plan has no figure and no camera")]
    EmptyTemplate,
    /// Nothing selected can be copied (a mark, figure, door or path handle has no copy).
    #[error("the selection cannot be copied")]
    NotCopyable,
    #[error("bad value for {}: {value}", field.name())]
    BadValue { field: Field, value: String },
    #[error(transparent)]
    Export(#[from] klappe_export::ExportError),
    /// The example's old copy was set aside, then the fresh one could not be written.
    #[error("{source}; the old copy is at {}", .aside.display())]
    ExampleAside {
        aside: std::path::PathBuf,
        source: Box<ServiceError>,
    },
    #[error("cannot write {path}: {source}")]
    Write {
        path: std::path::PathBuf,
        source: std::io::Error,
    },
}

/// A failed write of the store as the user knows it: "cannot write <path>". Other store
/// errors stay what they are.
pub(crate) fn write_failed(error: StoreError) -> ServiceError {
    match error {
        StoreError::Io { path, source } => ServiceError::Write { path, source },
        other => ServiceError::Store(other),
    }
}
