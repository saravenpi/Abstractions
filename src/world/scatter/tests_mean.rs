//! Tests of the colour a chunk of a living planet keeps across its levels of detail.

use bevy::prelude::*;

use super::tests_cover::planet;
use super::zones::hold;

/// Returns the mean colour of the first `top` vertices of `mesh`, its ground without the skirts.
fn mean_colour(mesh: &Mesh, top: usize) -> Vec3 {
    let colours = mesh.attribute(Mesh::ATTRIBUTE_COLOR);
    let Some(bevy::mesh::VertexAttributeValues::Float32x4(colours)) = colours else {
        panic!("the mesh has colours");
    };
    let sum: Vec3 = colours[..top]
        .iter()
        .map(|c| Vec3::new(c[0], c[1], c[2]))
        .sum();
    sum / top as f32
}

/// A chunk keeps its mean colour at every level of detail: its carpet colours the 8 m grid of a far chunk as
/// it does the 2 m grid of a near one.
#[test]
fn a_chunk_keeps_its_mean_colour_at_every_level() {
    use crate::world::mesh::{GRID_POINTS, Lod, chunk_heights, chunk_mesh};
    let _book = hold();
    for tile in planet(3, 4) {
        let (terrain, coord) = (&tile.terrain, tile.coord);
        let mean = |lod: Lod| {
            let heights = chunk_heights(terrain, coord, lod.stride());
            let cells = (GRID_POINTS - 1) / lod.stride();
            mean_colour(
                &chunk_mesh(terrain, coord, &heights, lod),
                cells * cells * 6,
            )
        };
        let (near, far) = (mean(Lod::Near), mean(Lod::Distant));
        assert!(
            (near - far).abs().max_element() < 0.08,
            "{near} against {far}"
        );
    }
}
