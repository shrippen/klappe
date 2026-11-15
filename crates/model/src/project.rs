use serde::{Deserialize, Serialize};

use crate::geometry::Point;
use crate::library::{LibraryItem, ObjectShape};
use crate::plan::FloorPlan;
use crate::template::Template;

/// One film: everything every module works on.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub title: String,
    #[serde(default)]
    pub characters: Vec<Character>,
    #[serde(default)]
    pub locations: Vec<Location>,
    #[serde(default)]
    pub scenes: Vec<Scene>,
    /// The project's own objects (LH-12), next to the built-in library.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub library: Vec<LibraryItem>,
    /// The project's own templates (LH-07), next to the built-in ones.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub templates: Vec<Template>,
}

impl Project {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_owned(),
            characters: Vec::new(),
            locations: Vec::new(),
            scenes: Vec::new(),
            library: Vec::new(),
            templates: Vec::new(),
        }
    }

    pub fn character(&self, id: &str) -> Option<&Character> {
        self.characters.iter().find(|c| c.id == id)
    }

    pub fn location(&self, id: &str) -> Option<&Location> {
        self.locations.iter().find(|l| l.id == id)
    }

    pub fn scene(&self, id: &str) -> Option<&Scene> {
        self.scenes.iter().find(|s| s.id == id)
    }
}

/// A role in the script, project-wide (LH-03).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Character {
    /// Stable id, e.g. `elisabeth`; names may change, ids do not.
    pub id: String,
    pub name: String,
    /// Short label drawn in the figure, e.g. `E`.
    pub short: String,
    pub color: FigureColor,
    /// How many people the character stands for (LH-23): a group such as the guests moves
    /// as one figure. `None` is a single person.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<u32>,
}

/// A group has two to 999 people; one person is a figure, more would not fit the label.
pub const MIN_GROUP: u32 = 2;
pub const MAX_GROUP: u32 = 999;

impl Character {
    /// The name with the head count of a group: `Gäste (12)`.
    pub fn title(&self) -> String {
        match self.group {
            Some(count) => format!("{} ({count})", self.name),
            None => self.name.clone(),
        }
    }
}

/// The colour a figure is drawn in. A role, not a hex value: one of Kante's label colours
/// (`KanteStyle.labels`), so both themes stay readable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FigureColor {
    Purple,
    Blue,
    Green,
    Yellow,
    Orange,
    Red,
    Cyan,
}

impl FigureColor {
    /// Every colour, in Kante's order of labels (the order the tool bar and forms offer).
    pub const ALL: [FigureColor; 7] = [
        FigureColor::Red,
        FigureColor::Orange,
        FigureColor::Yellow,
        FigureColor::Green,
        FigureColor::Cyan,
        FigureColor::Blue,
        FigureColor::Purple,
    ];

    /// The name the file format uses, e.g. `purple`; also Kante's label name.
    pub fn name(self) -> &'static str {
        match self {
            FigureColor::Purple => "purple",
            FigureColor::Blue => "blue",
            FigureColor::Green => "green",
            FigureColor::Yellow => "yellow",
            FigureColor::Orange => "orange",
            FigureColor::Red => "red",
            FigureColor::Cyan => "cyan",
        }
    }

    /// The colour `name` names; `None` for any other text.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|c| c.name() == name)
    }
}

/// A place with a set: the Sängersaal, the castle yard (LH-02).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Location {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub set: SetPlan,
}

/// The built set of a location, shared by every scene there.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct SetPlan {
    /// Width and depth of the drawing area in metres.
    pub width_m: f64,
    pub depth_m: f64,
    #[serde(default)]
    pub walls: Vec<Wall>,
    #[serde(default)]
    pub objects: Vec<SetObject>,
    /// An image under the set, e.g. a floor plan or an aerial photo (LH-13).
    #[serde(default)]
    pub background: Option<Background>,
    /// Free text on the plan, e.g. `Vorhang schließen` (LH-16).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<Note>,
}

/// A text pinned to a place on the set.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Note {
    /// Stable id within the set, e.g. `n1`.
    pub id: String,
    pub at: Point,
    pub text: String,
}

/// An image behind the plan, scaled so that its pixels measure what they do on site. The
/// height follows from the image's aspect ratio, which the model does not know: whoever
/// needs it reads the pixel size of `file`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Background {
    /// Relative to the project folder, e.g. `bilder/saengersaal.png`.
    pub file: String,
    /// The image's top left corner on the plan.
    pub at: Point,
    pub width_m: f64,
    /// Clockwise on screen about `at`.
    pub rotation_deg: f64,
    /// 0 = invisible, 1 = opaque.
    pub opacity: f64,
}

/// A wall segment (LH-10) with the doors and windows cut into it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Wall {
    pub from: Point,
    pub to: Point,
    pub thickness_m: f64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub openings: Vec<Opening>,
}

impl Wall {
    /// A wall without openings.
    pub fn new(from: Point, to: Point, thickness_m: f64) -> Self {
        Self {
            from,
            to,
            thickness_m,
            openings: Vec::new(),
        }
    }

    pub fn length_m(&self) -> f64 {
        self.from.distance_m(self.to)
    }

