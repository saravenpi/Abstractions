//! Meshes of creature parts: flat-shaded, coloured per face from their tissue, one shared by every
//! individual of a species.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

use crate::life::Tissue;
use crate::life::body::{self, Part};

/// Returns the mesh of `part` made of `tissue`, in the frame of the part: tapers, spikes and fans hang
/// along -Y from the socket at the origin, blocks and balls are centred on it.
#[cfg_attr(not(test), allow(dead_code))]
pub fn part_mesh(part: &Part, tissue: &Tissue) -> Mesh {
    part_mesh_toned(part, tissue, None)
}

/// Returns the mesh of `part` made of `tissue`, blending two-tone coat patterns with `accent`.
pub fn part_mesh_toned(part: &Part, tissue: &Tissue, accent: Option<&Tissue>) -> Mesh {
    let soup = body::soup_toned(part, tissue, accent);
    let faces = &soup.faces;
    let positions: Vec<[f32; 3]> = faces
        .iter()
        .flat_map(|f| f.corners.map(|c| c.to_array()))
        .collect();
    let normals: Vec<[f32; 3]> = faces
        .iter()
        .flat_map(|f| [f.normal.to_array(); 3])
        .collect();
    let colours: Vec<[f32; 4]> = faces
        .iter()
        .flat_map(|f| [[f.colour[0], f.colour[1], f.colour[2], 1.0]; 3])
        .collect();
    let count = positions.len() as u32;
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colours)
    .with_inserted_indices(Indices::U32((0..count).collect()))
}

/// Returns the number of triangles of the mesh of `part`, for budgets.
pub fn triangles(part: &Part) -> usize {
    body::triangles(part)
}
