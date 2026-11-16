//! Ray tracing for chunk meshes: the BLAS flag on new meshes and the Solari instance on spawned entities.

use bevy::pbr::MeshMaterial3d;
use bevy::prelude::*;
use bevy::solari::prelude::RaytracingMesh3d;

use super::ChunkData;
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

/// Gives every new world mesh entity a ray tracing mesh that shares its raster mesh.
pub fn add_instances(
    mut commands: Commands,
    assets: Res<PropAssets>,
    meshes: Query<(Entity, &Mesh3d, &MeshMaterial3d<StandardMaterial>), Added<Mesh3d>>,
) {
    for (entity, mesh, material) in &meshes {
        if material.0 == assets.material {
            commands
                .entity(entity)
                .insert(RaytracingMesh3d(mesh.0.clone()));
        }
    }
}
