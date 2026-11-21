//! Deterministic placement of plants, rocks and sea life inside one chunk, by biome, surface, depth, patch
//! density and climate.

mod air;
mod cover;
mod ctx;
mod density;
mod grow;
mod metric;
mod pass;
mod perch;
mod relief;
mod rig;
#[cfg(test)]
mod rules;
mod sampler;
mod shelf;
mod sow;
mod spacing;
mod spread;
mod stand;
mod table;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_class;
#[cfg(test)]
mod tests_climate;
#[cfg(test)]
mod tests_cost;
#[cfg(test)]
mod tests_cover;
#[cfg(test)]
mod tests_density;
#[cfg(test)]
mod tests_edge;
#[cfg(test)]
mod tests_frozen;
#[cfg(test)]
mod tests_life;
#[cfg(test)]
mod tests_mean;
#[cfg(test)]
mod tests_metric;
#[cfg(test)]
mod tests_perch;
#[cfg(test)]
pub(crate) mod tests_river;
#[cfg(test)]
mod tests_root;
#[cfg(test)]
mod tests_sphere;
#[cfg(test)]
pub(crate) mod tile;
mod turf;
#[cfg(test)]
pub(crate) mod zones;

use bevy::prelude::*;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use self::grow::growth;
#[cfg(test)]
use self::metric::Metric;
use self::pass::{Ctx, pass};
use self::relief::Relief;
use self::spacing::room;
use super::globe::Chart;
use super::mesh::chunk_heights;
use super::propmesh::Plant;
use super::propmesh::grown::grown;
#[cfg(test)]
use super::props::small_tris;
use super::{CHUNK_SIZE, Terrain};
#[cfg(test)]
use super::{SEA_LEVEL, Surface};

pub use self::grow::Flora;
pub use self::rig::Rig;
pub use self::spread::Spread;
pub use self::table::plants_for;
pub use self::turf::{Carpet, Turf, turf_of};

/// Most big props on one chunk of `CHUNK_SIZE` metres, sea floor ones included.
const BIG_CAP: usize = 55;

/// Most small plants on one chunk of `CHUNK_SIZE` metres.
const SMALL_CAP: usize = 650;

/// Most triangles the merged ground-cover mesh of one chunk of `CHUNK_SIZE` metres gets, sea life included.
const TRI_BUDGET: usize = 7000;

/// Sea floors deeper than this get nothing.
const DEEP_LIMIT: f32 = 22.0;

/// Tries of the pass that plants the big props on a chunk of `CHUNK_SIZE` metres.
const BIG_TRIES: usize = 160;

/// Tries of the pass that plants the ground cover on a chunk of `CHUNK_SIZE` metres.
const SMALL_TRIES: usize = 1400;

/// Position and look of one prop, in chunk space: `x` and `z` are chart offsets from the chunk center and `y` is
/// the height above sea level in metres.
///
/// The prop itself is rigid and metric: its mesh keeps its shape and stands along the local up at this origin.
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
    /// Which of the shared mesh variants or book pages to show.
    pub variant: u16,
    /// Rise over run of the ground under a stone along chart x and z, which it is tilted square to. Zero
    /// for everything that stands straight up along the local up.
    pub lean: Vec2,
}

/// Everything that stands on one chunk.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Scatter {
    /// Big props: trees, cacti, bushes, logs, rocks and sunken things, each merged into one mesh.
    pub props: Vec<Placement>,
    /// Small plants merged into one ground-cover mesh: grass, flowers, ferns, seagrass, coral and kelp.
    pub grass: Vec<Placement>,
}

/// Returns the world position of the center of chunk `coord`.
pub fn chunk_center(coord: IVec2) -> Vec2 {
    (coord.as_vec2() + Vec2::splat(0.5)) * CHUNK_SIZE
}

/// Returns the height below which a floor gets nothing: the deep limit under the sea, and no limit on a
/// planet without a sea.
fn deep(terrain: &Terrain) -> f32 {
    terrain.globe().sea_level() - DEEP_LIMIT
}

