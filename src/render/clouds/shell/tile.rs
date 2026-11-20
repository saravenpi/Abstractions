//! The tiles of the cloud shell: a cube sphere cut into tiles of [`TILE`] cells a side, each
//! pushed onto the vertices and triangles of the mesh on its own.

use std::f32::consts::FRAC_PI_4;

use bevy::prelude::*;

use super::lift;

/// Cells along one side of a tile.
pub const TILE: u32 = 8;

/// For each face of the cube, the axis it looks along, the axis its columns run along and the
/// axis its rows run along.
pub const FACES: [[Vec3; 3]; 6] = [
    [Vec3::X, Vec3::NEG_Z, Vec3::NEG_Y],
    [Vec3::NEG_X, Vec3::Z, Vec3::NEG_Y],
    [Vec3::Y, Vec3::X, Vec3::Z],
    [Vec3::NEG_Y, Vec3::X, Vec3::NEG_Z],
    [Vec3::Z, Vec3::X, Vec3::NEG_Y],
    [Vec3::NEG_Z, Vec3::NEG_X, Vec3::NEG_Y],
];

/// Returns the unit direction at corner `column`, `row` of a face of `cells` cells per side.
pub fn corner(face: usize, column: u32, row: u32, cells: u32) -> Vec3 {
    let [major, along, down] = FACES[face];
    let slope = |at: u32| (FRAC_PI_4 * (2.0 * at as f32 / cells as f32 - 1.0)).tan();
    (major + along * slope(column) + down * slope(row)).normalize()
}

/// Pushes the vertices and the triangles of tile `tile` of `face` onto `vertices` and `indices`.
pub fn push(
    (face, tile, cells): (usize, [u32; 2], u32),
    vertices: &mut Vec<[f32; 3]>,
    indices: &mut Vec<u32>,
) {
    let first = vertices.len() as u32;
    let side = TILE + 1;
    let lifted = 1.0 + lift(cells);
    for row in 0..side {
        for column in 0..side {
            let at = corner(face, tile[0] * TILE + column, tile[1] * TILE + row, cells);
            vertices.push((at * lifted).to_array());
        }
    }
    for row in 0..TILE {
        for column in 0..TILE {
            let a = first + row * side + column;
            let (b, c, d) = (a + 1, a + side, a + side + 1);
            indices.extend([a, b, c, b, d, c]);
        }
    }
}
