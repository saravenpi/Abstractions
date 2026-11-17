//! The ground rules, written once for the legacy plane and for the sphere.

use bevy::math::Vec2;

use super::Globe;
use super::site::Site;
use crate::world::SEA_LEVEL;
use crate::world::biome::Biome;
use crate::world::surface::{Surface, classify};

/// Finite difference step used to measure slopes, in metres.
const SLOPE_STEP: f32 = 1.5;

impl Globe {
    /// Returns the ground height in metres above sea level at `at`, ponds carved in.
    pub(in crate::world) fn ground<S: Site>(&self, at: S) -> f32 {
        self.carve(at, self.shape.elevation(at.point(), &self.climate))
    }

    /// Returns the biome at `at`, also over water.
    pub(in crate::world) fn biome_on<S: Site>(&self, at: S) -> Biome {
        let p = at.point();
        self.climate.biome(p, self.shape.base(p))
    }

    /// Returns what covers the ground at `at`, the slope being measured along the two local axes.
    pub(in crate::world) fn cover<S: Site>(&self, at: S) -> Surface {
        let h = self.ground(at);
        if h <= SEA_LEVEL && self.in_pond(at) {
            return Surface::Pond;
        }
        let e = SLOPE_STEP;
        let rise = |by: Vec2| (self.ground(at.moved(by)) - self.ground(at.moved(-by))) / (2.0 * e);
        let (dx, dz) = (rise(Vec2::new(e, 0.0)), rise(Vec2::new(0.0, e)));
        classify(h, dx.hypot(dz), self.biome_on(at))
    }
}
