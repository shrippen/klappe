use serde::{Deserialize, Serialize};

use crate::geometry::Point;

/// One stroke of the sketch pen: the points the pen passed, in metres, in order (LH-120).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sketch {
    pub points: Vec<Point>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plan::FloorPlan;

    /// A plan without sketches writes no `sketches` key; a file without it has none (LH-120).
    #[test]
    fn sketches_are_optional_in_json() {
        let mut plan = FloorPlan::new("p", "Plan A");

        let json = serde_json::to_string(&plan).unwrap();
        let back: FloorPlan = serde_json::from_str(&json).unwrap();
        assert!(!json.contains("sketches"), "{json}");
        assert!(back.sketches.is_empty());

        plan.sketches.push(Sketch {
            points: vec![Point::new(1.0, 2.0), Point::new(3.0, 4.0)],
        });
        let json = serde_json::to_string(&plan).unwrap();
        assert_eq!(serde_json::from_str::<FloorPlan>(&json).unwrap(), plan);
    }
}
