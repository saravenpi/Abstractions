//! Merging solid props into one mesh, from geometry built once and shared by all workers.

use std::sync::OnceLock;

use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;

use super::{BIG, PARTS, VARIANTS, big_mesh};
use crate::world::scatter::Placement;
use crate::world::shapes::textured_mesh;

/// Vertex arrays of a triangle soup.
#[derive(Default)]
struct Soup {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
}

impl Soup {
    /// Reads the vertex arrays of a mesh built by `FlatMesh`.
    fn of(mesh: &Mesh) -> Self {
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

    /// Appends one placed prop with its transform baked into the vertices.
    fn push(&mut self, p: &Placement) {
        let b = base(p.plant as usize, p.variant as usize);
        let rot = Quat::from_rotation_y(p.yaw);
        self.positions.extend(
            b.positions
                .iter()
                .map(|v| (p.pos + rot * (Vec3::from(*v) * p.scale)).to_array()),
        );
        self.normals
            .extend(b.normals.iter().map(|n| (rot * Vec3::from(*n)).to_array()));
        self.uvs.extend_from_slice(&b.uvs);
    }

    /// Converts the soup into a mesh.
    fn into_mesh(self) -> Mesh {
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
    &bases[slot * VARIANTS + variant]
}

/// Merges every placed solid prop into one mesh in chunk space, with each transform baked into the vertices.
pub fn merge_props(props: &[Placement]) -> Mesh {
    merge_into(Soup::default(), props)
}

/// Appends every placed solid prop to a mesh built by `FlatMesh`.
pub fn merge_with(mesh: Mesh, props: &[Placement]) -> Mesh {
    merge_into(Soup::of(&mesh), props)
}

/// Appends every placed solid prop to a soup and converts it into a mesh.
fn merge_into(mut soup: Soup, props: &[Placement]) -> Mesh {
    for p in props {
        soup.push(p);
    }
    soup.into_mesh()
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
