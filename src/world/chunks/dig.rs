//! Chunk building and live mending with felled trees and carved terrain craters.

use bevy::camera::primitives::{Aabb, MeshAabb};
use bevy::prelude::*;

use super::Chunks;
use super::build::{ChunkData, ground, planted, render_only};
use super::embed::Embedding;
use super::plan::Loaded;
use super::spawn::Spawner;
use crate::universe::BodyId;
use crate::world::Terrain;
use crate::world::canopy::canopy_on;
use crate::world::digs::{Digs, TerrainDug};
use crate::world::mesh::{Lod, chunk_heights, chunk_mesh, leaf_coord, subsample};
use crate::world::propmesh::merge_props_on;
use crate::world::props::{build_grass_on, stone_collider_on, trunk_collider_on};
use crate::world::rocks::{Stand, Taken, key};
use crate::world::scatter::Rig;
use crate::world::trees::{Felled, Tree, TreeChopped, spare_trees};

/// Dynamic world modifications sampled asynchronously by chunk worker tasks.
#[derive(Clone, Default)]
pub struct ChunkMods {
    /// Body currently being streamed.
    pub planet: BodyId,
    /// Quarried rocks.
    pub taken: Taken,
    /// Chopped and felled trees.
    pub felled: Felled,
    /// Carved terrain dig craters.
    pub digs: Digs,
}

/// Captures [`ChunkMods`] from a streaming [`Spawner`].
pub(super) fn spawner_mods(spawner: &Spawner) -> ChunkMods {
    ChunkMods {
        planet: spawner
            .planet
            .as_deref()
            .map_or(BodyId::default(), |p| p.body),
        taken: spawner.taken.as_deref().cloned().unwrap_or_default(),
        felled: spawner.felled.as_deref().cloned().unwrap_or_default(),
        digs: spawner.digs.as_deref().cloned().unwrap_or_default(),
    }
}

/// Builds the terrain mesh of `coord` at `lod`, applying any terrain digs on `mods`.
pub fn terrain_mesh_mods(
    terrain: &Terrain,
    coord: IVec2,
    lod: Lod,
    mods: &ChunkMods,
) -> (Mesh, Option<Aabb>) {
    let mut full = chunk_heights(terrain, coord, 1);
    let dug = mods
        .digs
        .apply_to_grid(terrain, coord, mods.planet, &mut full);
    let heights = subsample(&full, lod.stride());
    let mut mesh = chunk_mesh(terrain, coord, &heights, lod);
    if dug {
        let frame = Embedding::of(terrain, coord).frame();
        mods.digs
            .tint_mesh(&mut mesh, mods.planet, &frame, terrain.globe().radius());
    }
    let aabb = mesh.get_aabb();
    (render_only(mesh), aabb)
}

/// Builds a full chunk asynchronously with `taken` rocks, `felled` trees, and `digs` applied.
pub fn generate_mods(
    terrain: &Terrain,
    coord: IVec2,
    (lod, turfed): (Lod, bool),
    mods: &ChunkMods,
) -> ChunkData {
    let mut heights = chunk_heights(terrain, coord, 1);
    let dug = mods
        .digs
        .apply_to_grid(terrain, coord, mods.planet, &mut heights);
    let (mut scatter, warp) = planted(terrain, coord, &heights, &mods.taken);
    spare_trees(&mut scatter.props, key(terrain, coord), &mods.felled);
    let mut mesh = chunk_mesh(terrain, coord, &subsample(&heights, lod.stride()), lod);
    let frame = warp.frame();
    if dug {
        mods.digs
            .tint_mesh(&mut mesh, mods.planet, &frame, terrain.globe().radius());
    }
    let (embed, turn) = (|p| warp.point(p), |p| warp.turn(p));
    let rig = Rig {
        embed: &embed,
        turn: &turn,
    };
    let grass = || render_only(build_grass_on(&scatter, &rig));
    let props = || render_only(merge_props_on(&scatter.props, &rig));
    let aabb = mesh.get_aabb();
    ChunkData {
        mesh: render_only(mesh),
        grass: (turfed && !scatter.grass.is_empty()).then(grass),
        props: (!scatter.props.is_empty()).then(props),
        bodies: trunk_collider_on(&scatter.props, &rig),
        stone_bodies: stone_collider_on(&scatter.props, &rig),
        heights: ground(terrain, coord, &heights),
        canopy: canopy_on(terrain, coord, (&scatter.props, &scatter.grass), &rig),
        stand: Stand::with_trees(
            key(terrain, coord),
            &scatter.props,
            &rig,
            &mods.taken,
            &mods.felled,
        ),
        lod,
        frame,
        turfed,
        aabb,
    }
}

