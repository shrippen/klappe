//! The rows of the breakdown and the scene sheet (DB-60, D-22).

use serde::Serialize;

/// One element of the breakdown with the scenes that need it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ElementRow {
    pub id: String,
    /// The category's name, as `prop`.
    pub category: &'static str,
    pub name: String,
    pub description: String,
    /// The status' name, as `open`.
    pub status: &'static str,
    /// The scenes whose sheet lists it, in scene order; numbers may repeat (DB-33).
    pub scenes: Vec<SceneRef>,
    /// The id of the person of the team who answers for it.
    pub responsible: Option<String>,
}

/// A look of a character with the costume and make-up it is made of.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LookRow {
    pub id: String,
    /// The id of the character it belongs to.
    pub character: String,
    pub name: String,
    pub elements: Vec<LookElement>,
    /// The scenes whose sheet gives it to its character, in scene order.
    pub scenes: Vec<SceneRef>,
}

/// An element of a look.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LookElement {
    pub id: String,
    pub name: String,
    /// The category's name: `costume` or `makeup`.
    pub category: &'static str,
}

/// A scene named in a list: its id, and its number to show.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SceneRef {
    pub id: String,
    pub number: String,
}

/// The scene sheet of the scene on view (D-22).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SheetView {
    pub story_day: String,
    /// The pre-stop as `m:ss`, empty for none.
    pub prestop_text: String,
    /// The planned shooting time as `h:mm`, empty for none.
    pub planned_text: String,
    pub continuity: String,
    pub note: String,
    /// The characters with a cue in the book, in the order of their first cue.
    pub speaking: Vec<SheetFigure>,
    /// The characters without a cue the sheet names: silent ones and extras.
    pub present: Vec<SheetFigure>,
    /// The categories that have elements in this scene, in the order of `element_categories`.
    pub categories: Vec<SheetCategory>,
    /// Elements the sheet neither lists nor turned down that the set offers or whose name
    /// stands in the scene's text (D-23).
    pub suggestions: Vec<Suggestion>,
    /// The scene's text to mark elements in (D-23), one entry per line.
    pub text: Vec<TextLine>,
}

/// A line of a scene's text on the scene sheet (D-23).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TextLine {
    /// `action`, `character` (a cue), `parenthetical`, `dialogue`, `transition`, `centered` or
    /// `lyrics`, named like the kinds of the screenplay view.
    pub kind: &'static str,
    /// The line in pieces, marked and not; joined they are the line.
    pub runs: Vec<TextRun>,
}

/// A piece of a line: marked as an element or not.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TextRun {
    pub text: String,
    /// The element's id; `None` for text not marked.
    pub element: Option<String>,
    /// The element's category, e.g. `prop`.
    pub category: Option<&'static str>,
}

/// A character on a scene sheet.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SheetFigure {
    pub id: String,
    pub name: String,
    /// The colour's role name, e.g. `purple`.
    pub color: &'static str,
    /// Head count of a group of extras (LH-23); 0 for a single person.
    pub group: u32,
    /// The id of the look it wears in the scene; `None` for none.
    pub look: Option<String>,
}

/// The elements of one category in a scene.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SheetCategory {
    pub category: &'static str,
    pub elements: Vec<SheetElement>,
    /// The elements of the looks worn in the scene, not part of its sheet.
    pub from_looks: Vec<SheetLookElement>,
}

/// An element a character brings into a scene through its look.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SheetLookElement {
    pub id: String,
    pub name: String,
    pub status: &'static str,
    /// The name of the character that wears the look.
    pub figure: String,
}

/// An element in a scene.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SheetElement {
    pub id: String,
    pub name: String,
    pub status: &'static str,
    /// It was marked in the scene's text and no mark is found there any more (D-23).
    pub lost: bool,
}

/// An element the set of the scene offers through one of its objects.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Suggestion {
    /// The element's id.
    pub element: String,
    pub name: String,
    pub category: &'static str,
    /// The label of the set object that stands for it; `None`: its name is in the text.
    pub object: Option<String>,
}
