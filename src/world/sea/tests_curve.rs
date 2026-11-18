//! Tests of the curve of the near sea: the mesh on the sphere and the hand-over distance.

use std::f32::consts::TAU;
use std::f64::consts::FRAC_PI_2;

use bevy::math::DVec3;
use bevy::prelude::*;

use super::follow::{HANDOVER, placing, wind_in_seat};
use super::mesh::lines;
use super::uniform::WaterUniform;
use crate::wind::Wind;
use crate::world::Globe;
use crate::world::globe::quadtree::{MERGE, SPLIT};

/// The shader source, to pin the constants and formulas mirrored here.
const SHADER: &str = include_str!("water.wesl");

/// Share of the hand-over distance at which the fade starts, as in the shader.
const HAND_SOFT: f32 = 0.83;

/// Longest render distance the options menu offers, in chunks.
const LONGEST_VIEW: f64 = 8.0;

/// Mirrors `lift` of the shader: where chart point `p` lies in the frame of the seat.
fn lift(p: Vec2, radius: f32) -> Vec3 {
    let angle = p.length() / radius;
    let reach = angle.sin() / angle.max(0.000001);
    let half = (0.5 * angle).sin();
    Vec3::new(p.x * reach, -2.0 * radius * half * half, p.y * reach)
}

/// Returns the height above the sea level sphere of the still mesh, `q` meters from its
/// center, for a mesh centered on chart point `centre`: the triangle the shader draws there.
fn surface(q: Vec2, centre: Vec2, radius: f32) -> f64 {
    let lines = lines();
    let cell = |v: f32| lines.partition_point(|&l| l <= v).clamp(1, lines.len() - 1);
    let (i, j) = (cell(q.x), cell(q.y));
    let (x, z) = ([lines[i - 1], lines[i]], [lines[j - 1], lines[j]]);
    let (u, v) = ((q.x - x[0]) / (x[1] - x[0]), (q.y - z[0]) / (z[1] - z[0]));
    let corner = |a: usize, b: usize| lift(centre + Vec2::new(x[a], z[b]), radius).as_dvec3();
    let (u, v) = (f64::from(u), f64::from(v));
    let point = if u + v <= 1.0 {
        corner(0, 0) + (corner(1, 0) - corner(0, 0)) * u + (corner(0, 1) - corner(0, 0)) * v
    } else {
        let far = corner(1, 1);
        far + (corner(0, 1) - far) * (1.0 - u) + (corner(1, 0) - far) * (1.0 - v)
    };
    (point + DVec3::Y * f64::from(radius)).length() - f64::from(radius)
}

/// The mirrored formula and constant are the ones the shader runs.
#[test]
fn the_mirror_matches_the_shader() {
    assert!(SHADER.contains("let reach = sin(angle) / max(angle, 0.000001);"));
    assert!(
        SHADER.contains("return vec3<f32>(p.x * reach, -2.0 * radius * half * half, p.y * reach);")
    );
    assert!(SHADER.contains(&format!("const HAND_SOFT: f32 = {HAND_SOFT};")));
}

/// On small and large globes the still near sea, 200 m from the camera, lies within 2 cm of
/// the sea level sphere, between vertices too, and every vertex lies on it within 2 mm.
#[test]
fn the_near_sea_lies_on_the_sphere() {
    for radius in [3000.0_f32, 40_000.0] {
        for centre in [
            Vec2::ZERO,
            Vec2::new(412.0, -286.0),
            Vec2::new(-0.12, 0.15) * radius,
        ] {
            let camera = Vec2::new(0.7, -0.4);
            for step in 0..256 {
                let q = camera + Vec2::from_angle(step as f32 / 256.0 * TAU) * 200.0;
                let off = surface(q, centre, radius);
                assert!(off.abs() < 0.02, "{radius} {centre} {q}: {off}");
            }
            for line in lines() {
                let off = surface(Vec2::new(line, -line), centre, radius);
                assert!(off.abs() < 2e-3, "{radius} {centre} {line}: {off}");
            }
        }
    }
}

