//! Worker thread side of chunk streaming: everything a chunk needs, built without touching the world.

use avian3d::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;

use crate::world::mesh::{Lod, chunk_heights, chunk_mesh, subsample};
use crate::world::propmesh::merge_props;
use crate::world::props::{body_collider, build_grass};
use crate::world::scatter::{Placement, chunk_center, scatter_chunk};
use crate::world::structures::{StructureData, build_chunk, origins_near};
use crate::world::{CHUNK_SIZE, Terrain};

/// Radius around a structure origin kept free of plants.
const CLEARING: f32 = 11.0;

/// Everything a worker thread produces for one chunk.
pub struct ChunkData {
    pub(super) heights: Collider,
    pub(super) mesh: Mesh,
    pub(super) grass: Option<Mesh>,
    pub(super) props: Option<Mesh>,
    pub(super) bodies: Option<Collider>,
    pub(super) structures: Option<StructureData>,
    pub(super) lod: Lod,
}

/// Drops the CPU copy of a mesh once it is uploaded to the GPU.
fn render_only(mut mesh: Mesh) -> Mesh {
    mesh.asset_usage = RenderAssetUsages::RENDER_WORLD;
    mesh
}

/// Builds the terrain mesh of chunk `coord` at level `lod` with the ray tracing flag `on`.
pub fn terrain_mesh(terrain: &Terrain, coord: IVec2, lod: Lod, on: bool) -> Mesh {
    let heights = chunk_heights(terrain, coord, lod.stride());
    let mut mesh = render_only(chunk_mesh(terrain, coord, &heights, lod));
    mesh.enable_raytracing = on;
    mesh
}

/// Generates the mesh at level `lod`, collider and merged props of chunk `coord`.
pub fn generate(terrain: &Terrain, coord: IVec2, lod: Lod) -> ChunkData {
    let heights = chunk_heights(terrain, coord, 1);
    let mut scatter = scatter_chunk(terrain, coord);
    let taken = origins_near(terrain, coord, CLEARING);
    let center = chunk_center(coord);
    let free = |p: &Placement| {
        let at = center + Vec2::new(p.pos.x, p.pos.z);
        taken.iter().all(|o| o.distance(at) > CLEARING)
    };
    scatter.props.retain(free);
    scatter.grass.retain(free);
    let mesh = chunk_mesh(terrain, coord, &subsample(&heights, lod.stride()), lod);
    let scale = Vec3::new(CHUNK_SIZE, 1.0, CHUNK_SIZE);
    ChunkData {
        mesh: render_only(mesh),
        grass: (!scatter.grass.is_empty()).then(|| render_only(build_grass(&scatter))),
        props: (!scatter.props.is_empty()).then(|| render_only(merge_props(&scatter.props))),
        bodies: body_collider(&scatter.props),
        structures: build_chunk(terrain, coord),
        heights: Collider::heightfield(heights, scale),
        lod,
    }
}
