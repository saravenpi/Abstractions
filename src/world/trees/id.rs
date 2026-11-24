//! Deterministic tree identities and chunk placement filtering.

use bevy::prelude::*;

use super::{Felled, Tree, TreeId};
use crate::world::Terrain;
use crate::world::propmesh::Plant;
use crate::world::props::extent;
use crate::world::scatter::{Placement, Rig};

/// Returns true when `plant` is a woody tree that can be chopped for logs.
pub fn is_tree(plant: Plant) -> bool {
    matches!(
        plant,
        Plant::Oak
            | Plant::Pine
            | Plant::Cherry
            | Plant::SnowPine
            | Plant::Palm
            | Plant::Birch
            | Plant::GreatOak
            | Plant::DeadTree
            | Plant::Bamboo
            | Plant::Willow
    )
}

/// Returns the deterministic chunk key shared with rock props.
pub fn key(terrain: &Terrain, coord: IVec2) -> u64 {
    crate::world::rocks::key(terrain, coord)
}

/// SplitMix64 finalizer for stable 64-bit prop ids.
fn mix(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

/// Returns the deterministic identity of tree placement `p` in chunk `chunk_key`.
pub fn id_of(chunk_key: u64, p: &Placement) -> TreeId {
    let qx = (p.pos.x * 1000.0).round() as i64 as u64;
    let qz = (p.pos.z * 1000.0).round() as i64 as u64;
    let mut h = mix(chunk_key ^ 0x5452_4545_u64);
    h = mix(h ^ (p.plant as u64));
    h = mix(h ^ qx);
    TreeId(mix(h ^ qz))
}

/// Removes trees from `props` that are already felled in `felled`.
pub fn spare_trees(props: &mut Vec<Placement>, chunk_key: u64, felled: &Felled) {
    props.retain(|p| !is_tree(p.plant) || !felled.gone(id_of(chunk_key, p)));
}

/// Extracts standing choppable trees from chunk `props` placed via `rig`.
pub fn trees_of(props: &[Placement], chunk_key: u64, rig: &Rig, felled: &Felled) -> Vec<Tree> {
    props
        .iter()
        .filter(|p| is_tree(p.plant))
        .filter_map(|p| {
            let [radius, height] = extent(p.plant, p.scale);
            let id = id_of(chunk_key, p);
            if radius <= 0.0 || felled.gone(id) {
                return None;
            }
            let (at, rot) = rig.pose(p);
            let top = at + rot * Vec3::Y * height;
            Some(Tree {
                id,
                plant: p.plant,
                scale: p.scale,
                at,
                top,
                mid: (at + top) * 0.5,
                radius,
                hits: felled.hits(id),
            })
        })
        .collect()
}
