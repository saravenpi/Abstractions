//! Per-chunk level of detail: terrain mesh swaps and ground cover on demand.

use bevy::camera::primitives::Aabb;
use bevy::camera::visibility::{NoAutoAabb, NoFrustumCulling};
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future::poll_once};

use super::Chunks;
use super::budget::Spend;
use super::build::{ground_cover, terrain_mesh};
use super::dither::Way;
use super::fade::Fades;
use super::plan::Loaded;
use super::scene::Scene;
use super::spawn::Spawner;
use super::turf::Turf;
use super::work::start_work;
use crate::world::Terrain;
use crate::world::ground::GroundMaterial;
use crate::world::mesh::Lod;

/// Most terrain meshes and ground covers rebuilt at the same time.
const MAX_REMESH: usize = 4;

/// Terrain mesh paired with its precomputed local bounding box.
type BakedMesh = (Mesh, Option<Aabb>);

/// Detail state of one loaded chunk.
pub struct Detail {
    /// The level of its terrain mesh.
    pub(super) lod: Lod,
    /// The grass and flowers of the chunk.
    pub turf: Turf,
    remesh: Option<(Lod, Task<BakedMesh>)>,
}

impl Detail {
    /// Starts at level `lod` with its ground cover.
    pub fn new(lod: Lod, turf: Turf) -> Self {
        Self {
            lod,
            turf,
            remesh: None,
        }
    }

    /// Starts the task that does `work` for the chunk at `coord` of the chart `terrain`.
    pub(super) fn begin(
        &mut self,
        work: Work,
        (terrain, coord): (Terrain, IVec2),
        pool: &AsyncComputeTaskPool,
    ) {
        match work {
            Work::Terrain(lod) => {
                let task = pool.spawn(async move { terrain_mesh(&terrain, coord, lod) });
                self.remesh = Some((lod, task));
            }
            Work::Cover => self
                .turf
                .start(pool.spawn(async move { ground_cover(&terrain, coord) })),
        }
    }

    /// Returns true while a task rebuilds the terrain or the ground cover of the chunk.
    pub(super) fn busy(&self) -> bool {
        self.remesh.is_some() || self.turf.building()
    }
}

/// Swaps in a finished terrain mesh, the budget allowing. Returns true when it did.
fn swap_finished(
    commands: &mut Commands,
    l: &mut Loaded,
    spawner: &mut Spawner,
    spend: &mut Spend,
) -> bool {
    let Some((lod, task)) = l.detail.remesh.as_mut() else {
        return false;
    };
    if !task.is_finished() || !spend.more() {
        return false;
    }
    let lod = *lod;
    let Some((mesh, aabb)) = block_on(poll_once(task)) else {
        return false;
    };
    l.detail.remesh = None;
    l.detail.lod = lod;
    let handle = spawner.meshes.add(mesh);
    let mut chunk = commands.entity(l.entity);
    chunk.try_insert(Mesh3d(handle));
    if let Some(aabb) = aabb {
        chunk.try_insert((aabb, NoAutoAabb));
    }
    true
}

/// What a loaded chunk should rebuild: its terrain at another level, or its ground cover.
pub(super) enum Work {
    /// The terrain mesh at this level.
    Terrain(Lod),
    /// The ground cover.
    Cover,
}

/// What changed on a chunk in one frame.
struct Changed {
    /// The ground cover that came into view.
    grass: Option<Entity>,
    /// The mesh the chunk drew before its terrain was rebuilt.
    terrain: Option<Handle<Mesh>>,
}

/// Fades what `changed` on `chunk`, which the player can see, from `now` on: new ground cover
/// fades in, and rebuilt terrain fades in while a ghost that draws the old mesh fades out and
/// goes. A chunk that is still fading in swaps its terrain at once.
fn soften(
    commands: &mut Commands,
    fades: &mut Fades,
    chunk: Entity,
    changed: Changed,
    (material, now): (&Handle<GroundMaterial>, f64),
) {
    if let Some(grass) = changed.grass {
        fades.begin(commands, grass, vec![grass], (Way::In, now));
    }
    let Some(old) = changed.terrain.filter(|_| fades.way(chunk).is_none()) else {
        return;
    };
    let ghost = (
        Mesh3d(old),
        MeshMaterial3d(material.clone()),
        NoFrustumCulling,
        ChildOf(chunk),
    );
    let ghost = commands.spawn(ghost).id();
    fades.cross(commands, chunk, ghost, now);
}

/// Runs the per-frame detail work for every loaded chunk: ground cover follows the rings and
/// terrain meshes follow their level, within the swap budget of the frame. What changes where
/// the player can see it fades over a moment.
pub fn refine(
    mut commands: Commands,
    mut chunks: ResMut<Chunks>,
    mut spawner: Spawner,
    scene: Scene,
) {
    let Some(around) = scene.at().and_then(|at| chunks.around(&spawner, at)) else {
        return;
    };
    let cover = spawner.reach().cover();
    let mut spend = Spend::of(chunks.stretch.millis(spawner.shares().swaps));
    let (eye, now) = (scene.eye(), scene.now());
    let Chunks { loaded, fades, .. } = &mut *chunks;
    let mut busy = 0;
    for (id, l) in loaded.iter_mut() {
        let rings = (around.rank(*id), cover);
        let turf = &mut l.detail.turf;
        let grass = turf.tend(&mut commands, l.entity, rings, (&mut spawner, &mut spend));
        let swapped = swap_finished(&mut commands, l, &mut spawner, &mut spend);
        let changed = l.shown && (swapped || grass.is_some());
        if changed && eye.is_some_and(|eye| eye.sees(*id)) {
            let terrain = scene.mesh(l.entity).filter(|_| swapped);
            let with = (&spawner.assets.material, now);
            soften(
                &mut commands,
                fades,
                l.entity,
                Changed { grass, terrain },
                with,
            );
        }
        busy += usize::from(l.detail.busy());
    }
    start_work(loaded, &around, &spawner, MAX_REMESH.saturating_sub(busy));
}
