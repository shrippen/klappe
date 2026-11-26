//! The project session (planung/KONZEPT.md, section 4): the one layer GUI, TUI and CLI call.
//!
//! ```text
//! app / tui / cli
//!        │  open, select_scene, set_time, edit_shot, undo, save
//!        ▼
//!     Session ──► store (files)   stellprobe (display list, axis)   script (Fountain)
//!        │
//!        └─► views: ShotRow, Track, ScriptLine  (plain data, ready to show)
//! ```

mod answer_way;
mod book;
mod camera_edit;
mod cast_number;
mod catalog;
mod edit;
mod error;
mod error_text;
mod example;
mod export_format;
mod field;
mod figure_file;
mod figures;
mod folder;
mod library;
mod light_edit;
mod load;
mod measure;
mod number;
mod place_file;
mod places;
mod plan_edit;
mod project_edit;
mod report;
mod scene_location;
mod scene_text;
mod script_pages;
mod script_refs;
mod session;
mod sheet_time;
mod shot_edit;
mod sides;
mod start;
mod sun_input;
mod templates;
mod versions;
mod views;

pub use book::{BookReport, Candidate, Choice, ImportReport, PendingScene};
pub use catalog::Language;
pub use edit::ShotField;
pub use error::ServiceError;
pub use error_text::{book_report_text, error_text, load_notes_text, outside_files_text};
pub use example::{copy_example, ensure_example};
pub use export_format::ExportFormat;
pub use field::Field;
pub use figures::{name_from_capitals, FigureChoice, PendingFigure};
pub use folder::{archive_project, rename_project, trash_project};
pub use klappe_store::{builtin_camera_presets, FileKind};
pub use light_edit::LightChange;
pub use measure::measure_angle_deg;
pub use number::{read_number, Range, Unit};
pub use places::{PendingPlace, PlaceChoice, PlaceEdit};
pub use plan_edit::{layer_of_source, Grid, MarkInfo, ObjectChange, Pace, PlanEdit, Snapping};
pub use project_edit::{
    BreakdownEdit, LibraryEdit, MarkTarget, NameEdit, PlanShift, ProjectEdit, ShotsGo, TeamEdit,
    TemplateEdit,
};
pub use report::{Report, ReportFormat};
pub use script_pages::{HeaderLine, PageBreak, SceneNumbers, SceneSpan, ScriptPages, ScriptPrint};
pub use script_refs::{ChangeKind, ScriptChange};
pub use session::video_frame_count;
pub use session::{
    Answer, AnswerLine, Answered, Autosave, BackupState, BookQuestion, DialogueFigure, Draft,
    ExportOptions, Fade, FigureAnswer, Grab, Laid, Layer, Lock, Looping, OnOutside, Outside,
    OutsideFile, Pages, PlaceAnswer, Playing, QuestionKey, Revisions, Saved, Session, Size, Speech,
    SpokenLine, Step, Tick, TimeSnap, VideoJob, View, ViewStep, Visibility,
};
pub use start::{project_card, CardState, ProjectCard};
pub use sun_input::{read_site, read_sun_time};
pub use versions::color_text;
pub use views::{
    AxisChoice, CameraSpot, CastUse, CastingRow, CharacterRow, Chip, ElementRow, Elsewhere, Hint,
    LibraryEntry, LinkKind, LocationRow, LookElement, LookRow, Marker, NotShot, Overview,
    PersonRow, PlaceRow, PlanRow, SceneCard, SceneRef, SceneRow, SceneUse, ScriptLine, ScriptRow,
    Segment, SetupOption, SheetCategory, SheetElement, SheetFigure, SheetLookElement, SheetView,
    ShotRow, Suggestion, SunTexts, TemplateEntry, Track, TrackKind, ViewRow,
};

// What surfaces draw and print, handed through so they need no lower crate.
pub use klappe_export::{DrawingTexts, LampTexts, ShotColumns, ShotTexts, Texts, Theme, FFMPEG};
// The screenplay's pages as they print, for a surface's print view (D-15).
pub use klappe_export::{Face as PrintFace, Placed, SheetPage, SCRIPT_ASCENT_PT, SCRIPT_FONT_PT};
pub use klappe_model::{
    same_character_name, same_place_name, Angle, Arrival, CameraBody, CameraPresets, Contact,
    Curve, Dash, Department, Departure, DoorSide, DrawingShape, ElementCategory, ElementStatus,
    FigureColor, Fill, FloorPlan, Hinge, LensSet, LightKind, LightPreset, Movement, Numbering,
    ObjectShape, OpeningKind, Orientation, Paper, PaperSetup, Point, Pose, Rig, SceneNumbering,
    ScriptLanguage, ScriptPaper, ScriptSetup, ScriptWords, Sensor, ShotSize, Site, SunTime,
    MIN_IMAGE_OPACITY, MIN_OBJECT_HEIGHT_M,
};
pub use klappe_stellprobe::{
    horizontal_fov_deg, look, scene_look, Area, Bend, Cut, DisplayList, Frame, Hue, Item,
    LightGrip, ObjectGrip, OpeningEnd, Picture, PictureLabel, Role, SceneRole, Shape, ShotOrder,
    Source, NOTE_TEXT_M,
};

// Fountain, so the TUI editor can style and list what it types without the script crate;
// `parse` for its tests.
pub use klappe_script::{parse, parse_located, Element, Located};
// What `compare_drafts` and `revision` return (DB-41, DB-42, DB-44).
pub use klappe_model::{RevisionColor, Version};
pub use klappe_script::{Comparison, Edit, LineChange, SceneDiff, ScenePlace, SceneStatus};

// The writing aids of `script/` (completion, Enter and Tab flow, search, title page, inline
// marks), for the screenplay editors of the surfaces.
pub use klappe_script::{
    boneyard_lines, can_suggest, enter_step, find_all, force_line, heading_like, inline_runs,
    replace_hits, set_title_fields, suggest, tab_step, title_fields, title_page_lines, typed_caps,
    upper, Case, Context, EnterStep, Forced, Gap, Hit, Marks, Query, Run, Slot, Splice, Stage,
    Suggestions, TabStep, Vocabulary, FORCE_CHARACTER, NUMBER_MARK, TITLE_KEYS,
};

// The kind of each text line and the columns the page sets it in (D-15), for an editor that
// indents the elements like the page.
pub use klappe_script::{
    located_kinds, text_kinds, Align, Margin, PageKind, TextKind, TEXT_COLUMNS,
};

// The outline of a screenplay (DB-20, DB-21): the tree, what a filter or a collapsed
// section leaves, and moving a scene or a section.
pub use klappe_script::{
    length_text, move_entry, outline, outline_of, sections_around, visible as visible_entries,
    Entry as OutlineEntry, EntryKind as OutlineKind, Moved, Way as MoveWay,
};
