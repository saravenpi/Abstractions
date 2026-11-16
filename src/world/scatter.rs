//! Deterministic placement of plants, rocks and sea life inside one chunk, by biome, surface, depth and patch density.

mod density;
#[cfg(test)]
mod rules;
mod spacing;
mod table;
#[cfg(test)]
mod tests;

use bevy::prelude::*;
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;

use self::density::{Fields, choose};
use self::spacing::{crowded, ground};
use self::table::plants_for;
use super::propmesh::{Plant, VARIANTS};
use super::props::small_tris;
use super::{CHUNK_SIZE, SEA_LEVEL, Surface, Terrain};

/// Most big props on one chunk, sea floor ones included.
const BIG_CAP: usize = 55;

/// Most small plants on one chunk.
const SMALL_CAP: usize = 650;

/// Most triangles the merged ground-cover mesh of one chunk gets, sea life included.
const TRI_BUDGET: usize = 7000;

/// Sea floors deeper than this get nothing.
const DEEP_LIMIT: f32 = 22.0;

/// Highest ground a willow may stand on.
const WILLOW_CEILING: f32 = 5.0;

/// Distance from a willow to the pond it leans over.
const WILLOW_REACH: f32 = 6.0;

/// Returns true when a pond floor lies on a circle of `WILLOW_REACH` meters around `w`.
fn near_pond(terrain: &Terrain, w: Vec2) -> bool {
    (0..6).any(|k| {
        let q = w + Vec2::from_angle(k as f32 / 6.0 * std::f32::consts::TAU) * WILLOW_REACH;
        terrain.surface(q.x, q.y) == Surface::Pond
    })
}

/// Position and look of one prop, in chunk space (origin at the chunk center, `y` is world height).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placement {
    /// Chunk-space position on the ground.
    pub pos: Vec3,
    /// Rotation around the vertical axis in radians.
    pub yaw: f32,
    /// Uniform scale, or the height in meters for kelp.
    pub scale: f32,
    /// What stands here.
    pub plant: Plant,
    /// Which of the shared mesh variants to show.
    pub variant: u8,
}

/// Everything that stands on one chunk.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Scatter {
    /// Big props: trees, cacti, bushes, logs, rocks and sunken things, each merged into one mesh.
    pub props: Vec<Placement>,
    /// Small plants merged into one ground-cover mesh: grass, flowers, ferns, seagrass, coral and kelp.
    pub grass: Vec<Placement>,
}

/// What one pass over a chunk needs to look up the world.
struct Ctx<'a> {
    terrain: &'a Terrain,
    center: Vec2,
    fields: Fields,
}

/// Returns the world position of the center of chunk `coord`.
pub fn chunk_center(coord: IVec2) -> Vec2 {
    (coord.as_vec2() + Vec2::splat(0.5)) * CHUNK_SIZE
}

impl Ctx<'_> {
    /// Returns the plant, surface and ground height the roll lands on at `w`, if any.
    fn pick(&self, w: Vec2, roll: f32, big: bool) -> Option<(Plant, Surface, f32)> {
        let height = self.terrain.height(w.x, w.y);
        if height < -DEEP_LIMIT {
            return None;
        }
        let surface = self.terrain.surface(w.x, w.y);
        let table = plants_for(self.terrain.biome(w.x, w.y), surface);
        let density = self.fields.density(w, -height);
        choose(table, roll, &density, big)
            .filter(|&plant| {
                plant != Plant::Willow || (height < WILLOW_CEILING && near_pond(self.terrain, w))
            })
            .map(|plant| (plant, surface, height))
    }

    /// Returns the prop the roll plants at chunk position `local`, if any, with its look drawn from `rng`.
    fn place(&self, rng: &mut ChaCha8Rng, local: Vec2, roll: f32, big: bool) -> Option<Placement> {
        let w = self.center + local;
        let (plant, surface, height) = self.pick(w, roll, big)?;
        let yaw = rng.random_range(0.0..std::f32::consts::TAU);
        let scale = size(rng, plant, surface, height);
        if height < SEA_LEVEL && -height < plant.draft() * scale {
            return None;
        }
        let variant = rng.random_range(0..VARIANTS) as u8;
        let y = ground(self.terrain, w, (plant, scale), height);
        Some(Placement {
            pos: Vec3::new(local.x, y, local.y),
            yaw,
            scale,
            plant,
            variant,
        })
    }
}

/// Returns the scale of a new plant, or the height of a kelp, which stops one to two meters under the surface.
fn size(rng: &mut ChaCha8Rng, plant: Plant, surface: Surface, height: f32) -> f32 {
    if plant == Plant::Kelp {
        return (-height - 1.0 - rng.random_range(0.0..1.5)).max(1.5);
    }
    let boulder = 1.0 + f32::from(surface == Surface::Rock) * height * 0.06;
    rng.random_range(0.8..1.3) * boulder
}

/// Draws `tries` random points of the chunk and plants what the biome, surface and density allow there, keeping big
/// props apart.
fn pass(ctx: &Ctx, rng: &mut ChaCha8Rng, tries: u32, big: bool) -> Vec<Placement> {
    let half = CHUNK_SIZE * 0.5;
    let mut out = Vec::new();
    let mut spent = 0;
    for _ in 0..tries {
        let local = Vec2::new(rng.random_range(-half..half), rng.random_range(-half..half));
        let roll: f32 = rng.random();
        if out.len() >= [SMALL_CAP, BIG_CAP][big as usize] || spent >= TRI_BUDGET {
            break;
        }
        let Some(placement) = ctx.place(rng, local, roll, big) else {
            continue;
        };
        if big && crowded(&out, &placement) {
            continue;
        }
        spent += if big { 0 } else { small_tris(&placement) };
        out.push(placement);
    }
    out
}

/// Places every prop of chunk `coord`. The result only depends on the terrain seed and the coordinate.
pub fn scatter_chunk(terrain: &Terrain, coord: IVec2) -> Scatter {
    let center = chunk_center(coord);
    let corner = center - Vec2::splat(CHUNK_SIZE * 0.5);
    let shallow = (0..25).any(|i| {
        let p = corner + Vec2::new((i % 5) as f32, (i / 5) as f32) * CHUNK_SIZE / 4.0;
        terrain.height(p.x, p.y) > -DEEP_LIMIT
    });
    if !shallow {
        return Scatter::default();
    }
    let mix = (coord.x as u32 as u64) << 32 | coord.y as u32 as u64;
    let seed = terrain.seed();
    let mut rng = ChaCha8Rng::seed_from_u64(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ mix);
    let ctx = Ctx {
        terrain,
        center,
        fields: Fields::new(seed),
    };
    Scatter {
        props: pass(&ctx, &mut rng, 160, true),
        grass: pass(&ctx, &mut rng, 1400, false),
    }
}
