//! Merging every big prop of a chunk into one mesh, from geometry built once and shared by all workers.

use std::sync::OnceLock;

use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;

use super::{BIG, VARIANTS, big_mesh};
use crate::world::scatter::Placement;
use crate::world::shapes::textured_mesh;

/// Vertex arrays of one prop variant, shared by every chunk worker.
struct Base {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
}

/// Returns the shared geometry of big plant `kind`, variant `variant`, building all of them once.
fn base(kind: usize, variant: usize) -> &'static Base {
    static BASES: OnceLock<Vec<Base>> = OnceLock::new();
    let bases = BASES.get_or_init(|| {
        (0..BIG * VARIANTS)
            .map(|i| {
                let mesh = big_mesh(i / VARIANTS, i % VARIANTS);
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
                Base {
                    positions: positions.clone(),
                    normals: normals.clone(),
                    uvs: uvs.clone(),
                }
            })
            .collect()
    });
    &bases[kind * VARIANTS + variant]
}

/// Merges every placed big prop into one mesh in chunk space, with each transform baked into the vertices.
pub fn merge_props(props: &[Placement]) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    for p in props {
        let b = base(p.plant as usize, p.variant as usize);
        let rot = Quat::from_rotation_y(p.yaw);
        positions.extend(
            b.positions
                .iter()
                .map(|v| (p.pos + rot * (Vec3::from(*v) * p.scale)).to_array()),
        );
        normals.extend(b.normals.iter().map(|n| (rot * Vec3::from(*n)).to_array()));
        uvs.extend_from_slice(&b.uvs);
    }
    textured_mesh(positions, normals, uvs)
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
