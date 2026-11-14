use serde::{Deserialize, Serialize};

/// A camera body format to choose from (LH-32): the sensor it sets and the frame it suggests.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CameraBody {
    pub id: String,
    /// A format's name is the same in every language, e.g. `Super 35`.
    pub name: String,
    pub width_mm: f64,
    /// Width to height the body is usually shot in, e.g. 2.39.
    pub aspect: f64,
}

/// A set of lenses to choose from (LH-32): prime focal lengths, or a zoom range. Named by the
/// catalogue (`lensSet.<id>`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LensSet {
    pub id: String,
    /// Focal lengths of a prime set, shortest first; empty for a zoom.
    #[serde(default)]
    pub primes_mm: Vec<f64>,
    /// Shortest and longest focal length of a zoom.
    #[serde(default)]
    pub zoom_mm: Option<[f64; 2]>,
}

impl LensSet {
    /// The focal lengths to offer: the primes, or the two ends of a zoom.
    pub fn stops_mm(&self) -> Vec<f64> {
        match self.zoom_mm {
            Some(ends) => ends.to_vec(),
            None => self.primes_mm.clone(),
        }
    }
}

/// The bodies and lens sets Klappe ships with.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CameraPresets {
    pub bodies: Vec<CameraBody>,
    pub lens_sets: Vec<LensSet>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A prime set offers its focal lengths, a zoom its two ends.
    #[test]
    fn a_set_offers_its_stops() {
        let primes = LensSet {
            id: "p".into(),
            primes_mm: vec![25.0, 50.0],
            zoom_mm: None,
        };
        let zoom = LensSet {
            id: "z".into(),
            primes_mm: vec![],
            zoom_mm: Some([24.0, 70.0]),
        };

        assert_eq!(primes.stops_mm(), [25.0, 50.0]);
        assert_eq!(zoom.stops_mm(), [24.0, 70.0]);
    }
}
