//! Merged emissive bioluminescent cave flora (hanging leaf vines, ceiling moss, fungal caps,
//! crystal spires) and 3 pooled non-shadow natural lights with gentle breathing pulse.

use std::f32::consts::TAU;

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

use super::plan::{CaveBiome, CavePlan, SpineNode};
use crate::universe::hash::{Rng, child};

/// Breathing animation metadata for a pooled non-shadow cave light.
#[derive(Component, Debug, Clone, Copy)]
pub struct CaveLight {
    /// Base luminous intensity in lumens.
    pub base_intensity: f32,
    /// Seeded phase offset in radians.
    pub phase: f32,
}

/// Triangle soup builder for one merged bioluminescent flora and crystal mesh.
#[derive(Default)]
struct FloraBuf {
    /// Vertex positions in cave tangent frame.
    positions: Vec<[f32; 3]>,
    /// Flat normals.
    normals: Vec<[f32; 3]>,
    /// Linear RGBA vertex colours.
    colors: Vec<[f32; 4]>,
}

/// Emits one flat-shaded triangle into `buf`.
fn push_tri(buf: &mut FloraBuf, a: Vec3, b: Vec3, c: Vec3, col: [f32; 4]) {
    let n = (b - a).cross(c - a).normalize_or(Vec3::Y);
    buf.positions
        .extend([a.to_array(), b.to_array(), c.to_array()]);
    buf.normals.extend([n.to_array(); 3]);
    buf.colors.extend([col; 3]);
}

/// Adds a hanging bioluminescent leaf vine draped downward from `top` for `len` metres.
fn add_vine(buf: &mut FloraBuf, top: Vec3, len: f32, col: [f32; 4], seed: u64) {
    let mut rng = Rng::new(seed);
    let leaves = ((len * 2.2) as usize).clamp(3, 8);
    let step = len / leaves as f32;
    for k in 0..leaves {
        let sway = Vec3::new(
            (k as f32 * 1.1 + seed as f32).sin() * 0.14,
            -(k as f32 * step),
            (k as f32 * 0.9).cos() * 0.14,
        );
        let p = top + sway;
        let ang = rng.unit() as f32 * TAU;
        let side = Vec3::new(ang.cos(), 0.0, ang.sin()) * (0.22 + rng.unit() as f32 * 0.16);
        let tip = p - Vec3::Y * (step * 1.15);
        push_tri(buf, p, p + side, tip, col);
        push_tri(buf, p, tip, p - side, col);
    }
}

/// Adds a pointed 5-sided glowing crystal spire growing from `base` along `dir`.
fn add_crystal(
    buf: &mut FloraBuf,
    base: Vec3,
    dir: Vec3,
    (radius, len): (f32, f32),
    col: [f32; 4],
) {
    let axis = dir.normalize_or(Vec3::Y);
    let right = axis.any_orthonormal_vector();
    let fwd = axis.cross(right);
    let tip = base + axis * len;
    let ring: [Vec3; 5] = std::array::from_fn(|k| {
        let a = TAU * k as f32 / 5.0;
        base + (right * a.cos() + fwd * a.sin()) * radius
    });
    for k in 0..5 {
        push_tri(buf, ring[k], ring[(k + 1) % 5], tip, col);
    }
}

/// Adds a giant glowing fungal cap with a tapered stem at `base`.
fn add_fungus(
    buf: &mut FloraBuf,
    base: Vec3,
    (height, cap_r): (f32, f32),
    (stem_col, cap_col): ([f32; 4], [f32; 4]),
) {
    let top = base + Vec3::Y * height;
    let peak = top + Vec3::Y * (cap_r * 0.48);
    let ring: [Vec3; 6] = std::array::from_fn(|k| {
        let a = TAU * k as f32 / 6.0;
        Vec3::new(a.cos(), 0.0, a.sin())
    });
    for k in 0..6 {
        let (d0, d1) = (ring[k], ring[(k + 1) % 6]);
        push_tri(
            buf,
            base + d0 * (cap_r * 0.22),
            base + d1 * (cap_r * 0.22),
            top,
            stem_col,
        );
        push_tri(buf, top + d0 * cap_r, top + d1 * cap_r, peak, cap_col);
        push_tri(
            buf,
            top + d1 * cap_r,
            top + d0 * cap_r,
            top - Vec3::Y * 0.08,
            stem_col,
        );
    }
}