/// On every planet size, at the longest render distance, no detail chunk reaches the fade of
/// the near sea, and the fade ends inside the mesh.
#[test]
fn the_hand_over_clears_the_detail_chunks() {
    for radius in (3000..=40_000).step_by(50) {
        let radius = f64::from(radius);
        let level = Globe::new(1, radius, 9.8).leaf_level();
        let arc = radius * FRAC_PI_2 / f64::from(1u32 << level);
        let parent = 2.0 * arc;
        let range = (parent * SPLIT).max((LONGEST_VIEW + 1.0) * arc + parent * 0.75);
        let farthest = range * (MERGE / SPLIT) + parent * 0.75;
        assert!(
            farthest < f64::from(HANDOVER * HAND_SOFT),
            "{radius}: {farthest}"
        );
    }
    assert!(lines().last().is_some_and(|rim| HANDOVER <= *rim));
}

/// The sea reads the wind in the axes of its seat, and its uniform puts the planet center one
/// radius under the seat and the mesh on the fine grid.
#[test]
fn the_sea_is_placed_in_its_seat() {
    let pose = Transform::from_xyz(12.0, -3.0, 40.0).with_rotation(Quat::from_rotation_y(0.5));
    let wind = Wind {
        direction: Vec2::X,
        strength: 0.4,
        air: 1.0,
    };
    let seen = wind_in_seat(&wind, pose.rotation);
    let back = pose.rotation * Vec3::new(seen.direction.x, 0.0, seen.direction.y);
    assert!((back - Vec3::X).length() < 1e-5 && seen.strength == 0.4);
    let water = WaterUniform::default();
    let placed = placing(
        water,
        &pose,
        3000.0,
        Vec2::new(8.0, -6.0),
        Vec2::new(7.3, -4.9),
    );
    assert_eq!(placed.map, Vec4::new(8.0, -6.0, 8.0, -4.0));
    assert!((placed.globe.truncate() - Vec3::new(12.0, -3003.0, 40.0)).length() < 1e-3);
    assert_eq!((placed.globe.w, placed.frame.w), (3000.0, HANDOVER));
    assert!((placed.frame.truncate() - pose.rotation * Vec3::X).length() < 1e-6);
}

/// Mirrors `keep` of the shader: the share of a wave of wave number `k` left in the normal of a
/// fragment whose pixel covers `foot` meters of sea.
fn keep(k: f32, foot: f32) -> f32 {
    let t = ((foot * k / std::f32::consts::TAU - 0.15) / (0.5 - 0.15)).clamp(0.0, 1.0);
    1.0 - t * t * (3.0 - 2.0 * t)
}

/// A wave leaves the water normal before a pixel covers half its length, so far water cannot
/// alias into rings: every swell and ripple term of the normal goes through that fade, the
/// glints and flecks fade with the pixel too, and nothing but the fades reads the distance.
#[test]
fn far_water_normals_fade_before_they_alias() {
    for wavelength in [22.0_f32, 11.0, 6.0, 4.0, 1.9, 1.1] {
        let k = std::f32::consts::TAU / wavelength;
        assert_eq!(keep(k, wavelength * 0.5), 0.0);
        assert_eq!(keep(k, wavelength * 0.15), 1.0);
        assert!(keep(k, wavelength * 0.3) > 0.0 && keep(k, wavelength * 0.3) < 1.0);
    }
    assert_eq!(keep(std::f32::consts::TAU / 22.0, 40.0), 0.0);
    for line in [
        "return 1.0 - smoothstep(KEEP_START, KEEP_END, foot * k / TAU);",
        "const KEEP_START: f32 = 0.15;",
        "const KEEP_END: f32 = 0.5;",
        "return wave_term(w, p, t, seed).yz * keep(w.z, foot);",
        "let foot = max(length(dpdx(p)), length(dpdy(p)));",
        "var n = normalize(up - east * slope.x - south * slope.y);",
        "let fine = detail * (1.0 - smoothstep(SPARK_START, SPARK_END, foot));",
    ] {
        assert!(SHADER.contains(line), "{line}");
    }
    assert_eq!(
        SHADER.matches("ripple(rotate(").count(),
        SHADER.matches("* keep(").count() - 1
    );
    let fragment = &SHADER[SHADER.find("@fragment").unwrap_or_default()..];
    assert_eq!(fragment.matches("vertex.position").count(), 0);
    assert_eq!(
        fragment.matches("dist").count(),
        6,
        "only the view vector, depth and fades"
    );
}
