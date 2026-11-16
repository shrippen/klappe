//! The keys of the plan's shapes: which element a shape on the canvas stands for.

use klappe_services::{Bend, LightGrip, ObjectGrip, OpeningEnd, Source};

/// The kinds of element a key names; `name` is the key's first part.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementKind {
    Object,
    ObjectGrip,
    Light,
    LightGrip,
    Camera,
    Figure,
    Mark,
    FigureVia,
    CameraVia,
    Wall,
    Opening,
    OpeningEdge,
    Note,
    NoteWidth,
    Image,
    ImageSize,
    Drawing,
    DrawingPoint,
    DrawingEdge,
}

impl ElementKind {
    const ALL: [ElementKind; 19] = [
        Self::Object,
        Self::ObjectGrip,
        Self::Light,
        Self::LightGrip,
        Self::Camera,
        Self::Figure,
        Self::Mark,
        Self::FigureVia,
        Self::CameraVia,
        Self::Wall,
        Self::Opening,
        Self::OpeningEdge,
        Self::Note,
        Self::NoteWidth,
        Self::Image,
        Self::ImageSize,
        Self::Drawing,
        Self::DrawingPoint,
        Self::DrawingEdge,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Object => "object",
            Self::ObjectGrip => "objectgrip",
            Self::Light => "light",
            Self::LightGrip => "lightgrip",
            Self::Camera => "camera",
            Self::Figure => "figure",
            Self::Mark => "mark",
            Self::FigureVia => "figurevia",
            Self::CameraVia => "cameravia",
            Self::Wall => "wall",
            Self::Opening => "opening",
            Self::OpeningEdge => "openingedge",
            Self::Note => "note",
            Self::NoteWidth => "notewidth",
            Self::Image => "image",
            Self::ImageSize => "imagesize",
            Self::Drawing => "drawing",
            Self::DrawingPoint => "drawingpoint",
            Self::DrawingEdge => "drawingedge",
        }
    }

    fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.name() == name)
    }
}

/// `camera:a`, `figure:elisabeth`, `mark:elisabeth:0` …; the app's tooltips read it back.
pub fn key(source: &Source) -> String {
    let (kind, rest) = match source {
        Source::Object(id) => (ElementKind::Object, id.clone()),
        Source::ObjectGrip { object, grip } => (
            ElementKind::ObjectGrip,
            format!("{object}:{}", object_grip_part(*grip)),
        ),
        Source::Light(id) => (ElementKind::Light, id.clone()),
        Source::LightGrip { light, grip } => (
            ElementKind::LightGrip,
            format!("{light}:{}", light_grip_part(*grip)),
        ),
        Source::Camera(id) => (ElementKind::Camera, id.clone()),
        Source::Figure(id) => (ElementKind::Figure, id.clone()),
        Source::Mark { character, index } => (ElementKind::Mark, format!("{character}:{index}")),
        Source::FigureVia {
            character,
            index,
            bend,
        } => (
            ElementKind::FigureVia,
            format!("{character}:{index}:{}", bend_part(*bend)),
        ),
        Source::CameraVia {
            camera,
            index,
            bend,
        } => (
            ElementKind::CameraVia,
            format!("{camera}:{index}:{}", bend_part(*bend)),
        ),
        Source::Wall(index) => (ElementKind::Wall, index.to_string()),
        Source::Opening { wall, index } => (ElementKind::Opening, format!("{wall}:{index}")),
        Source::OpeningEdge { wall, index, end } => (
            ElementKind::OpeningEdge,
            format!("{wall}:{index}:{}", end_part(*end)),
        ),
        Source::Note(id) => (ElementKind::Note, id.clone()),
        Source::NoteWidth(id) => (ElementKind::NoteWidth, id.clone()),
        Source::Image(id) => (ElementKind::Image, id.clone()),
        Source::ImageSize(id) => (ElementKind::ImageSize, id.clone()),
        Source::Drawing(id) => (ElementKind::Drawing, id.clone()),
        Source::DrawingPoint { drawing, index } => {
            (ElementKind::DrawingPoint, format!("{drawing}:{index}"))
        }
        Source::DrawingEdge { drawing, index } => {
            (ElementKind::DrawingEdge, format!("{drawing}:{index}"))
        }
    };

    format!("{}:{rest}", kind.name())
}

/// The last part of a grip's key: `at2` for bend 2, `new0` for the grip of piece 0.
fn bend_part(bend: Bend) -> String {
    match bend {
        Bend::At(n) => format!("at{n}"),
        Bend::New(n) => format!("new{n}"),
    }
}

/// The last part of an opening grip's key: the end of the opening it sits at.
fn end_part(end: OpeningEnd) -> &'static str {
    match end {
        OpeningEnd::Start => "start",
        OpeningEnd::End => "end",
    }
}

/// The last part of a lamp grip's key: which grip of the lamp it is.
fn light_grip_part(grip: LightGrip) -> &'static str {
    match grip {
        LightGrip::Direction => "direction",
        LightGrip::Beam => "beam",
    }
}

/// The last part of an object grip's key: the size it sets, or `point2` for point 2 and `side2` for side 2 of a form.
fn object_grip_part(grip: ObjectGrip) -> String {
    match grip {
        ObjectGrip::Width => "width".to_owned(),
        ObjectGrip::Depth => "depth".to_owned(),
        ObjectGrip::Point(n) => format!("point{n}"),
        ObjectGrip::Side(n) => format!("side{n}"),
    }
}

