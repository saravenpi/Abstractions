//! Positions in the universe: a galaxy grid cell plus 64-bit metres inside it.

use std::ops::{Add, Sub};

use bevy::math::{DVec3, IVec3};

use crate::universe::GALAXY_CELL;

/// A point anywhere in the universe.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct UniPos {
    /// Galaxy grid cell, each `GALAXY_CELL` metres wide.
    pub cell: IVec3,
    /// Metres from the minimum corner of the cell.
    pub local: DVec3,
}

impl UniPos {
    /// Builds a position and moves whole cells out of `local`, so that it lies inside the cell.
    pub fn new(cell: IVec3, local: DVec3) -> Self {
        let carry = (local / GALAXY_CELL).floor();
        Self {
            cell: cell + carry.as_ivec3(),
            local: local - carry * GALAXY_CELL,
        }
    }

    /// Returns the distance to `other` in metres.
    pub fn distance(self, other: Self) -> f64 {
        (self - other).length()
    }
}

impl Add<DVec3> for UniPos {
    type Output = UniPos;

    /// Returns the position moved by `delta` metres.
    fn add(self, delta: DVec3) -> UniPos {
        UniPos::new(self.cell, self.local + delta)
    }
}

impl Sub<DVec3> for UniPos {
    type Output = UniPos;

    /// Returns the position moved back by `delta` metres.
    fn sub(self, delta: DVec3) -> UniPos {
        UniPos::new(self.cell, self.local - delta)
    }
}

impl Sub for UniPos {
    type Output = DVec3;

    /// Returns the metres from `other` to this position.
    fn sub(self, other: UniPos) -> DVec3 {
        (self.cell - other.cell).as_dvec3() * GALAXY_CELL + (self.local - other.local)
    }
}
