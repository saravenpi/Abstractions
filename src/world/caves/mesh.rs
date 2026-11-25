//! Merged procedural cave shell mesh (14-sided organic rings, boulder arch collar, stalactites,
//! stalagmites) and matching Avian3d trimesh collider.

use std::f32::consts::TAU;

use avian3d::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

use super::plan::{CaveBiome, CavePlan, SpineNode};
use crate::universe::hash::{Rng, child};

/// Number of vertices around each tunnel or cavern cross-section ring.
const RING_SIDES: usize = 14;

/// Triangle soup builder for one merged cave shell and its trimesh collider.
#[derive(Default)]
struct ShellBuf {
    /// Vertex positions in cave tangent frame.
    positions: Vec<[f32; 3]>,
    /// Flat per-face normals.
    normals: Vec<[f32; 3]>,
    /// Linear RGBA vertex colours.
    colors: Vec<[f32; 4]>,
    /// Collider vertex positions.
    col_verts: Vec<Vec3>,
    /// Collider triangle indices.
    col_tris: Vec<[u32; 3]>,
}

/// Returns subterranean linear RGBA rock colour for `biome` at `pos` and ring vertical factor `sy`.
fn rock_color(biome: CaveBiome, pos: Vec3, seed: u64, sy: f32) -> [f32; 4] {
    let base = match biome {
        CaveBiome::BioluminescentGrotto => [0.16, 0.21, 0.23],
        CaveBiome::CrystalSpire => [0.18, 0.17, 0.23],
        CaveBiome::MycelialDepths => [0.22, 0.18, 0.15],
    };
    let wave = (pos.x * 0.37 + pos.y * 0.53 + pos.z * 0.41 + (seed % 97) as f32).sin();
    let ao = if sy < -0.2 {
        0.88
    } else {
        0.96 - sy.max(0.0) * 0.16
    };
    let tone = (ao + wave * 0.08).clamp(0.55, 1.15);
    LinearRgba::from(Color::srgb(base[0] * tone, base[1] * tone, base[2] * tone)).to_f32_array()
}

/// Computes the 14-sided organic ring vertices around `node` with a flattened walkable floor.
fn ring_verts(node: SpineNode, tangent: Vec3, seed: u64) -> [Vec3; RING_SIDES] {
    let right = tangent.cross(Vec3::Y).normalize_or(Vec3::X);
    let up = right.cross(tangent).normalize_or(Vec3::Y);
    let phase = (seed % 251) as f32 * 0.1;
    std::array::from_fn(|s| {
        let theta = TAU * s as f32 / RING_SIDES as f32;
        let (cx, raw_sy) = (theta.cos(), theta.sin());
        let sy = if raw_sy < -0.25 {
            -0.25 + (raw_sy + 0.25) * 0.22
        } else {
            raw_sy
        };
        let p = node.pos + right * (cx * node.rx) + up * (sy * node.ry);
        let n = (p.x * 0.45 + phase).sin() * (p.z * 0.43 - phase).cos() + (p.y * 0.6).sin() * 0.5;
        let scale = 1.0 + n * if raw_sy < -0.2 { 0.04 } else { 0.18 };
        node.pos + right * (cx * node.rx * scale) + up * (sy * node.ry * scale)
    })
}

/// Emits one triangle oriented so its normal agrees with `hint`.
fn push_tri(buf: &mut ShellBuf, (a, b, c): (Vec3, Vec3, Vec3), hint: Vec3, col: [f32; 4]) {
    let raw = (b - a).cross(c - a);
    let (v1, v2, n) = if raw.dot(hint) >= 0.0 {
        (b, c, raw.normalize_or(Vec3::Y))
    } else {
        (c, b, (-raw).normalize_or(Vec3::Y))
    };
    let idx = buf.col_verts.len() as u32;
    buf.col_verts.extend([a, v1, v2]);
    buf.col_tris.push([idx, idx + 1, idx + 2]);
    buf.positions
        .extend([a.to_array(), v1.to_array(), v2.to_array()]);
    buf.normals.extend([n.to_array(); 3]);
    buf.colors.extend([col; 3]);
}

/// Extrudes inward-facing 14-sided tunnel rings along `spine` and closes the far end cap.
fn extrude_spine(buf: &mut ShellBuf, spine: &[SpineNode], biome: CaveBiome, seed: u64) {
    let rings: Vec<[Vec3; RING_SIDES]> = spine
        .iter()
        .enumerate()
        .map(|(k, node)| {
            let tang = (spine[(k + 1).min(spine.len() - 1)].pos - spine[k.saturating_sub(1)].pos)
                .normalize_or(Vec3::Z);
            ring_verts(*node, tang, seed)
        })
        .collect();
    for k in 0..rings.len().saturating_sub(1) {
        let mid = (spine[k].pos + spine[k + 1].pos) * 0.5;
        for s in 0..RING_SIDES {
            let (a, b, c, d) = (
                rings[k][s],
                rings[k][(s + 1) % RING_SIDES],
                rings[k + 1][s],
                rings[k + 1][(s + 1) % RING_SIDES],
            );
            let col = rock_color(
                biome,
                (a + d) * 0.5,
                seed,
                (TAU * s as f32 / RING_SIDES as f32).sin(),
            );
            let hint = mid - (a + b + c + d) * 0.25;
            push_tri(buf, (a, b, d), hint, col);
            push_tri(buf, (a, d, c), hint, col);
        }
    }
    if let (Some(last), Some(ring)) = (spine.last(), rings.last()) {
        let col = rock_color(biome, last.pos, seed, 0.0);
        let hint = spine[spine.len().saturating_sub(2)].pos - last.pos;
        for s in 0..RING_SIDES {
            push_tri(
                buf,
                (ring[s], ring[(s + 1) % RING_SIDES], last.pos),
                hint,
                col,
            );
        }
    }
}

