//! Positions in the universe: a galaxy grid cell, a star sector inside it, and 64-bit metres
//! inside the sector.
//!
//! One 64-bit float cannot hold a point of a galaxy cell (2.8e22 m) to the metre: its step there
//! is 4e6 m. The metres inside one sector (at most 9.46e16 m) have a step of 16 m at the far end
//! and finer closer in. The sector and the cell are whole numbers, so they lose nothing. A
//! difference of two positions is exact in the integers and rounds once, to 1e-16 of the result,
//! so two points a metre apart stay a metre apart wherever they lie in the galaxy.

use std::ops::{Add, Sub};

use bevy::math::{DVec3, IVec3};

use crate::universe::SECTOR_M;

/// Sectors along one side of a galaxy cell: `GALAXY_CELL / SECTOR_M`.
pub const SECTORS_PER_CELL: i32 = 300_000;

/// A point anywhere in the universe.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct UniPos {
    /// Galaxy grid cell, each `GALAXY_CELL` metres wide.
    pub cell: IVec3,
    /// Sector inside the cell, from 0 to `SECTORS_PER_CELL - 1` on each axis, each `SECTOR_M`
    /// metres wide. It counts from the minimum corner of the cell.
    pub sector: IVec3,
    /// Metres from the minimum corner of the sector, from 0 to `SECTOR_M`.
    pub local: DVec3,
}

/// Splits `metres` into whole sectors and the metres left, from 0 to `SECTOR_M`.
///
/// The product `sectors * SECTOR_M` is fused with the subtraction, so the rest has one rounding.
fn split(metres: f64) -> (i32, f64) {
    let mut sectors = (metres / SECTOR_M).floor();
    let mut rest = (-sectors).mul_add(SECTOR_M, metres);
    if rest < 0.0 {
        sectors -= 1.0;
        rest += SECTOR_M;
    } else if rest >= SECTOR_M {
        sectors += 1.0;
        rest -= SECTOR_M;
    }
    (sectors as i32, rest)
}

impl UniPos {
    /// Builds a position from a cell and metres from its minimum corner. Whole sectors and whole
    /// cells are moved out of `local`, so that the position is stored in its three parts.
    pub fn new(cell: IVec3, local: DVec3) -> Self {
        Self::sectors(cell, IVec3::ZERO, local)
    }

    /// Builds a position from a cell, a sector counted from the corner of the cell, and metres
    /// from the corner of that sector. Each part may lie outside its range: it is carried into
    /// the next one.
    pub fn sectors(cell: IVec3, sector: IVec3, local: DVec3) -> Self {
        let [(cx, x), (cy, y), (cz, z)] = local.to_array().map(split);
        let sector = sector + IVec3::new(cx, cy, cz);
        let whole = IVec3::splat(SECTORS_PER_CELL);
        Self {
            cell: cell + sector.div_euclid(whole),
            sector: sector.rem_euclid(whole),
            local: DVec3::new(x, y, z),
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
        UniPos::sectors(self.cell, self.sector, self.local + delta)
    }
}

impl Sub<DVec3> for UniPos {
    type Output = UniPos;

    /// Returns the position moved back by `delta` metres.
    fn sub(self, delta: DVec3) -> UniPos {
        UniPos::sectors(self.cell, self.sector, self.local - delta)
    }
}

impl Sub for UniPos {
    type Output = DVec3;

    /// Returns the metres from `other` to this position.
    ///
    /// The cells and sectors are subtracted as whole numbers and turned into metres once, then
    /// the metres inside the sectors are added.
    fn sub(self, other: UniPos) -> DVec3 {
        let cells = (self.cell - other.cell).as_dvec3() * f64::from(SECTORS_PER_CELL);
        let sectors = cells + (self.sector - other.sector).as_dvec3();
        sectors * SECTOR_M + (self.local - other.local)
    }
}
