//! The look of the ship: a flat-shaded dart with a chined hull, a canopy that runs back into a
//! spine and a swept fin, wings, engine nacelles and gear, and the cockpit under the canopy.

mod bars;
mod blade;
mod dome;
mod fittings;
mod frame;
mod lames;
mod levers;
mod lod;
mod loft;
mod looks;
mod outline;
mod outside;
mod parts;
mod plating;
mod ports;
mod shell;
mod struts;
mod tail;
mod tub;
mod wing;

use bevy::prelude::*;

pub use dome::glass;
pub use levers::{lever_mesh, stick_mesh};
pub use lod::{FAR, far_mesh};
pub use looks::{Looks, dress};
pub use outline::hulls;
pub use outside::{HEAD_LAMP, TIP_LAMPS};
pub use ports::{LIFT, LIFT_AT, Port, ports};
pub use tail::{ENGINES, EXIT, NOZZLE_Z};

use super::cockpit::HINGE;
use super::gear::{GEAR, STRUT};
use super::solid::Solid;
use parts::{Paint, parts};

/// The materials a ship is made of. Each one is drawn as its own mesh.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Skin {
    /// Bone hull plates, toned by their vertex colours.
    Hull,
    /// The accent colour.
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
/// legs of the gear, which fold and are drawn from [`leg_mesh`], and without the lights, which
/// are drawn from [`lamp_meshes`]. The hull, wings, fin, engines, pods and fittings are all
/// there: [`parts`] is the one list they come from.
pub fn hull_meshes() -> [Mesh; 5] {
    let mut skins: [Solid; 5] = Default::default();
    for part in parts() {
        let skin = match part.paint {
            Paint::Hull => Skin::Hull,
            Paint::Trim => Skin::Trim,
            Paint::Dark => Skin::Dark,
            Paint::Glass => Skin::Glass,
            Paint::Glow => Skin::Glow,
            _ => continue,
        };
        skins[skin as usize].merge(&part.solid, Vec3::ZERO);
    }
    skins.map(Solid::into_mesh)
}

/// Builds the meshes of the three kinds of light, in the order red, green and white.
pub fn lamp_meshes() -> [Mesh; 3] {
    let mut lamps: [Solid; 3] = Default::default();
    for part in parts() {
        let at = match part.paint {
            Paint::Red => 0,
            Paint::Green => 1,
            Paint::White => 2,
            _ => continue,
        };
        lamps[at].merge(&part.solid, Vec3::ZERO);
    }
    lamps.map(Solid::into_mesh)
}

/// Builds what is inside the cockpit and does not move, each mesh with its skin: the dark
/// frame of the furniture, its light plates and the accents of the seat.
pub fn cabin_meshes() -> [(Skin, Mesh); 3] {
    [
        (Skin::Dark, fittings::furniture().into_mesh()),
        (Skin::Hull, fittings::plates().into_mesh()),
        (Skin::Trim, fittings::accents().into_mesh()),
    ]
}

/// Builds the canopy that lifts, each mesh with its skin: the glass and its frame, around the
/// hinge of the canopy at the origin.
pub fn canopy_meshes() -> [(Skin, Mesh); 2] {
    let at = |solid: Solid| solid.into_mesh().translated_by(-HINGE);
    [
        (Skin::Glass, at(dome::glass())),
        (Skin::Dark, at(bars::frame())),
    ]
}

/// Builds the tub of the cockpit, which takes its own grey lining.
pub fn tub_mesh() -> Mesh {
    tub::tub().into_mesh()
}

/// Builds one leg of the gear, strut and pad, hanging from its hinge at the origin: scaled
/// along y it folds up into its pod.
pub fn leg_mesh() -> Mesh {
    struts::leg().into_mesh()
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
        Skin::Hull => paint(0.86, 0.86, 0.82, 0.55),
        Skin::Trim => paint(0.95, 0.48, 0.16, 0.6),
        Skin::Dark => paint(0.09, 0.1, 0.13, 0.8),
        Skin::Glass => StandardMaterial {
            base_color: Color::srgba(0.42, 0.6, 0.72, 0.16),
            alpha_mode: AlphaMode::Blend,
            perceptual_roughness: 0.3,
            reflectance: 0.3,
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
mod tests_attach;
#[cfg(test)]
mod tests_cabin;
#[cfg(test)]
mod tests_collide;
#[cfg(test)]
mod tests_fit;
#[cfg(test)]
mod tests_gear;
#[cfg(test)]
mod tests_glow;
#[cfg(test)]
mod tests_graph;
#[cfg(test)]
mod tests_shell;
#[cfg(test)]
mod tests_view;
