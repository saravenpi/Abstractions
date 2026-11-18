//! Flame and lantern meshes: layered low-poly fire shapes shared by every torch, fire and lamp.

use std::f32::consts::TAU;

use bevy::prelude::*;

use super::glow::LightKind;
use crate::world::shapes::textured_mesh;

/// Sides of a flame shell.
const SIDES: u32 = 6;

/// Radius and height of the rings of the unit teardrop: a wide low belly that tapers to the tip at height 1.
const PROFILE: [(f32, f32); 3] = [(0.55, 0.0), (1.0, 0.22), (0.6, 0.56)];

/// Width and height of the outer, middle and core shells, as shares of a tongue.
const SHELLS: [(f32, f32); 3] = [(1.0, 1.0), (0.68, 0.74), (0.38, 0.5)];

/// Linear HDR colors of the shells from the outside in: deep orange-red, orange and a pale yellow core.
pub const SHELL_COLORS: [[f32; 3]; 3] = [[1.7, 0.2, 0.03], [3.2, 1.05, 0.12], [7.0, 4.6, 1.6]];

/// One shell of a flame: a teardrop child of a light, scaled and leaned about its base.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Layer {
    /// Base of the shell relative to the light.
    pub base: Vec3,
    /// Radius and height at rest, in meters.
    pub size: Vec2,
    /// Offset of its motion in seconds, so no two shells move together.
    pub seed: f32,
    /// How much it moves, from 0 (still) to 1 (open fire).
    pub lively: f32,
}

/// Returns the triangles of a closed solid of revolution, wound outward.
///
/// `rings` are (radius, height) pairs from the bottom up; `ends` are the heights of the bottom and top points.
fn lathe(sides: u32, rings: &[(f32, f32)], ends: (f32, f32)) -> Vec<[Vec3; 3]> {
    let at = |ring: usize, i: u32| {
        let (radius, height) = rings[ring];
        let a = i as f32 / sides as f32 * TAU;
        Vec3::new(a.cos() * radius, height, a.sin() * radius)
    };
    let (bottom, top) = (Vec3::Y * ends.0, Vec3::Y * ends.1);
    let last = rings.len() - 1;
    let mut tris = Vec::new();
    for i in 0..sides {
        tris.push([at(0, i), at(0, i + 1), bottom]);
        tris.push([at(last, i + 1), at(last, i), top]);
        for ring in 0..last {
            tris.push([at(ring, i + 1), at(ring, i), at(ring + 1, i)]);
            tris.push([at(ring, i + 1), at(ring + 1, i), at(ring + 1, i + 1)]);
        }
    }
    tris
}

/// Builds a flat-shaded mesh.
fn solid(tris: &[[Vec3; 3]]) -> Mesh {
    let positions = tris.iter().flatten().map(|p| p.to_array()).collect();
    let normals = tris
        .iter()
        .flat_map(|[a, b, c]| [(*b - *a).cross(*c - *a).normalize_or_zero().to_array(); 3])
        .collect();
    textured_mesh(positions, normals, vec![[0.5, 0.5]; tris.len() * 3])
}

/// Builds the unit flame shell: base at the origin, tip at height 1, widest radius 1.
pub fn teardrop() -> Mesh {
    solid(&lathe(SIDES, &PROFILE, (0.0, 1.0)))
}

/// Returns how far the flame base sits under the light, how lively it is and its tongues as x, z, radius, height.
fn layout(kind: LightKind) -> (f32, f32, &'static [[f32; 4]]) {
    match kind {
        LightKind::Lamp => (0.14, 0.15, &[[0.0, 0.0, 0.1, 0.28]]),
        LightKind::Torch => (0.15, 1.0, &[[0.0, 0.0, 0.085, 0.36]]),
        LightKind::Candle => (0.05, 0.3, &[[0.0, 0.0, 0.03, 0.11]]),
        LightKind::Brazier => (
            0.275,
            1.0,
            &[[0.0, 0.0, 0.2, 0.62], [0.1, -0.06, 0.12, 0.4]],
        ),
        LightKind::Campfire => (
            0.45,
            1.0,
            &[
                [0.0, 0.0, 0.3, 0.95],
                [0.21, 0.1, 0.18, 0.6],
                [-0.17, -0.15, 0.15, 0.45],
            ],
        ),
        LightKind::Hearth => (
            0.3,
            0.9,
            &[
                [0.0, 0.0, 0.26, 0.7],
                [0.19, 0.04, 0.15, 0.46],
                [-0.16, -0.05, 0.13, 0.36],
            ],
        ),
    }
}

/// Returns the shells of the flame of a light kind, each with the index of its color, for a light with `phase`.
pub fn shells(kind: LightKind, phase: f32) -> Vec<(usize, Layer)> {
    let (drop, lively, tongues) = layout(kind);
    let shell = move |tongue: usize, [x, z, radius, height]: [f32; 4], index: usize| Layer {
        base: Vec3::new(x, -drop, z),
        size: Vec2::new(radius * SHELLS[index].0, height * SHELLS[index].1),
        seed: phase + tongue as f32 * 2.4 + index as f32 * 0.9,
        lively,
    };
    tongues
        .iter()
        .enumerate()
        .flat_map(|(tongue, at)| {
            (0..SHELLS.len()).map(move |index| (index, shell(tongue, *at, index)))
        })
        .collect()
}

/// Returns the transform of a shell at `seconds`: stretched, squeezed and leaning from its base, bent by `wind`.
///
/// The base never moves: the unit shell has its base at the origin, so scale and lean pivot there.
pub fn pose(layer: &Layer, seconds: f32, wind: Vec3) -> Transform {
    let (t, k) = (seconds + layer.seed * 1.7, layer.lively);
    let stretch = 0.14 * (t * 7.1).sin() + 0.09 * (t * 12.7 + 1.3).sin() + 0.05 * (t * 19.3).sin();
    let squeeze = 1.0 - k * 0.07 * (t * 9.4 + 0.7).sin();
    let sway = Vec3::new((t * 3.1).sin(), 0.0, (t * 4.3 + 2.0).cos()) * 0.1;
    let lean = Vec3::Y + (sway + Vec3::new(wind.x, 0.0, wind.z) * 0.035) * k;
    Transform {
        translation: layer.base,
        rotation: Quat::from_rotation_arc(Vec3::Y, lean.normalize()),
        scale: Vec3::new(
            layer.size.x * squeeze,
            layer.size.y * (1.0 + k * stretch),
            layer.size.x * squeeze,
        ),
    }
}
