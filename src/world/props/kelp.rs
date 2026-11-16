//! Kelp: a swaying ribbon with leaves whose height follows the water depth.

use bevy::prelude::*;

use crate::world::shapes::FlatMesh;

/// Height of one ribbon segment.
const SEGMENT: f32 = 1.5;

/// Returns how many ribbon segments a kelp of `height` has.
fn segments(height: f32) -> usize {
    (height / SEGMENT).ceil().clamp(2.0, 14.0) as usize
}

/// Returns how many triangles a kelp of `height` has: four for the ribbon and two for the leaf of each segment.
pub fn kelp_tris(height: f32) -> usize {
    segments(height) * 6
}

/// Adds a kelp ribbon rising `height` from `base`, facing `yaw`, with both sides of every triangle visible.
pub fn kelp(mesh: &mut FlatMesh, base: Vec3, height: f32, yaw: f32, color: [f32; 4]) {
    let n = segments(height);
    let step = height / n as f32;
    let side = Vec3::new(yaw.cos(), 0.0, yaw.sin());
    let lean = Vec3::new(-yaw.sin(), 0.0, yaw.cos());
    let spine = |i: usize| {
        let t = i as f32 / n as f32;
        base + Vec3::Y * step * i as f32 + lean * (i as f32 * 0.8 + yaw).sin() * 0.45 * t
    };
    let width = |i: usize| 0.22 * (1.0 - 0.5 * i as f32 / n as f32);
    let leaf = [color[0] * 1.25, color[1] * 1.25, color[2] * 1.25, color[3]];
    let mut both = |a: Vec3, b: Vec3, c: Vec3, tone: [f32; 4]| {
        mesh.tri(a, b, c, tone);
        mesh.tri(c, b, a, tone);
    };
    for i in 0..n {
        let (a, b) = (spine(i), spine(i + 1));
        let (wa, wb) = (width(i), width(i + 1));
        both(a - side * wa, a + side * wa, b - side * wb, color);
        both(a + side * wa, b + side * wb, b - side * wb, color);
        let out = if i % 2 == 0 { side } else { -side };
        both(
            a,
            a + out * 0.7 + Vec3::Y * step * 0.5,
            a + Vec3::Y * step * 0.8,
            leaf,
        );
    }
}
