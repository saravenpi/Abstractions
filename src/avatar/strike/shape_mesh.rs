//! Procedural unit meshes shared by 3D held and ground tools, each fitted in a unit cube.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

/// Kind of unit geometry a tool part uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormMesh {
    /// Unit box.
    Cube,
    /// Unit octagonal shaft along Y.
    Rod,
    /// Single-edged wedge blade along Y, flat spine on -X and edge on +X.
    Wedge,
    /// Flared bearded axe bit from neck at -X to curved cutting edge at +X.
    AxeBit,
    /// Double-edged diamond cross-section blade along Y.
    Blade,
    /// Four-sided diamond pyramid tapering along +Y to a tip.
    Point,
}

/// Reusable unit mesh handles for every tool part shape.
#[derive(Debug, Clone)]
pub struct Meshes {
    /// Unit cube.
    pub cube: Handle<Mesh>,
    /// Unit cylindrical shaft.
    pub rod: Handle<Mesh>,
    /// Unit single-edge wedge blade.
    pub wedge: Handle<Mesh>,
    /// Unit flared axe bit.
    pub axe_bit: Handle<Mesh>,
    /// Unit double-edged blade.
    pub blade: Handle<Mesh>,
    /// Unit four-sided spear or blade point.
    pub point: Handle<Mesh>,
}

impl Meshes {
    /// Builds and registers every unit tool mesh once.
    pub fn new(meshes: &mut Assets<Mesh>) -> Self {
        Self {
            cube: meshes.add(Cuboid::from_length(1.0)),
            rod: meshes.add(Cylinder::new(0.5, 1.0).mesh().resolution(8).build()),
            wedge: meshes.add(wedge_mesh()),
            axe_bit: meshes.add(axe_bit_mesh()),
            blade: meshes.add(blade_mesh()),
            point: meshes.add(point_mesh()),
        }
    }

    /// Returns the mesh handle for `kind`.
    pub fn get(&self, kind: FormMesh) -> &Handle<Mesh> {
        match kind {
            FormMesh::Cube => &self.cube,
            FormMesh::Rod => &self.rod,
            FormMesh::Wedge => &self.wedge,
            FormMesh::AxeBit => &self.axe_bit,
            FormMesh::Blade => &self.blade,
            FormMesh::Point => &self.point,
        }
    }
}

/// Builds a flat-shaded closed solid mesh from convex faces, orienting every triangle outward
/// from the mean vertex centroid.
fn poly_solid(faces: &[&[Vec3]]) -> Mesh {
    let mut sum = Vec3::ZERO;
    let mut count = 0usize;
    for face in faces {
        for &v in *face {
            sum += v;
            count += 1;
        }
    }
    let interior = sum / count.max(1) as f32;
    let mut pos = Vec::new();
    let mut nor = Vec::new();
    let mut uv = Vec::new();
    let mut idx = Vec::new();
    for face in faces {
        let anchor = face[0];
        for pair in face[1..].windows(2) {
            let (mut b, mut c) = (pair[0], pair[1]);
            let mid = (anchor + b + c) / 3.0;
            if (b - anchor).cross(c - anchor).dot(mid - interior) < 0.0 {
                std::mem::swap(&mut b, &mut c);
            }
            let n = (b - anchor).cross(c - anchor).normalize_or(Vec3::Y);
            let base = pos.len() as u32;
            pos.extend([anchor.to_array(), b.to_array(), c.to_array()]);
            nor.extend([n.to_array(); 3]);
            uv.extend([[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]]);
            idx.extend([base, base + 1, base + 2]);
        }
    }
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, pos);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, nor);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uv);
    mesh.insert_indices(Indices::U32(idx));
    mesh
}

/// Returns a unit single-edge wedge prism: spine at -X and bevelled edge at +X.
fn wedge_mesh() -> Mesh {
    let p = |x, y, z| Vec3::new(x, y, z);
    let (lt, lb, rt, rb) = (
        p(-0.5, 0.5, 0.5),
        p(-0.5, -0.5, 0.5),
        p(-0.5, 0.5, -0.5),
        p(-0.5, -0.5, -0.5),
    );
    let (et, eb) = (p(0.5, 0.5, 0.0), p(0.5, -0.5, 0.0));
    poly_solid(&[
        &[lb, lt, rt, rb],
        &[lt, lb, eb, et],
        &[rb, rt, et, eb],
        &[lt, et, rt],
        &[lb, rb, eb],
    ])
}

/// Returns a unit flared bearded axe bit spanning -X at the eye neck to +X along a curved edge.
fn axe_bit_mesh() -> Mesh {
    let p = |x, y, z| Vec3::new(x, y, z);
    let (lt, lb, rt, rb) = (
        p(-0.5, 0.24, 0.5),
        p(-0.5, -0.24, 0.5),
        p(-0.5, 0.24, -0.5),
        p(-0.5, -0.24, -0.5),
    );
    let (et, em, eb) = (p(0.42, 0.34, 0.0), p(0.50, -0.06, 0.0), p(0.42, -0.50, 0.0));
    poly_solid(&[
        &[lb, lt, rt, rb],
        &[lt, rt, et],
        &[lb, rb, eb],
        &[lt, et, em],
        &[lt, em, lb],
        &[lb, em, eb],
        &[rt, em, et],
        &[rt, rb, em],
        &[rb, eb, em],
    ])
}

/// Returns a unit diamond-section double-edged sword blade aligned along Y.
fn blade_mesh() -> Mesh {
    let p = |x, y, z| Vec3::new(x, y, z);
    let (bl, bf, br, bk) = (
        p(-0.5, -0.5, 0.0),
        p(0.0, -0.5, 0.5),
        p(0.5, -0.5, 0.0),
        p(0.0, -0.5, -0.5),
    );
    let (tl, tf, tr, tk) = (
        p(-0.5, 0.5, 0.0),
        p(0.0, 0.5, 0.5),
        p(0.5, 0.5, 0.0),
        p(0.0, 0.5, -0.5),
    );
    poly_solid(&[
        &[bl, bf, tf, tl],
        &[bf, br, tr, tf],
        &[br, bk, tk, tr],
        &[bk, bl, tl, tk],
        &[tl, tf, tr, tk],
        &[bl, bk, br, bf],
    ])
}

/// Returns a unit four-sided diamond pyramid tapering from y = -0.5 to a tip at y = +0.5.
fn point_mesh() -> Mesh {
    let p = |x, y, z| Vec3::new(x, y, z);
    let (l, f, r, b) = (
        p(-0.5, -0.5, 0.0),
        p(0.0, -0.5, 0.5),
        p(0.5, -0.5, 0.0),
        p(0.0, -0.5, -0.5),
    );
    let tip = p(0.0, 0.5, 0.0);
    poly_solid(&[
        &[l, f, tip],
        &[f, r, tip],
        &[r, b, tip],
        &[b, l, tip],
        &[l, b, r, f],
    ])
}
