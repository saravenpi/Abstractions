//! Shared meshes and materials of the flight effects: one handle per kind, built once.

use std::f32::consts::TAU;

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

/// Segments around the rush ring.
const RING_SIDES: u32 = 32;

/// Inner and outer radius of the rush ring: clear inside, glowing outside.
const RING_RADII: (f32, f32) = (1.0, 2.2);

/// Linear colour and strongest alpha of the rush ring.
const RING_COLOR: [f32; 4] = [1.1, 1.2, 1.4, 0.22];

/// Radius and height of the unit nozzle flare.
const FLARE_SIZE: (f32, f32) = (0.5, 1.0);

/// Everything the flight effects draw with.
#[derive(Resource, Debug, Clone)]
pub struct Kit {
    /// Unit square for streaks, sparks and puffs.
    pub quad: Handle<Mesh>,
    /// Unit cone for the nozzle flares, tip up.
    pub cone: Handle<Mesh>,
    /// Ring for the rush of air at the edge of the view.
    pub ring: Handle<Mesh>,
    /// Additive material whose colour and alpha come from the vertices: ribbons and the ring.
    pub glow: Handle<StandardMaterial>,
    /// Wind speed lines.
    pub streak: Handle<StandardMaterial>,
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

/// Builds the rush ring: transparent at its inner edge, glowing at its outer one.
fn ring() -> Mesh {
    let [r, g, b, a] = RING_COLOR;
    let around = |i: u32| Vec2::from_angle(i as f32 / RING_SIDES as f32 * TAU);
    let rim =
        |radius: f32| (0..=RING_SIDES).map(move |i| (around(i) * radius).extend(0.0).to_array());
    let shade = |alpha: f32| (0..=RING_SIDES).map(move |_| [r, g, b, alpha]);
    let count = (RING_SIDES + 1) as usize * 2;
    let step = RING_SIDES + 1;
    let faces = (0..RING_SIDES).flat_map(|i| [i, i + step, i + 1, i + 1, i + step, i + step + 1]);
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        rim(RING_RADII.0)
            .chain(rim(RING_RADII.1))
            .collect::<Vec<_>>(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 0.0, 1.0]; count])
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.5, 0.5]; count])
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_COLOR,
        shade(0.0).chain(shade(a)).collect::<Vec<_>>(),
    )
    .with_inserted_indices(Indices::U32(faces.collect()))
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
        ring: meshes.add(ring()),
        glow: add(Color::WHITE, AlphaMode::Add),
        streak: add(Color::linear_rgba(1.4, 1.5, 1.7, 0.35), AlphaMode::Add),
        spark: add(Color::linear_rgba(8.0, 4.5, 1.2, 1.0), AlphaMode::Add),
        flare: add(Color::linear_rgba(6.0, 3.0, 0.9, 0.85), AlphaMode::Add),
        dust: add(Color::srgba(0.8, 0.68, 0.45, 0.55), AlphaMode::Blend),
        spray: add(Color::srgba(0.92, 0.96, 1.0, 0.5), AlphaMode::Blend),
    });
}
