//! A flat-shaded, vertex-colored triangle soup builder shared by terrain and props.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::PrimitiveTopology;
use bevy::prelude::*;

/// Converts an sRGB color to the linear RGBA array used by vertex colors.
pub fn srgb(r: f32, g: f32, b: f32) -> [f32; 4] {
    LinearRgba::from(Color::srgb(r, g, b)).to_f32_array()
}

/// Scales an sRGB color by `tone` and converts it to a linear RGBA vertex color.
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
    colors: Vec<[f32; 4]>,
}

impl FlatMesh {
    /// Adds a counter-clockwise triangle and returns its unit normal.
    pub fn tri(&mut self, a: Vec3, b: Vec3, c: Vec3, color: [f32; 4]) -> Vec3 {
        let normal = (b - a).cross(c - a).normalize_or_zero();
        self.positions
            .extend([a.to_array(), b.to_array(), c.to_array()]);
        self.normals.extend([normal.to_array(); 3]);
        self.colors.extend([color; 3]);
        normal
    }

    /// Converts the triangles into a Bevy mesh.
    pub fn into_mesh(self) -> Mesh {
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, self.colors)
    }
}
