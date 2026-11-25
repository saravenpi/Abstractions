//! Procedural 3D triangle geometry for flintlock pistols, mineral rifles, and shared item meshes.

use std::f32::consts::TAU;

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

use super::GunKind;
use crate::craft::matter::Mineral;
use crate::craft::pack::tune::STONE_TONES;

/// Flat-shaded linear-RGB triangle in local object space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tri {
    /// Counter-clockwise vertex positions.
    pub v: [Vec3; 3],
    /// Linear RGB facet reflectance.
    pub color: [f32; 3],
}

/// Appends an axis-aligned box spanning `[min..=max]` as 12 outward triangles.
pub fn push_box(out: &mut Vec<Tri>, min: Vec3, max: Vec3, color: [f32; 3]) {
    let c = [
        Vec3::new(min.x, min.y, min.z),
        Vec3::new(max.x, min.y, min.z),
        Vec3::new(max.x, max.y, min.z),
        Vec3::new(min.x, max.y, min.z),
        Vec3::new(min.x, min.y, max.z),
        Vec3::new(max.x, min.y, max.z),
        Vec3::new(max.x, max.y, max.z),
        Vec3::new(min.x, max.y, max.z),
    ];
    let quads = [
        [0, 3, 2, 1],
        [4, 5, 6, 7],
        [0, 1, 5, 4],
        [3, 7, 6, 2],
        [0, 4, 7, 3],
        [1, 2, 6, 5],
    ];
    for [a, b, d, e] in quads {
        out.push(Tri {
            v: [c[a], c[b], c[d]],
            color,
        });
        out.push(Tri {
            v: [c[a], c[d], c[e]],
            color,
        });
    }
}

/// Appends a capped tapered prism along segment `a..=b` with `sides` radial facets.
pub fn push_prism(
    out: &mut Vec<Tri>,
    a: Vec3,
    b: Vec3,
    (ra, rb): (f32, f32),
    sides: usize,
    color: [f32; 3],
) {
    let axis = (b - a).normalize_or(Vec3::Z);
    let (u, v) = axis.any_orthonormal_pair();
    let ring = |center: Vec3, r: f32, i: usize| -> Vec3 {
        let ang = (i % sides) as f32 * TAU / sides as f32;
        center + (u * ang.cos() + v * ang.sin()) * r
    };
    for i in 0..sides {
        let (a0, a1) = (ring(a, ra, i), ring(a, ra, i + 1));
        let (b0, b1) = (ring(b, rb, i), ring(b, rb, i + 1));
        out.push(Tri {
            v: [a0, b0, b1],
            color,
        });
        out.push(Tri {
            v: [a0, b1, a1],
            color,
        });
        if i >= 1 && i + 1 < sides {
            out.push(Tri {
                v: [ring(a, ra, 0), ring(a, ra, i + 1), a0],
                color,
            });
            out.push(Tri {
                v: [ring(b, rb, 0), b0, ring(b, rb, i + 1)],
                color,
            });
        }
    }
}

/// Builds 3D geometry for a compact single-shot flintlock mineral pistol.
fn pistol_tris(metal: [f32; 3]) -> Vec<Tri> {
    let mut out = Vec::with_capacity(96);
    let wood = [0.22, 0.11, 0.05];
    let brass = [0.68, 0.50, 0.14];
    let iron = [0.18, 0.19, 0.22];
    push_prism(
        &mut out,
        Vec3::new(0.0, -0.14, 0.14),
        Vec3::new(0.0, 0.0, 0.06),
        (0.024, 0.021),
        8,
        wood,
    );
    push_prism(
        &mut out,
        Vec3::new(0.0, -0.14, 0.14),
        Vec3::new(0.0, -0.16, 0.16),
        (0.028, 0.018),
        8,
        brass,
    );
    push_box(
        &mut out,
        Vec3::new(-0.02, -0.02, -0.18),
        Vec3::new(0.02, 0.025, 0.07),
        wood,
    );
    push_box(
        &mut out,
        Vec3::new(-0.024, -0.01, -0.04),
        Vec3::new(0.024, 0.032, 0.06),
        brass,
    );
    push_prism(
        &mut out,
        Vec3::new(0.0, 0.035, 0.03),
        Vec3::new(0.0, 0.032, -0.26),
        (0.017, 0.014),
        8,
        metal,
    );
    push_box(
        &mut out,
        Vec3::new(0.016, 0.015, 0.01),
        Vec3::new(0.030, 0.065, 0.045),
        iron,
    );
    push_box(
        &mut out,
        Vec3::new(-0.008, -0.052, -0.025),
        Vec3::new(0.008, -0.015, 0.035),
        brass,
    );
    out
}

