//! Baking the far bodies: each solid planet and moon of a star system gets a small sphere of its
//! real surface, built off the main thread when the camera enters the system, with its cloud
//! shell around it. Until it is ready the body shows the flat colour of its class.

use std::sync::Arc;

use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future::poll_once};

use super::material::BodyMaterial;
use crate::render::air::Air;
use crate::render::clouds::CELL;
use crate::universe::{PlanetClass, PlanetSpec};
use crate::world::Globe;
use crate::world::globe::far::{Ball, ball};

/// Texture coordinates that mark a triangle of the cloud shell; the ground has none.
pub const SHELL_UV: [f32; 2] = [2.0, 2.0];

/// Segments around and along the cloud shell of a ball: 960 triangles.
const SHELL: (u32, u32) = (32, 16);

/// A ball being baked for a body.
#[derive(Component)]
pub(super) struct Baking(Task<Baked>);

/// A baked body: its ball and the cells of the cloud noise along the radius of its cloud shell,
/// 0 for a body without clouds.
pub(super) struct Baked {
    /// The ball, wrapped in its cloud shell.
    pub(super) ball: Ball,
    /// Cells of the cloud noise along the radius of the cloud shell.
    pub(super) cells: f32,
}

/// Returns the radius of the cloud shell of `body` in metres, halfway up its cloud band, or
/// nothing for a body whose class or whose band has no clouds.
pub fn shell_radius(body: &PlanetSpec) -> Option<f64> {
    let cloudy = Air::of(&body.record).clouds > 0.0 && body.cloud_top > body.cloud_base;
    cloudy.then_some(body.radius + 0.5 * (body.cloud_base + body.cloud_top))
}

/// Returns `mesh`, a unit ball, with a smooth sphere `ratio` times as large around it whose
/// triangles carry [`SHELL_UV`]: the cloud shell, drawn in the same call as the ground.
fn shelled(mut mesh: Mesh, ratio: f32) -> Mesh {
    let mut shell = Sphere::new(ratio).mesh().uv(SHELL.0, SHELL.1);
    shell.remove_attribute(Mesh::ATTRIBUTE_TANGENT);
    let count = shell.count_vertices();
    shell.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![SHELL_UV; count]);
    shell.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![[1.0, 1.0, 1.0, 0.0]; count]);
    if let Err(error) = mesh.merge(&shell) {
        warn!("could not wrap a far body in its cloud shell: {error}");
    }
    mesh
}

/// Bakes the ball of `body` with its cloud shell. It reads only its argument and is safe on any
/// thread.
pub(super) fn bake(body: &PlanetSpec) -> Baked {
    let mut ball = ball(&Arc::new(Globe::of(body)));
    let shell = shell_radius(body);
    if let Some(shell) = shell {
        ball.mesh = shelled(ball.mesh, (shell / body.radius) as f32);
    }
    Baked {
        ball,
        cells: shell.map_or(0.0, |shell| shell as f32 / CELL),
    }
}

/// Starts baking the ball of `body` off the main thread, or nothing for a gas giant, which has
/// no ground and keeps its bands.
pub(super) fn start(body: &PlanetSpec) -> Option<Baking> {
    let solid = body.record.class != PlanetClass::GasGiant;
    let body = body.clone();
    solid.then(|| Baking(AsyncComputeTaskPool::get().spawn(async move { bake(&body) })))
}

impl BodyMaterial {
    /// Turns this material into the one of the baked `ball`: the mean colour and the mean glow
    /// of the real surface for the dot, the sea and its light for the disc, and the cloud noise.
    pub(super) fn baked(&mut self, baked: &Baked) {
        let ball = &baked.ball;
        let body = &mut self.body;
        body.color = ball.mean.extend(0.0);
        body.ember = (ball.glow * ball.sea).extend(0.0);
        body.map = Vec4::new(0.0, 0.0, 0.0, baked.cells);
        body.glow = ball.glow.extend(1.0);
    }
}

/// Gives every body whose ball is baked its mesh and its material values, a frame after the
/// task is done. Nothing else happens on the main thread.
pub(super) fn finish(
    mut commands: Commands,
    mut baking: Query<(Entity, &mut Baking, &MeshMaterial3d<BodyMaterial>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<BodyMaterial>>,
) {
    for (entity, mut task, handle) in &mut baking {
        let Some(baked) = block_on(poll_once(&mut task.0)) else {
            continue;
        };
        if let Some(mut material) = materials.get_mut(&handle.0) {
            material.baked(&baked);
        }
        let mesh = Mesh3d(meshes.add(baked.ball.mesh));
        commands.entity(entity).insert(mesh).remove::<Baking>();
    }
}

/// Returns the number of triangles of `mesh` whose texture coordinates mark the cloud shell.
#[cfg(test)]
pub(super) fn shell_triangles(mesh: &Mesh) -> usize {
    use bevy::mesh::VertexAttributeValues;
    let Some(VertexAttributeValues::Float32x2(uvs)) = mesh.attribute(Mesh::ATTRIBUTE_UV_0) else {
        return 0;
    };
    let Ok(Some(indices)) = mesh.try_indices_option() else {
        return 0;
    };
    let marked = |i: usize| uvs[i] == SHELL_UV;
    let all: Vec<usize> = indices.iter().collect();
    all.chunks(3)
        .filter(|t| t.iter().all(|i| marked(*i)))
        .count()
}
