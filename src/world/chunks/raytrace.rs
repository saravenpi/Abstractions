//! Ray tracing for chunk meshes: the BLAS flag on new meshes, the Solari instance on spawned entities and the switch.

use bevy::material::OpaqueRendererMethod;
use bevy::pbr::MeshMaterial3d;
use bevy::prelude::*;
use bevy::solari::prelude::RaytracingMesh3d;

use super::Chunks;
use super::build::ChunkData;
use super::nodes::{Coarse, Node};
use crate::render::RayTracing;
use crate::world::props::PropAssets;

/// Sets whether the meshes of a finished chunk get acceleration structures.
pub fn prepare(data: &mut ChunkData, on: bool) {
    data.mesh.enable_raytracing = on;
    for mesh in [&mut data.grass, &mut data.props].into_iter().flatten() {
        mesh.enable_raytracing = on;
    }
    if let Some(structures) = data.structures.as_mut() {
        structures.set_raytracing(on);
    }
}

/// Points a chunk entity at a rebuilt terrain mesh and, with ray tracing on, at its ray tracing twin.
pub fn swap(commands: &mut Commands, entity: Entity, mesh: Handle<Mesh>, on: bool) {
    let mut chunk = commands.entity(entity);
    if on {
        chunk.try_insert(RaytracingMesh3d(mesh.clone()));
    }
    chunk.try_insert(Mesh3d(mesh));
}

/// Applies the visibility of a coarse node. Solari ignores visibility, so with ray tracing `on`
/// the node only has its ray tracing twin while it is drawn.
pub fn show(commands: &mut Commands, node: &Node, on: bool) {
    let mut entity = commands.entity(node.entity);
    if node.visible {
        entity.try_insert(Visibility::Inherited);
    } else {
        entity.try_insert(Visibility::Hidden);
        entity.try_remove::<RaytracingMesh3d>();
    }
    if node.visible && on {
        entity.try_insert(RaytracingMesh3d(node.mesh.clone()));
    }
}

/// Entities that just got a mesh and are not coarse nodes.
type NewMesh = (Added<Mesh3d>, Without<Coarse>);

/// Gives every new world mesh entity a ray tracing mesh that shares its raster mesh.
///
/// Coarse nodes are left out: [`show`] gives them theirs while they are drawn.
pub fn add_instances(
    mut commands: Commands,
    assets: Res<PropAssets>,
    meshes: Query<(Entity, &Mesh3d, &MeshMaterial3d<StandardMaterial>), NewMesh>,
) {
    for (entity, mesh, material) in &meshes {
        if material.0 == assets.material {
            commands
                .entity(entity)
                .insert(RaytracingMesh3d(mesh.0.clone()));
        }
    }
}

/// Points the shared material at the deferred renderer while ray tracing is active and at the forward one otherwise.
pub fn retarget(
    ray_tracing: Res<RayTracing>,
    assets: Res<PropAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let method = match ray_tracing.0 {
        true => OpaqueRendererMethod::Deferred,
        false => OpaqueRendererMethod::Forward,
    };
    if let Some(mut material) = materials.get_mut(&assets.material)
        && material.opaque_render_method != method
    {
        material.opaque_render_method = method;
    }
}

/// Builds every chunk again with the new ray tracing flag: the meshes keep no CPU copy to change it on.
pub fn rebuild(mut commands: Commands, mut chunks: ResMut<Chunks>) {
    chunks.rebuild(&mut commands);
}
