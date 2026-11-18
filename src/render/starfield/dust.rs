//! The mesh of the dust of a warp: one quad per grain, built once. The quads carry nothing but
//! their grain number; `starfield.wesl` places every grain from the flow and draws it as a
//! thin grey line.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

use crate::render::warp::MOTES;

/// Marker the shader reads in place of the light of a star to know a quad is a grain of dust.
pub const DUST: f32 = -100.0;

/// The dust entity, drawn with the material of the starfield.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Dust;

/// Builds the mesh of the dust: [`MOTES`] quads with the grain number and [`DUST`] in the second
/// pair of texture coordinates, and the attributes of the starfield mesh, so both share a shader.
pub fn mesh() -> Mesh {
    let corners = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
    let quads = 0..MOTES;
    let uvs: Vec<[f32; 2]> = quads.clone().flat_map(|_| corners).collect();
    let grains: Vec<[f32; 2]> = quads.clone().flat_map(|i| [[i as f32, DUST]; 4]).collect();
    let first = |i: u32| [i * 4, i * 4 + 2, i * 4 + 1, i * 4, i * 4 + 3, i * 4 + 2];
    let indices: Vec<u32> = quads.flat_map(first).collect();
    let count = uvs.len();
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.0, 0.0, 1.0]; count]);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 0.0, -1.0]; count]);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_1, grains);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![[1.0, 1.0, 1.0, 0.6]; count]);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}
