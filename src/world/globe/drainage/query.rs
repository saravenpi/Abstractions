//! Reading the drainage grid: where a direction falls, which cells are land, coast or river, and the path
//! a drop of water takes to the sea.

use bevy::math::DVec3;

use super::Drainage;
use super::grid::Grid;
use super::tune::RIVER;

impl Drainage {
    /// Returns the grid.
    pub fn grid(&self) -> Grid {
        self.grid
    }

    /// Returns the cell holding unit direction `dir`.
    pub fn find(&self, dir: DVec3) -> usize {
        self.grid.find(dir)
    }

    /// Returns the unit direction of the middle of `cell`.
    pub fn dir(&self, cell: usize) -> DVec3 {
        self.grid.dir(cell)
    }

    /// Returns true when `cell` is above the sea.
    pub fn is_land(&self, cell: usize) -> bool {
        self.height[cell] > self.sea
    }

    /// Returns the cell the water of `cell` flows into, or nothing at the sea and at the bottom of a basin.
    pub fn next(&self, cell: usize) -> Option<usize> {
        let to = self.next[cell] as usize;
        (to != cell).then_some(to)
    }

    /// Returns true when `cell` is land that drains at least [`RIVER`] cells of land: a trunk river runs
    /// through it.
    pub fn is_river(&self, cell: usize) -> bool {
        self.is_land(cell) && self.area[cell] >= RIVER
    }

    /// Returns a sea cell next to the land cell `cell`, or nothing inland.
    pub fn sea_beside(&self, cell: usize) -> Option<usize> {
        let beside = self.grid.around(cell);
        beside
            .into_iter()
            .find(|k| !self.is_land(*k))
            .filter(|_| self.is_land(cell))
    }

    /// Returns true when every cell of the block of nine around the one holding `dir` is sea and a cell is
    /// wider than `reach` metres on a planet of `radius` metres, so no land lies within `reach` of `dir`
    /// but for islands smaller than a cell.
    pub fn sea_around(&self, dir: DVec3, reach: f64, radius: f64) -> bool {
        let cell = radius * std::f64::consts::FRAC_PI_2 / f64::from(self.grid.n);
        let middle = self.find(dir);
        reach <= cell
            && !self.is_land(middle)
            && self
                .grid
                .around(middle)
                .into_iter()
                .all(|k| !self.is_land(k))
    }
}
