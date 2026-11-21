//! The vertex buffers of the near cloud: one soft disc per puff, its middle opaque and its rim
//! clear, facing the camera.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::fill::{PUFFS, Puff, SPACING, opacity};

/// Corners of the rim of a disc.
const SIDES: usize = 6;

/// Vertices of one disc: its middle, then its rim.
const STRIDE: usize = SIDES + 1;

/// Radius of a disc in lattice steps, for a puff that is nearly empty and for a full one.
const SIZE: (f32, f32) = (0.55, 0.95);

/// Strongest alpha of the middle of a disc.
const OPACITY: f32 = 0.7;

/// Returns the empty mesh of the discs: every vertex at the origin, clear.
pub fn mesh() -> Mesh {
    use bevy::asset::RenderAssetUsages;
    use bevy::mesh::{Indices, PrimitiveTopology};
    let count = PUFFS * STRIDE;
    let faces = (0..PUFFS as u32).flat_map(|puff| {
        let first = puff * STRIDE as u32;
        (0..SIDES as u32).flat_map(move |side| {
            [
                first,
                first + 1 + side,
                first + 1 + (side + 1) % SIDES as u32,
            ]
        })
    });
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.0f32; 3]; count])
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0f32, 1.0, 0.0]; count])
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, vec![[0.0f32; 4]; count])
    .with_inserted_indices(Indices::U32(faces.collect()))
}

/// Writes the disc of `puff` as vertex `index` onwards of `positions` and `colours`: facing the
/// camera along `right` and `up`, `offset` from the camera, in the linear `colour`.
fn disc(
    (positions, colours): (&mut [[f32; 3]], &mut [[f32; 4]]),
    (index, puff, offset): (usize, &Puff, Vec3),
    (right, up): (Vec3, Vec3),
    colour: Vec3,
) {
    let size = SPACING as f32 * (SIZE.0 + (SIZE.1 - SIZE.0) * puff.fill);
    let alpha = OPACITY * opacity(puff.fill, offset.length());
    let first = index * STRIDE;
    positions[first] = offset.to_array();
    colours[first] = colour.extend(alpha).to_array();
    for side in 0..SIDES {
        let angle = side as f32 / SIDES as f32 * std::f32::consts::TAU;
        let rim = offset + (right * angle.cos() + up * angle.sin()) * size;
        positions[first + 1 + side] = rim.to_array();
        colours[first + 1 + side] = colour.extend(0.0).to_array();
    }
}

/// The vertex buffers of the discs, made once and refilled every frame.
pub struct Discs {
    /// Positions from the camera.
    positions: Vec<[f32; 3]>,
    /// Linear colours with alpha.
    colours: Vec<[f32; 4]>,
}

impl Default for Discs {
    /// Buffers for every disc, empty.
    fn default() -> Self {
        Self {
            positions: vec![[0.0; 3]; PUFFS * STRIDE],
            colours: vec![[0.0; 4]; PUFFS * STRIDE],
        }
    }
}

impl Discs {
    /// Lays `puffs`, placed from the camera by `render`, facing it along `view`, each in the
    /// colour `shade` gives it at its offset from the camera, and clears the discs left over.
    pub fn lay(
        &mut self,
        (puffs, render): (&[Puff], &dyn Fn(DVec3) -> Vec3),
        view: (Vec3, Vec3),
        shade: &dyn Fn(&Puff, Vec3) -> Vec3,
    ) {
        for (index, puff) in puffs.iter().enumerate() {
            let buffers = (&mut self.positions[..], &mut self.colours[..]);
            let offset = render(puff.at);
            disc(buffers, (index, puff, offset), view, shade(puff, offset));
        }
        for vertex in puffs.len() * STRIDE..PUFFS * STRIDE {
            (self.positions[vertex], self.colours[vertex]) = ([0.0; 3], [0.0; 4]);
        }
    }

    /// Copies the buffers into `mesh` without changing their size.
    pub fn write(&self, mesh: &mut Mesh) {
        use bevy::mesh::VertexAttributeValues::{Float32x3, Float32x4};
        if let Some(Float32x3(positions)) = mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION) {
            positions.copy_from_slice(&self.positions);
        }
        if let Some(Float32x4(colours)) = mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR) {
            colours.copy_from_slice(&self.colours);
        }
    }
}
