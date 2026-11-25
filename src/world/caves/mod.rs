//! Procedural subterranean caves: deterministic planner, merged shell + trimesh collider,
//! bioluminescent flora, pooled natural lighting, deep mineral nodes, and cave fauna streaming.

pub mod fauna;
pub mod flora;
pub mod mesh;
pub mod plan;
pub mod props;
#[cfg(test)]
mod tests;

use std::collections::HashMap;

use avian3d::prelude::*;
use bevy::ecs::system::SystemParam;
use bevy::math::DVec3;
use bevy::prelude::*;

pub use plan::{
    CAVE_CELL, CaveBiome, CaveId, CavePlan, CavePortal, SpineNode, cave_portal_at, caves_near,
};

use crate::player::Player;
use crate::screens::Screen;
use crate::sim::PlayerId;
use crate::space::{Anchor, Frame as SpaceFrame, Placed};
use crate::universe::BodyId;
use crate::world::mesh::Frame as ChunkFrame;
use crate::world::rocks::{RockTaken, Taken};
use crate::world::{NOT_ANIMALS, Terrain};

/// Distance in metres within which a cave streams in.
const STREAM_IN_M: f32 = 160.0;

/// Distance in metres past which a loaded cave despawns.
const STREAM_OUT_M: f64 = 190.0;

/// Distance in metres within which cave fauna spawn on living worlds.
const FAUNA_IN_M: f64 = 150.0;

/// Distance in metres past which cave fauna despawn.
const FAUNA_OUT_M: f64 = 180.0;

/// One currently streamed cave instance in the world.
struct LoadedCave {
    /// Root entity carrying [`Placed`] for automatic origin rebasing.
    root: Entity,
    /// Entrance center in planet frame metres.
    entry_planet: DVec3,
    /// Tangent frame of the cave root.
    frame: ChunkFrame,
    /// Spawned cave creature entities.
    fauna: Vec<Entity>,
    /// Deterministic layout used for lazy fauna streaming.
    plan: CavePlan,
}

/// Resource tracking streamed caves and shared cave materials.
#[derive(Resource, Default)]
pub struct ActiveCaves {
    /// Planet body currently streamed.
    planet: Option<BodyId>,
    /// Active caves keyed by [`CaveId`].
    loaded: HashMap<CaveId, LoadedCave>,
    /// Shared subterranean rock material using vertex colours.
    shell_mat: Option<Handle<StandardMaterial>>,
    /// Shared mineral crystal material using vertex colours.
    mineral_mat: Option<Handle<StandardMaterial>>,
}

/// Bundled system parameters for cave streaming.
#[derive(SystemParam)]
struct CaveStream<'w, 's> {
    /// Entity command queue.
    commands: Commands<'w, 's>,
    /// Active terrain view.
    terrain: Option<Res<'w, Terrain>>,
    /// Render anchor and active frame.
    anchor: Option<Res<'w, Anchor>>,
    /// Local player transform query.
    players: Query<'w, 's, (&'static Transform, &'static PlayerId), With<Player>>,
    /// Quarry ledger for persistent cave mineral depletion.
    taken: Res<'w, Taken>,
    /// Active cave state resource.
    active: ResMut<'w, ActiveCaves>,
    /// Mesh asset store.
    meshes: ResMut<'w, Assets<Mesh>>,
    /// Material asset store.
    materials: ResMut<'w, Assets<StandardMaterial>>,
}

/// Returns shared cave rock and mineral materials, initializing them once.
fn shared_materials(
    active: &mut ActiveCaves,
    materials: &mut Assets<StandardMaterial>,
) -> (Handle<StandardMaterial>, Handle<StandardMaterial>) {
    let shell = active
        .shell_mat
        .get_or_insert_with(|| {
            materials.add(StandardMaterial {
                base_color: Color::WHITE,
                perceptual_roughness: 0.92,
                ..default()
            })
        })
        .clone();
    let mineral = active
        .mineral_mat
        .get_or_insert_with(|| {
            materials.add(StandardMaterial {
                base_color: Color::WHITE,
                emissive: LinearRgba::rgb(0.18, 0.22, 0.26),
                perceptual_roughness: 0.35,
                metallic: 0.25,
                ..default()
            })
        })
        .clone();
    (shell, mineral)
}

/// Despawns all loaded caves and clears the active map.
fn despawn_all(commands: &mut Commands, active: &mut ActiveCaves) {
    for (_, cave) in active.loaded.drain() {
        for critter in cave.fauna {
            commands.entity(critter).try_despawn();
        }
        commands.entity(cave.root).try_despawn();
    }
    active.planet = None;
}

