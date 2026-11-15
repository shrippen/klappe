use klappe_model::Point;

use super::Snapping;

/// Grid spacing of the plan, in the order the tool bar offers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Grid {
    Cm5,
    Cm10,
    #[default]
    Cm25,
    Cm50,
    M1,
}

impl Grid {
    pub const ALL: [Grid; 5] = [Grid::Cm5, Grid::Cm10, Grid::Cm25, Grid::Cm50, Grid::M1];

    /// The spacing in whole centimetres.
    fn step_cm(self) -> f64 {
        match self {
            Grid::Cm5 => 5.0,
            Grid::Cm10 => 10.0,
            Grid::Cm25 => 25.0,
            Grid::Cm50 => 50.0,
            Grid::M1 => 100.0,
        }
    }

    /// The spacing in metres.
    pub fn step_m(self) -> f64 {
        self.step_cm() / CM_PER_M
    }

    /// The name the surface uses, e.g. `cm25`; also the end of the text key.
    pub fn name(self) -> &'static str {
        match self {
            Grid::Cm5 => "cm5",
            Grid::Cm10 => "cm10",
            Grid::Cm25 => "cm25",
            Grid::Cm50 => "cm50",
            Grid::M1 => "m1",
        }
    }

    /// The grid `name` names; `None` for any other text.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|g| g.name() == name)
    }
}

/// Centimetres in a metre.
const CM_PER_M: f64 = 100.0;

/// `at` on the grid when snapping is on.
pub(crate) fn snap(at: Point, snapping: Snapping, grid: Grid) -> Point {
    if snapping == Snapping::Off || !at.x_m.is_finite() || !at.y_m.is_finite() {
        return at;
    }

    // Whole centimetres: 0.1 * 3 would store 0.30000000000000004, 30 / 100 stores 0.3.
    let on_grid = |v: f64| ((v * CM_PER_M / grid.step_cm()).round() * grid.step_cm()) / CM_PER_M;
    Point::new(on_grid(at.x_m), on_grid(at.y_m))
}
