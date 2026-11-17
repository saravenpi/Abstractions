//! Per-chunk level of detail: terrain mesh swaps, ground cover visibility, and the structures and lights of a new
//! chunk moved onto the globe.

use std::collections::HashMap;

use avian3d::prelude::*;
use bevy::ecs::system::SystemParam;
use bevy::light::NotShadowCaster;
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future::poll_once};

use super::Chunks;
use super::build::terrain_mesh;
use super::embed::Embedding;
use super::near::Around;
use super::plan::Loaded;
use super::raytrace;
use super::spawn::Spawner;
use crate::world::focus::Focus;
use crate::world::globe::ChunkId;
use crate::world::mesh::{Lod, leaf_coord, next_lod, ring_distance};

/// Most terrain meshes rebuilt at the same time.
const MAX_REMESH: usize = 4;

/// Chunks within this many rings show their grass and flowers.
const COVER_RINGS: i32 = 3;

/// Detail state of one loaded chunk.
pub struct Detail {
    lod: Lod,
    grass: Option<Entity>,
    shown: bool,
    structure: Option<(Entity, Collider)>,
    adopted: bool,
    remesh: Option<(Lod, Task<Mesh>)>,
}

impl Detail {
    /// Starts at level `lod`; `children` tells whether the chunk spawned child entities to look up.
    pub fn new(lod: Lod, children: bool) -> Self {
        Self {
            lod,
            grass: None,
            shown: true,
            structure: None,
            adopted: !children,
            remesh: None,
        }
    }

    /// Returns the structure child and its collider, once found.
    pub fn structure(&self) -> Option<&(Entity, Collider)> {
        self.structure.as_ref()
    }
}

/// What the detail pass reads from a child of a chunk.
type Part = (
    Option<&'static Mesh3d>,
    Option<&'static Collider>,
    Has<NotShadowCaster>,
    &'static Transform,
);

/// Reads the children of chunk entities.
#[derive(SystemParam)]
pub struct Probe<'w, 's> {
    children: Query<'w, 's, &'static Children>,
    parts: Query<'w, 's, Part>,
}

/// Moves the structure child onto the globe: its mesh, and its collider, kept only while solid.
fn fit(
    commands: &mut Commands,
    l: &mut Loaded,
    child: Entity,
    collider: Option<&Collider>,
    warp: &Embedding,
) {
    l.detail.structure = collider.map(|collider| (child, warp.collider(collider)));
    match l.detail.structure().filter(|_| l.solid) {
        Some((_, collider)) => commands.entity(child).try_insert(collider.clone()),
        None => commands.entity(child).try_remove::<Collider>(),
    };
}

/// Finds the children of a new chunk once they exist. The grass is remembered, the structure
/// mesh, its collider and the lights, all built in chart coordinates, are embedded on the globe.
fn adopt(
    commands: &mut Commands,
    l: &mut Loaded,
    probe: &Probe,
    fitter: (&Embedding, &mut Assets<Mesh>),
) {
    let Ok(children) = probe.children.get(l.entity) else {
        return;
    };
    let ((warp, meshes), props) = (fitter, l.props);
    for child in children.iter().filter(|c| Some(*c) != props) {
        match probe.parts.get(child) {
            Ok((_, _, true, _)) => l.detail.grass = Some(child),
            Ok((Some(mesh), collider, false, _)) => {
                if let Some(mut mesh) = meshes.get_mut(&mesh.0) {
                    warp.mesh(&mut mesh);
                }
                fit(commands, l, child, collider, warp);
            }
            Ok((None, _, false, at)) => {
                let (translation, rotation) = warp.pose(at.translation, at.rotation);
                let moved = Transform {
                    translation,
                    rotation,
                    ..*at
                };
                commands.entity(child).try_insert(moved);
            }
            Err(_) => {}
        }
    }
    l.detail.adopted = true;
}

/// Shows ground cover within `COVER_RINGS` of the focus and hides it beyond.
fn show_cover(commands: &mut Commands, l: &mut Loaded, ring: i32) {
    let show = ring <= COVER_RINGS;
    let Some(grass) = l.detail.grass.filter(|_| show != l.detail.shown) else {
        return;
    };
    l.detail.shown = show;
    let visibility = if show {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    commands.entity(grass).try_insert(visibility);
}

/// Swaps in a finished terrain mesh.
fn swap_finished(commands: &mut Commands, l: &mut Loaded, spawner: &mut Spawner) {
    let Some((lod, task)) = l.detail.remesh.as_mut() else {
        return;
    };
    let lod = *lod;
    let Some(mesh) = block_on(poll_once(task)) else {
        return;
    };
    l.detail.remesh = None;
    l.detail.lod = lod;
    let handle = spawner.meshes.add(mesh);
    raytrace::swap(commands, l.entity, handle, spawner.ray_tracing.0);
}

/// Starts terrain rebuilds for chunks whose wanted level changed, nearest first, while `room` allows.
fn start_remesh(
    loaded: &mut HashMap<ChunkId, Loaded>,
    around: &Around,
    spawner: &Spawner,
    room: usize,
) {
    let mut stale: Vec<(i32, ChunkId, Lod)> = loaded
        .iter()
        .filter(|(_, l)| l.detail.remesh.is_none())
        .map(|(id, l)| {
            let d = ring_distance(around.coord(*id), around.center);
            (d, *id, next_lod(l.detail.lod, d), l.detail.lod)
        })
        .filter(|(_, _, next, now)| next != now)
        .map(|(d, id, next, _)| (d, id, next))
        .collect();
    stale.sort_by_key(|(d, ..)| *d);
    let pool = AsyncComputeTaskPool::get();
    for (_, id, lod) in stale.into_iter().take(room) {
        let (terrain, on) = (spawner.view(id.face), spawner.ray_tracing.0);
        let task = pool.spawn(async move { terrain_mesh(&terrain, leaf_coord(id), lod, on) });
        if let Some(l) = loaded.get_mut(&id) {
            l.detail.remesh = Some((lod, task));
        }
    }
}

/// Runs the per-frame detail work for every loaded chunk: new children are found and embedded
/// in the frame they spawn, ground cover follows the rings and terrain meshes follow their level.
pub fn refine(
    mut commands: Commands,
    mut chunks: ResMut<Chunks>,
    mut spawner: Spawner,
    focus: Focus,
    probe: Probe,
) {
    let Some(around) = focus
        .planet()
        .and_then(|at| Around::new(&spawner.terrain, at))
    else {
        return;
    };
    let mut busy = 0;
    for (id, l) in chunks.loaded.iter_mut() {
        if !l.detail.adopted {
            let warp = Embedding::of(&spawner.view(id.face), leaf_coord(*id));
            adopt(&mut commands, l, &probe, (&warp, &mut spawner.meshes));
        }
        show_cover(
            &mut commands,
            l,
            ring_distance(around.coord(*id), around.center),
        );
        swap_finished(&mut commands, l, &mut spawner);
        busy += usize::from(l.detail.remesh.is_some());
    }
    let room = MAX_REMESH.saturating_sub(busy);
    start_remesh(&mut chunks.loaded, &around, &spawner, room);
}