    /// The point `along_m` metres from `from` along the wall.
    pub fn point_at(&self, along_m: f64) -> Point {
        let length_m = self.length_m();
        if length_m == 0.0 {
            return self.from;
        }

        self.from.lerp(self.to, along_m / length_m)
    }

    /// Where `at` is nearest the wall: metres from `from` along it, kept within the wall, and
    /// how far `at` lies from that point.
    pub fn foot(&self, at: Point) -> (f64, f64) {
        let length_m = self.length_m();
        if length_m == 0.0 {
            return (0.0, self.from.distance_m(at));
        }

        let (dx, dy) = (self.to.x_m - self.from.x_m, self.to.y_m - self.from.y_m);
        let along_m = ((at.x_m - self.from.x_m) * dx + (at.y_m - self.from.y_m) * dy) / length_m;
        let along_m = along_m.clamp(0.0, length_m);

        (along_m, self.point_at(along_m).distance_m(at))
    }
}

/// What an opening in a wall is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpeningKind {
    Door,
    Window,
}

impl OpeningKind {
    /// The width a new opening of this kind has.
    pub fn default_width_m(self) -> f64 {
        match self {
            OpeningKind::Door => DOOR_WIDTH_M,
            OpeningKind::Window => WINDOW_WIDTH_M,
        }
    }
}

const DOOR_WIDTH_M: f64 = 0.9;
const WINDOW_WIDTH_M: f64 = 1.2;

/// A door or window in a wall (LH-10): a gap `width_m` wide, centred `at_m` metres from the
/// wall's `from` along the wall.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Opening {
    pub kind: OpeningKind,
    pub at_m: f64,
    pub width_m: f64,
}

impl Opening {
    /// Where the gap begins and ends along the wall, in metres from `from`.
    pub fn span_m(&self) -> (f64, f64) {
        (
            self.at_m - self.width_m / 2.0,
            self.at_m + self.width_m / 2.0,
        )
    }
}

/// Furniture, props, pillars: anything from the library (LH-11).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetObject {
    pub id: String,
    /// Library item, e.g. `table`, `pillar`, `door`, `window`, `fireplace`.
    pub kind: String,
    pub label: String,
    /// Centre of the object.
    pub at: Point,
    pub width_m: f64,
    pub depth_m: f64,
    pub rotation_deg: f64,
    /// Drawn as a rectangle unless it says otherwise.
    #[serde(default, skip_serializing_if = "is_rect")]
    pub shape: ObjectShape,
    /// How tall the object is (LH-17); `None` when not given.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height_m: Option<f64>,
}

fn is_rect(shape: &ObjectShape) -> bool {
    *shape == ObjectShape::Rect
}

/// A scene from the script, at one location (LH-01).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Scene {
    pub id: String,
    /// Scene number as in the script, e.g. `12` or `12A`.
    pub number: String,
    /// Scene heading, e.g. `INT. SÄNGERSAAL – TAG`.
    pub heading: String,
    pub location: String,
    #[serde(default)]
    pub plans: Vec<FloorPlan>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A wall from (1, 1) to (5, 4) is 5 m long.
    #[test]
    fn a_wall_finds_the_nearest_place_on_it() {
        let wall = Wall::new(Point::new(1.0, 1.0), Point::new(5.0, 4.0), 0.2);

        assert_eq!(wall.length_m(), 5.0);
        assert_eq!(wall.point_at(2.5), Point::new(3.0, 2.5));

        // On the wall: its foot is the point itself.
        let (along_m, off_m) = wall.foot(Point::new(3.0, 2.5));
        assert!((along_m - 2.5).abs() < 1e-9 && off_m < 1e-9);

        // Beside the middle by 1 m at a right angle (to the lower left of a wall that runs down
        // to the right), and beyond the end: kept at the end.
        let (along_m, off_m) = wall.foot(Point::new(3.0 + 0.6, 2.5 - 0.8));
        assert!((along_m - 2.5).abs() < 1e-9 && (off_m - 1.0).abs() < 1e-9);
        assert_eq!(wall.foot(Point::new(9.0, 4.0)).0, 5.0);
        assert_eq!(wall.foot(Point::new(-9.0, 1.0)).0, 0.0);
    }

    #[test]
    fn an_opening_spans_its_width_about_its_centre() {
        let door = Opening {
            kind: OpeningKind::Door,
            at_m: 4.0,
            width_m: 0.9,
        };

        assert_eq!(door.span_m(), (3.55, 4.45));
        assert_eq!(OpeningKind::Door.default_width_m(), 0.9);
        assert_eq!(OpeningKind::Window.default_width_m(), 1.2);
    }

    /// `name` is what the file holds, `from_name` reads it back, and `ALL` has no colour twice.
    #[test]
    fn names_are_the_file_names() {
        for color in FigureColor::ALL {
            let json = serde_json::to_string(&color).unwrap();
            assert_eq!(json, format!("\"{}\"", color.name()));
            assert_eq!(FigureColor::from_name(color.name()), Some(color));
        }
        assert_eq!(FigureColor::from_name("magenta"), None);

        let mut names: Vec<_> = FigureColor::ALL.iter().map(|c| c.name()).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), FigureColor::ALL.len());
    }
}
