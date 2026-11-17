//! Which terrain level a chunk gets from its distance to the focus, in rings of chunks.

use super::lod::Lod;

/// The last ring of each terrain level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rings {
    /// Last ring drawn with the finest grid.
    pub near: i32,
    /// Last ring drawn with the middle grid.
    pub mid: i32,
    /// Last ring that draws the bare sea bed, under the near sea. Farther chunks draw a flat sea.
    pub bare: i32,
}

impl Rings {
    /// The rings of the default render distance, where no chunk lies past the near sea.
    pub const DEFAULT: Rings = Rings {
        near: 1,
        mid: 3,
        bare: i32::MAX,
    };

    /// Returns the level wanted for a chunk `ring` chunks away from the focus.
    pub fn lod(&self, ring: i32) -> Lod {
        if ring > self.bare {
            Lod::Distant
        } else if ring <= self.near {
            Lod::Near
        } else if ring <= self.mid {
            Lod::Mid
        } else {
            Lod::Far
        }
    }

    /// Returns the level a chunk at `ring` should use now: finer levels apply at once, coarser
    /// ones one ring later, so a focus that paces a border changes nothing.
    pub fn next(&self, current: Lod, ring: i32) -> Lod {
        let wanted = self.lod(ring);
        if wanted < current {
            wanted
        } else {
            current.max(self.lod(ring - 1))
        }
    }
}
