//! Cover preference: grazers crop the grass of open land, browsers the leaves of woods.
//!
//! Large grazers live mostly in open grassland and browsers in closed woodland, as in African
//! antelope (Jarman 1974, re-tested by Brashares and others 2000,
//! <https://biology.ucr.edu/people/faculty/Garland/BrasEA00.pdf>).

use crate::life::{Guild, Habitat, Species};
use crate::world::Biome;

/// Returns how well land of `biome` suits the feeding of `species`, 0 to 1: grazers like open
/// meadows, browsers like woods, and every other species takes any cover.
pub(super) fn cover(species: &Species, biome: Biome) -> f32 {
    if species.role.habitat == Habitat::Water {
        return 1.0;
    }
    match species.role.guild {
        Guild::Grazer => grass(biome),
        Guild::Browser => leaves(biome),
        _ => 1.0,
    }
}

/// Returns how much grass `biome` holds for grazers, 0 to 1.
fn grass(biome: Biome) -> f32 {
    match biome {
        Biome::Meadow => 1.0,
        Biome::Tropical => 0.7,
        Biome::Cherry => 0.6,
        Biome::Taiga => 0.5,
        Biome::Desert => 0.4,
        Biome::Forest => 0.3,
    }
}

/// Returns how many leaves `biome` holds for browsers, 0 to 1.
fn leaves(biome: Biome) -> f32 {
    match biome {
        Biome::Forest | Biome::Tropical => 1.0,
        Biome::Cherry | Biome::Taiga => 0.8,
        Biome::Meadow => 0.5,
        Biome::Desert => 0.3,
    }
}
