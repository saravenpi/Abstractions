//! A soft round disc: the mesh a puff or a drop is drawn with, so that none is a square card or
//! a streak.

use std::f32::consts::TAU;

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

/// Sides of the rim of the disc: few, for the low resolution look.
const SIDES: u32 = 8;

/// Radius of the inner ring as a share of the rim, and its opacity.
const CORE: (f32, f32) = (0.5, 0.8);

/// Returns the points of a ring of `radius` metres in the XY plane, from +X anticlockwise seen
/// from +Z.
fn ring(radius: f32) -> impl Iterator<Item = [f32; 3]> {
    (0..SIDES).map(move |k| {
        let turn = k as f32 * TAU / SIDES as f32;
        [radius * turn.cos(), radius * turn.sin(), 0.0]
    })
}

/// Returns the triangles of the disc, all wound anticlockwise seen from +Z: a fan from the
/// centre to the inner ring, then a band from the inner ring to the rim.
fn triangles() -> Vec<u32> {
    (0..SIDES)
        .flat_map(|k| {
            let next = (k + 1) % SIDES;
            let (inner, inner_next) = (1 + k, 1 + next);
            let (rim, rim_next) = (1 + SIDES + k, 1 + SIDES + next);
            [
                [0, inner, inner_next],
                [inner, rim, rim_next],
                [inner, rim_next, inner_next],
            ]
        })
        .flatten()
        .collect()
}

/// Returns a disc one metre across in the XY plane, facing +Z: opaque at its centre and clear at
/// its rim through its vertex colours, with every normal along +Y. Turned to face the camera, a
/// puff is lit as the top of a cloud, as the ground under it is lit, and never turns into a dark
/// card when the sun is behind it.
pub fn soft_disc() -> Mesh {
    let positions: Vec<[f32; 3]> = std::iter::once([0.0; 3])
        .chain(ring(0.5 * CORE.0))
        .chain(ring(0.5))
        .collect();
    let alpha = |index: usize| match index {
        0 => 1.0,
        i if i <= SIDES as usize => CORE.1,
        _ => 0.0,
    };
    let colours: Vec<[f32; 4]> = (0..positions.len())
        .map(|index| [1.0, 1.0, 1.0, alpha(index)])
        .collect();
    let normals = vec![[0.0, 1.0, 0.0]; positions.len()];
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colours)
    .with_inserted_indices(Indices::U32(triangles()))
}
