//! Moons: the bodies that orbit a planet, each a planet spec of its own with its host's orbit.

use super::spec::{PlanetSpec, StarSpec};
use super::{BodyId, Universe};

/// Returns the moons of `planet`, a planet of `star`, ordered outward. Every moon has
/// `id.moon` from 1 up and `host` set to the orbit of its planet.
pub(super) fn of(_star: &StarSpec, _planet: &PlanetSpec) -> Vec<PlanetSpec> {
    Vec::new()
}

impl Universe {
    /// Returns the moons of the planet that `id` is or orbits, ordered outward.
    pub fn moons(&self, id: BodyId) -> Vec<PlanetSpec> {
        let system = self.system(id.star);
        let planet = self.planet(id.planet());
        of(&system.star, &planet)
    }
}
