//! The meshes of the body, built in code: one per segment at each level of detail, and the few
//! parts that glow or stretch.

mod arms;
mod chest;
mod head;
mod hips;
mod legs;

#[cfg(test)]
mod tests_fit;
#[cfg(test)]
mod tests_piece;

pub use crate::skin::{Detail, Soup};
pub use head::SCREEN_FRONT;
#[cfg(test)]
pub use tests_piece::held;

use std::f32::consts::TAU;

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

use super::rig::Seg;
use super::tune::{HEAD, SCREEN};
use super::wing::RIM;

/// Number of triangles of everything that is not a segment: the screen, two nozzle glows, two
/// plumes and three panels of fabric.
#[cfg(test)]
pub const EXTRA_TRIANGLES: usize = 2 + 2 * 6 + 2 * 6 + 3 * RIM;

/// Builds the triangles of a segment in its own frame at the level `detail`. The limbs are
/// written for the right side; the left side mirrors them where they differ.
pub fn build(seg: Seg, detail: Detail) -> Soup {
    match seg {
        Seg::Pelvis => hips::pelvis(detail),
        Seg::Torso => chest::torso(detail),
        Seg::Head => head::head(detail),
        Seg::UpperArmL => arms::upper_arm(detail, true).mirrored(),
        Seg::UpperArmR => arms::upper_arm(detail, false),
        Seg::ForearmL | Seg::ForearmR => arms::forearm(detail),
        Seg::HandL | Seg::HandR => arms::hand(detail),
        Seg::ThighL => legs::thigh(detail).mirrored(),
        Seg::ThighR => legs::thigh(detail),
        Seg::ShinL | Seg::ShinR => legs::shin(detail),
        Seg::BootL | Seg::BootR => legs::boot(detail),
    }
}

/// Builds the triangles of a segment as seen up close.
pub fn segment(seg: Seg) -> Soup {
    build(seg, Detail::Near)
}

/// Builds a mesh of loose triangles from positions, normals and picture coordinates.
fn loose(positions: Vec<[f32; 3]>, normals: Vec<[f32; 3]>, uvs: Vec<[f32; 2]>) -> Mesh {
    let count = positions.len() as u32;
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32((0..count).collect()))
}

/// Builds the lit screen in the frame of the head: a quad facing -Z whose picture reads the right
/// way round from the front.
pub fn screen() -> Mesh {
    let (w, h, z) = (SCREEN[0], SCREEN[1], -SCREEN_FRONT);
    let corners = [
        ([w, HEAD.0 + h, z], [0.0, 0.0]),
        ([-w, HEAD.0 + h, z], [1.0, 0.0]),
        ([-w, HEAD.0 - h, z], [1.0, 1.0]),
        ([w, HEAD.0 - h, z], [0.0, 1.0]),
    ];
    let order = [0, 2, 1, 0, 3, 2];
    loose(
        order.map(|i| corners[i].0).to_vec(),
        vec![[0.0, 0.0, -1.0]; 6],
        order.map(|i| corners[i].1).to_vec(),
    )
}

/// Builds the glow of a boot: a six-sided cone of radius one from the sole at the origin down to
/// `depth`. A depth of zero gives the flat glow of the nozzle, a depth of one the plume.
pub fn glow(depth: f32) -> Mesh {
    let ring = |i: u32| {
        let angle = i as f32 / 6.0 * TAU;
        [angle.cos(), 0.0, angle.sin()]
    };
    let positions: Vec<[f32; 3]> = (0..6)
        .flat_map(|i| [ring(i + 1), [0.0, -depth, 0.0], ring(i)])
        .collect();
    let count = positions.len();
    loose(
        positions,
        vec![[0.0, -1.0, 0.0]; count],
        vec![[0.5, 0.5]; count],
    )
}

/// Builds a panel of fabric with every corner at the origin. The fabric system moves the corners.
pub fn fabric() -> Mesh {
    let count = RIM * 3;
    loose(
        vec![[0.0; 3]; count],
        vec![[0.0, 1.0, 0.0]; count],
        vec![[0.5, 0.5]; count],
    )
}
