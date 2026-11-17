//! The look of the ship: a small flat-shaded hull with wings, a fin, a canopy, engines and gear.

use std::f32::consts::TAU;

use bevy::prelude::*;

use super::landing::GEAR;
use super::solid::Solid;

/// The materials a ship is made of. Each one is drawn as its own mesh.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Skin {
    /// Light grey hull plates.
    Hull,
    /// Orange trim.
    Trim,
    /// Dark metal of the engines and the gear.
    Dark,
    /// See-through canopy glass.
    Glass,
    /// The engine glow, whose brightness follows the throttle.
    Glow,
}

impl Skin {
    /// Every skin, in the order of [`hull_meshes`].
    pub const ALL: [Skin; 5] = [Skin::Hull, Skin::Trim, Skin::Dark, Skin::Glass, Skin::Glow];
}

/// Sides of the fuselage.
const SIDES: u32 = 8;

/// Stations of the fuselage from nose to tail: z, half width, half height and height of the centre.
const FUSELAGE: [[f32; 4]; 5] = [
    [-3.2, 0.42, 0.3, -0.05],
    [-1.8, 0.85, 0.55, 0.05],
    [0.4, 1.0, 0.65, 0.1],
    [2.4, 0.9, 0.6, 0.15],
    [3.2, 0.6, 0.45, 0.2],
];

/// Stations of the canopy bubble, like [`FUSELAGE`].
const CANOPY: [[f32; 4]; 2] = [[-2.1, 0.5, 0.42, 0.62], [-1.0, 0.58, 0.5, 0.72]];

/// Places of the two engines on the tail: x and y of their axes.
const ENGINES: [Vec2; 2] = [Vec2::new(-0.52, 0.2), Vec2::new(0.52, 0.2)];

/// Where the engines end and the exhaust starts, along z.
pub const NOZZLE_Z: f32 = 3.7;

/// Returns the point of a station of a lathe along z at corner `i` of `sides`.
fn ring([z, w, h, y]: [f32; 4], i: u32, sides: u32) -> Vec3 {
    let a = i as f32 / sides as f32 * TAU;
    Vec3::new(a.cos() * w, y + a.sin() * h, z)
}

/// Adds a closed body lathed along z through `stations`, closed by fans to the two `ends`: the
/// first in front of the stations, the second behind them.
fn lathe(solid: &mut Solid, stations: &[[f32; 4]], ends: (Vec3, Vec3), sides: u32) {
    let (first, last) = (stations[0], stations[stations.len() - 1]);
    for i in 0..sides {
        let out = ring([0.0, 1.0, 1.0, 0.0], i, sides) + ring([0.0, 1.0, 1.0, 0.0], i + 1, sides);
        solid.tri(
            [ends.0, ring(first, i, sides), ring(first, i + 1, sides)],
            Vec3::NEG_Z,
        );
        solid.tri(
            [ends.1, ring(last, i, sides), ring(last, i + 1, sides)],
            Vec3::Z,
        );
        for pair in stations.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            let corners = [
                ring(a, i, sides),
                ring(a, i + 1, sides),
                ring(b, i + 1, sides),
                ring(b, i, sides),
            ];
            solid.quad(corners, out);
        }
    }
}

/// Adds both wings with their upturned tips.
fn wings(hull: &mut Solid, trim: &mut Solid) {
    for side in [-1.0, 1.0] {
        let wing = [
            Vec3::new(0.9 * side, -0.1, -0.2),
            Vec3::new(0.9 * side, -0.1, 2.6),
            Vec3::new(3.2 * side, -0.3, 2.2),
            Vec3::new(3.2 * side, -0.3, 1.0),
        ];
        hull.plate(wing, Vec3::Y * 0.14);
        let tip = [
            wing[3],
            wing[2],
            wing[2] + Vec3::new(0.0, 0.5, 0.2),
            wing[3] + Vec3::new(0.0, 0.5, 0.3),
        ];
        trim.plate(tip, Vec3::X * 0.08 * side);
    }
}

