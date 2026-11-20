//! The ground rules, written once for the legacy plane and for the sphere.

use bevy::math::Vec2;

use super::Globe;
use super::site::Site;
use crate::world::SEA_LEVEL;
use crate::world::biome::Biome;
use crate::world::surface::Surface;

/// Finite difference step used to measure slopes, in metres.
const SLOPE_STEP: f32 = 1.5;

impl Globe {
    /// Returns the ground height in metres above sea level at `at`, ponds carved in. A planet without
    /// ground is its smooth deck, at height 0 everywhere, read without sampling any noise.
    pub(in crate::world) fn ground<S: Site>(&self, at: S) -> f32 {
        if !self.has_ground() {
            return SEA_LEVEL;
        }
        self.carve(at, self.uncarved(at))
    }

    /// Returns the ground height in metres above sea level at `at` before any pond is carved: the shape
    /// of the land, moved by the sea datum of the planet. On the sphere the landform of the class shapes it
    /// and the flows cut their channels.
    pub(in crate::world) fn uncarved<S: Site>(&self, at: S) -> f32 {
        match at.ball() {
            Some(p) => self.flowed(p, self.shaped(p)),
            None => self.shape.elevation(at.point(), &self.climate) - self.relief.datum,
        }
    }

    /// Returns the biome at `at`, also over water: the one biome of the class, or else on the sphere what
    /// the annual mean climate gives, and on the legacy plane what its climate noise says.
    pub(in crate::world) fn biome_on<S: Site>(&self, at: S) -> Biome {
        self.lands.biome().unwrap_or_else(|| match at.ball() {
            Some(p) => self.biome_at(p),
            None => {
                let p = at.point();
                self.climate.biome(p, self.shape.base(p))
            }
        })
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
        let surface = self.lands.surface(h, dx.hypot(dz), self.biome_on(at));
        at.ball().and_then(|p| self.marked(p, h)).unwrap_or(surface)
    }
}