/// Returns true when nothing is scattered on the planet at all.
fn bare(terrain: &Terrain) -> bool {
    growth(terrain) <= 0.0
}

/// Returns true when one of 25 points spread over chunk `coord` has a floor above the deep limit.
fn shallow(terrain: &Terrain, coord: IVec2) -> bool {
    let corner = chunk_center(coord) - Vec2::splat(CHUNK_SIZE * 0.5);
    (0..25).any(|i| {
        let p = corner + Vec2::new((i % 5) as f32, (i / 5) as f32) * CHUNK_SIZE / 4.0;
        terrain.height(p.x, p.y) > deep(terrain)
    })
}

/// Returns the random stream of chunk `coord`, which differs between the faces of a globe. It is drawn from
/// the chunk counted from the middle of the chart, so it is the same wherever the view is rooted.
fn stream(terrain: &Terrain, coord: IVec2) -> ChaCha8Rng {
    let id = coord + terrain.shift(CHUNK_SIZE);
    let mix = (id.x as u32 as u64) << 32 | id.y as u32 as u64;
    let face = match terrain.chart() {
        Chart::Face { face, .. } => (face as u64 + 1).wrapping_mul(0xD6E8_FEB8_6659_FD93),
        _ => 0,
    };
    let seed = terrain.seed().wrapping_mul(0x9E37_79B9_7F4A_7C15);
    ChaCha8Rng::seed_from_u64(seed ^ mix ^ face)
}

/// Runs the two passes of chunk `coord`: the big props, then the ground cover in the shade they cast, each
/// with the share of the tries of home that the class of the planet lets grow.
fn run(terrain: &Terrain, coord: IVec2, relief: Option<Relief>) -> Scatter {
    let mut rng = stream(terrain, coord);
    let mut ctx = Ctx::new(terrain, coord, relief);
    let share = growth(terrain);
    let thin = |tries: usize| (tries as f32 * share) as usize;
    let props = pass(&ctx, &mut rng, thin(BIG_TRIES), true);
    let grown_big = props.iter().filter(|p| grown(p.variant));
    ctx.canopy = grown_big
        .map(|p| (ctx.center + p.pos.xz(), room(p)))
        .collect();
    Scatter {
        grass: pass(&ctx, &mut rng, thin(SMALL_TRIES), false),
        props,
    }
}

/// Places every prop of chunk `coord`. The result only depends on the terrain, its chart and the coordinate.
///
/// On the legacy plane every decision reads the analytic ground. On a globe the chunk is measured in true
/// metres and every decision reads the visible terrain mesh, which this call samples itself: a caller that
/// already holds the heights of the chunk calls [`scatter_over`] instead and saves that work.
pub fn scatter_chunk(terrain: &Terrain, coord: IVec2) -> Scatter {
    if bare(terrain) || !shallow(terrain, coord) {
        return Scatter::default();
    }
    if terrain.chart() == Chart::Flat {
        return run(terrain, coord, None);
    }
    scatter_over(terrain, coord, &chunk_heights(terrain, coord, 1))
}

/// Places every prop of chunk `coord` over `heights`, the heights of its finest terrain mesh as
/// [`chunk_heights`] gives them at stride 1.
///
/// Tries, caps and spacing are in true metres, a prop belongs to the chunk whose square holds it, land and
/// sea props stand on the mesh, and no ground height is sampled again. On the legacy plane this is
/// [`scatter_chunk`]. It reads only its arguments and is safe on any thread.
pub fn scatter_over(terrain: &Terrain, coord: IVec2, heights: &[Vec<f32>]) -> Scatter {
    if terrain.chart() == Chart::Flat {
        return scatter_chunk(terrain, coord);
    }
    let relief = Relief::new(coord, heights);
    if bare(terrain) || !relief.shallow(deep(terrain)) {
        return Scatter::default();
    }
    run(terrain, coord, Some(relief))
}