/// A key taken apart: `mark:elisabeth:0` is `Mark`, `elisabeth` and index 0.
pub struct ParsedKey<'a> {
    pub kind: ElementKind,
    pub id: &'a str,
    /// The third part, when it is a number.
    pub index: Option<usize>,
}

/// What `key` wrote, read back; `None` for a key of no known kind or without an id.
pub fn parse_key(key: &str) -> Option<ParsedKey<'_>> {
    let mut parts = key.split(':');
    let kind = ElementKind::from_name(parts.next()?)?;
    let id = parts.next()?;
    let index = parts.next().and_then(|i| i.parse().ok());

    Some(ParsedKey { kind, id, index })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grip_keys_name_track_index_and_grip() {
        let figure = Source::FigureVia {
            character: "e".into(),
            index: 1,
            bend: Bend::At(0),
        };
        let camera = Source::CameraVia {
            camera: "a".into(),
            index: 1,
            bend: Bend::New(2),
        };

        assert_eq!(key(&figure), "figurevia:e:1:at0");
        assert_eq!(key(&camera), "cameravia:a:1:new2");
        assert_eq!(key(&object_grip()), "objectgrip:o:depth");
        let point = Source::ObjectGrip {
            object: "o".into(),
            grip: ObjectGrip::Point(2),
        };
        assert_eq!(key(&point), "objectgrip:o:point2");
        let side = Source::ObjectGrip {
            object: "o".into(),
            grip: ObjectGrip::Side(1),
        };
        assert_eq!(key(&side), "objectgrip:o:side1");
    }

    fn opening() -> Source {
        Source::Opening { wall: 4, index: 1 }
    }

    fn opening_edge() -> Source {
        Source::OpeningEdge {
            wall: 4,
            index: 1,
            end: OpeningEnd::End,
        }
    }

    fn light_grip() -> Source {
        Source::LightGrip {
            light: "l".into(),
            grip: LightGrip::Beam,
        }
    }

    fn object_grip() -> Source {
        Source::ObjectGrip {
            object: "o".into(),
            grip: ObjectGrip::Depth,
        }
    }

    /// Too long for one line of the table below.
    fn note_width() -> Source {
        Source::NoteWidth("n".into())
    }

    #[test]
    fn an_image_and_its_grip_read_back() {
        for (source, kind) in [
            (Source::Image("i".into()), ElementKind::Image),
            (Source::ImageSize("i".into()), ElementKind::ImageSize),
        ] {
            let text = key(&source);
            let parsed = parse_key(&text);

            assert_eq!(parsed.map(|p| (p.kind, p.id)), Some((kind, "i")));
        }
    }

    /// Every kind of element reads back from the key it is written as.
    #[test]
    fn every_key_reads_back() {
        let sources = [
            (Source::Object("o".into()), ElementKind::Object, "o", None),
            (object_grip(), ElementKind::ObjectGrip, "o", None),
            (Source::Light("l".into()), ElementKind::Light, "l", None),
            (light_grip(), ElementKind::LightGrip, "l", None),
            (Source::Camera("a".into()), ElementKind::Camera, "a", None),
            (Source::Figure("e".into()), ElementKind::Figure, "e", None),
            (
                Source::Mark {
                    character: "e".into(),
                    index: 2,
                },
                ElementKind::Mark,
                "e",
                Some(2),
            ),
            (
                Source::FigureVia {
                    character: "e".into(),
                    index: 1,
                    bend: Bend::At(0),
                },
                ElementKind::FigureVia,
                "e",
                Some(1),
            ),
            (
                Source::CameraVia {
                    camera: "a".into(),
                    index: 3,
                    bend: Bend::New(1),
                },
                ElementKind::CameraVia,
                "a",
                Some(3),
            ),
            (Source::Wall(4), ElementKind::Wall, "4", None),
            (opening(), ElementKind::Opening, "4", Some(1)),
            (opening_edge(), ElementKind::OpeningEdge, "4", Some(1)),
            (Source::Note("n".into()), ElementKind::Note, "n", None),
            (note_width(), ElementKind::NoteWidth, "n", None),
            (Source::Drawing("d".into()), ElementKind::Drawing, "d", None),
            (
                Source::DrawingPoint {
                    drawing: "d".into(),
                    index: 2,
                },
                ElementKind::DrawingPoint,
                "d",
                Some(2),
            ),
            (
                Source::DrawingEdge {
                    drawing: "d".into(),
                    index: 1,
                },
                ElementKind::DrawingEdge,
                "d",
                Some(1),
            ),
        ];
        // Image and its grip read back in their own test (this table is at its length limit).
        assert_eq!(sources.len() + 2, ElementKind::ALL.len());

        for (source, kind, id, index) in sources {
            let text = key(&source);
            let parsed = parse_key(&text);

            let Some(parsed) = parsed else {
                panic!("{text} does not read back");
            };
            assert_eq!(parsed.kind, kind, "{text}");
            assert_eq!(parsed.id, id, "{text}");
            assert_eq!(parsed.index, index, "{text}");
        }
        assert!(parse_key("nonsense:x").is_none());
        assert!(parse_key("camera").is_none());
    }

    #[test]
    fn mark_keys_name_character_and_index() {
        let k = key(&Source::Mark {
            character: "e".into(),
            index: 2,
        });

        assert_eq!(k, "mark:e:2");
    }
}
