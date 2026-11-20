//! Keeping the mesh of the cloud shell: it is built over the part of the sphere the camera sees,
//! off the main thread, and built again when the camera sees past it or has come so near or
//! gone so far that its facets are too coarse or finer than they need to be. The system parameter
//! that holds it also holds the material of the shell.

use std::f32::consts::PI;

use bevy::asset::AssetId;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::tasks::{
    AsyncComputeTaskPool, Task, TaskPool, block_on, futures_lite::future::poll_once,
};

use super::material::{CloudMaterial, CloudUniform};
use super::shell::{Cap, GROWTH, MARGIN, TOLERANCE, build_within, reach};
use super::tune::{COARSEN, SEEN_ERROR, SPARE};

/// What a mesh of the shell was built for.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Built {
    /// The mesh asset that holds it.
    mesh: AssetId<Mesh>,
    /// Radius of the shell in metres.
    radius: f32,
    /// The part of the sphere it covers.
    cap: Cap,
    /// Most metres a point of it lies from the true sphere.
    tolerance: f32,
}

impl Built {
    /// Returns true while this mesh covers a camera under direction `up` that sees `need`
    /// radians of the shell.
    fn covers(&self, mesh: AssetId<Mesh>, radius: f32, up: Vec3, need: f32) -> bool {
        let leaning = self.cap.centre.angle_between(up);
        self.mesh == mesh
            && self.radius.to_bits() == radius.to_bits()
            && leaning + need <= self.cap.half
    }

    /// Returns true while this mesh covers the camera, is not much wider than what it sees, and
    /// is neither coarser than the `wanted` tolerance in metres nor more than [`COARSEN`] times
    /// finer.
    fn fits(&self, mesh: AssetId<Mesh>, radius: f32, view: (Vec3, f32, f32)) -> bool {
        let (up, need, wanted) = view;
        self.covers(mesh, radius, up, need)
            && self.cap.half <= 3.0 * (need + MARGIN)
            && self.tolerance <= wanted
            && wanted <= COARSEN * self.tolerance
    }
}

/// A mesh being built, and what it is built for.
struct Pending {
    /// What the mesh is built for.
    built: Built,
    /// The task that builds it.
    task: Task<Mesh>,
}

/// The mesh of the shell in the asset store and the one being built to replace it.
#[derive(Default)]
pub struct Mending {
    /// What the mesh in the asset store was built for, once it holds one.
    built: Option<Built>,
    /// The mesh being built.
    pending: Option<Pending>,
}

/// Returns the most metres the mesh may lie from the true sphere for a camera `altitude`
/// metres above a planet of `ground` metres under a shell of `radius` metres: [`SEEN_ERROR`] of
/// the distance to the nearest point of the shell, and never finer than [`TOLERANCE`].
pub fn wanted(ground: f32, altitude: f32, radius: f32) -> f32 {
    let nearest = (ground + altitude.max(0.0) - radius).abs();
    (SEEN_ERROR * nearest).max(TOLERANCE)
}

/// Starts building the mesh for a camera under `up` that sees `need` radians of the shell of
/// `radius` metres and wants a tolerance of `wanted` metres, and returns what it is built for.
fn start(handle: AssetId<Mesh>, radius: f32, view: (Vec3, f32, f32)) -> Pending {
    let (up, need, wanted) = view;
    let cap = Cap {
        centre: up,
        half: (need * GROWTH + MARGIN).min(PI),
    };
    let tolerance = (wanted / SPARE).max(TOLERANCE);
    let pool = AsyncComputeTaskPool::get_or_init(TaskPool::default);
    Pending {
        built: Built {
            mesh: handle,
            radius,
            cap,
            tolerance,
        },
        task: pool.spawn(async move { build_within(radius, cap, tolerance) }),
    }
}

/// Swaps the mesh the task of `state` has finished into `handle`.
fn land(state: &mut Mending, meshes: &mut Assets<Mesh>, handle: &Handle<Mesh>) {
    let done = (state.pending.as_mut()).and_then(|pending| {
        block_on(poll_once(&mut pending.task)).map(|mesh| (pending.built, mesh))
    });
    let Some((built, mesh)) = done else {
        return;
    };
    state.pending = None;
    if let Some(mut slot) = meshes.get_mut(handle) {
        *slot = mesh;
        state.built = Some(built);
    }
}

/// Takes a finished mesh into `handle`, starts building another when the camera has seen past
/// what the held one and the one being built cover, or needs another tolerance, and returns
/// whether the mesh held covers the camera now.
fn refresh(
    state: &mut Mending,
    meshes: &mut Assets<Mesh>,
    handle: &Handle<Mesh>,
    sight: (f32, f32, f32),
    up: Vec3,
) -> bool {
    land(state, meshes, handle);
    let (ground, altitude, radius) = sight;
    let need = reach(ground, altitude, radius);
    let view = (up, need, wanted(ground, altitude, radius));
    let fits = |built: &Built| built.fits(handle.id(), radius, view);
    let held = state.built.as_ref().is_some_and(fits);
    let coming = state
        .pending
        .as_ref()
        .is_some_and(|pending| fits(&pending.built));
    if !held && !coming {
        state.pending = Some(start(handle.id(), radius, view));
    }
    (state.built.as_ref()).is_some_and(|built| built.covers(handle.id(), radius, up, need))
}

/// The assets of the cloud shell: its material, its mesh and what the mesh was built for.
#[derive(SystemParam)]
pub struct Shell<'w, 's> {
    /// The material of the shell.
    materials: ResMut<'w, Assets<CloudMaterial>>,
    /// The meshes, one of which is the mesh of the shell.
    meshes: ResMut<'w, Assets<Mesh>>,
    /// The mesh of the shell and the one being built.
    mending: Local<'s, Mending>,
}

impl Shell<'_, '_> {
    /// Builds the mesh `handle` again, off the main thread, when the camera has seen past what it
    /// covers, and returns whether the mesh covers the camera now: `sight` is the radius of the
    /// planet, the altitude of the camera and the radius of the shell in metres, and `up` the
    /// unit direction under the camera.
    pub fn remesh(&mut self, handle: &Handle<Mesh>, sight: (f32, f32, f32), up: Vec3) -> bool {
        refresh(&mut self.mending, &mut self.meshes, handle, sight, up)
    }

    /// Gives the material `handle` its shader values and the cover map it reads, when they
    /// differ from what it holds.
    pub fn paint(
        &mut self,
        handle: &Handle<CloudMaterial>,
        clouds: CloudUniform,
        cover: &Option<Handle<Image>>,
    ) {
        let same = |material: &CloudMaterial| material.clouds == clouds && material.cover == *cover;
        if self.materials.get(handle).is_none_or(same) {
            return;
        }
        if let Some(mut material) = self.materials.get_mut(handle) {
            material.clouds = clouds;
            material.cover.clone_from(cover);
        }
    }
}
