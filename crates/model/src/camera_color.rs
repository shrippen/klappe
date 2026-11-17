//! The colour of a camera's shots (LH-111).

use crate::plan::FloorPlan;
use crate::project::FigureColor;

impl FloorPlan {
    /// The colour of camera `id` (LH-111): its own, else by its place in the plan from the
    /// colours Kante offers, round again after the last.
    pub fn camera_color(&self, id: &str) -> Option<FigureColor> {
        let at = self.cameras.iter().position(|c| c.id == id)?;
        let own = self.cameras[at].color;

        own.or(Some(FigureColor::ALL[at % FigureColor::ALL.len()]))
    }
}