/// Populates biome-specific glowing foliage, crystals, or fungal caps at one spine node.
fn add_node_flora(
    buf: &mut FloraBuf,
    biome: CaveBiome,
    (node, i, seed): (SpineNode, usize, u64),
    (c1, c2): ([f32; 4], [f32; 4]),
) {
    let mut rng = Rng::new(child(seed, 0x1000 + i as u64));
    let side = if i % 2 == 0 { 1.0 } else { -1.0 };
    match biome {
        CaveBiome::BioluminescentGrotto => {
            let off = Vec3::new(
                (rng.unit() as f32 - 0.5) * node.rx * 0.9,
                node.ry * 0.86,
                (rng.unit() as f32 - 0.5) * node.rx * 0.9,
            );
            add_vine(
                buf,
                node.pos + off,
                1.4 + rng.unit() as f32 * 2.0,
                if i % 2 == 0 { c1 } else { c2 },
                child(seed, 0x2000 + i as u64),
            );
        }
        CaveBiome::CrystalSpire => {
            let base = Vec3::new(
                node.pos.x + side * node.rx * 0.72,
                node.pos.y - node.ry * 0.38,
                node.pos.z,
            );
            let dir = Vec3::new(-side * 0.35, 1.0, (rng.unit() as f32 - 0.5) * 0.4);
            add_crystal(
                buf,
                base,
                dir,
                (
                    0.28 + rng.unit() as f32 * 0.24,
                    1.5 + rng.unit() as f32 * 2.1,
                ),
                if i % 3 == 0 { c2 } else { c1 },
            );
        }
        CaveBiome::MycelialDepths => {
            let base = Vec3::new(
                node.pos.x + side * node.rx * 0.66,
                node.pos.y - node.ry * 0.42,
                node.pos.z,
            );
            add_fungus(
                buf,
                base,
                (
                    0.9 + rng.unit() as f32 * 1.3,
                    0.55 + rng.unit() as f32 * 0.75,
                ),
                (c2, c1),
            );
        }
    }
}

/// Builds ONE merged emissive bioluminescent foliage and crystal mesh for `plan`.
pub fn build_flora_mesh(plan: &CavePlan, accent: [f32; 3]) -> (Mesh, StandardMaterial) {
    let (c1, c2, emissive) = match plan.biome {
        CaveBiome::BioluminescentGrotto => (
            LinearRgba::from(Color::srgb(0.18, 0.92, 0.78)).to_f32_array(),
            LinearRgba::from(Color::srgb(0.32, 0.98, 0.56)).to_f32_array(),
            LinearRgba::rgb(0.35, 1.45, 1.15),
        ),
        CaveBiome::CrystalSpire => (
            LinearRgba::from(Color::srgb(accent[0], accent[1], accent[2])).to_f32_array(),
            LinearRgba::from(Color::srgb(0.74, 0.86, 1.0)).to_f32_array(),
            LinearRgba::rgb(accent[0] * 1.4, accent[1] * 1.4, accent[2] * 1.6),
        ),
        CaveBiome::MycelialDepths => (
            LinearRgba::from(Color::srgb(0.95, 0.66, 0.24)).to_f32_array(),
            LinearRgba::from(Color::srgb(0.24, 0.84, 0.76)).to_f32_array(),
            LinearRgba::rgb(1.35, 0.92, 0.38),
        ),
    };
    let mut buf = FloraBuf::default();
    for k in 0..8 {
        let a = TAU * k as f32 / 8.0;
        let lip = Vec3::new(
            a.cos() * plan.entry_radius * 0.84,
            plan.entry_y + 0.1,
            a.sin() * plan.entry_radius * 0.84,
        );
        add_vine(
            &mut buf,
            lip,
            1.4 + (k % 3) as f32 * 0.4,
            c1,
            child(plan.seed, 0xE00 + k as u64),
        );
    }
    for (i, &node) in plan.main_spine.iter().enumerate().skip(2) {
        add_node_flora(&mut buf, plan.biome, (node, i, plan.seed), (c1, c2));
    }
    let count = buf.positions.len() as u32;
    let mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, buf.positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, buf.normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, buf.colors)
    .with_inserted_indices(Indices::U32((0..count).collect()));
    let mat = StandardMaterial {
        base_color: Color::WHITE,
        emissive,
        perceptual_roughness: 0.45,
        cull_mode: None,
        ..default()
    };
    (mesh, mat)
}

/// Spawns 3 pooled non-shadow natural lights inside the cave (threshold, mid-tunnel, main chamber).
pub fn spawn_cave_lights(commands: &mut Commands, plan: &CavePlan, root: Entity) {
    let color = match plan.biome {
        CaveBiome::BioluminescentGrotto => Color::srgb(0.32, 0.96, 0.82),
        CaveBiome::CrystalSpire => Color::srgb(0.76, 0.84, 1.0),
        CaveBiome::MycelialDepths => Color::srgb(0.98, 0.72, 0.36),
    };
    let mid = plan.main_spine[plan.main_spine.len() / 2].pos;
    let spots = [
        (plan.main_spine[1].pos + Vec3::Y * 1.2, 6_500.0, 22.0, 0.0),
        (mid + Vec3::Y * 1.4, 9_500.0, 28.0, 2.1),
        (plan.chamber_center + Vec3::Y * 2.2, 16_000.0, 38.0, 4.2),
    ];
    for (pos, intensity, range, phase) in spots {
        commands.spawn((
            PointLight {
                color,
                intensity,
                range,
                shadow_maps_enabled: false,
                ..default()
            },
            CaveLight {
                base_intensity: intensity,
                phase: phase + (plan.seed % 63) as f32 * 0.1,
            },
            Transform::from_translation(pos),
            ChildOf(root),
        ));
    }
}

/// Gently pulses active cave point lights for living bioluminescent ambience.
pub fn pulse_cave_lights(time: Res<Time>, mut lights: Query<(&CaveLight, &mut PointLight)>) {
    let t = time.elapsed_secs();
    for (cfg, mut light) in &mut lights {
        light.intensity = cfg.base_intensity * (0.86 + 0.14 * (t * 1.35 + cfg.phase).sin());
    }
}
