//! Flows on the globe: the channels rivers and lava cut into the sphere, and the surfaces they leave.

use bevy::math::DVec3;

use super::Globe;
use super::atlas::Flowing;
use super::landform::Landform;
use crate::world::SEA_LEVEL;
use crate::world::surface::Surface;

impl Globe {
    /// Returns what runs down the high ground of the planet.
    pub(super) fn flowing(&self) -> Flowing {
        Flowing::of(&self.record, self.has_sea())
    }

    /// Returns the ground `height` at sphere point `p` with the channels of the flows cut in. Ground at or
    /// under the sea is left as it is, without reading any flow.
    pub(super) fn flowed(&self, p: DVec3, height: f32) -> f32 {
        if height <= SEA_LEVEL || self.flowing() == Flowing::Nothing {
            return height;
        }
        let dir = p / self.radius();
        self.with_map(dir, |map, local| map.carve(p, local, height))
    }

    /// Returns the surface of a channel bed in unit direction `dir`, or nothing outside every bed.
    pub(in crate::world) fn mark(&self, dir: DVec3) -> Option<Surface> {
        let flowing = self.flowing();
        if flowing == Flowing::Nothing {
            return None;
        }
        let p = dir * self.radius();
        self.with_map(dir, |map, local| map.bed(p, local))
            .then(|| flowing.bed())
    }

    /// Returns the surface the landform or a flow puts on ground `height` high at sphere point `p`, if any:
    /// the frozen sea on a world of ice, a channel bed above the sea.
    pub(super) fn marked(&self, p: DVec3, height: f32) -> Option<Surface> {
        let frozen = self.landform() == Landform::Glaciers && !self.has_sea();
        if frozen && height <= SEA_LEVEL {
            return Some(Surface::Snow);
        }
        (height > SEA_LEVEL)
            .then(|| self.mark(p / self.radius()))
            .flatten()
    }
}
