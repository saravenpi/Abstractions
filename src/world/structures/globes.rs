//! Test-only globes: the six face charts of a planet, the villages found on them and the terrain mesh under
//! a point.

use std::sync::Arc;

use bevy::math::DVec3;
use bevy::prelude::*;

use super::site::{CELL, Site};
use super::village::{self, SPAN};
use crate::world::globe::{Chart, Face, Globe};
use crate::world::mesh::{Frame, chunk_heights, chunk_patch};
use crate::world::scatter::chunk_center;
use crate::world::{CHUNK_SIZE, Terrain};

/// Radii of the globes the structure tests run on, in meters.
pub const RADII: [f64; 2] = [3_000.0, 40_000.0];

/// Returns the six face charts of the planet of `seed` and `radius`.
pub fn faces(seed: u64, radius: f64) -> Vec<Terrain> {
    let globe = Arc::new(Globe::new(seed, radius, 22.0));
    let chart = |face: Face| Terrain::on(globe.clone(), globe.face_chart(face));
    Face::ALL.into_iter().map(chart).collect()
}

/// Returns half the width of the face chart `face` in chart units.
pub fn half_width(face: &Terrain) -> f32 {
    match face.chart() {
        Chart::Face { scale, .. } => scale as f32,
        _ => 0.0,
    }
}

/// Returns up to `want` villages of the chart `face`, found among the village cells nearest its middle.
pub fn villages(face: &Terrain, want: usize) -> Vec<Site> {
    let n = ((half_width(face) / (SPAN as f32 * CELL)).ceil() as i32).min(7);
    let cells = (-n..n).flat_map(|x| (-n..n).map(move |y| IVec2::new(x, y)));
    let found = cells.filter_map(|c| village::village_in_cell(face, c));
    found.take(want).collect()
}

/// Returns the vertex normals of a built mesh.
pub fn normals(mesh: &Mesh) -> Vec<Vec3> {
    mesh.attribute(Mesh::ATTRIBUTE_NORMAL)
        .and_then(|a| a.as_float3())
        .map(|v| v.iter().map(|n| Vec3::from_array(*n)).collect())
        .unwrap_or_default()
}

/// Returns the planet position of the point `p` of `frame`.
pub fn planet(frame: &Frame, p: Vec3) -> DVec3 {
    frame.origin + frame.rot * p.as_dvec3()
}

/// Returns the distance along the ray `from + dir * t` to the triangle `tri`, behind the start or ahead.
fn hit(tri: [Vec3; 3], from: Vec3, dir: Vec3) -> Option<f32> {
    let (e1, e2) = (tri[1] - tri[0], tri[2] - tri[0]);
    let p = dir.cross(e2);
    let det = e1.dot(p);
    let s = from - tri[0];
    let (u, q) = (s.dot(p) / det, s.cross(e1));
    let v = dir.dot(q) / det;
    let inside = det.abs() > 1e-9 && u >= -1e-4 && v >= -1e-4 && u + v <= 1.0 + 1e-4;
    inside.then(|| e2.dot(q) / det)
}

/// Returns how far the planet position `pos` lies above the triangles the terrain mesh of the chunk under it
/// draws, measured along the local up.
pub fn above_mesh(face: &Terrain, pos: DVec3) -> f32 {
    let coord = (face.locate(pos).xz() / CHUNK_SIZE).floor().as_ivec2();
    let patch = chunk_patch(face, coord, &chunk_heights(face, coord, 1));
    let frame = Frame::at(face, chunk_center(coord));
    let (from, up) = (frame.local(pos), frame.turn(pos.normalize()));
    let corners = patch.positions();
    let tris = patch
        .triangles()
        .into_iter()
        .map(|t| t.map(|i| corners[i as usize]));
    let hits = tris.filter_map(|tri| hit(tri, from, up));
    -hits.fold(
        f32::MAX,
        |best, t| if t.abs() < best.abs() { t } else { best },
    )
}
