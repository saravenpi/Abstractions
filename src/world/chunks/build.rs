//! Worker thread side of chunk streaming: everything a chunk needs, built without touching the world.

use avian3d::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;

use super::embed::Embedding;
use crate::world::Terrain;
use crate::world::canopy::{Canopy, canopy_on};
use crate::world::globe::ChunkId;
use crate::world::mesh::{
    Frame, Lod, chunk_heights, chunk_mesh, chunk_patch, node_frame, node_mesh, subsample,
};
use crate::world::propmesh::merge_props_on;
use crate::world::props::{body_collider_on, build_grass_on};
use crate::world::scatter::{Placement, Rig, Scatter, scatter_over};
use crate::world::structures::{StructureData, build_in, origins_near};

/// Radius in metres around a structure origin kept free of plants.
const CLEARING: f32 = 11.0;

/// Radius in metres around the reserved ground of a planet kept free of plants.
const RESERVE_CLEARING: f64 = 3.0;

/// How many chart units the clearing may span where the chart is squeezed the most.
const SQUEEZE: f32 = 1.5;

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
    pub(super) turfed: bool,
    pub(super) canopy: Canopy,
}

/// What a worker thread produces for one coarse quadtree node.
pub struct NodeData {
    pub(super) mesh: Mesh,
    pub(super) frame: Frame,
}

/// Drops the CPU copy of a mesh once it is uploaded to the GPU.
pub fn render_only(mut mesh: Mesh) -> Mesh {
    mesh.asset_usage = RenderAssetUsages::RENDER_WORLD;
    mesh
}

/// Builds the terrain mesh of chunk `coord` at level `lod`.
pub fn terrain_mesh(terrain: &Terrain, coord: IVec2, lod: Lod) -> Mesh {
    let heights = chunk_heights(terrain, coord, lod.stride());
    render_only(chunk_mesh(terrain, coord, &heights, lod))
}

/// Builds the terrain-only mesh of quadtree node `id`.
///
/// `terrain` must be the face chart view of the face of the node.
pub fn coarse(terrain: &Terrain, id: ChunkId) -> NodeData {
    NodeData {
        mesh: render_only(node_mesh(terrain, id)),
        frame: node_frame(terrain, id),
    }
}

/// Returns the ground collider of chunk `coord`: the triangles of its finest mesh, in its frame.
fn ground(terrain: &Terrain, coord: IVec2, heights: &[Vec<f32>]) -> Collider {
    let patch = chunk_patch(terrain, coord, heights);
    Collider::trimesh(patch.positions().to_vec(), patch.triangles())
}

/// Removes from `scatter` what stands within the clearing of a structure of chunk `coord`, or on
/// the reserved ground of the planet, both measured in metres on the sea level sphere.
fn clear(scatter: &mut Scatter, terrain: &Terrain, coord: IVec2, warp: &Embedding) {
    let sea = |p: Vec3| warp.point(Vec3::new(p.x, 0.0, p.z));
    let center = warp.center();
    let taken: Vec<Vec3> = origins_near(terrain, coord, CLEARING * SQUEEZE)
        .into_iter()
        .map(|o| sea(Vec3::new(o.x - center.x, 0.0, o.y - center.y)))
        .collect();
    let frame = warp.frame();
    let kept = terrain.globe().reserve().filter(|reserve| {
        let reach = f64::from(crate::world::CHUNK_SIZE) * 2.0 + RESERVE_CLEARING;
        reserve.covers(frame.origin, reach)
    });
    if taken.is_empty() && kept.is_none() {
        return;
    }
    let free = |p: &Placement| {
        let at = sea(p.pos);
        let planet = frame.origin + frame.rot * at.as_dvec3();
        taken.iter().all(|o| o.distance(at) > CLEARING)
            && kept.is_none_or(|reserve| !reserve.covers(planet, RESERVE_CLEARING))
    };
    scatter.props.retain(free);
    scatter.grass.retain(free);
}

/// Returns the scatter of chunk `coord` over its `heights`, cleared around structures and the
/// reserved ground, and the embedding that places it.
fn planted(terrain: &Terrain, coord: IVec2, heights: &[Vec<f32>]) -> (Scatter, Embedding) {
    let warp = Embedding::of(terrain, coord);
    let mut scatter = scatter_over(terrain, coord, heights);
    clear(&mut scatter, terrain, coord, &warp);
    (scatter, warp)
}

/// Builds the ground cover mesh of chunk `coord` alone, for a chunk that was generated without
/// it. None when nothing small grows there.
pub fn ground_cover(terrain: &Terrain, coord: IVec2) -> Option<Mesh> {
    let (scatter, warp) = planted(terrain, coord, &chunk_heights(terrain, coord, 1));
    let (embed, turn) = (|p| warp.point(p), |p| warp.turn(p));
    let rig = Rig {
        embed: &embed,
        turn: &turn,
    };
    (!scatter.grass.is_empty()).then(|| render_only(build_grass_on(&scatter, &rig)))
}

/// Generates the mesh at level `lod`, the colliders, the merged props and the structures of chunk
/// `coord`, all in the frame of the chunk: the main thread only inserts them. The ground cover is
/// built only when `turfed` is set.
///
/// `terrain` is the face chart view of the face of the chunk. Runs on any thread.
pub fn generate(terrain: &Terrain, coord: IVec2, lod: Lod, turfed: bool) -> ChunkData {
    let heights = chunk_heights(terrain, coord, 1);
    let (scatter, warp) = planted(terrain, coord, &heights);
    let mesh = chunk_mesh(terrain, coord, &subsample(&heights, lod.stride()), lod);
    let (embed, turn) = (|p| warp.point(p), |p| warp.turn(p));
    let rig = Rig {
        embed: &embed,
        turn: &turn,
    };
    let grass = || render_only(build_grass_on(&scatter, &rig));
    let props = || render_only(merge_props_on(&scatter.props, &rig));
    let frame = warp.frame();
    ChunkData {
        mesh: render_only(mesh),
        grass: (turfed && !scatter.grass.is_empty()).then(grass),
        props: (!scatter.props.is_empty()).then(props),
        bodies: body_collider_on(&scatter.props, &rig),
        structures: build_in(terrain, coord, &frame),
        heights: ground(terrain, coord, &heights),
        canopy: canopy_on(terrain, coord, (&scatter.props, &scatter.grass), &rig),
        lod,
        frame,
        turfed,
    }
}
