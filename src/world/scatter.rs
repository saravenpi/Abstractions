//! Deterministic placement of trees, rocks, bushes and grass inside one chunk.

use bevy::prelude::*;
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;

use super::islands::islands_near;
use super::{CHUNK_SIZE, Surface, Terrain};

/// Position and look of one prop, in chunk space (origin at the chunk center, `y` is world height).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placement {
    /// Chunk-space position on the ground.
    pub pos: Vec3,
    /// Rotation around the vertical axis in radians.
    pub yaw: f32,
    /// Uniform scale.
    pub scale: f32,
}

/// Everything that stands on one chunk.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Scatter {
    /// Trees, standing on grass.
    pub trees: Vec<Placement>,
    /// Rocks, on any dry ground.
    pub rocks: Vec<Placement>,
    /// Bushes, standing on grass.
    pub bushes: Vec<Placement>,
    /// Grass tufts, standing on grass.
    pub grass: Vec<Placement>,
}

/// Returns the world position of the center of chunk `coord`.
pub fn chunk_center(coord: IVec2) -> Vec2 {
    (coord.as_vec2() + Vec2::splat(0.5)) * CHUNK_SIZE
}

/// Returns true when some island is close enough to put dry land inside the chunk.
fn near_land(terrain: &Terrain, center: Vec2) -> bool {
    let reach = CHUNK_SIZE * 0.75;
    islands_near(terrain.seed(), center.x, center.y)
        .any(|i| i.center.distance(center) < i.radius + reach)
}

/// Draws up to `tries` random points of the chunk and keeps those on a wanted surface.
fn pick(
    terrain: &Terrain,
    rng: &mut ChaCha8Rng,
    center: Vec2,
    tries: u32,
    wants: &[Surface],
) -> Vec<Placement> {
    let half = CHUNK_SIZE * 0.5;
    (0..tries)
        .filter_map(|_| {
            let local = Vec2::new(rng.random_range(-half..half), rng.random_range(-half..half));
            let (yaw, scale) = (
                rng.random_range(0.0..std::f32::consts::TAU),
                rng.random_range(0.8..1.3),
            );
            let world = center + local;
            wants
                .contains(&terrain.surface(world.x, world.y))
                .then(|| Placement {
                    pos: Vec3::new(local.x, terrain.height(world.x, world.y), local.y),
                    yaw,
                    scale,
                })
        })
        .collect()
}

/// Places every prop of chunk `coord`. The result only depends on the terrain seed and the coordinate.
pub fn scatter_chunk(terrain: &Terrain, coord: IVec2) -> Scatter {
    let center = chunk_center(coord);
    if !near_land(terrain, center) {
        return Scatter::default();
    }
    let mix = (coord.x as u32 as u64) << 32 | coord.y as u32 as u64;
    let mut rng =
        ChaCha8Rng::seed_from_u64(terrain.seed().wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ mix);
    let land = [Surface::Grass, Surface::Rock, Surface::Sand];
    Scatter {
        trees: pick(terrain, &mut rng, center, 28, &[Surface::Grass]),
        rocks: pick(terrain, &mut rng, center, 10, &land),
        bushes: pick(terrain, &mut rng, center, 18, &[Surface::Grass]),
        grass: pick(terrain, &mut rng, center, 260, &[Surface::Grass]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The same seed and chunk always give the same props, and islands get some.
    #[test]
    fn scatter_is_deterministic() {
        let t = Terrain::new(5);
        let spawn = t.find_spawn();
        let coord = IVec2::new(
            (spawn.x / CHUNK_SIZE).floor() as i32,
            (spawn.z / CHUNK_SIZE).floor() as i32,
        );
        let a = scatter_chunk(&t, coord);
        assert_eq!(a, scatter_chunk(&Terrain::new(5), coord));
        assert!(!a.grass.is_empty());
    }

    /// Open sea holds no props.
    #[test]
    fn deep_sea_is_empty() {
        let t = Terrain::new(5);
        let far = IVec2::new(100_000, 100_000);
        assert_eq!(scatter_chunk(&t, far), Scatter::default());
    }
}
