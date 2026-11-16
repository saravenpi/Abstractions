//! Deterministic placement of plants and rocks inside one chunk, by biome, surface and patch density.

use bevy::prelude::*;
use noise::{NoiseFn, Perlin};
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;

use super::propmesh::Plant::{self, *};
use super::propmesh::{BIG, FLOWER, TREES, VARIANTS};
use super::{Biome, CHUNK_SIZE, SEA_LEVEL, Surface, Terrain};

/// Most big props on one chunk.
const BIG_CAP: usize = 45;

/// Most small plants on one chunk, which keeps the merged mesh under about 6000 triangles.
const SMALL_CAP: usize = 600;

/// Plants and their chance per try, for no surface at all.
const NONE: &[(Plant, f32)] = &[];

/// Boulders on bare rock.
const ROCKY: &[(Plant, f32)] = &[(Rock, 0.12)];

/// Ice-grey stones on snow.
const SNOWY: &[(Plant, f32)] = &[(IceRock, 0.06)];

/// Plants and their chance per try, by biome then by surface in the order water, sand, grass, rock, snow.
#[rustfmt::skip]
const TABLE: [[&[(Plant, f32)]; 5]; 6] = [
    [NONE, &[(Rock, 0.03), (Grass, 0.08)], &[(Oak, 0.1), (Bush, 0.07), (Rock, 0.01), (Grass, 0.5),
        (Poppy, 0.12), (Daisy, 0.12), (Buttercup, 0.1), (Bluebell, 0.1)], ROCKY, SNOWY],
    [NONE, &[(Rock, 0.03), (Fern, 0.05)], &[(Oak, 0.28), (Pine, 0.26), (Bush, 0.05), (Log, 0.03), (Rock, 0.01),
        (Grass, 0.3), (Fern, 0.12), (Mushroom, 0.04), (Daisy, 0.02)], ROCKY, SNOWY],
    [NONE, &[(Rock, 0.03), (Grass, 0.08)], &[(Cherry, 0.2), (Oak, 0.02), (Bush, 0.04), (Rock, 0.01),
        (Grass, 0.35), (Pink, 0.2), (Daisy, 0.12)], ROCKY, SNOWY],
    [NONE, &[(Saguaro, 0.05), (RoundCactus, 0.06), (DeadBush, 0.06), (Sandstone, 0.04), (Grass, 0.02)],
        &[(Saguaro, 0.04), (RoundCactus, 0.05), (DeadBush, 0.05), (Sandstone, 0.03), (Grass, 0.03)],
        &[(Sandstone, 0.12)], SNOWY],
    [NONE, &[(IceRock, 0.04)], &[(SnowPine, 0.28), (Pine, 0.08), (DeadBush, 0.04), (IceRock, 0.03),
        (Grass, 0.12), (Fern, 0.03)], &[(IceRock, 0.12)], &[(SnowPine, 0.2), (IceRock, 0.04)]],
    [NONE, &[(Palm, 0.14), (Rock, 0.02), (Reed, 0.2), (Orchid, 0.02)], &[(Palm, 0.12), (Bush, 0.08),
        (Jungle, 0.1), (Rock, 0.01), (Grass, 0.3), (Orchid, 0.12), (Poppy, 0.08), (Buttercup, 0.08),
        (Reed, 0.03)], ROCKY, SNOWY],
];

/// Position and look of one prop, in chunk space (origin at the chunk center, `y` is world height).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placement {
    /// Chunk-space position on the ground.
    pub pos: Vec3,
    /// Rotation around the vertical axis in radians.
    pub yaw: f32,
    /// Uniform scale.
    pub scale: f32,
    /// What stands here.
    pub plant: Plant,
    /// Which of the shared mesh variants to show.
    pub variant: u8,
}

/// Everything that stands on one chunk.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Scatter {
    /// Big props: trees, cacti, bushes, logs and rocks, each its own entity.
    pub props: Vec<Placement>,
    /// Small plants merged into one ground-cover mesh: grass, flowers, ferns, mushrooms and reeds.
    pub grass: Vec<Placement>,
}

/// What one pass over a chunk needs to look up the world.
struct Ctx<'a> {
    /// Terrain to query.
    terrain: &'a Terrain,
    /// World position of the chunk center.
    center: Vec2,
    /// Low-frequency fields of grove density and flower density.
    fields: [Perlin; 2],
}

/// Returns the world position of the center of chunk `coord`.
pub fn chunk_center(coord: IVec2) -> Vec2 {
    (coord.as_vec2() + Vec2::splat(0.5)) * CHUNK_SIZE
}

/// Returns the plants that may grow on a surface of a biome, with their chance per try.
pub fn plants_for(biome: Biome, surface: Surface) -> &'static [(Plant, f32)] {
    TABLE[biome as usize][surface as usize]
}

