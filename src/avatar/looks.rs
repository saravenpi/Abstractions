//! The shared look of the robot: one handle per mesh and per material that every body wears.
//!
//! They are created once and cloned into each body, so sixteen robots cost sixteen sets of
//! entities and one set of assets. Every segment has two meshes: the rich one a body wears
//! near the camera and the light one it wears from afar. Only what a body changes by itself is its own: the picture
//! and the tint of its screen, the glow of its nozzles, and the three fabric meshes it drapes.

use bevy::material::OpaqueRendererMethod;
use bevy::prelude::*;

use super::mesh::{Detail, build, glow, screen};
use super::rig::{SEGS, Seg};
use super::tune::ACCENT;

/// The meshes and materials every body shares.
#[derive(Resource, Debug, Clone)]
pub struct Looks {
    /// The mesh of every segment as seen up close, in the order of [`Seg::ALL`].
    pub segments: [Handle<Mesh>; SEGS],
    /// The mesh of every segment as seen from afar: the same shapes, a quarter of the triangles.
    pub far: [Handle<Mesh>; SEGS],
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
        let mut level = |detail| Seg::ALL.map(|seg| meshes.add(build(seg, detail).into_mesh()));
        let (segments, far) = (level(Detail::Near), level(Detail::Far));
        let accent = Color::srgb(ACCENT[0], ACCENT[1], ACCENT[2]);
        Self {
            segments,
            far,
            shell: materials.add(paint(Color::WHITE, true, None)),
            screen: meshes.add(screen()),
            disc: meshes.add(glow(0.0)),
            cone: meshes.add(glow(1.0)),
            cloth: materials.add(sheet(accent)),
        }
    }
}
