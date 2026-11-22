//! The shared spheres every body is drawn on: a cube sphere at a few levels of detail, grown just
//! enough that every facet lies outside the ball it stands for. The shader of the bodies finds the
//! true ball inside it pixel by pixel, so the mesh only has to cover it: a coarse mesh still draws
//! a round rim, and a body costs a few thousand triangles whatever its size on the screen. Each
//! level is built once off the main thread the first time a body needs it.

use std::f32::consts::FRAC_PI_2;

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future::poll_once};

use super::bake::SHELL_UV;
use crate::world::globe::Face;

/// Segments along one side of a face of the cube sphere at each level: 192 triangles for a dot,
/// 12,288 for the finest.
pub const SEGMENTS: [u32; 4] = [4, 8, 16, 32];

/// Most a sphere may stand out of the ball it covers, in game pixels: what the shader cuts away
/// around the rim, and what the halo, drawn on the facets themselves, is off by.
pub const BULGE: f32 = 0.5;

/// Widest angular radius of the circle through the corners of a facet, against the angle of one
/// segment: half the diagonal of a square cell, with a margin for the cells that are not square.
pub const REACH: f32 = 0.75;

/// Returns how much a sphere of `segments` segments along a face is grown at most, as a share of
/// the radius over 1: a facet whose corners lie on a circle of angular radius `r` has its plane
/// `cos r` from the centre.
pub fn bulge(segments: u32) -> f32 {
    1.0 / (REACH * FRAC_PI_2 / segments as f32).cos() - 1.0
}

/// Returns the level of [`SEGMENTS`] whose sphere stands out of a body whose radius covers `pixels`
/// game pixels by no more than [`BULGE`], or the last level.
pub fn level(pixels: f32) -> usize {
    let fits = |segments: &u32| bulge(*segments) * pixels <= BULGE;
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

/// Returns how much the facets `indices` of the unit `points` must grow for each to lie outside
/// the unit ball: one over the distance of the nearest facet plane from the centre.
pub fn growth(points: &[[f32; 3]], indices: &[u32]) -> f32 {
    let at = |i: u32| Vec3::from(points[i as usize]);
    let plane = |t: &[u32; 3]| {
        let [a, b, c] = t.map(at);
        (b - a).cross(c - a).normalize_or_zero().dot(a)
    };
    let nearest = indices
        .as_chunks::<3>()
        .0
        .iter()
        .map(plane)
        .fold(1.0, f32::min);
    1.0 / nearest.max(f32::MIN_POSITIVE) + 1.0e-5
}

/// Returns the unit `points` moved out by the [`growth`] of the facets `indices`.
fn grown(points: &[[f32; 3]], indices: &[u32]) -> Vec<[f32; 3]> {
    let share = growth(points, indices);
    let out = |p: &[f32; 3]| (Vec3::from(*p) * share).to_array();
    points.iter().map(out).collect()
}

/// Returns the cube sphere of `segments` segments along each side of a face, in equal angles so
/// every facet spans about the same angle, grown by [`growth`] so it covers the unit ball, with
/// the normals of the smooth sphere. The ground has texture coordinates 0 and the cloud shell,
/// when `shell` is set, [`SHELL_UV`].
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
    let corners = grown(&points, &indices);
    let uv = [[0.0, 0.0], SHELL_UV][usize::from(shell)];
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![uv; points.len()])
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, points)
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, corners)
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
