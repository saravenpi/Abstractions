//! Worker thread side of chunk streaming: everything a chunk needs, built without touching the world.

use avian3d::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;

use super::embed::Embedding;
use crate::world::Terrain;
use crate::world::globe::ChunkId;
use crate::world::mesh::{
    Frame, Lod, chunk_heights, chunk_mesh, chunk_patch, node_frame, node_mesh, subsample,
};
use crate::world::propmesh::merge_props;
use crate::world::props::{body_collider, build_grass};
use crate::world::scatter::{Placement, chunk_center, scatter_chunk};
use crate::world::structures::{StructureData, build_chunk, origins_near};

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
    pub(super) frame: Frame,
}

/// What a worker thread produces for one coarse quadtree node.
pub struct NodeData {
    pub(super) mesh: Mesh,
    pub(super) frame: Frame,
}

/// Drops the CPU copy of a mesh once it is uploaded to the GPU and sets its ray tracing flag.
pub fn render_only(mut mesh: Mesh, on: bool) -> Mesh {
    mesh.asset_usage = RenderAssetUsages::RENDER_WORLD;
    mesh.enable_raytracing = on;
    mesh
}

/// Builds the terrain mesh of chunk `coord` at level `lod` with the ray tracing flag `on`.
pub fn terrain_mesh(terrain: &Terrain, coord: IVec2, lod: Lod, on: bool) -> Mesh {
    let heights = chunk_heights(terrain, coord, lod.stride());
    render_only(chunk_mesh(terrain, coord, &heights, lod), on)
}

/// Builds the terrain-only mesh of quadtree node `id` with the ray tracing flag `on`.
///
/// `terrain` must be the face chart view of the face of the node.
pub fn coarse(terrain: &Terrain, id: ChunkId, on: bool) -> NodeData {
    NodeData {
        mesh: render_only(node_mesh(terrain, id), on),
        frame: node_frame(terrain, id),
    }
}

/// Returns the ground collider of chunk `coord`: the triangles of its finest mesh, in its frame.
fn ground(terrain: &Terrain, coord: IVec2, heights: &[Vec<f32>]) -> Collider {
    let patch = chunk_patch(terrain, coord, heights);
    Collider::trimesh(patch.positions().to_vec(), patch.triangles())
}

/// Returns the meshes of `scatter` embedded by `warp`: the merged big props and the ground cover.
fn cover(scatter: &crate::world::scatter::Scatter, warp: &Embedding) -> [Option<Mesh>; 2] {
    let embedded = |mut mesh: Mesh| {
        warp.mesh(&mut mesh);
        render_only(mesh, false)
    };
    [
        (!scatter.props.is_empty()).then(|| embedded(merge_props(&scatter.props))),
        (!scatter.grass.is_empty()).then(|| embedded(build_grass(scatter))),
    ]
}

/// Generates the mesh at level `lod`, collider and merged props of chunk `coord`.
///
/// `terrain` is the face chart view of the face of the chunk. The generators work in its chart
/// and their output is embedded into the frame of the chunk on the globe.
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
    let warp = Embedding::of(terrain, coord);
    let [props, grass] = cover(&scatter, &warp);
    ChunkData {
        mesh: render_only(mesh, false),
        grass,
        props,
        bodies: body_collider(&scatter.props).map(|bodies| warp.collider(&bodies)),
        structures: build_chunk(terrain, coord),
        heights: ground(terrain, coord, &heights),
        lod,
        frame: warp.frame(),
    }
}
