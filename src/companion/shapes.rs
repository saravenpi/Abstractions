//! The shared look of the companion: one handle per mesh and material every companion wears.
//!
//! They are built once and cloned into each companion. Only what a companion changes by itself
//! is its own: its picture, its two tinted materials and the mesh of its trail.

use bevy::material::OpaqueRendererMethod;
use bevy::prelude::*;

use super::body::body;
use super::orb::{orb, screen};
use super::tail;
use super::tune::HALF;

/// Radius of the shared orb in meters: the antenna tip as it is, the hands and the thruster scaled from it.
const ORB_RADIUS: f32 = 0.03;

/// Where the antenna ends, above and slightly behind the top of the head.
pub const ANTENNA_TIP: Vec3 = Vec3::new(0.0, HALF[1] + 0.11, 0.025);

/// Returns an unlit forward material, which shows its base colour as it is and so can glow through bloom.
pub fn unlit(picture: Option<Handle<Image>>) -> StandardMaterial {
    StandardMaterial {
        base_color: Color::BLACK,
        base_color_texture: picture,
        unlit: true,
        opaque_render_method: OpaqueRendererMethod::Forward,
        ..default()
    }
}

/// Returns the matte lit material of the shell and the hands, coloured by the vertex colours.
fn matte() -> StandardMaterial {
    StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.85,
        reflectance: 0.2,
        opaque_render_method: OpaqueRendererMethod::Forward,
        ..default()
    }
}

/// The meshes and materials every companion shares, built once.
#[derive(Resource, Debug, Clone)]
pub struct Shapes {
    /// The shell of the head-body with its antenna.
    pub body: Handle<Mesh>,
    /// The screen.
    pub screen: Handle<Mesh>,
    /// The orb the antenna tip, the hands and the thruster are scaled from.
    pub orb: Handle<Mesh>,
    /// The matte shell.
    pub shell: Handle<StandardMaterial>,
    /// The additive material of the trail.
    pub trail: Handle<StandardMaterial>,
}

impl Shapes {
    /// Builds the shared meshes and materials.
    pub fn new(meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>) -> Self {
        Self {
            body: meshes.add(body(ANTENNA_TIP)),
            screen: meshes.add(screen()),
            orb: meshes.add(orb(ORB_RADIUS)),
            shell: materials.add(matte()),
            trail: materials.add(tail::material()),
        }
    }
}
