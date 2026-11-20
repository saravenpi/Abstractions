//! The id of a rock prop, and the filter that leaves the taken ones out.

use bevy::prelude::*;

use super::{RockId, Taken};
use crate::world::globe::Chart;
use crate::world::propmesh::Plant;
use crate::world::scatter::Placement;
use crate::world::{CHUNK_SIZE, Terrain};

/// Odd multiplier that spreads the bits of a hash, the golden ratio of 64 bits.
const SPREAD: u64 = 0x9E37_79B9_7F4A_7C15;

/// Steps per metre at which a position goes into an id: half a metre.
const STEPS: f32 = 2.0;

/// Folds `value` into the hash `hash`.
fn mix(hash: u64, value: u64) -> u64 {
    let spread = (hash.rotate_left(27) ^ value).wrapping_mul(SPREAD);
    spread ^ (spread >> 32)
}

/// Returns the key of chunk `coord` of the chart of `terrain`: the planet, the cube face and the chunk
/// counted from the middle of the chart, so it is the same wherever the view is rooted.
pub fn key(terrain: &Terrain, coord: IVec2) -> u64 {
    let cell = coord + terrain.shift(CHUNK_SIZE);
    let face = match terrain.chart() {
        Chart::Face { face, .. } => face as u64 + 1,
        _ => 0,
    };
    let parts = [face, cell.x as u32 as u64, cell.y as u32 as u64];
    parts.into_iter().fold(terrain.seed(), mix)
}

/// Returns the id of placement `p` of the chunk with key `key`: its kind, its variant and its
/// position in chunk space to half a metre.
pub fn id_of(key: u64, p: &Placement) -> RockId {
    let at = |v: f32| (v * STEPS).round() as i32 as u32 as u64;
    let parts = [
        at(p.pos.x),
        at(p.pos.y),
        at(p.pos.z),
        p.plant as u64,
        u64::from(p.variant),
    ];
    RockId(parts.into_iter().fold(key, mix))
}

/// Returns true for the kinds of prop the robot can break up: the stones that stand on land and on
/// the deep floor. The smooth stones of the shallows have no body to aim at.
pub fn is_rock(plant: Plant) -> bool {
    matches!(
        plant,
        Plant::Rock | Plant::IceRock | Plant::Sandstone | Plant::Boulder
    )
}

/// Removes from `props` the rocks of `taken`, for the chunk with key `key`.
pub fn spare(props: &mut Vec<Placement>, key: u64, taken: &Taken) {
    props.retain(|p| !is_rock(p.plant) || !taken.has(id_of(key, p)));
}
