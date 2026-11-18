//! The shared look of the robot: one handle per mesh and per material that every body wears.
//!
//! They are created once and cloned into each body, so sixteen robots cost sixteen sets of
//! entities and one set of assets. Only what a body changes by itself is its own: the picture
//! and the tint of its screen, the glow of its nozzles, and the three fabric meshes it drapes.

use bevy::material::OpaqueRendererMethod;
use bevy::prelude::*;

use super::mesh::{glow, screen, segment};
use super::rig::{SEGS, Seg};
use super::tune::ACCENT;

/// The meshes and materials every body shares.
#[derive(Resource, Debug, Clone)]
pub struct Looks {
    /// The mesh of every segment, in the order of [`Seg::ALL`]. Left and right share one.
    pub segments: [Handle<Mesh>; SEGS],
    /// The matte shell, coloured by the vertex colours.
    pub shell: Handle<StandardMaterial>,
    /// The screen on the head.
    pub screen: Handle<Mesh>,
    /// The glowing disc in a sole.
    pub disc: Handle<Mesh>,
    /// The plume under a sole.
    pub cone: Handle<Mesh>,
    /// The fabric of the wingsuit, lit from both sides.
    pub cloth: Handle<StandardMaterial>,
}

/// Returns a forward material: `lit` for the matte shell coloured by the vertex colours, or unlit,
/// which shows its base colour as it is and so can glow through bloom.
pub fn paint(base: Color, lit: bool, picture: Option<Handle<Image>>) -> StandardMaterial {
    StandardMaterial {
        base_color: base,
        base_color_texture: picture,
        unlit: !lit,
        perceptual_roughness: 0.85,
        reflectance: 0.2,
        opaque_render_method: OpaqueRendererMethod::Forward,
        ..default()
    }
}

/// Returns the material of the fabric: lit `accent` seen from both sides.
pub fn sheet(accent: Color) -> StandardMaterial {
    StandardMaterial {
        double_sided: true,
        cull_mode: None,
        ..paint(accent, true, None)
    }
}

impl Looks {
    /// Builds the shared meshes and materials.
    pub fn new(meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>) -> Self {
        let mut segments: [Handle<Mesh>; SEGS] = default();
        for seg in Seg::ALL {
            segments[seg as usize] = match seg.side() {
                Some(1) => segments[seg as usize - 1].clone(),
                _ => meshes.add(segment(seg).into_mesh()),
            };
        }
        let accent = Color::srgb(ACCENT[0], ACCENT[1], ACCENT[2]);
        Self {
            segments,
            shell: materials.add(paint(Color::WHITE, true, None)),
            screen: meshes.add(screen()),
            disc: meshes.add(glow(0.0)),
            cone: meshes.add(glow(1.0)),
            cloth: materials.add(sheet(accent)),
        }
    }
}