/// Adds the fuselage, the wings, the nose trim and the tail fin.
fn airframe(hull: &mut Solid, trim: &mut Solid) {
    lathe(
        hull,
        &FUSELAGE[1..],
        (Vec3::new(0.0, 0.05, -1.8), Vec3::new(0.0, 0.2, 3.3)),
        SIDES,
    );
    lathe(
        trim,
        &FUSELAGE[..2],
        (Vec3::new(0.0, -0.1, -4.2), Vec3::new(0.0, 0.05, -1.8)),
        SIDES,
    );
    wings(hull, trim);
    let fin = [
        Vec3::new(0.0, 0.7, 1.5),
        Vec3::new(0.0, 0.7, 3.2),
        Vec3::new(0.0, 1.9, 3.4),
        Vec3::new(0.0, 1.9, 2.7),
    ];
    trim.plate(fin.map(|p| p - Vec3::X * 0.05), Vec3::X * 0.1);
}

/// Adds the engines with their glowing ends, and the landing gear.
fn machinery(dark: &mut Solid, glow: &mut Solid) {
    for at in ENGINES {
        let station = |z: f32, r: f32| [z, r, r, 0.0];
        let (front, back) = (Vec3::new(0.0, 0.0, 2.6), Vec3::new(0.0, 0.0, NOZZLE_Z));
        let mut engine = Solid::default();
        lathe(
            &mut engine,
            &[station(2.8, 0.42), station(NOZZLE_Z, 0.34)],
            (front, back),
            6,
        );
        dark.merge(&engine, at.extend(0.0));
        let mut disc = Solid::default();
        lathe(
            &mut disc,
            &[station(NOZZLE_Z + 0.01, 0.27)],
            (back, back + Vec3::Z * 0.06),
            6,
        );
        glow.merge(&disc, at.extend(0.0));
    }
    for foot in GEAR.map(|foot| foot.as_vec3()) {
        dark.slab(
            foot + Vec3::new(-0.07, 0.08, -0.07),
            foot + Vec3::new(0.07, 0.95, 0.07),
        );
        dark.slab(
            foot + Vec3::new(-0.26, 0.0, -0.32),
            foot + Vec3::new(0.26, 0.1, 0.32),
        );
    }
}

/// Builds the meshes of the ship, one per skin in the order of [`Skin::ALL`].
pub fn hull_meshes() -> [Mesh; 5] {
    let mut parts: [Solid; 5] = Default::default();
    let [hull, trim, dark, glass, glow] = &mut parts;
    airframe(hull, trim);
    machinery(dark, glow);
    lathe(
        glass,
        &CANOPY,
        (Vec3::new(0.0, 0.5, -2.9), Vec3::new(0.0, 0.75, 0.2)),
        6,
    );
    parts.map(Solid::into_mesh)
}

/// Builds the exhaust of both engines: unit cones that start at the nozzles and point back.
pub fn plume_mesh() -> Mesh {
    let mut plume = Solid::default();
    for at in ENGINES {
        let mut cone = Solid::default();
        lathe(
            &mut cone,
            &[[0.0, 0.25, 0.25, 0.0]],
            (Vec3::NEG_Z * 0.05, Vec3::Z),
            6,
        );
        plume.merge(&cone, at.extend(0.0));
    }
    plume.into_mesh()
}

/// Returns the material of a skin. The glow is unlit and takes its color from the throttle.
pub fn material(skin: Skin) -> StandardMaterial {
    let paint = |r: f32, g: f32, b: f32, roughness: f32| StandardMaterial {
        base_color: Color::srgb(r, g, b),
        perceptual_roughness: roughness,
        ..default()
    };
    match skin {
        Skin::Hull => paint(0.78, 0.8, 0.82, 0.55),
        Skin::Trim => paint(0.92, 0.45, 0.12, 0.6),
        Skin::Dark => paint(0.14, 0.15, 0.18, 0.8),
        Skin::Glass => StandardMaterial {
            base_color: Color::srgba(0.45, 0.78, 0.95, 0.32),
            alpha_mode: AlphaMode::Blend,
            perceptual_roughness: 0.1,
            ..default()
        },
        Skin::Glow => StandardMaterial {
            base_color: super::glow::color(0.0),
            unlit: true,
            opaque_render_method: bevy::material::OpaqueRendererMethod::Forward,
            ..default()
        },
    }
}

#[cfg(test)]
mod tests;
