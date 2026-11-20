//! The foam on the sea: one pooled mesh of quads, wake segments and pieces of ripple rings.
//!
//! The quads are written once, when laid, and never touched again: the vertex shader widens,
//! spreads and fades each one from the time it was laid (the technique of the wakes of MotoGP,
//! https://www.shawnhargreaves.com/blog/motogp-spray-and-wakes.html). The oldest quad is reused
//! when the pool is full.

use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::NoFrustumCulling;
use bevy::light::NotShadowCaster;
use bevy::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use bevy::prelude::*;

use super::material::FoamMaterial;
use super::tune::QUADS;
use crate::screens::Screen;
use crate::space::Rebased;

/// Marks the one foam mesh. It is `Rebased`, and the quads are written relative to it, so an
/// anchor rebase moves them all with it.
#[derive(Component, Debug, Clone, Copy)]
pub struct FoamMesh;

/// The foam mesh, its material, the next quad to reuse and how long the foam still shows.
#[derive(Resource, Debug, Default, Clone)]
pub struct Foam {
    /// The pooled mesh.
    pub mesh: Handle<Mesh>,
    /// Its material.
    pub material: Handle<FoamMaterial>,
    /// Next quad slot to write.
    pub next: usize,
    /// Foam clock in seconds until which some quad still shows.
    pub until: f32,
}

/// Vertices of the foam mesh: four per quad.
pub const VERTICES: usize = QUADS * 4;

/// One quad of foam: a strip between two ends, each pushed out along its own direction.
///
/// A vertex at side `s` and age `t` sits at its end plus its direction times
/// `shape.x + shape.y * t + s * (shape.z + shape.w * t)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quad {
    /// The two ends, in the local space of the foam mesh.
    pub ends: [Vec3; 2],
    /// Unit direction each end is pushed out along.
    pub dirs: [Vec3; 2],
    /// Offset, its growth per second, the half width and its growth per second, in metres.
    pub shape: Vec4,
    /// Time each end was laid, in seconds of the foam clock.
    pub births: [f32; 2],
    /// Strength from 0 to 1 and life in seconds.
    pub fade: Vec2,
}

/// Returns the triangle indices: two triangles per quad.
fn indices() -> Vec<u32> {
    (0..QUADS as u32)
        .flat_map(|quad| {
            let a = quad * 4;
            [a, a + 1, a + 2, a + 2, a + 1, a + 3]
        })
        .collect()
}

/// Builds the foam mesh once: every quad parked, with no life.
pub fn mesh() -> Mesh {
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.0f32; 3]; VERTICES])
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0f32, 1.0, 0.0]; VERTICES])
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0f32; 2]; VERTICES])
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_1, vec![[0.0f32; 2]; VERTICES])
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, vec![[0.0f32; 4]; VERTICES])
        .with_inserted_indices(Indices::U32(indices()))
}

/// Writes the four vertices of `quad` into slot `slot` of `mesh`, in place.
pub fn write(mesh: &mut Mesh, slot: usize, quad: &Quad) {
    use VertexAttributeValues::{Float32x2, Float32x3, Float32x4};
    let base = (slot % QUADS) * 4;
    let corner = |i: usize| (i / 2, [-1.0f32, 1.0][i % 2]);
    if let Some(Float32x3(positions)) = mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION) {
        (0..4).for_each(|i| positions[base + i] = quad.ends[corner(i).0].to_array());
    }
    if let Some(Float32x3(normals)) = mesh.attribute_mut(Mesh::ATTRIBUTE_NORMAL) {
        (0..4).for_each(|i| normals[base + i] = quad.dirs[corner(i).0].to_array());
    }
    if let Some(Float32x2(uvs)) = mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0) {
        (0..4).for_each(|i| uvs[base + i] = [corner(i).1, quad.births[corner(i).0]]);
    }
    if let Some(Float32x2(fades)) = mesh.attribute_mut(Mesh::ATTRIBUTE_UV_1) {
        (0..4).for_each(|i| fades[base + i] = quad.fade.to_array());
    }
    if let Some(Float32x4(shapes)) = mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR) {
        (0..4).for_each(|i| shapes[base + i] = quad.shape.to_array());
    }
}

/// Returns the directions of the two edges of each of the `pieces` sectors of a ring in the
/// plane whose unit normal is `up`, going round once.
pub fn sectors(up: Vec3, pieces: usize) -> impl Iterator<Item = [Vec3; 2]> {
    let side = up.any_orthonormal_vector();
    let turn = move |piece: usize| {
        let angle = std::f32::consts::TAU * piece as f32 / pieces.max(1) as f32;
        Quat::from_axis_angle(up, angle) * side
    };
    (0..pieces).map(move |piece| [turn(piece), turn(piece + 1)])
}

/// Spawns the foam mesh when gameplay starts, unless it already exists.
pub fn spawn(
    mut commands: Commands,
    mut assets: (ResMut<Assets<Mesh>>, ResMut<Assets<FoamMaterial>>),
    existing: Query<(), With<FoamMesh>>,
    mut foam: ResMut<Foam>,
) {
    if !existing.is_empty() {
        return;
    }
    *foam = Foam {
        mesh: assets.0.add(mesh()),
        material: assets.1.add(FoamMaterial::default()),
        ..Foam::default()
    };
    commands.spawn((
        FoamMesh,
        Mesh3d(foam.mesh.clone()),
        MeshMaterial3d(foam.material.clone()),
        Transform::IDENTITY,
        Visibility::Hidden,
        Rebased,
        NoFrustumCulling,
        NotShadowCaster,
        DespawnOnEnter(Screen::Title),
    ));
}

/// Writes `quad` into the next slot of the foam and keeps the foam shown for its life.
pub fn lay(meshes: &mut Assets<Mesh>, foam: &mut Foam, quad: &Quad) {
    if let Some(mut mesh) = meshes.get_mut(&foam.mesh) {
        write(&mut mesh, foam.next, quad);
    }
    foam.next = (foam.next + 1) % QUADS;
    foam.until = foam.until.max(quad.births[1] + quad.fade.y);
}
