//! Exact merged triangle-mesh colliders for stone and mineral props, built once per variant.

use std::collections::HashMap;
use std::sync::OnceLock;

use avian3d::prelude::*;
use bevy::prelude::*;

use super::merge::base;
use super::stones::{STONES, stone_index};
use super::{Plant, VARIANTS};
use crate::world::scatter::{Placement, Rig};

/// Scale factor that quantizes vertex positions to 0.1 mm when deduplicating shared stone corners.
const QUANTIZE: f32 = 10_000.0;

/// Cached indexed triangle soup and origin bounding radius for one stone `(plant, variant)`.
struct StoneShape {
    vertices: Vec<Vec3>,
    indices: Vec<[u32; 3]>,
    radius: f32,
}

/// Builds a deduplicated vertex list and triangle index buffer from flat triangle vertices `raw`.
fn index_triangles(raw: &[[f32; 3]]) -> StoneShape {
    let mut map: HashMap<[i32; 3], u32> = HashMap::new();
    let mut vertices = Vec::new();
    let mut indices = Vec::with_capacity(raw.len() / 3);
    let mut intern = |p: [f32; 3]| -> u32 {
        let v = Vec3::from(p);
        let key = (v * QUANTIZE).round().as_ivec3().to_array();
        *map.entry(key).or_insert_with(|| {
            let idx = vertices.len() as u32;
            vertices.push(v);
            idx
        })
    };
    for tri in raw.as_chunks::<3>().0 {
        let [a, b, c] = [intern(tri[0]), intern(tri[1]), intern(tri[2])];
        if a != b && b != c && c != a {
            indices.push([a, b, c]);
        }
    }
    let radius = vertices.iter().map(|v| v.length()).fold(0.0, f32::max);
    StoneShape {
        vertices,
        indices,
        radius,
    }
}

/// Returns the cached table of indexed stone shapes indexed by `stone_index * VARIANTS + variant`.
fn catalog() -> &'static [StoneShape] {
    static TABLE: OnceLock<Vec<StoneShape>> = OnceLock::new();
    TABLE.get_or_init(|| {
        STONES
            .iter()
            .flat_map(|&plant| {
                (0..VARIANTS).map(move |v| index_triangles(&base(plant as usize, v).positions))
            })
            .collect()
    })
}

/// Returns true when `plant` is one of the twelve sculpted stone or mineral props.
pub fn is_stone(plant: Plant) -> bool {
    stone_index(plant).is_some()
}

/// Returns a merged exact triangle-mesh collider for every stone or mineral in `props` in `rig`,
/// or nothing when `props` holds no non-zero stone.
pub fn stone_trimesh_on(props: &[Placement], rig: &Rig) -> Option<Collider> {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let table = catalog();
    for p in props {
        let Some(kind) = stone_index(p.plant) else {
            continue;
        };
        if p.scale <= 1e-4 {
            continue;
        }
        let shape = &table[kind * VARIANTS + usize::from(p.variant) % VARIANTS];
        let (origin, rot) = rig.pose(p);
        let base_idx = vertices.len() as u32;
        vertices.extend(shape.vertices.iter().map(|v| origin + rot * (*v * p.scale)));
        indices.extend(shape.indices.iter().map(|tri| tri.map(|i| base_idx + i)));
    }
    (!indices.is_empty())
        .then(|| Collider::trimesh_with_config(vertices, indices, TrimeshFlags::FIX_INTERNAL_EDGES))
}

/// Returns the maximum distance in metres from the base origin of `plant` across all variants at
/// unit scale, or nothing when `plant` is not a stone.
pub fn stone_reach(plant: Plant) -> Option<f32> {
    let base = stone_index(plant)? * VARIANTS;
    catalog()[base..base + VARIANTS]
        .iter()
        .map(|s| s.radius)
        .reduce(f32::max)
}
