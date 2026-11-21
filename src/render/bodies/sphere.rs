//! The shared spheres every body is drawn on: a unit cube sphere at a few levels of detail, cut
//! by the size the body covers on the screen so its edge stays round, each level built once off
//! the main thread the first time a body needs it.

use std::f32::consts::FRAC_PI_2;

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future::poll_once};

use super::bake::SHELL_UV;
use crate::world::globe::Face;

/// Segments along one side of a face of the cube sphere at each level: 192 triangles for a dot,
/// 786,432 for a disc filling the screen.
pub const SEGMENTS: [u32; 7] = [4, 8, 16, 32, 64, 128, 256];

/// Longest a facet edge on the rim of a body may be, in game pixels.
pub const EDGE: f32 = 2.0;

/// Widest edge of a facet against the side of a segment, a quarter turn over the segments: the
/// diagonal of a square cell, each cell cut along its shorter one.
pub const WIDEST: f32 = 1.415;

/// Returns the level of [`SEGMENTS`] whose facets stay under [`EDGE`] on the rim of a body whose
/// radius covers `pixels` game pixels: a face spans a quarter turn in that many segments, and a
/// facet spans at most [`WIDEST`] segments of it. Past the last level it is the last.
pub fn level(pixels: f32) -> usize {
    let fits = |segments: &u32| pixels * FRAC_PI_2 * WIDEST / *segments as f32 <= EDGE;
    SEGMENTS.iter().position(fits).unwrap_or(SEGMENTS.len() - 1)
}

/// Returns the corners of the two facets of the cell `[a, b, c, d]`, corners at `(i, j)`,
/// `(i + 1, j)`, `(i, j + 1)` and `(i + 1, j + 1)`, cut along its shorter diagonal and turning
/// counter-clockwise seen from outside.
fn cut(points: &[[f32; 3]], [a, b, c, d]: [u32; 4]) -> [u32; 6] {
    let at = |i: u32| Vec3::from(points[i as usize]);
    match at(b).dot(at(c)) >= at(a).dot(at(d)) {
        true => [a, c, b, b, c, d],
        false => [a, c, d, a, d, b],
    }
}

/// Returns the unit cube sphere of `segments` segments along each side of a face, in equal angles so
/// every facet spans about the same angle, with the normals of the smooth sphere. The ground has
/// texture coordinates 0 and the cloud shell, when `shell` is set, [`SHELL_UV`].
pub fn sphere(segments: u32, shell: bool) -> Mesh {
    let side = segments + 1;
    let mut points = Vec::with_capacity((6 * side * side) as usize);
    let mut indices = Vec::with_capacity((36 * segments * segments) as usize);
    let step = 2.0 / f64::from(segments);
    for face in Face::ALL {
        let first = points.len() as u32;
        for j in 0..side {
            for i in 0..side {
                let (u, v) = (f64::from(i) * step - 1.0, f64::from(j) * step - 1.0);
                points.push(face.dir(u, v).as_vec3().to_array());
            }
        }
        let at = |i: u32, j: u32| first + j * side + i;
        for (i, j) in (0..segments).flat_map(|j| (0..segments).map(move |i| (i, j))) {
            let cell = [at(i, j), at(i + 1, j), at(i, j + 1), at(i + 1, j + 1)];
            indices.extend(cut(&points, cell));
        }
    }
    let uv = [[0.0, 0.0], SHELL_UV][usize::from(shell)];
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![uv; points.len()])
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, points.clone())
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, points)
    .with_inserted_indices(Indices::U32(indices))
}

/// One level of the spheres: not asked for yet, being built, or ready.
#[derive(Default)]
enum Slot {
    /// No body has needed it yet.
    #[default]
    Idle,
    /// Being built off the main thread.
    Building(Task<Mesh>),
    /// Built and stored.
    Ready(Handle<Mesh>),
}

/// The levels of the ground sphere, then those of the cloud shell.
#[derive(Resource, Default)]
pub(super) struct Spheres([[Slot; SEGMENTS.len()]; 2]);

impl Spheres {
    /// Returns the finest sphere ready at or under `level`, the cloud shell when `shell` is set,
    /// and starts building `level` when nothing has asked for it before.
    pub(super) fn pick(&mut self, level: usize, shell: bool) -> Option<Handle<Mesh>> {
        let slots = &mut self.0[usize::from(shell)];
        if matches!(slots[level], Slot::Idle) {
            let segments = SEGMENTS[level];
            let task = AsyncComputeTaskPool::get().spawn(async move { sphere(segments, shell) });
            slots[level] = Slot::Building(task);
        }
        slots[..=level].iter().rev().find_map(|slot| match slot {
            Slot::Ready(handle) => Some(handle.clone()),
            _ => None,
        })
    }
}

/// Stores every sphere whose build is done.
pub(super) fn finish(mut spheres: ResMut<Spheres>, mut meshes: ResMut<Assets<Mesh>>) {
    for slot in spheres.0.iter_mut().flatten() {
        let Slot::Building(task) = slot else {
            continue;
        };
        if let Some(mesh) = block_on(poll_once(task)) {
            *slot = Slot::Ready(meshes.add(mesh));
        }
    }
}
