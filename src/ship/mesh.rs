//! The look of the ship: a small flat-shaded hull with wings, a fin, a canopy, engines and gear.

mod frame;
mod machinery;

use bevy::prelude::*;

pub use machinery::{NOZZLE_Z, plume_mesh};

use super::landing::GEAR;
use super::solid::Solid;
use frame::{canopy, fin, fuselage, wing};
use machinery::{engines, leg};

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

/// Builds the meshes of the ship, one per skin in the order of [`Skin::ALL`].
pub fn hull_meshes() -> [Mesh; 5] {
    let mut parts: [Solid; 5] = Default::default();
    let [hull, trim, dark, glass, glow] = &mut parts;
    let ([body, nose], [engine, disc]) = (fuselage(), engines());
    hull.merge(&body, Vec3::ZERO);
    trim.merge(&nose, Vec3::ZERO);
    trim.merge(&fin(), Vec3::ZERO);
    for [wing, winglet] in [wing(-1.0), wing(1.0)] {
        hull.merge(&wing, Vec3::ZERO);
        trim.merge(&winglet, Vec3::ZERO);
    }
    let legs = GEAR.map(|foot| leg(foot.as_vec3()));
    for part in legs.as_flattened().iter().chain([&engine]) {
        dark.merge(part, Vec3::ZERO);
    }
    glass.merge(&canopy(), Vec3::ZERO);
    glow.merge(&disc, Vec3::ZERO);
    parts.map(Solid::into_mesh)
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
#[cfg(test)]
mod tests_fit;