/// Picks the plant a roll in `[0, 1)` lands on, scaling trees by the grove and flowers by the bloom density.
fn choose(table: &[(Plant, f32)], roll: f32, density: [f32; 2], big: bool) -> Option<Plant> {
    let mut acc = 0.0;
    for &(plant, chance) in table.iter().filter(|(p, _)| ((*p as usize) < BIG) == big) {
        let i = plant as usize;
        let gain = if i < TREES {
            density[0]
        } else if i >= FLOWER {
            density[1]
        } else {
            1.0
        };
        acc += chance * gain;
        if roll < acc {
            return Some(plant);
        }
    }
    None
}

/// Draws `tries` random points of the chunk and plants what the biome, surface and density allow there.
fn pass(ctx: &Ctx, rng: &mut ChaCha8Rng, tries: u32, big: bool) -> Vec<Placement> {
    let half = CHUNK_SIZE * 0.5;
    let field = |i: usize, w: Vec2, freq: f64| {
        let n = ctx.fields[i].get([w.x as f64 * freq, w.y as f64 * freq]) as f32;
        (n * 2.0 + 0.5).clamp(0.0, 1.0)
    };
    let mut out = Vec::new();
    for _ in 0..tries {
        let local = Vec2::new(rng.random_range(-half..half), rng.random_range(-half..half));
        let roll: f32 = rng.random();
        let w = ctx.center + local;
        let surface = ctx.terrain.surface(w.x, w.y);
        if surface == Surface::Water || out.len() >= [SMALL_CAP, BIG_CAP][big as usize] {
            continue;
        }
        let table = plants_for(ctx.terrain.biome(w.x, w.y), surface);
        let density = [field(0, w, 0.045), field(1, w, 0.07)];
        let Some(plant) = choose(table, roll, density, big) else {
            continue;
        };
        let height = ctx.terrain.height(w.x, w.y);
        let boulder = 1.0 + f32::from(surface == Surface::Rock) * height * 0.06;
        out.push(Placement {
            pos: Vec3::new(local.x, height, local.y),
            yaw: rng.random_range(0.0..std::f32::consts::TAU),
            scale: rng.random_range(0.8..1.3) * boulder,
            plant,
            variant: rng.random_range(0..VARIANTS) as u8,
        });
    }
    out
}

/// Places every prop of chunk `coord`. The result only depends on the terrain seed and the coordinate.
pub fn scatter_chunk(terrain: &Terrain, coord: IVec2) -> Scatter {
    let center = chunk_center(coord);
    let corner = center - Vec2::splat(CHUNK_SIZE * 0.5);
    let dry = (0..25).any(|i| {
        let p = corner + Vec2::new((i % 5) as f32, (i / 5) as f32) * CHUNK_SIZE / 4.0;
        terrain.height(p.x, p.y) > SEA_LEVEL
    });
    if !dry {
        return Scatter::default();
    }
    let mix = (coord.x as u32 as u64) << 32 | coord.y as u32 as u64;
    let seed = terrain.seed();
    let mut rng = ChaCha8Rng::seed_from_u64(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ mix);
    let fields = [
        Perlin::new(seed as u32 ^ 0x51),
        Perlin::new(seed as u32 ^ 0xA3),
    ];
    let ctx = Ctx {
        terrain,
        center,
        fields,
    };
    Scatter {
        props: pass(&ctx, &mut rng, 160, true),
        grass: pass(&ctx, &mut rng, 1100, false),
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
        assert!(a.props.len() <= BIG_CAP && a.grass.len() <= SMALL_CAP);
    }

    /// Open sea holds no props.
    #[test]
    fn deep_sea_is_empty() {
        let t = Terrain::new(5);
        let far = IVec2::new(100_000, 100_000);
        assert_eq!(scatter_chunk(&t, far), Scatter::default());
    }

    /// Water is bare, cacti are desert-only, trees avoid rock and snow, pines own the taiga snow.
    #[test]
    fn plants_follow_biome_and_surface() {
        let dry = [Surface::Sand, Surface::Grass, Surface::Rock, Surface::Snow];
        let has = |b: Biome, s: Surface, p: Plant| plants_for(b, s).iter().any(|e| e.0 == p);
        let anywhere = |b: Biome, p: Plant| dry.iter().any(|&s| has(b, s, p));
        for b in Biome::ALL {
            assert!(plants_for(b, Surface::Water).is_empty());
            assert_eq!(anywhere(b, Saguaro), b == Biome::Desert);
            let rock_trees = plants_for(b, Surface::Rock)
                .iter()
                .any(|e| (e.0 as usize) < TREES);
            let snow_trees = plants_for(b, Surface::Snow)
                .iter()
                .any(|e| (e.0 as usize) < TREES);
            assert!(!rock_trees && snow_trees == (b == Biome::Taiga));
        }
        assert!(has(Biome::Taiga, Surface::Snow, SnowPine));
        assert!(anywhere(Biome::Tropical, Palm) && anywhere(Biome::Cherry, Cherry));
    }
}
