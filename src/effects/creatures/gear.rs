//! The few meshes and materials every creature shares.

use std::f32::consts::FRAC_PI_2;

use bevy::material::OpaqueRendererMethod;
use bevy::prelude::*;

/// Butterfly wing colors: white, yellow, orange and blue.
const WING_COLORS: [[f32; 3]; 4] = [
    [0.96, 0.96, 0.92],
    [0.97, 0.85, 0.2],
    [0.95, 0.5, 0.1],
    [0.3, 0.55, 0.95],
];

/// Shared handles for fireflies, butterflies and birds.
#[derive(Resource, Debug, Clone)]
pub struct Gear {
    /// Low-poly ball for a firefly.
    pub glow_mesh: Handle<Mesh>,
    /// Emissive yellow-green material of fireflies.
    pub glow: Handle<StandardMaterial>,
    /// Flat wing hinged at its inner edge, spanning `x` from 0 to 1.
    pub wing: Handle<Mesh>,
    /// Thin butterfly body.
    pub moth_body: Handle<Mesh>,
    /// Butterfly wing materials by color.
    pub moth_wings: Vec<Handle<StandardMaterial>>,
    /// Dark butterfly body material.
    pub moth_dark: Handle<StandardMaterial>,
    /// Small bird body.
    pub bird_body: Handle<Mesh>,
    /// Dark feathers.
    pub crow: Handle<StandardMaterial>,
    /// White feathers.
    pub gull: Handle<StandardMaterial>,
}

/// Returns a rough double sided material of one color.
fn sheet(color: Color) -> StandardMaterial {
    StandardMaterial {
        base_color: color,
        perceptual_roughness: 1.0,
        double_sided: true,
        cull_mode: None,
        opaque_render_method: OpaqueRendererMethod::Forward,
        ..default()
    }
}

/// Builds the shared meshes and materials once at startup.
pub fn build(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let wing = Mesh::from(Rectangle::new(1.0, 1.0))
        .rotated_by(Quat::from_rotation_x(-FRAC_PI_2))
        .translated_by(Vec3::X * 0.5);
    let glow = StandardMaterial {
        emissive: LinearRgba::rgb(5.0, 6.5, 1.2),
        ..sheet(Color::srgb(0.6, 0.75, 0.2))
    };
    let moth_wings = WING_COLORS
        .iter()
        .map(|&[r, g, b]| materials.add(sheet(Color::srgb(r, g, b))))
        .collect();
    commands.insert_resource(Gear {
        glow_mesh: meshes.add(Sphere::new(0.5).mesh().uv(6, 4)),
        glow: materials.add(glow),
        wing: meshes.add(wing),
        moth_body: meshes.add(Cuboid::new(0.015, 0.015, 0.09)),
        moth_wings,
        moth_dark: materials.add(sheet(Color::srgb(0.12, 0.1, 0.1))),
        bird_body: meshes.add(Cuboid::new(0.22, 0.2, 0.8)),
        crow: materials.add(sheet(Color::srgb(0.1, 0.1, 0.13))),
        gull: materials.add(sheet(Color::srgb(0.95, 0.96, 0.98))),
    });
}