/// Spawns one cave root, shell + trimesh collider, flora, 3 pooled lights, and deep mineral nodes.
fn spawn_cave(s: &mut CaveStream, terrain: &Terrain, plan: CavePlan) -> LoadedCave {
    let frame = ChunkFrame::at(terrain, plan.entry_xz);
    let entry_planet = terrain.embed(plan.entry_xz.x, plan.entry_y, plan.entry_xz.y);
    let (shell_mat, mineral_mat) = shared_materials(&mut s.active, &mut s.materials);
    let (shell_mesh, collider) = mesh::build_cave_shell(&plan);
    let (flora_mesh, flora_mat) =
        flora::build_flora_mesh(&plan, props::cave_accent(terrain.globe(), &frame, &plan));
    let (shell_h, flora_h, flora_m) = (
        s.meshes.add(shell_mesh),
        s.meshes.add(flora_mesh),
        s.materials.add(flora_mat),
    );
    let placed = Placed {
        pos: frame.origin,
        rot: frame.rot,
    };
    let root = s
        .commands
        .spawn((
            Name::new("Cave"),
            placed,
            Transform::default(),
            Visibility::default(),
            DespawnOnEnter(Screen::Title),
        ))
        .id();
    s.commands.spawn((
        Mesh3d(shell_h),
        MeshMaterial3d(shell_mat),
        RigidBody::Static,
        collider,
        CollisionLayers::new(NOT_ANIMALS, LayerMask::ALL),
        Transform::default(),
        ChildOf(root),
    ));
    s.commands.spawn((
        Mesh3d(flora_h),
        MeshMaterial3d(flora_m),
        Transform::default(),
        ChildOf(root),
    ));
    flora::spawn_cave_lights(&mut s.commands, &plan, root);
    props::spawn_cave_minerals(
        &mut s.commands,
        &plan,
        (terrain.globe(), &frame),
        (&mut s.meshes, mineral_mat),
        (&s.taken, root),
    );
    LoadedCave {
        root,
        entry_planet,
        frame,
        fauna: Vec::new(),
        plan,
    }
}

/// Despawns caves past [`STREAM_OUT_M`] and cave fauna past [`FAUNA_OUT_M`].
fn prune_distant(commands: &mut Commands, active: &mut ActiveCaves, player_planet: DVec3) {
    active.loaded.retain(|_, cave| {
        let dist = cave.entry_planet.distance(player_planet);
        if dist > STREAM_OUT_M {
            cave.fauna
                .drain(..)
                .for_each(|e| commands.entity(e).try_despawn());
            commands.entity(cave.root).try_despawn();
            return false;
        }
        if dist > FAUNA_OUT_M {
            cave.fauna
                .drain(..)
                .for_each(|e| commands.entity(e).try_despawn());
        }
        true
    });
}

/// Lazily populates cave fauna for active caves within [`FAUNA_IN_M`] on living planets.
fn populate_fauna(s: &mut CaveStream, terrain: &Terrain, player_planet: DVec3) {
    let Some(life) = terrain.globe().life() else {
        return;
    };
    for cave in s.active.loaded.values_mut() {
        if cave.fauna.is_empty() && cave.entry_planet.distance(player_planet) <= FAUNA_IN_M {
            cave.fauna = fauna::spawn_cave_fauna(
                &mut s.commands,
                &cave.plan,
                &cave.frame,
                life,
                (&mut s.meshes, &mut s.materials),
            );
        }
    }
}

/// Streams caves near the local player while on a planet surface, doing zero work in space.
fn stream_caves(mut s: CaveStream) {
    let Some(anchor) = s.anchor.as_deref().copied() else {
        return;
    };
    let SpaceFrame::Surface(body) = anchor.frame else {
        if !s.active.loaded.is_empty() {
            despawn_all(&mut s.commands, &mut s.active);
        }
        return;
    };
    let (Some(terrain), Some((tf, _))) = (
        s.terrain.as_deref().cloned(),
        s.players.iter().find(|(_, id)| **id == PlayerId::LOCAL),
    ) else {
        return;
    };
    if s.active.planet != Some(body) {
        despawn_all(&mut s.commands, &mut s.active);
        s.active.planet = Some(body);
    }
    let player_planet = anchor.to_frame(tf.translation);
    prune_distant(&mut s.commands, &mut s.active, player_planet);
    let chart = terrain.locate(player_planet);
    for plan in caves_near(&terrain, Vec2::new(chart.x, chart.z), STREAM_IN_M) {
        if !s.active.loaded.contains_key(&plan.id) {
            let id = plan.id;
            let loaded = spawn_cave(&mut s, &terrain, plan);
            s.active.loaded.insert(id, loaded);
        }
    }
    populate_fauna(&mut s, &terrain, player_planet);
}

/// Clears active cave tracking when returning to the title screen.
fn clear_caves(mut active: ResMut<ActiveCaves>) {
    active.loaded.clear();
    active.planet = None;
}

/// Registers procedural cave streaming, lighting, mineral syncing, and cave fauna systems.
pub fn plugin(app: &mut App) {
    app.init_resource::<ActiveCaves>()
        .add_systems(OnEnter(Screen::Title), clear_caves)
        .add_systems(
            Update,
            (
                stream_caves
                    .run_if(resource_exists::<Assets<Mesh>>)
                    .run_if(resource_exists::<Assets<StandardMaterial>>),
                flora::pulse_cave_lights,
                props::sync_taken_nodes.run_if(on_message::<RockTaken>),
                fauna::step_cave_fauna,
            )
                .run_if(in_state(Screen::Gameplay)),
        );
}