/// Builds 3D geometry for a long four-round bolt-action mineral rifle with crystal sights.
fn rifle_tris(metal: [f32; 3]) -> Vec<Tri> {
    let mut out = Vec::with_capacity(128);
    let wood = [0.26, 0.13, 0.06];
    let brass = [0.72, 0.54, 0.16];
    let sight = [0.40, 0.85, 1.00];
    push_box(
        &mut out,
        Vec3::new(-0.022, -0.09, 0.14),
        Vec3::new(0.022, 0.015, 0.36),
        wood,
    );
    push_box(
        &mut out,
        Vec3::new(-0.024, -0.095, 0.35),
        Vec3::new(0.024, 0.015, 0.375),
        brass,
    );
    push_box(
        &mut out,
        Vec3::new(-0.023, -0.032, -0.22),
        Vec3::new(0.023, 0.028, 0.15),
        wood,
    );
    push_box(
        &mut out,
        Vec3::new(-0.021, 0.005, -0.04),
        Vec3::new(0.021, 0.048, 0.13),
        brass,
    );
    push_prism(
        &mut out,
        Vec3::new(0.0, 0.042, 0.08),
        Vec3::new(0.0, 0.040, -0.48),
        (0.019, 0.014),
        8,
        metal,
    );
    push_prism(
        &mut out,
        Vec3::new(0.02, 0.035, 0.06),
        Vec3::new(0.065, 0.02, 0.07),
        (0.008, 0.010),
        6,
        metal,
    );
    push_box(
        &mut out,
        Vec3::new(-0.024, -0.025, -0.18),
        Vec3::new(0.024, 0.056, -0.16),
        brass,
    );
    push_box(
        &mut out,
        Vec3::new(-0.007, 0.052, -0.45),
        Vec3::new(0.007, 0.072, -0.42),
        sight,
    );
    out
}

/// Returns flat-shaded triangles representing firearm `(kind, barrel)`.
pub fn gun_tris(kind: GunKind, barrel: Mineral) -> Vec<Tri> {
    let base = STONE_TONES[barrel as usize];
    let metal = [
        (base[0] * 0.75 + 0.14).min(0.95),
        (base[1] * 0.75 + 0.14).min(0.95),
        (base[2] * 0.75 + 0.15).min(0.95),
    ];
    match kind {
        GunKind::FlintlockPistol => pistol_tris(metal),
        GunKind::MineralRifle => rifle_tris(metal),
    }
}

/// Builds a Bevy `Mesh` with positions, flat normals, and vertex colors for a held firearm.
pub fn gun_mesh(kind: GunKind, barrel: Mineral) -> Mesh {
    let tris = gun_tris(kind, barrel);
    let count = tris.len() * 3;
    let mut pos = Vec::with_capacity(count);
    let mut norm = Vec::with_capacity(count);
    let mut col = Vec::with_capacity(count);
    for tri in tris {
        let n = (tri.v[1] - tri.v[0])
            .cross(tri.v[2] - tri.v[0])
            .normalize_or(Vec3::Y);
        let rgba = [tri.color[0], tri.color[1], tri.color[2], 1.0];
        for v in tri.v {
            pos.push(v.to_array());
            norm.push(n.to_array());
            col.push(rgba);
        }
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, pos)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, norm)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, col)
    .with_inserted_indices(Indices::U32((0..count as u32).collect()))
}
