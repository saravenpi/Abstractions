//! Per-chunk level of detail: terrain mesh swaps, ground cover visibility and structure colliders.

use std::collections::HashMap;

use avian3d::prelude::*;
use bevy::ecs::system::SystemParam;
use bevy::light::NotShadowCaster;
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future::poll_once};

use super::build::terrain_mesh;
use super::plan::Loaded;
use super::raytrace;
use super::spawn::Spawner;
use crate::world::mesh::{Lod, next_lod, ring_distance};

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

/// Reads the children of chunk entities.
#[derive(SystemParam)]
pub struct Probe<'w, 's> {
    children: Query<'w, 's, &'static Children>,
    parts: Query<'w, 's, (Option<&'static Collider>, Has<NotShadowCaster>)>,
}

/// Finds the grass and structure children once they exist; far structures lose their collider.
fn adopt(commands: &mut Commands, l: &mut Loaded, probe: &Probe) {
    if l.detail.adopted {
        return;
    }
    let Ok(children) = probe.children.get(l.entity) else {
        return;
    };
    for child in children.iter().filter(|c| Some(*c) != l.props) {
        match probe.parts.get(child) {
            Ok((_, true)) => l.detail.grass = Some(child),
            Ok((Some(collider), false)) => {
                l.detail.structure = Some((child, collider.clone()));
                if !l.solid {
                    commands.entity(child).try_remove::<Collider>();
                }
            }
            _ => {}
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
    loaded: &mut HashMap<IVec2, Loaded>,
    center: IVec2,
    spawner: &Spawner,
    room: usize,
) {
    let mut stale: Vec<(i32, IVec2, Lod)> = loaded
        .iter()
        .filter(|(_, l)| l.detail.remesh.is_none())
        .map(|(c, l)| {
            let d = ring_distance(*c, center);
            (d, *c, next_lod(l.detail.lod, d), l.detail.lod)
        })
        .filter(|(_, _, next, now)| next != now)
        .map(|(d, c, next, _)| (d, c, next))
        .collect();
    stale.sort_by_key(|(d, ..)| *d);
    let pool = AsyncComputeTaskPool::get();
    for (_, coord, lod) in stale.into_iter().take(room) {
        let (terrain, on) = (spawner.terrain.clone(), spawner.ray_tracing.0);
        let task = pool.spawn(async move { terrain_mesh(&terrain, coord, lod, on) });
        if let Some(l) = loaded.get_mut(&coord) {
            l.detail.remesh = Some((lod, task));
        }
    }
}

/// Runs the per-frame detail work for every loaded chunk around `center`.
pub fn refine(
    commands: &mut Commands,
    loaded: &mut HashMap<IVec2, Loaded>,
    center: IVec2,
    spawner: &mut Spawner,
    probe: &Probe,
) {
    let mut busy = 0;
    for (coord, l) in loaded.iter_mut() {
        adopt(commands, l, probe);
        show_cover(commands, l, ring_distance(*coord, center));
        swap_finished(commands, l, spawner);
        busy += usize::from(l.detail.remesh.is_some());
    }
    start_remesh(loaded, center, spawner, MAX_REMESH.saturating_sub(busy));
}
