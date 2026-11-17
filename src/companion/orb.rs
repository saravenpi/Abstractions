//! Small shared meshes: the octahedron of the hands and the glowing parts, and the screen quad.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

use super::body::{SCREEN, SCREEN_FRONT};

/// Builds a mesh of flat triangles in the layout Solari accepts: position, normal, uv, tangent, `u32` indices.
fn solid(positions: Vec<[f32; 3]>, normals: Vec<[f32; 3]>, uvs: Vec<[f32; 2]>) -> Mesh {
    let count = positions.len();
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_attribute(Mesh::ATTRIBUTE_TANGENT, vec![[1.0, 0.0, 0.0, 1.0]; count])
    .with_inserted_indices(Indices::U32((0..count as u32).collect()))
}

/// Returns the surface area in square meters of the octahedron of `radius`.
pub fn orb_area(radius: f32) -> f32 {
    4.0 * 3.0_f32.sqrt() * radius * radius
}

/// Builds a flat-shaded octahedron of `radius`. Only a `traced` one gets a ray tracing acceleration structure.
pub fn orb(radius: f32, traced: bool) -> Mesh {
    let (mut positions, mut normals) = (Vec::new(), Vec::new());
    for (x, y, z) in (0..8).map(|i| (i & 1, (i >> 1) & 1, i >> 2)) {
        let sign = |bit: i32| 1.0 - 2.0 * bit as f32;
        let (a, b, c) = (Vec3::X * sign(x), Vec3::Y * sign(y), Vec3::Z * sign(z));
        let flip = (b - a).cross(c - a).dot(a + b + c) < 0.0;
        let corners = if flip { [a, c, b] } else { [a, b, c] };
        positions.extend(corners.map(|p| (p * radius).to_array()));
        normals.extend([(a + b + c).normalize().to_array(); 3]);
    }
    let uvs = vec![[0.5, 0.5]; positions.len()];
    let mut mesh = solid(positions, normals, uvs);
    mesh.enable_raytracing = traced;
    mesh
}

/// Builds the lit screen: a quad facing -Z whose picture reads the right way round from the front.
pub fn screen() -> Mesh {
    let (w, h, z) = (SCREEN.x, SCREEN.y, -SCREEN_FRONT);
    let corners = [
        ([w, h, z], [0.0, 0.0]),
        ([-w, h, z], [1.0, 0.0]),
        ([-w, -h, z], [1.0, 1.0]),
        ([w, -h, z], [0.0, 1.0]),
    ];
    let order = [0, 2, 1, 0, 3, 2];
    let mut mesh = solid(
        order.map(|i| corners[i].0).to_vec(),
        vec![[0.0, 0.0, -1.0]; 6],
        order.map(|i| corners[i].1).to_vec(),
    );
    mesh.enable_raytracing = false;
    mesh
}
