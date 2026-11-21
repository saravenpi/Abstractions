//! Shared meshes and materials of the flight effects: one handle per kind, built once.

use bevy::prelude::*;

/// Radius and height of the unit nozzle flare.
const FLARE_SIZE: (f32, f32) = (0.5, 1.0);

/// Everything the flight effects draw with.
#[derive(Resource, Debug, Clone)]
pub struct Kit {
    /// Unit square for sparks and puffs.
    pub quad: Handle<Mesh>,
    /// Unit cone for the nozzle flares, tip up.
    pub cone: Handle<Mesh>,
    /// Additive material whose colour and alpha come from the vertices: the ribbons.
    pub glow: Handle<StandardMaterial>,
    /// Sparks under braking boots.
    pub spark: Handle<StandardMaterial>,
    /// Nozzle flares.
    pub flare: Handle<StandardMaterial>,
    /// Dust kicked up from the ground.
    pub dust: Handle<StandardMaterial>,
    /// Spray kicked up from water.
    pub spray: Handle<StandardMaterial>,
}

/// Returns an unlit, fog-free, two-sided material of `color`. Colours above 1 are HDR and bloom.
fn unlit(color: Color, alpha_mode: AlphaMode) -> StandardMaterial {
    StandardMaterial {
        base_color: color,
        alpha_mode,
        unlit: true,
        fog_enabled: false,
        double_sided: true,
        cull_mode: None,
        ..default()
    }
}

/// Builds the shared meshes and materials once at startup.
pub fn build(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut add = |color, mode| materials.add(unlit(color, mode));
    commands.insert_resource(Kit {
        quad: meshes.add(Rectangle::new(1.0, 1.0)),
        cone: meshes.add(Cone::new(FLARE_SIZE.0, FLARE_SIZE.1)),
        glow: add(Color::WHITE, AlphaMode::Add),
        spark: add(Color::linear_rgba(8.0, 4.5, 1.2, 1.0), AlphaMode::Add),
        flare: add(Color::linear_rgba(6.0, 3.0, 0.9, 0.85), AlphaMode::Add),
        dust: add(Color::srgba(0.8, 0.68, 0.45, 0.55), AlphaMode::Blend),
        spray: add(Color::srgba(0.92, 0.96, 1.0, 0.5), AlphaMode::Blend),
    });
}
