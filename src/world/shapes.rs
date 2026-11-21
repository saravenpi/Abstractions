//! A flat-shaded triangle soup builder shared by terrain and props, colored through the lookup texture.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

use super::colormap;

/// Converts an sRGB color to the linear RGBA array passed to the color lookup.
pub fn srgb(r: f32, g: f32, b: f32) -> [f32; 4] {
    LinearRgba::from(Color::srgb(r, g, b)).to_f32_array()
}

/// Scales an sRGB color by `tone` and converts it to a linear RGBA color.
pub fn shade(rgb: [f32; 3], tone: f32) -> [f32; 4] {
    srgb(
        (rgb[0] * tone).min(1.0),
        (rgb[1] * tone).min(1.0),
        (rgb[2] * tone).min(1.0),
    )
}

/// Non-indexed triangles with one flat normal per triangle.
#[derive(Default, Clone)]
pub struct FlatMesh {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    turf: Vec<[f32; 2]>,
}

impl FlatMesh {
    /// Adds a counter-clockwise triangle and returns its unit normal.
    pub fn tri(&mut self, a: Vec3, b: Vec3, c: Vec3, color: [f32; 4]) -> Vec3 {
        let normal = (b - a).cross(c - a).normalize_or_zero();
        self.positions
            .extend([a.to_array(), b.to_array(), c.to_array()]);
        self.normals.extend([normal.to_array(); 3]);
        self.uvs.extend([colormap::uv(color); 3]);
        normal
    }

    /// Gives the last two triangles added the normal `normal` at every corner, so they are lit
    /// like the ground they hang from rather than by their own slant.
    pub fn relight(&mut self, normal: Vec3) {
        let from = self.normals.len().saturating_sub(6);
        self.normals[from..].fill(normal.to_array());
    }

    /// Gives the next triangle added the turf coordinates `uvs` of its three corners: the second uv set
    /// the ground shader reads its carpet from. Triangles added without any read as bare.
    pub fn turf(&mut self, uvs: [[f32; 2]; 3]) {
        self.turf.resize(self.positions.len(), [0.0; 2]);
        self.turf.extend(uvs);
    }

    /// Converts the triangles into a Bevy mesh, with the turf coordinates when any triangle has them.
    pub fn into_mesh(mut self) -> Mesh {
        let turfed = !self.turf.is_empty();
        self.turf.resize(self.positions.len(), [0.0; 2]);
        let mesh = textured_mesh(self.positions, self.normals, self.uvs);
        match turfed {
            true => mesh.with_inserted_attribute(Mesh::ATTRIBUTE_UV_1, self.turf),
            false => mesh,
        }
    }
}

/// Builds a mesh from non-indexed triangles: position, normal, uv, tangent and `u32` indices.
pub fn textured_mesh(positions: Vec<[f32; 3]>, normals: Vec<[f32; 3]>, uvs: Vec<[f32; 2]>) -> Mesh {
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
