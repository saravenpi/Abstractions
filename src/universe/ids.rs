//! Stable identifiers of galaxies, stars and planets: enough to regenerate each from the seed.

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

/// A planet, named by its star and its rank outward from that star.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct BodyId {
    /// Star the planet orbits.
    pub star: StarId,
    /// Rank of the planet, 0 being the closest to the star.
    pub index: u8,
}
