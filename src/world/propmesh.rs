//! Vertex-colored meshes for trees, rocks, bushes and grass.

use bevy::prelude::*;

use super::shapes::{FlatMesh, srgb};
use super::solids::{cone, icosa, prism};

/// Height of the tree trunk, which is also where the foliage starts.
pub const TRUNK_HEIGHT: f32 = 1.6;

/// Builds a tree mesh with its base at the origin. Variant 1 is a squat two-tier pine.
pub fn tree_mesh(variant: usize) -> Mesh {
    let mut m = FlatMesh::default();
    prism(
        &mut m,
        Vec3::ZERO,
        Vec2::new(0.22, TRUNK_HEIGHT),
        6,
        srgb(0.36, 0.24, 0.14),
    );
    let tiers: &[(f32, f32, f32)] = if variant == 0 {
        &[(1.7, 1.8, 0.0), (1.35, 1.6, 1.0), (0.95, 1.4, 1.9)]
    } else {
        &[(1.9, 2.0, 0.0), (1.3, 1.8, 1.2)]
    };
    let green = if variant == 0 {
        srgb(0.16, 0.40, 0.18)
    } else {
        srgb(0.22, 0.46, 0.20)
    };
    for (radius, height, lift) in tiers {
        cone(
            &mut m,
            Vec3::Y * (TRUNK_HEIGHT * 0.7 + lift),
            Vec2::new(*radius, *height),
            7,
            green,
        );
    }
    m.into_mesh()
}

/// Builds a rock mesh resting on the origin. The seed changes the shape.
pub fn rock_mesh(seed: u32) -> Mesh {
    let mut m = FlatMesh::default();
    icosa(
        &mut m,
        Vec3::Y * 0.4,
        Vec3::new(0.9, 0.6, 0.8),
        (0.28, seed),
        srgb(0.47, 0.47, 0.50),
    );
    m.into_mesh()
}

/// Builds a squashed green bush resting on the origin.
pub fn bush_mesh() -> Mesh {
    let mut m = FlatMesh::default();
    icosa(
        &mut m,
        Vec3::Y * 0.3,
        Vec3::new(0.6, 0.45, 0.6),
        (0.2, 11),
        srgb(0.20, 0.45, 0.17),
    );
    m.into_mesh()
}

/// Builds one mesh holding a three-blade tuft at every position, `(x, y, z, yaw)` in chunk space.
pub fn grass_mesh(tufts: &[(Vec3, f32)]) -> Mesh {
    let mut m = FlatMesh::default();
    for (i, (p, yaw)) in tufts.iter().enumerate() {
        let tone = 0.9 + (i % 5) as f32 * 0.05;
        let color = srgb(0.30 * tone, 0.58 * tone, 0.22 * tone);
        for blade in 0..3 {
            let a = yaw + blade as f32 * 2.1;
            let side = Vec3::new(a.cos(), 0.0, a.sin()) * 0.09;
            let lean = Vec3::new(-a.sin(), 0.0, a.cos()) * 0.12;
            m.tri(*p - side, *p + side, *p + lean + Vec3::Y * 0.55, color);
            m.tri(*p + side, *p - side, *p + lean + Vec3::Y * 0.55, color);
        }
    }
    m.into_mesh()
}
