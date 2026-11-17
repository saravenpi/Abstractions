//! Test-only bookkeeping of closed boxes and double-sided sheets, to check their winding.

use bevy::prelude::*;

use super::brush::Build;

/// What a recorded piece of mesh is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Piece {
    /// A closed box whose triangles must wind outward.
    Closed,
    /// A thin sheet drawn once per side.
    Sheet,
}

impl Build<'_> {
    /// Returns the number of vertices drawn so far.
    pub fn vertex_mark(&self) -> usize {
        self.mesh.count()
    }

    /// Records the vertices drawn since `start` as one piece.
    pub fn note(&mut self, piece: Piece, start: usize) {
        let end = self.vertex_mark();
        self.audit.push((piece, start, end));
    }
}

/// Returns the vertex positions of a built mesh.
pub fn positions(mesh: &Mesh) -> Vec<Vec3> {
    mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        .and_then(|a| a.as_float3())
        .map(|v| v.iter().map(|p| Vec3::from_array(*p)).collect())
        .unwrap_or_default()
}

/// Returns the signed volume enclosed by the triangles `verts`, positive when they wind outward.
pub fn signed_volume(verts: &[Vec3]) -> f32 {
    verts
        .as_chunks::<3>()
        .0
        .iter()
        .map(|t| t[0].dot(t[1].cross(t[2])) / 6.0)
        .sum()
}

/// Returns true when every triangle of `verts` has a twin with the opposite winding.
pub fn has_twins(verts: &[Vec3]) -> bool {
    let tris: Vec<&[Vec3; 3]> = verts.as_chunks::<3>().0.iter().collect();
    let same = |a: &[Vec3; 3], b: &[Vec3; 3]| a.iter().zip(b).all(|(p, q)| p.distance(*q) < 1e-4);
    tris.iter().all(|t| {
        let flipped = [t[0], t[2], t[1]];
        let rolled = [[t[2], t[1], t[0]], [t[1], t[0], t[2]], flipped];
        tris.iter().any(|o| rolled.iter().any(|r| same(o, r)))
    })
}
