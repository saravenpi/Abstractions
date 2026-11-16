//! Ground classification from height, slope and biome, and the spawn search.

use bevy::prelude::*;

use super::biome::Biome;
use super::{SEA_LEVEL, Terrain};

/// Slope (rise over run) above which ground counts as rock.
const ROCK_SLOPE: f32 = 1.0;

/// Height above sea level above which ground counts as rock.
const ROCK_HEIGHT: f32 = 38.0;

/// Height above sea level above which ground counts as snow.
const SNOW_LINE: f32 = 55.0;

/// Height above sea level below which land counts as sand.
const SAND_HEIGHT: f32 = 1.5;

/// Spacing of the spawn search rings.
const SPAWN_STEP: f32 = 16.0;

/// What covers the ground at a point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Surface {
    /// Below sea level.
    Water,
    /// Beach near the shore, and all of the desert.
    Sand,
    /// Flat land above the beach.
    Grass,
    /// Steep or high land.
    Rock,
    /// Snow on cold or very high land.
    Snow,
}

/// Returns the surface for a ground height, a slope (rise over run) and a biome.
pub fn classify(height: f32, slope: f32, biome: Biome) -> Surface {
    if height <= SEA_LEVEL {
        Surface::Water
    } else if slope > ROCK_SLOPE {
        Surface::Rock
    } else if height > SEA_LEVEL + SNOW_LINE
        || (biome == Biome::Taiga && height > SEA_LEVEL + SAND_HEIGHT)
    {
        Surface::Snow
    } else if height > SEA_LEVEL + ROCK_HEIGHT {
        Surface::Rock
    } else if biome == Biome::Desert || height < SEA_LEVEL + SAND_HEIGHT {
        Surface::Sand
    } else {
        Surface::Grass
    }
}

impl Terrain {
    /// Returns true when `(x, z)` is gentle dry ground of grass or sand with dry neighbours.
    fn good_spawn(&self, x: f32, z: f32) -> bool {
        let dry = |dx: f32, dz: f32| self.height(x + dx, z + dz) > SEA_LEVEL + 1.5;
        matches!(self.surface(x, z), Surface::Grass | Surface::Sand)
            && self.height(x, z) >= SEA_LEVEL + 2.0
            && [(6.0, 0.0), (-6.0, 0.0), (0.0, 6.0), (0.0, -6.0)]
                .iter()
                .all(|&(dx, dz)| dry(dx, dz))
    }

    /// Returns the nearest good dry spot to the origin, searching outward in rings.
    pub fn find_spawn(&self) -> Vec3 {
        let ring = |k: u32| {
            let n = (k * 6).max(1);
            (0..n).map(move |a| {
                Vec2::from_angle(a as f32 * std::f32::consts::TAU / n as f32)
                    * (k as f32 * SPAWN_STEP)
            })
        };
        let spot = (0..600)
            .flat_map(ring)
            .find(|p| self.good_spawn(p.x, p.y))
            .unwrap_or(Vec2::ZERO);
        Vec3::new(spot.x, self.height(spot.x, spot.y), spot.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every spawn is on dry land.
    #[test]
    fn spawn_is_always_land() {
        for seed in 0..50 {
            let t = Terrain::new(seed);
            let p = t.find_spawn();
            assert!(p.y >= SEA_LEVEL + 2.0, "seed {seed} spawn {p}");
            assert_eq!(p, Terrain::new(seed).find_spawn());
        }
    }
}
