//! Stable identifiers of galaxies, stars, planets and moons: enough to regenerate each from the seed.

use bevy::math::IVec3;
use serde::{Deserialize, Serialize};

/// A galaxy, named by the universe grid cell that holds it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct GalaxyId(pub IVec3);

/// A star, named by its galaxy, its sector in that galaxy and its rank in the sector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct StarId {
    /// Galaxy the star belongs to.
    pub galaxy: GalaxyId,
    /// Sector of the galaxy, in steps of `SECTOR_GLY` game light-years from the galaxy centre.
    pub sector: IVec3,
    /// Rank of the star inside its sector.
    pub index: u8,
}

/// A planet or one of its moons, named by its star, the rank of the planet outward from that
/// star, and the rank of the moon outward from the planet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct BodyId {
    /// Star the planet orbits.
    pub star: StarId,
    /// Rank of the planet, 0 being the closest to the star.
    pub index: u8,
    /// 0 for the planet itself, else 1 for its closest moon, 2 for the next one, and so on.
    /// Saves written before moons existed have no such field and name the planet.
    #[serde(default)]
    pub moon: u8,
}

impl BodyId {
    /// Returns the planet this body is or orbits.
    pub fn planet(self) -> Self {
        Self { moon: 0, ..self }
    }

    /// Returns true when this body is a moon.
    pub fn is_moon(self) -> bool {
        self.moon > 0
    }
}
