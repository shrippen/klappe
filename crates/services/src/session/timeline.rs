use klappe_model::{FloorPlan, LinkTarget};
use klappe_script::Element;
use klappe_stellprobe::Source;

use super::{Layer, Session};
use crate::error::ServiceError;
use crate::views::{Chip, LinkKind, Marker, ScriptLine, Segment, Track, TrackKind};

/// Id of the cut-order track.
const CUT_TRACK: &str = "cut";

/// Colours the cut order gives its cameras, in camera order.
const CAMERA_LABELS: [&str; 4] = ["yellow", "green", "cyan", "orange"];

/// Colour of the lights' tracks.
const LIGHT_LABEL: &str = "orange";

/// Colour of the cut track.
const CUT_LABEL: &str = "yellow";

impl Session {
    /// The timeline: cut order first, then figures, cameras and lights.
    pub fn tracks(&self) -> Vec<Track> {
        let Some(plan) = self.plan() else {
            return Vec::new();
        };
        let mut tracks = vec![self.cut_track(plan)];

        for figure in &plan.figures {
            let Some(character) = self.project.character(&figure.character) else {
                continue;
            };
            tracks.push(Track {
                kind: TrackKind::Figure,
                id: figure.character.clone(),
                name: character.title(),
                label: character.color.name().to_owned(),
                shown: self.track_shown(&figure.character, Layer::Figures),
                segments: walks(figure.marks.iter().map(|m| (m.at_s, m.hold_s))),
                markers: figure
                    .marks
                    .iter()
                    .enumerate()
                    .map(|(i, m)| Marker {
                        at_s: m.at_s,
                        text: format!("{} {}", character.short, i + 1),
                    })
                    .collect(),
            });
        }

        for (i, camera) in plan.cameras.iter().enumerate() {
            tracks.push(Track {
                kind: TrackKind::Camera,
                id: camera.id.clone(),
                name: camera.label.clone(),
                label: CAMERA_LABELS[i % CAMERA_LABELS.len()].to_owned(),
                shown: self.track_shown(&camera.id, Layer::Cameras),
                segments: walks(camera.marks.iter().map(|m| (m.at_s, m.hold_s))),
                markers: camera
                    .marks
                    .iter()
                    .enumerate()
                    .map(|(n, m)| Marker {
                        at_s: m.at_s,
                        text: format!("{} {}", camera.label, n + 1),
                    })
                    .collect(),
            });
        }

        for light in &plan.lights {
            // Bounce and flag make no light: no colour to name.
            let text = if light.kind.gives_light() {
                format!("{:.0} K", light.color_temp_k)
            } else {
                String::new()
            };
            tracks.push(Track {
                kind: TrackKind::Light,
                id: light.id.clone(),
                name: light.label.clone(),
                label: LIGHT_LABEL.to_owned(),
                shown: self.track_shown(&light.id, Layer::Lights),
                segments: vec![Segment {
                    from_s: 0.0,
                    to_s: self.duration_s(),
                    text,
                    key: String::new(),
                    selected: false,
                }],
                markers: Vec::new(),
            });
        }

        tracks
    }

    fn track_shown(&self, id: &str, layer: Layer) -> bool {
        !self.plan_view.track_hidden(id) && !self.plan_view.layer_hidden(layer)
    }

    fn cut_track(&self, plan: &FloorPlan) -> Track {
        let segments = plan
            .shots
            .iter()
            .map(|shot| {
                let camera = plan
                    .camera(&shot.camera)
                    .map(|c| c.label.as_str())
                    .unwrap_or("?");
                Segment {
                    from_s: shot.from_s,
                    to_s: shot.to_s,
                    text: format!("{} · {}", shot.number, camera),
                    key: shot.id.clone(),
                    selected: self.selected_shot.as_deref() == Some(&shot.id),
                }
            })
            .collect();

        Track {
            kind: TrackKind::Cut,
            id: CUT_TRACK.to_owned(),
            name: String::new(),
            label: CUT_LABEL.to_owned(),
            shown: true,
            segments,
            markers: Vec::new(),
        }
    }

    /// The elements of the current scene's script; none for a scene outside the book.
    pub(super) fn scene_elements(&self) -> &[Element] {
        let Some(at) = self.book_scene_of(self.scene) else {
            return &[];
        };

        self.script
            .scenes()
            .get(at)
            .map_or(&[], |view| view.elements)
    }

    /// The earliest time the scene's element `index` is linked to.
    pub(super) fn link_s(&self, index: usize) -> Option<f64> {
        let chips = self.plan().map(|p| self.chips(p, index))?;

        chips.iter().map(|c| c.at_s).min_by(f64::total_cmp)
    }

