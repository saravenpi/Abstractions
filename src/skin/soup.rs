//! A flat-shaded triangle soup with one colour per triangle: what every part of a body is built from.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

/// Flat-shaded triangles with one colour each.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Soup {
    /// Corner positions, three per triangle.
    positions: Vec<[f32; 3]>,
    /// Corner normals, the same for the three corners of a triangle.
    normals: Vec<[f32; 3]>,
    /// Linear corner colours.
    colors: Vec<[f32; 4]>,
}

impl Soup {
    /// Adds a triangle of the sRGB colour `rgb`, wound so that its front faces `out`.
    pub fn tri(&mut self, p: [Vec3; 3], out: Vec3, rgb: [f32; 3]) {
        let flip = (p[1] - p[0]).cross(p[2] - p[0]).dot(out) < 0.0;
        let [a, b, c] = if flip { [p[0], p[2], p[1]] } else { p };
        let normal = (b - a).cross(c - a).normalize_or_zero().to_array();
        let color = LinearRgba::from(Color::srgb(rgb[0], rgb[1], rgb[2])).to_f32_array();
        self.positions.extend([a, b, c].map(|p| p.to_array()));
        self.normals.extend([normal; 3]);
        self.colors.extend([color; 3]);
    }

    /// Adds a quad of the sRGB colour `rgb`, wound so that its front faces `out`.
    pub fn quad(&mut self, p: [Vec3; 4], out: Vec3, rgb: [f32; 3]) {
        self.tri([p[0], p[1], p[2]], out, rgb);
        self.tri([p[0], p[2], p[3]], out, rgb);
    }

    /// Returns the number of triangles.
    #[cfg(test)]
    pub fn triangles(&self) -> usize {
        self.positions.len() / 3
    }

    /// Returns every triangle as its three corners.
    #[cfg(test)]
    pub fn tris(&self) -> impl Iterator<Item = [Vec3; 3]> + '_ {
        self.positions
            .chunks(3)
            .map(|c| [c[0], c[1], c[2]].map(Vec3::from_array))
    }

    /// Adds every triangle of `other`, turned by `turn` and then moved by `at`.
    pub fn extend(&mut self, other: &Soup, (at, turn): (Vec3, Quat)) {
        let place = |p: &[f32; 3]| (turn * Vec3::from_array(*p) + at).to_array();
        self.positions.extend(other.positions.iter().map(place));
        let aim = |n: &[f32; 3]| (turn * Vec3::from_array(*n)).to_array();
        self.normals.extend(other.normals.iter().map(aim));
        self.colors.extend(&other.colors);
    }

    /// Returns every corner position.
    #[cfg(test)]
    pub fn points(&self) -> impl Iterator<Item = Vec3> + '_ {
        self.positions.iter().map(|p| Vec3::from_array(*p))
    }

    /// Converts the triangles into a mesh with vertex colours.
    pub fn into_mesh(self) -> Mesh {
        let count = self.positions.len() as u32;
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, self.colors)
        .with_inserted_indices(Indices::U32((0..count).collect()))
    }
}
