//! Light entities of structures: flame and lamp meshes with point lights, spawned per chunk and dressed once.

use bevy::light::{NotShadowCaster, PointLight};
use bevy::material::OpaqueRendererMethod;
use bevy::prelude::*;

use super::flicker::power;
use super::glow::{LightKind, LightPoint};

/// A structure light. `near` says whether it is within reach of the camera.
#[derive(Component, Debug, Clone, Copy)]
pub struct Glow {
    /// Which kind of light.
    pub kind: LightKind,
    /// Offset of its flicker in radians.
    pub phase: f32,
    /// True while the light is switched on.
    pub near: bool,
}

/// Shared meshes and materials of every flame and smoke puff.
#[derive(Resource)]
pub struct FireAssets {
    /// Flame meshes for torch, campfire and brazier.
    pub flames: [Handle<Mesh>; 3],
    /// Glowing lamp box.
    pub lamp: Handle<Mesh>,
    /// Unlit flame material.
    pub flame_material: Handle<StandardMaterial>,
    /// Unlit lamp glass material.
    pub lamp_material: Handle<StandardMaterial>,
    /// Smoke puff mesh.
    pub puff: Handle<Mesh>,
    /// Unlit pale grey smoke material.
    pub puff_material: Handle<StandardMaterial>,
}

/// Creates the shared flame and smoke assets.
pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut cone = |r: f32, h: f32| meshes.add(Cone::new(r, h).mesh().resolution(5));
    let flames = [cone(0.08, 0.3), cone(0.3, 0.9), cone(0.2, 0.55)];
    let unlit = |rgb: [f32; 3]| StandardMaterial {
        base_color: Color::srgb(rgb[0], rgb[1], rgb[2]),
        unlit: true,
        opaque_render_method: OpaqueRendererMethod::Forward,
        ..default()
    };
    commands.insert_resource(FireAssets {
        flames,
        lamp: meshes.add(Cuboid::new(0.2, 0.32, 0.2)),
        flame_material: materials.add(unlit([1.0, 0.62, 0.15])),
        lamp_material: materials.add(unlit([1.0, 0.88, 0.5])),
        puff: meshes.add(Sphere::new(1.0).mesh().uv(6, 4)),
        puff_material: materials.add(unlit([0.62, 0.62, 0.64])),
    });
}

/// Spawns one switched-off light entity per point as a child of `chunk`.
pub fn spawn_lights(commands: &mut Commands, chunk: Entity, points: &[LightPoint]) {
    commands.entity(chunk).with_children(|c| {
        for p in points {
            let (_, range) = power(p.kind);
            let phase = (p.pos.x * 12.9898 + p.pos.z * 78.233).sin() * 43.0;
            c.spawn((
                Glow {
                    kind: p.kind,
                    phase,
                    near: false,
                },
                PointLight {
                    color: Color::srgb(1.0, 0.7, 0.4),
                    intensity: 0.0,
                    range,
                    shadow_maps_enabled: false,
                    ..default()
                },
                Transform::from_translation(p.pos),
                Visibility::Hidden,
            ));
        }
    });
}

/// Gives every new light its flame or lamp mesh, which casts no shadow.
pub fn dress(
    mut commands: Commands,
    assets: Res<FireAssets>,
    added: Query<(Entity, &Glow), Added<Glow>>,
) {
    for (entity, glow) in &added {
        let (mesh, material) = match glow.kind {
            LightKind::Lamp => (assets.lamp.clone(), assets.lamp_material.clone()),
            LightKind::Torch => (assets.flames[0].clone(), assets.flame_material.clone()),
            LightKind::Campfire => (assets.flames[1].clone(), assets.flame_material.clone()),
            LightKind::Brazier | LightKind::Hearth => {
                (assets.flames[2].clone(), assets.flame_material.clone())
            }
            LightKind::Candle => (assets.flames[0].clone(), assets.flame_material.clone()),
        };
        commands
            .entity(entity)
            .insert((Mesh3d(mesh), MeshMaterial3d(material), NotShadowCaster));
    }
}