/// Adds a faceted low-poly boulder around `center` for the portal entrance arch collar.
fn add_boulder(buf: &mut ShellBuf, center: Vec3, radii: Vec3, col: [f32; 4], seed: u64) {
    let mut rng = Rng::new(seed);
    let ring: [Vec3; 6] = std::array::from_fn(|k| {
        let a = TAU * k as f32 / 6.0;
        let j = 0.82 + rng.unit() as f32 * 0.34;
        center + Vec3::new(a.cos() * radii.x * j, 0.0, a.sin() * radii.z * j)
    });
    let (top, bot) = (center + Vec3::Y * radii.y, center - Vec3::Y * radii.y * 0.7);
    for k in 0..6 {
        let (a, b) = (ring[k], ring[(k + 1) % 6]);
        push_tri(buf, (a, b, top), (a + b + top) / 3.0 - center, col);
        push_tri(buf, (b, a, bot), (a + b + bot) / 3.0 - center, col);
    }
}

/// Adds a 6-sided tapered rocky stalactite or stalagmite from `base` to `tip`.
fn add_cone(buf: &mut ShellBuf, base: Vec3, tip: Vec3, radius: f32, col: [f32; 4]) {
    let axis = (tip - base).normalize_or(Vec3::Y);
    let right = axis.any_orthonormal_vector();
    let fwd = axis.cross(right);
    let ring: [Vec3; 6] = std::array::from_fn(|k| {
        let a = TAU * k as f32 / 6.0;
        base + (right * a.cos() + fwd * a.sin()) * radius
    });
    let mid = (base + tip) * 0.5;
    for k in 0..6 {
        let (a, b) = (ring[k], ring[(k + 1) % 6]);
        push_tri(buf, (a, b, tip), (a + b + tip) / 3.0 - mid, col);
    }
}

/// Adds the entrance boulder arch collar plus ceiling stalactites and edge stalagmites.
fn add_details(buf: &mut ShellBuf, plan: &CavePlan) {
    let mut rng = Rng::new(child(plan.seed, 0xB01D));
    for k in 0..12 {
        let a = TAU * k as f32 / 12.0;
        let dist = plan.entry_radius * (0.94 + rng.unit() as f32 * 0.16);
        let pos = Vec3::new(
            a.cos() * dist,
            plan.entry_y + a.sin().max(-0.2) * 0.45,
            a.sin() * dist,
        );
        let r = Vec3::new(
            1.2 + rng.unit() as f32 * 0.7,
            0.9 + rng.unit() as f32 * 0.6,
            1.2 + rng.unit() as f32 * 0.7,
        );
        add_boulder(
            buf,
            pos,
            r,
            rock_color(plan.biome, pos, plan.seed, 0.2),
            child(plan.seed, 0xC01 + k as u64),
        );
    }
    for (i, node) in plan.main_spine.iter().enumerate().skip(3) {
        let off = Vec3::new(
            (rng.unit() as f32 - 0.5) * node.rx * 0.7,
            0.0,
            (rng.unit() as f32 - 0.5) * node.rx * 0.7,
        );
        let ceil = node.pos + Vec3::Y * (node.ry * 0.92) + off;
        let len = (1.1 + rng.unit() as f32 * 1.4).min(node.ry * 0.45);
        add_cone(
            buf,
            ceil,
            ceil - Vec3::Y * len,
            0.36,
            rock_color(plan.biome, ceil, plan.seed, 0.7),
        );
        if i >= 10 && i % 2 == 0 {
            let side = if i % 4 == 0 { 1.0 } else { -1.0 };
            let floor = Vec3::new(
                node.pos.x + side * node.rx * 0.68,
                node.pos.y - node.ry * 0.42,
                node.pos.z,
            );
            add_cone(
                buf,
                floor,
                floor + Vec3::Y * (0.8 + rng.unit() as f32 * 1.1),
                0.34,
                rock_color(plan.biome, floor, plan.seed, -0.4),
            );
        }
    }
}

/// Builds ONE merged cave shell mesh (with inward normals and vertex colours) and its trimesh collider.
pub fn build_cave_shell(plan: &CavePlan) -> (Mesh, Collider) {
    let mut buf = ShellBuf::default();
    extrude_spine(&mut buf, &plan.main_spine, plan.biome, plan.seed);
    for (idx, alcove) in plan.alcoves.iter().enumerate() {
        extrude_spine(
            &mut buf,
            &alcove.spine,
            plan.biome,
            child(plan.seed, 0xA00 + idx as u64),
        );
    }
    add_details(&mut buf, plan);
    let count = buf.positions.len() as u32;
    let mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, buf.positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, buf.normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, buf.colors)
    .with_inserted_indices(Indices::U32((0..count).collect()));
    let collider = Collider::trimesh_with_config(
        buf.col_verts,
        buf.col_tris,
        TrimeshFlags::FIX_INTERNAL_EDGES,
    );
    (mesh, collider)
}
