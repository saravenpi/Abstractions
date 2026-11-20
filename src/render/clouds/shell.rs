//! The mesh of the cloud shell: a cube sphere cut into tiles, of which only the tiles the camera
//! can see are built. Its cells are small enough that no point of the mesh lies more than a
//! stated tolerance in metres from the true sphere, whatever the size of the planet. The
//! tolerance grows with the distance of the camera from the shell, see [`super::mesh`].

mod tile;

use std::f32::consts::{FRAC_PI_2, SQRT_2};

use bevy::asset::RenderAssetUsages;
use bevy::math::DVec3;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

use self::tile::{FACES, TILE, corner, push, unit};

/// Most metres any point of the shell mesh lies from the true sphere of the shell when the
/// camera is close to it: the finest the mesh is ever built.
pub const TOLERANCE: f32 = 50.0;

/// Radians added round the cap the camera sees, so a camera that moves does not rebuild the
/// mesh every frame.
pub(super) const MARGIN: f32 = 0.03;

/// How many times wider than what the camera sees a new cap is built.
pub(super) const GROWTH: f32 = 1.5;

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
/// above it. The worst sag is in the triangle at a corner of the cube, where a cell is widest
/// across its short diagonal: with `d` the distance of its plane from the middle of the sphere,
/// the balance is `(1 - d) / (1 + d)`.
pub fn lift(cells: u32) -> f32 {
    let at = |column, row| unit(0, column, row, cells);
    let (a, b, c) = (at(0, 0), at(1, 0), at(0, 1));
    let flat = (b - a).cross(c - a).normalize().dot(a).abs();
    ((1.0 - flat) / (1.0 + flat)) as f32
}

/// Returns the most metres a point of the mesh of `cells` cells per face side lies from a
/// sphere of `radius` metres.
pub fn error(radius: f32, cells: u32) -> f32 {
    radius * lift(cells)
}

/// Returns the cells per face side the shell of a sphere of `radius` metres is built with: the
/// fewest, in whole tiles, that keep it within `tolerance` metres.
pub fn cells_within(radius: f32, tolerance: f32) -> u32 {
    (1..)
        .map(|tiles| tiles * TILE)
        .find(|cells| error(radius, *cells) < tolerance)
        .unwrap_or(TILE)
}

/// Returns the cells per face side the shell of `radius` metres has at the finest tolerance.
#[cfg(test)]
pub fn cells(radius: f32) -> u32 {
    cells_within(radius, TOLERANCE)
}

/// Returns the angle in radians from the point under a camera `altitude` metres above a planet
/// of `ground` metres to the farthest point of the shell of `shell` metres that the camera can
/// see: along its line of sight past the horizon of the ground, up to where it leaves the shell.
pub fn reach(ground: f32, altitude: f32, shell: f32) -> f32 {
    let angle = |low: f32, high: f32| (low / high.max(low)).clamp(0.0, 1.0).acos();
    angle(ground, ground + altitude.max(0.0)) + angle(ground, shell)
}

/// Builds the mesh of the unit sphere over `cap`, no point of it more than `tolerance` metres
/// from the true sphere of `radius` metres, which it is turned to by scaling it by `radius`.
/// It carries the unit direction of every vertex as its normal, and stays in the render world.
pub fn build_within(radius: f32, cap: Cap, tolerance: f32) -> Mesh {
    let cells = cells_within(radius, tolerance);
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
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_indices(Indices::U32(indices))
}

/// Builds the mesh of the unit sphere over `cap` at the finest tolerance.
#[cfg(test)]
pub fn build(radius: f32, cap: Cap) -> Mesh {
    build_within(radius, cap, TOLERANCE)
}
