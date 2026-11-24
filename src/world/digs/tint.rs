//! Darkening dug terrain facets via the palette lookup coordinates in `ATTRIBUTE_UV_0`.

use bevy::math::DVec3;
use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;

use super::DigStamp;
use super::map::{CHUNK_CULL_MARGIN, sample_stamps};
use crate::universe::BodyId;
use crate::world::colormap;
use crate::world::mesh::Frame;
use crate::world::shapes::srgb;

/// Exposed subsoil sRGB tint blended into dug crater facets.
const SOIL_TINT: [f32; 3] = [0.24, 0.19, 0.14];

/// Blends an encoded colormap `[u, v]` texel toward [`SOIL_TINT`] by `mix` (`0.0..=1.0`).
fn blend_uv(uv: [f32; 2], mix: f32) -> [f32; 2] {
    let bx_r = (uv[0] * 1024.0).floor().clamp(0.0, 1023.0) as u32;
    let r = (bx_r % 32) as f32 / 31.0;
    let g = (uv[1] * 32.0).floor().clamp(0.0, 31.0) / 31.0;
    let b = (bx_r / 32) as f32 / 31.0;
    let nr = r * (1.0 - mix) + SOIL_TINT[0] * mix;
    let ng = g * (1.0 - mix) + SOIL_TINT[1] * mix;
    let nb = b * (1.0 - mix) + SOIL_TINT[2] * mix;
    colormap::uv(srgb(nr, ng, nb))
}

/// Evaluates dig depth at each mesh vertex for stamps overlapping `frame`.
pub(super) fn vertex_depths(
    stamps: &[DigStamp],
    mesh: &Mesh,
    planet: BodyId,
    frame: &Frame,
    globe_radius: f64,
) -> Option<Vec<f32>> {
    let origin_dir = frame.origin.normalize_or(DVec3::Y);
    let active: Vec<&DigStamp> = stamps
        .iter()
        .filter(|s| {
            let sd = DVec3::from_array(s.dir).normalize_or(DVec3::Y);
            s.planet == planet
                && origin_dir.distance(sd) * globe_radius
                    <= CHUNK_CULL_MARGIN + f64::from(s.radius.max(0.0))
        })
        .collect();
    if active.is_empty() {
        return None;
    }
    let VertexAttributeValues::Float32x3(pos) = mesh.attribute(Mesh::ATTRIBUTE_POSITION)? else {
        return None;
    };
    Some(
        pos.iter()
            .map(|p| {
                let world = frame.origin + frame.rot * Vec3::from_array(*p).as_dvec3();
                sample_stamps(&active, world.normalize_or(DVec3::Y), globe_radius)
            })
            .collect(),
    )
}

/// Updates facet UVs on `mesh` wherever `depths` indicate freshly dug craters.
pub(super) fn apply_uv_tint(mesh: &mut Mesh, depths: &[f32]) {
    let Some(VertexAttributeValues::Float32x2(uvs)) = mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0)
    else {
        return;
    };
    for (tri_idx, tri) in uvs.chunks_exact_mut(3).enumerate() {
        let base = tri_idx * 3;
        let peak = depths[base].max(depths[base + 1]).max(depths[base + 2]);
        if peak <= 0.04 {
            continue;
        }
        let mix = (peak / 1.6).clamp(0.22, 0.65);
        let tinted = blend_uv(tri[0], mix);
        tri.fill(tinted);
    }
}
