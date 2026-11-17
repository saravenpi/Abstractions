//! Flat-shaded triangles with one colour each: what every piece of the base is drawn with.

use bevy::asset::RenderAssetUsages;
use bevy::math::Affine3A;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

/// An sRGB colour.
pub type Rgb = [f32; 3];

/// Corner numbers of the six faces of a box, each in outline order. Bit 0 of a corner picks
/// the high x, bit 1 the high y and bit 2 the high z.
const BOX_FACES: [[usize; 4]; 6] = [
    [0, 1, 3, 2],
    [4, 5, 7, 6],
    [0, 1, 5, 4],
    [2, 3, 7, 6],
    [0, 2, 6, 4],
    [1, 3, 7, 5],
];

/// Triangles wound counter-clockwise seen from outside, each with its sRGB colour.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Soup {
    /// Corners and colour of every triangle.
    tris: Vec<([Vec3; 3], Rgb)>,
}

impl Soup {
    /// Adds a triangle wound so that its front faces the direction `out`.
    pub fn tri(&mut self, p: [Vec3; 3], out: Vec3, rgb: Rgb) {
        let front = (p[1] - p[0]).cross(p[2] - p[0]).dot(out) >= 0.0;
        let corners = if front { p } else { [p[0], p[2], p[1]] };
        self.tris.push((corners, rgb));
    }

    /// Adds a quad wound so that its front faces the direction `out`.
    pub fn quad(&mut self, p: [Vec3; 4], out: Vec3, rgb: Rgb) {
        self.tri([p[0], p[1], p[2]], out, rgb);
        self.tri([p[0], p[2], p[3]], out, rgb);
    }

    /// Adds a four-sided bar of half width `half` from `a` to `b`, closed at both ends.
    pub fn beam(&mut self, a: Vec3, b: Vec3, half: f32, rgb: Rgb) {
        let along = (b - a).normalize_or(Vec3::Y);
        let side = along.any_orthonormal_vector() * half;
        let other = along.cross(side);
        let rim = [side + other, other - side, -side - other, side - other];
        for i in 0..4 {
            let (p, q) = (rim[i], rim[(i + 1) % 4]);
            self.quad([a + p, a + q, b + q, b + p], p + q, rgb);
        }
        self.quad(rim.map(|p| a + p), -along, rgb);
        self.quad(rim.map(|p| b + p), along, rgb);
    }

    /// Adds a closed box between two corners.
    pub fn slab(&mut self, lo: Vec3, hi: Vec3, rgb: Rgb) {
        let corner =
            |i: usize| Vec3::select(BVec3::new(i & 1 != 0, i & 2 != 0, i & 4 != 0), hi, lo);
        for face in BOX_FACES {
            let p = face.map(corner);
            self.quad(p, (p[0] + p[2] - lo - hi) * 0.5, rgb);
        }
    }

    /// Adds the triangles of `other`, moved by `pose`.
    pub fn merge(&mut self, other: &Soup, pose: Affine3A) {
        let moved = |(tri, rgb): &([Vec3; 3], Rgb)| (tri.map(|p| pose.transform_point3(p)), *rgb);
        self.tris.extend(other.tris.iter().map(moved));
    }

    /// Converts the triangles into a mesh with flat normals and linear vertex colours. The mesh
    /// asks for no ray tracing acceleration structure.
    pub fn into_mesh(self) -> Mesh {
        let count = self.tris.len() as u32 * 3;
        let flat = |[a, b, c]: &[Vec3; 3]| (*b - *a).cross(*c - *a).normalize_or_zero().to_array();
        let linear = |[r, g, b]: Rgb| LinearRgba::from(Color::srgb(r, g, b)).to_f32_array();
        let corners = self
            .tris
            .iter()
            .flat_map(|(tri, _)| tri.map(|p| p.to_array()));
        let positions: Vec<[f32; 3]> = corners.collect();
        let normals: Vec<[f32; 3]> = self
            .tris
            .iter()
            .flat_map(|(tri, _)| [flat(tri); 3])
            .collect();
        let colors: Vec<[f32; 4]> = self
            .tris
            .iter()
            .flat_map(|(_, rgb)| [linear(*rgb); 3])
            .collect();
        let usage = RenderAssetUsages::default();
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, usage)
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
            .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
            .with_inserted_indices(Indices::U32((0..count).collect()));
        mesh.enable_raytracing = false;
        mesh
    }
}