/// Rebuilds the mesh and height collider of one loaded chunk overlapped by a new dig crater.
fn remesh_dug_chunk(
    commands: &mut Commands,
    spawner: &mut Spawner,
    l: &mut Loaded,
    (view, coord, mods): (&Terrain, IVec2, &ChunkMods),
) {
    let mut heights = chunk_heights(view, coord, 1);
    let dug = mods
        .digs
        .apply_to_grid(view, coord, mods.planet, &mut heights);
    l.heights = ground(view, coord, &heights);
    if l.solid {
        commands.entity(l.entity).try_insert(l.heights.clone());
    }
    let mut mesh = chunk_mesh(
        view,
        coord,
        &subsample(&heights, l.detail.lod.stride()),
        l.detail.lod,
    );
    if dug {
        let frame = Embedding::of(view, coord).frame();
        mods.digs
            .tint_mesh(&mut mesh, mods.planet, &frame, view.globe().radius());
    }
    let aabb = mesh.get_aabb();
    let handle = spawner.meshes.add(render_only(mesh));
    let mut entity = commands.entity(l.entity);
    entity.try_insert(Mesh3d(handle));
    if let Some(aabb) = aabb {
        entity.try_insert((aabb, bevy::camera::visibility::NoAutoAabb));
    }
}

impl Chunks {
    /// Returns the choppable tree hit at `at` on chunk props entity `props`.
    pub fn tree_under(&self, props: Entity, at: Vec3) -> Option<Tree> {
        let held = self.loaded.values().find(|l| l.props == Some(props))?;
        held.stand.near_tree(at)
    }

    /// Returns true if `entity` is a streamed terrain chunk collider entity.
    pub fn is_terrain(&self, entity: Entity) -> bool {
        self.loaded.values().any(|l| l.entity == entity)
    }
}

/// Propagates chopped trees into loaded chunk stands and queues visual/collider repair when felled.
pub fn note_trees(mut chunks: ResMut<Chunks>, mut told: MessageReader<TreeChopped>) {
    let Chunks { loaded, owed, .. } = &mut *chunks;
    for chop in told.read() {
        loaded
            .values_mut()
            .any(|l| l.stand.fell(chop.id, chop.power));
    }
    *owed |= loaded.values().any(|l| l.stand.dirty());
}

/// Rebuilds loaded chunk terrain meshes and height colliders overlapped by fresh [`TerrainDug`] stamps.
pub fn mend_digs(
    mut commands: Commands,
    mut chunks: ResMut<Chunks>,
    mut spawner: Spawner,
    mut dug: MessageReader<TerrainDug>,
) {
    let stamps: Vec<_> = dug.read().map(|m| m.0).collect();
    if stamps.is_empty() {
        return;
    }
    let mods = spawner_mods(&spawner);
    for (id, l) in chunks.loaded.iter_mut() {
        let view = spawner.view(*id);
        let coord = leaf_coord(&view, *id);
        if stamps
            .iter()
            .any(|s| mods.planet == s.planet && mods.digs.overlaps_chunk(&view, coord, s))
        {
            remesh_dug_chunk(&mut commands, &mut spawner, l, (&view, coord, &mods));
        }
    }
}
