//! Merging solid props into one mesh, from geometry built once and shared by all workers.

use std::sync::OnceLock;

use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;

use super::{BIG, PARTS, VARIANTS, big_mesh, grown};
use crate::world::scatter::Placement;
use crate::world::shapes::textured_mesh;

/// Vertex arrays of a triangle soup.
#[derive(Default)]
pub(super) struct Soup {
    /// Position of every vertex.
    pub(super) positions: Vec<[f32; 3]>,
    /// Normal of every vertex.
    pub(super) normals: Vec<[f32; 3]>,
    /// Color lookup coordinates of every vertex.
    pub(super) uvs: Vec<[f32; 2]>,
}

impl Soup {
    /// Reads the vertex arrays of a mesh built by `FlatMesh`.
    pub(super) fn of(mesh: &Mesh) -> Self {
        let (
            Some(VertexAttributeValues::Float32x3(positions)),
            Some(VertexAttributeValues::Float32x3(normals)),
            Some(VertexAttributeValues::Float32x2(uvs)),
        ) = (
            mesh.attribute(Mesh::ATTRIBUTE_POSITION),
            mesh.attribute(Mesh::ATTRIBUTE_NORMAL),
            mesh.attribute(Mesh::ATTRIBUTE_UV_0),
        )
        else {
            unreachable!("prop meshes always carry position, normal and uv")
        };
        Self {
            positions: positions.clone(),
            normals: normals.clone(),
            uvs: uvs.clone(),
        }
    }

    /// Appends prop `p` as one rigid piece, its base at `origin` and its axes turned by `rot`: the shared mesh
    /// of its plant and variant, or the grown look its variant names.
    pub(super) fn push(&mut self, p: &Placement, origin: Vec3, rot: Quat) {
        match grown::soup(p.variant) {
            Some(look) => self.lay(&look, p, origin, rot),
            None => self.lay(base(p.plant as usize, p.variant as usize), p, origin, rot),
        }
    }

    /// Appends the geometry `b` of prop `p`, scaled by it, its base at `origin` and its axes turned by `rot`.
    fn lay(&mut self, b: &Soup, p: &Placement, origin: Vec3, rot: Quat) {
        self.positions.extend(
            b.positions
                .iter()
                .map(|v| (origin + rot * (Vec3::from(*v) * p.scale)).to_array()),
        );
        self.normals
            .extend(b.normals.iter().map(|n| (rot * Vec3::from(*n)).to_array()));
        self.uvs.extend_from_slice(&b.uvs);
    }

    /// Converts the soup into a mesh.
    pub(super) fn into_mesh(self) -> Mesh {
        textured_mesh(self.positions, self.normals, self.uvs)
    }
}

/// Returns the shared geometry of solid plant `kind`, variant `variant`, building all of them once.
fn base(kind: usize, variant: usize) -> &'static Soup {
    static BASES: OnceLock<Vec<Soup>> = OnceLock::new();
    let bases = BASES.get_or_init(|| {
        (0..BIG)
            .chain(PARTS)
            .flat_map(|k| (0..VARIANTS).map(move |v| Soup::of(&big_mesh(k, v))))
            .collect()
    });
    let slot = if kind < BIG {
        kind
    } else {
        kind - PARTS.start + BIG
    };
    &bases[slot * VARIANTS + variant % VARIANTS]
}

/// Merges every placed solid prop into one mesh in chunk space, with each transform baked into the vertices.
#[cfg(test)]
pub fn merge_props(props: &[Placement]) -> Mesh {
    super::merge_props_on(props, &crate::world::scatter::Rig::PLAIN)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::propmesh::Plant;

    /// The merged mesh holds every prop's vertices, moved to its position.
    #[test]
    fn merge_bakes_transforms() {
        let at = |x: f32| Placement {
            pos: Vec3::new(x, 1.0, 0.0),
            yaw: 0.0,
            scale: 1.0,
            plant: Plant::Oak,
            variant: 1,
        };
        let one = merge_props(&[at(0.0)]);
        let two = merge_props(&[at(0.0), at(10.0)]);
        assert_eq!(two.count_vertices(), one.count_vertices() * 2);
        let max_x = |m: &Mesh| {
            let Some(VertexAttributeValues::Float32x3(v)) = m.attribute(Mesh::ATTRIBUTE_POSITION)
            else {
                return 0.0;
            };
            v.iter().map(|p| p[0]).fold(f32::MIN, f32::max)
        };
        assert!(max_x(&two) > max_x(&one) + 9.0);
    }
}
