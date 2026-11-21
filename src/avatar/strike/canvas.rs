//! The mesh and the material of a tool ribbon: a fixed set of vertices, written in place.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use bevy::prelude::*;

use super::ribbon::fill;
use super::trail::{ROWS, Trail};

/// Vertices of the ribbon: two for each row.
const VERTICES: usize = ROWS * 2;

/// Returns the triangle indices of the ribbon: two triangles between each row and the next.
fn indices() -> Vec<u32> {
    (0..ROWS as u32 - 1)
        .flat_map(|row| {
            let (a, b) = (row * 2, row * 2 + 2);
            [a, b, a + 1, a + 1, b, b + 1]
        })
        .collect()
}

/// Builds the mesh of a ribbon once: a fixed number of vertices, all clear.
pub fn mesh() -> Mesh {
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.0f32; 3]; VERTICES])
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0f32, 1.0, 0.0]; VERTICES])
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, vec![[0.0f32; 4]; VERTICES])
    .with_inserted_indices(Indices::U32(indices()))
}

/// Writes the ribbon of `trail` into the vertex buffers of `mesh` without changing their size, at
/// the opacity `look` gives, as [`fill`] reads it.
pub fn write(mesh: &mut Mesh, trail: &Trail, look: [f32; 2]) {
    let (
        Some(VertexAttributeValues::Float32x3(positions)),
        Some(VertexAttributeValues::Float32x4(colours)),
    ) = (
        mesh.remove_attribute(Mesh::ATTRIBUTE_POSITION),
        mesh.remove_attribute(Mesh::ATTRIBUTE_COLOR),
    )
    else {
        return;
    };
    let (mut positions, mut colours) = (positions, colours);
    fill(trail, look, &mut positions, &mut colours);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colours);
}

/// Returns the material of the ribbons: unlit and translucent, with colour and opacity in the vertices.
pub fn material() -> StandardMaterial {
    StandardMaterial {
        base_color: Color::WHITE,
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        fog_enabled: false,
        double_sided: true,
        cull_mode: None,
        ..default()
    }
}
