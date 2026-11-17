//! The ribbon: a camera-facing strip of fixed size drawn along a trail.
//!
//! Three vertices per row: the middle one carries the glow and the two edges are transparent,
//! which gives the soft falloff across the width. The glow fades along the length with age,
//! with the place in the ring and over the first rows, so both ends are soft too.

use bevy::asset::RenderAssetUsages;
use bevy::math::DVec3;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

use super::trail::{POINTS, Trail};

/// Rows of the ribbon: the live tip, then every point of the trail.
pub const ROWS: usize = POINTS + 1;

/// Vertices of the ribbon, the same whatever the trail holds.
pub const VERTICES: usize = ROWS * 3;

/// Rows over which the glow fades in behind the emitter.
const FADE_IN: f32 = 3.0;

/// How a ribbon is drawn this frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Look {
    /// Camera position in render space.
    pub eye: Vec3,
    /// Half width at the emitter in metres.
    pub width: f32,
    /// How many times wider the ribbon is at the end of its life.
    pub spread: f32,
    /// Seconds a point glows for.
    pub life: f32,
    /// Linear HDR colour of the glow.
    pub tint: Vec3,
}

/// The vertex data of one ribbon, filled in place each frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Strip {
    /// Render positions.
    pub positions: [[f32; 3]; VERTICES],
    /// Linear colours with alpha.
    pub colors: [[f32; 4]; VERTICES],
}

impl Default for Strip {
    /// Everything at the origin and transparent.
    fn default() -> Self {
        Self {
            positions: [[0.0; 3]; VERTICES],
            colors: [[0.0; 4]; VERTICES],
        }
    }
}

/// Returns the glow of row `row` of `trail`: its heat, faded by age, by its place along the
/// ribbon and over the first rows behind the emitter.
fn glow(trail: &Trail, row: usize, life: f32) -> f32 {
    let sample = if row == 0 {
        trail.tip()
    } else {
        trail.point(row - 1)
    };
    let young = (1.0 - sample.age / life).clamp(0.0, 1.0);
    let along = 1.0 - row as f32 / (ROWS - 1) as f32;
    let begun = (row as f32 / FADE_IN).min(1.0);
    let laid = if row > trail.len() { 0.0 } else { 1.0 };
    sample.heat * young * young * along.sqrt() * begun * laid
}

/// Returns the position of row `row` of `trail` in the planet frame.
fn place(trail: &Trail, row: usize) -> DVec3 {
    if row == 0 {
        trail.tip().at
    } else {
        trail.point(row - 1).at
    }
}

/// Fills `strip` with the ribbon of `trail`, seen from `look.eye`. `to_render` maps the planet
/// frame to render space.
///
/// Each row is widened across the view direction and along neither the trail nor the view, so
/// the strip always faces the camera. Rows the trail has not laid yet collapse onto its end.
pub fn fill(trail: &Trail, look: &Look, to_render: impl Fn(DVec3) -> Vec3, strip: &mut Strip) {
    for row in 0..ROWS {
        let at = to_render(place(trail, row));
        let ahead = to_render(place(trail, row.saturating_sub(1)));
        let behind = to_render(place(trail, (row + 1).min(ROWS - 1)));
        let age = if row == 0 {
            0.0
        } else {
            trail.point(row - 1).age
        };
        let wide = look.width * (1.0 + (look.spread - 1.0) * (age / look.life).clamp(0.0, 1.0));
        let side = (ahead - behind).cross(look.eye - at).normalize_or_zero() * wide;
        let alpha = glow(trail, row, look.life);
        for (slot, (offset, shade)) in [(-1.0, 0.0), (0.0, alpha), (1.0, 0.0)]
            .into_iter()
            .enumerate()
        {
            strip.positions[row * 3 + slot] = (at + side * offset).to_array();
            strip.colors[row * 3 + slot] = look.tint.extend(shade).to_array();
        }
    }
}

/// Returns the triangle indices of the strip: four triangles between two rows.
fn indices() -> Vec<u32> {
    (0..ROWS as u32 - 1)
        .flat_map(|row| {
            let (a, b) = (row * 3, row * 3 + 3);
            [
                a,
                b,
                a + 1,
                a + 1,
                b,
                b + 1,
                a + 1,
                b + 1,
                a + 2,
                a + 2,
                b + 1,
                b + 2,
            ]
        })
        .collect()
}

/// Builds the ribbon mesh once: fixed vertex and index counts, updated in place afterwards.
pub fn mesh() -> Mesh {
    let strip = Strip::default();
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, strip.positions.to_vec())
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 1.0, 0.0]; VERTICES])
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.5, 0.5]; VERTICES])
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, strip.colors.to_vec())
    .with_inserted_indices(Indices::U32(indices()))
}

/// Writes `strip` into the vertex buffers of `mesh` without changing their size.
pub fn write(mesh: &mut Mesh, strip: &Strip) {
    use bevy::mesh::VertexAttributeValues::{Float32x3, Float32x4};
    if let Some(Float32x3(positions)) = mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION) {
        positions.copy_from_slice(&strip.positions);
    }
    if let Some(Float32x4(colors)) = mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR) {
        colors.copy_from_slice(&strip.colors);
    }
}
