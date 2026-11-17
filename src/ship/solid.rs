//! A flat-shaded triangle soup: the building blocks of the ship meshes.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

/// Triangles with one flat normal each, wound counter-clockwise seen from outside.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Solid {
    /// The corners of every triangle.
    tris: Vec<[Vec3; 3]>,
}

impl Solid {
    /// Adds a triangle wound so that its front faces the direction `out`.
    pub fn tri(&mut self, p: [Vec3; 3], out: Vec3) {
        let front = (p[1] - p[0]).cross(p[2] - p[0]).dot(out) >= 0.0;
        self.tris.push(if front { p } else { [p[0], p[2], p[1]] });
    }

    /// Adds a quad wound so that its front faces the direction `out`.
    pub fn quad(&mut self, p: [Vec3; 4], out: Vec3) {
        self.tri([p[0], p[1], p[2]], out);
        self.tri([p[0], p[2], p[3]], out);
    }

    /// Adds a closed plate: the flat outline `base`, extruded by `depth`.
    pub fn plate(&mut self, base: [Vec3; 4], depth: Vec3) {
        let top = base.map(|p| p + depth);
        let centre = (base[0] + base[2] + depth) * 0.5;
        self.quad(base, -depth);
        self.quad(top, depth);
        for i in 0..4 {
            let j = (i + 1) % 4;
            let side = [base[i], base[j], top[j], top[i]];
            self.quad(side, (base[i] + top[j]) * 0.5 - centre);
        }
    }

    /// Adds a closed box between two corners.
    pub fn slab(&mut self, lo: Vec3, hi: Vec3) {
        let base = [
            lo,
            Vec3::new(hi.x, lo.y, lo.z),
            Vec3::new(hi.x, lo.y, hi.z),
            Vec3::new(lo.x, lo.y, hi.z),
        ];
        self.plate(base, Vec3::Y * (hi.y - lo.y));
    }

    /// Adds the triangles of `other`, moved by `offset`.
    pub fn merge(&mut self, other: &Solid, offset: Vec3) {
        self.tris
            .extend(other.tris.iter().map(|tri| tri.map(|p| p + offset)));
    }

    /// Converts the triangles into a mesh in the layout the ray tracer accepts, without asking
    /// for an acceleration structure: positions, flat normals, one UV, tangents and `u32` indices.
    pub fn into_mesh(self) -> Mesh {
        let count = self.tris.len() * 3;
        let normal =
            |[a, b, c]: &[Vec3; 3]| (*b - *a).cross(*c - *a).normalize_or_zero().to_array();
        let normals: Vec<[f32; 3]> = self.tris.iter().flat_map(|tri| [normal(tri); 3]).collect();
        let positions: Vec<[f32; 3]> = self.tris.iter().flatten().map(|p| p.to_array()).collect();
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.5, 0.5]; count])
        .with_inserted_attribute(Mesh::ATTRIBUTE_TANGENT, vec![[1.0, 0.0, 0.0, 1.0]; count])
        .with_inserted_indices(Indices::U32((0..count as u32).collect()));
        mesh.enable_raytracing = false;
        mesh
    }
}