    /// The current scene's script with its links to marks and shots (design B).
    pub fn script_lines(&self) -> Vec<ScriptLine> {
        let mut lines: Vec<ScriptLine> = self
            .scene_elements()
            .iter()
            .enumerate()
            .map(|(index, element)| {
                let (kind, text) = line_of(element);
                ScriptLine {
                    index,
                    kind,
                    text,
                    // A scene without a plan has no links yet.
                    chips: self
                        .plan()
                        .map(|p| self.chips(p, index))
                        .unwrap_or_default(),
                    current: false,
                    character: self.cue_character(element),
                }
            })
            .collect();

        mark_current(&mut lines, self.time_s);
        lines
    }

    /// Goes to what a script line's chip links: a mark is the one to select and the time
    /// goes to its arrival; a shot is selected as in the shot list, so nothing else stays
    /// selected (`None`). `index` counts for marks only.
    pub fn show_link(
        &mut self,
        kind: LinkKind,
        target: &str,
        index: usize,
    ) -> Result<Option<Source>, ServiceError> {
        if kind == LinkKind::Shot {
            self.select_shot(target)?;
            return Ok(None);
        }

        let mark = self
            .plan()
            .and_then(|plan| plan.figure(target))
            .and_then(|figure| figure.marks.get(index))
            .ok_or(ServiceError::Gone)?;

        self.set_time(mark.at_s);
        Ok(Some(Source::Mark {
            character: target.to_owned(),
            index,
        }))
    }

    fn chips(&self, plan: &FloorPlan, index: usize) -> Vec<Chip> {
        plan.links
            .iter()
            .filter(|l| l.element == index)
            .filter_map(|l| match &l.target {
                LinkTarget::Mark { character, index } => {
                    let c = self.project.character(character)?;
                    let mark = plan.figure(character)?.marks.get(*index)?;
                    Some(Chip {
                        kind: LinkKind::Mark,
                        text: format!("{} · M{}", c.short, index + 1),
                        target: character.clone(),
                        label: c.color.name().to_owned(),
                        index: *index,
                        at_s: mark.at_s,
                    })
                }
                LinkTarget::Shot { id } => {
                    let shot = plan.shots.iter().find(|s| &s.id == id)?;
                    Some(Chip {
                        kind: LinkKind::Shot,
                        text: shot.number.clone(),
                        target: id.clone(),
                        label: String::new(),
                        index: 0,
                        at_s: shot.from_s,
                    })
                }
            })
            .collect()
    }
}

/// Moving stretches between marks: leave after the hold, arrive at the next mark.
fn walks(marks: impl Iterator<Item = (f64, f64)>) -> Vec<Segment> {
    let marks: Vec<(f64, f64)> = marks.collect();

    marks
        .windows(2)
        .filter_map(|pair| {
            let ((at, hold), (next, _)) = (pair[0], pair[1]);
            let leave = (at + hold).min(next);
            (next > leave).then(|| Segment {
                from_s: leave,
                to_s: next,
                text: String::new(),
                key: String::new(),
                selected: false,
            })
        })
        .collect()
}

fn line_of(element: &Element) -> (&'static str, String) {
    match element {
        Element::Action(t) => ("action", t.clone()),
        Element::Character {
            name, extension, ..
        } => match extension {
            Some(ext) => ("character", format!("{name} ({ext})")),
            None => ("character", name.clone()),
        },
        Element::Parenthetical(t) => ("parenthetical", t.clone()),
        Element::Dialogue(t) => ("dialogue", t.clone()),
        Element::Transition(t) => ("transition", t.clone()),
        Element::SceneHeading { text, .. } => ("other", text.clone()),
        Element::Centered(t) | Element::Synopsis(t) | Element::Note(t) | Element::Lyrics(t) => {
            ("other", t.clone())
        }
        Element::Section { text, .. } => ("other", text.clone()),
        Element::PageBreak => ("other", String::new()),
    }
}

/// The last line with a link whose time has come is current.
fn mark_current(lines: &mut [ScriptLine], t_s: f64) {
    let current = lines
        .iter()
        .enumerate()
        .filter_map(|(i, l)| {
            let start = l.chips.iter().map(|c| c.at_s).fold(f64::INFINITY, f64::min);
            (start <= t_s).then_some((i, start))
        })
        .max_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)))
        .map(|(i, _)| i);

    if let Some(i) = current {
        lines[i].current = true;
    }
}
