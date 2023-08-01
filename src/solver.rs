use std::fmt;

use crate::grid::Grid;

pub struct Solver {
    grid: Grid,
}

impl fmt::Display for Solver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.grid)
    }
}

impl Solver {
    #[must_use]
    pub fn new(grid: &Grid) -> Self {
        Solver { grid: grid.clone() }
    }
}
