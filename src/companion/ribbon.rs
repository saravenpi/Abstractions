//! The ribbon of the wake: one thin camera-facing strip along the points it flew through.
//!
//! This is the strip of `gear::fx::ribbon`, which is private to the gear, for the shorter ring
//! of the wake. Three vertices per row: the middle one carries the glow and the two edges are
//! transparent. The glow fades with age, along the length and over the first rows, so both
//! ends are soft and nothing pops out.

use bevy::asset::RenderAssetUsages;
use bevy::math::DVec3;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

use super::spring::ramp;
use super::wake::{POINTS, Wake};

/// Rows of the ribbon: the live tip, then every point of the wake.
pub const ROWS: usize = POINTS + 1;

/// Vertices of the ribbon, the same whatever the wake holds.
pub const VERTICES: usize = ROWS * 3;

/// Rows over which the glow fades in behind the companion.
const FADE_IN: f32 = 2.0;

/// Widest the ribbon is drawn to each side, as a share of its distance to the camera.
const SPAN: f32 = 0.06;

/// Distances from the camera between which the ribbon fades in, in metres.
const NEAR: (f32, f32) = (0.5, 1.5);

/// How the ribbon is drawn this frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Look {
    /// Camera position in render space.
    pub eye: Vec3,
    /// Half width in metres.
    pub width: f32,
    /// Seconds a point glows for.
    pub life: f32,
    /// Linear HDR colour of the glow.
    pub tint: Vec3,
}

/// The vertex data of the ribbon, filled in place each frame.
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

/// Returns the position in the planet frame, the age and the heat of row `row` of `wake`.
fn row_of(wake: &Wake, row: usize) -> (DVec3, f32, f32) {
    let sample = match row {
        0 => wake.tip(),
        _ => wake.point(row - 1),
    };
    (sample.at, sample.age, sample.heat)
}

/// Returns the glow of row `row` of `wake`: its heat, faded by age, by its place along the
/// ribbon and over the first rows behind the companion. Rows not laid yet are dark.
pub fn glow(wake: &Wake, row: usize, life: f32) -> f32 {
    let (_, age, heat) = row_of(wake, row);
    let young = (1.0 - age / life).clamp(0.0, 1.0);
    let along = 1.0 - row as f32 / (ROWS - 1) as f32;
    let begun = (row as f32 / FADE_IN).min(1.0);
    let laid = if row >= wake.len() { 0.0 } else { 1.0 };
    heat * young * young * along.sqrt() * begun * laid
}

/// Fills `strip` with the ribbon of `wake`, seen from `look.eye`. `to_render` maps the planet
/// frame to render space.
///
/// Each row is widened across the view direction, so the strip always faces the camera. It
/// tapers to nothing with age, and a row close to the camera narrows and fades out.
pub fn fill(wake: &Wake, look: &Look, to_render: impl Fn(DVec3) -> Vec3, strip: &mut Strip) {
    for row in 0..ROWS {
        let (place, age, _) = row_of(wake, row);
        let at = to_render(place);
        let ahead = to_render(row_of(wake, row.saturating_sub(1)).0);
        let behind = to_render(row_of(wake, (row + 1).min(ROWS - 1)).0);
        let away = look.eye.distance(at);
        let aged = look.width * (1.0 - age / look.life).clamp(0.0, 1.0);
        let side = (ahead - behind).cross(look.eye - at).normalize_or_zero();
        let side = side * aged.min(away * SPAN);
        let alpha = glow(wake, row, look.life) * ramp(NEAR.0, NEAR.1, away);
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
    let quad = |a: u32, b: u32| [a, b, a + 1, a + 1, b, b + 1];
    (0..ROWS as u32 - 1)
        .flat_map(|row| {
            let (a, b) = (row * 3, row * 3 + 3);
            quad(a, b).into_iter().chain(quad(a + 1, b + 1))
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
