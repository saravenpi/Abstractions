//! The mesh of the cloud shell: a cube sphere cut into tiles, of which only the tiles the camera
//! can see are built. Its cells are small enough that no point of the mesh lies more than
//! [`TOLERANCE`] metres from the true sphere, whatever the size of the planet.

use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, SQRT_2};

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

/// Most metres any point of the shell mesh lies from the true sphere of the shell.
pub const TOLERANCE: f32 = 50.0;

/// Cells along one side of a tile.
const TILE: u32 = 8;

/// Radians added round the cap the camera sees, so a camera that moves does not rebuild the
/// mesh every frame.
pub(super) const MARGIN: f32 = 0.03;

/// How many times wider than what the camera sees a new cap is built.
pub(super) const GROWTH: f32 = 1.5;

/// For each face of the cube, the axis it looks along, the axis its columns run along and the
/// axis its rows run along.
const FACES: [[Vec3; 3]; 6] = [
    [Vec3::X, Vec3::NEG_Z, Vec3::NEG_Y],
    [Vec3::NEG_X, Vec3::Z, Vec3::NEG_Y],
    [Vec3::Y, Vec3::X, Vec3::Z],
    [Vec3::NEG_Y, Vec3::X, Vec3::NEG_Z],
    [Vec3::Z, Vec3::X, Vec3::NEG_Y],
    [Vec3::NEG_Z, Vec3::NEG_X, Vec3::NEG_Y],
];

/// The part of the sphere a mesh covers: every direction within `half` radians of `centre`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cap {
    /// Unit direction at the middle of the cap.
    pub centre: Vec3,
    /// Angular radius in radians.
    pub half: f32,
}

/// Returns the share of the radius the vertices of a mesh of `cells` cells per face side lie
/// above the true sphere, so that the flat faces sag below it by as much as the vertices stand
/// above it. The worst sag is at the middle of the longest face of a cell, which spans
/// `FRAC_PI_2 / cells` radians along the diagonal of a cell.
pub fn lift(cells: u32) -> f32 {
    let span = FRAC_PI_2 / cells as f32 / SQRT_2;
    (span * 0.5).tan().powi(2)
}

/// Returns the most metres a point of the mesh of `cells` cells per face side lies from a
/// sphere of `radius` metres.
pub fn error(radius: f32, cells: u32) -> f32 {
    radius * lift(cells)
}

/// Returns the cells per face side the shell of a sphere of `radius` metres is built with: the
/// fewest, in whole tiles, that keep it within [`TOLERANCE`].
pub fn cells(radius: f32) -> u32 {
    (1..)
        .map(|tiles| tiles * TILE)
        .find(|cells| error(radius, *cells) < TOLERANCE)
        .unwrap_or(TILE)
}

/// Returns the angle in radians from the point under a camera `altitude` metres above a planet
/// of `ground` metres to the farthest point of the shell of `shell` metres that the camera can
/// see: along its line of sight past the horizon of the ground, up to where it leaves the shell.
pub fn reach(ground: f32, altitude: f32, shell: f32) -> f32 {
    let angle = |low: f32, high: f32| (low / high.max(low)).clamp(0.0, 1.0).acos();
    angle(ground, ground + altitude.max(0.0)) + angle(ground, shell)
}

/// Returns the unit direction at corner `column`, `row` of a face of `cells` cells per side.
fn corner(face: usize, column: u32, row: u32, cells: u32) -> Vec3 {
    let [major, along, down] = FACES[face];
    let slope = |at: u32| (FRAC_PI_4 * (2.0 * at as f32 / cells as f32 - 1.0)).tan();
    (major + along * slope(column) + down * slope(row)).normalize()
}

/// Pushes the vertices and the triangles of tile `tile` of `face` onto `vertices` and `indices`.
fn push(
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

/// Builds the mesh of the unit sphere over `cap`, turned to the true sphere of `radius` metres
/// by scaling it by `radius`. It carries the unit direction of every vertex as its normal.
pub fn build(radius: f32, cap: Cap) -> Mesh {
    let cells = cells(radius);
    let tiles = cells / TILE;
    let limit = cap.half + 0.75 * TILE as f32 * FRAC_PI_2 / cells as f32;
    let (mut vertices, mut indices) = (Vec::new(), Vec::new());
    for face in 0..FACES.len() {
        for (x, y) in (0..tiles).flat_map(|x| (0..tiles).map(move |y| (x, y))) {
            let middle = corner(face, x * TILE + TILE / 2, y * TILE + TILE / 2, cells);
            if middle.angle_between(cap.centre) <= limit {
                push((face, [x, y], cells), &mut vertices, &mut indices);
            }
        }
    }
    let normals: Vec<[f32; 3]> = vertices
        .iter()
        .map(|at| Vec3::from(*at).normalize().to_array())
        .collect();
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_indices(Indices::U32(indices))
}
