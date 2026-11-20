//! The look of the ship: a small flat-shaded hull with wings, a fin, a canopy, engines and gear,
//! and the cockpit under the canopy.

mod bars;
mod fittings;
mod frame;
mod levers;
mod looks;
mod machinery;
mod outline;
mod shell;
mod tub;

use bevy::prelude::*;

pub use levers::{bar_mesh, lever_mesh, stick_mesh};
pub use looks::{Looks, dress};
pub use machinery::{NOZZLE_Z, plume_mesh};
pub use outline::hulls;

use super::gear::{GEAR, STRUT};
use super::solid::Solid;
use frame::{fin, wing};
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

/// Builds the meshes of the ship, one per skin in the order of [`Skin::ALL`], without the
/// legs of the gear, which fold and are drawn from [`leg_mesh`].
pub fn hull_meshes() -> [Mesh; 5] {
    let mut parts: [Solid; 5] = Default::default();
    let [hull, trim, dark, glass, glow] = &mut parts;
    let ([body, nose], [engine, disc]) = (shell::hull(), engines());
    hull.merge(&body, Vec3::ZERO);
    trim.merge(&nose, Vec3::ZERO);
    trim.merge(&fin(), Vec3::ZERO);
    for [wing, winglet] in [wing(-1.0), wing(1.0)] {
        hull.merge(&wing, Vec3::ZERO);
        trim.merge(&winglet, Vec3::ZERO);
    }
    dark.merge(&engine, Vec3::ZERO);
    glass.merge(&shell::glass(), Vec3::ZERO);
    glow.merge(&disc, Vec3::ZERO);
    parts.map(Solid::into_mesh)
}

/// Builds what is inside the cockpit and does not move, each mesh with its skin: the dark
/// furniture with the frame of the canopy, and the cushions of the seat.
pub fn cabin_meshes() -> [(Skin, Mesh); 2] {
    let mut dark = fittings::furniture();
    dark.merge(&bars::frame(), Vec3::ZERO);
    [
        (Skin::Dark, dark.into_mesh()),
        (Skin::Trim, fittings::seat().into_mesh()),
    ]
}

/// Builds the tub of the cockpit, which takes its own grey lining.
pub fn tub_mesh() -> Mesh {
    tub::tub().into_mesh()
}

/// Builds the black faces of the instruments, which take their own unlit black.
pub fn faces_mesh() -> Mesh {
    fittings::faces().into_mesh()
}

/// Builds one leg of the gear, strut and pad, hanging from its hinge at the origin: scaled
/// along y it folds up into the hull.
pub fn leg_mesh() -> Mesh {
    let mut solid = Solid::default();
    for part in leg(-Vec3::Y * STRUT as f32) {
        solid.merge(&part, Vec3::ZERO);
    }
    solid.into_mesh()
}

/// Returns where the hinges of the three legs are in body axes, at the top of each strut.
pub fn hinges() -> [Vec3; 3] {
    GEAR.map(|foot| (foot + bevy::math::DVec3::Y * STRUT).as_vec3())
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
            base_color: Color::srgba(0.9, 0.9, 0.9, 0.08),
            alpha_mode: AlphaMode::Blend,
            perceptual_roughness: 0.05,
            reflectance: 0.5,
            double_sided: true,
            cull_mode: None,
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
mod tests_cabin;
#[cfg(test)]
mod tests_fit;
#[cfg(test)]
mod tests_view;
