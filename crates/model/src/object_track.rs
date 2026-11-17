//! The way of a set object through a scene (LH-24).

use serde::{Deserialize, Serialize};

use crate::geometry::Point;

/// The way of a set object through the scene: marks in time order (LH-24). Without marks
/// the object stays where the set puts it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObjectTrack {
    /// Id of the object in the set of the scene's location.
    pub object: String,
    pub marks: Vec<ObjectMark>,
}

/// Where an object is at a moment: arrives at `at_s`, stays until it moves on to the next
/// mark. The way between two marks is straight.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObjectMark {
    pub at_s: f64,
    pub at: Point,
    pub rotation_deg: f64,
    /// Seconds the object stays before it moves on.
    #[serde(default)]
    pub hold_s: f64,
    /// Id of the character who carries the object from this mark until the next one: it
    /// keeps its offset to that figure (LH-24).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub carried_by: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plan::FloorPlan;

    /// An object's track counts in the scene's length like a figure's (LH-24).
    #[test]
    fn an_object_track_ends_the_scene_too() {
        let mut plan = FloorPlan::new("p", "Plan A");
        plan.object_tracks = vec![ObjectTrack {
            object: "o".into(),
            marks: vec![ObjectMark {
                at_s: 7.0,
                at: Point::new(0.0, 0.0),
                rotation_deg: 0.0,
                hold_s: 2.0,
                carried_by: None,
            }],
        }];

        assert_eq!(plan.duration_s(), 9.0);
    }
}
