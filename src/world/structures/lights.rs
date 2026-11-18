//! Light entities of structures: point lights spawned per chunk, dressed with layered flames.

use bevy::light::{NotShadowCaster, PointLight};
use bevy::material::OpaqueRendererMethod;
use bevy::prelude::*;
use bevy::render::render_resource::Face;

use super::Embedded;
use super::flame::{SHELL_COLORS, pose, shells, teardrop};
use super::flicker::power;
use super::glow::{LightKind, LightPoint};

/// Radius of every structure light in meters, which softens its highlights close to a wall.
const LIGHT_RADIUS: f32 = 0.15;

/// Base color and opacity of a smoke puff when young, middle-aged and old: dark soot that thins to pale grey.
const SMOKE: [(f32, f32); 3] = [(0.3, 0.36), (0.45, 0.24), (0.6, 0.13)];

/// A structure light. `near` says whether it should be lit, `level` how far it has faded in.
#[derive(Component, Debug, Clone, Copy)]
pub struct Glow {
    /// Which kind of light.
    pub kind: LightKind,
    /// Offset of its flicker in radians.
    pub phase: f32,
    /// True while the light is among the nearest ones and should be on.
    pub near: bool,
    /// Fade from 0 (off and hidden) to 1 (fully lit).
    pub level: f32,
}

/// Shared meshes and materials of every flame, ember and smoke puff.
#[derive(Resource)]
pub struct FireAssets {
    /// Unit teardrop every flame shell is scaled from.
    pub flame: Handle<Mesh>,
    /// Unlit HDR flame materials from the outer shell to the core.
    pub shells: [Handle<StandardMaterial>; 3],
    /// Smoke puff and ember mesh.
    pub puff: Handle<Mesh>,
    /// Lit smoke materials from young to old.
    pub smoke: [Handle<StandardMaterial>; 3],
    /// Unlit HDR ember material.
    pub ember: Handle<StandardMaterial>,
}

/// Returns an unlit forward material of a linear HDR color that shows the faces not culled by `cull`.
fn bright(rgb: [f32; 3], cull: Face) -> StandardMaterial {
    StandardMaterial {
        base_color: Color::linear_rgb(rgb[0], rgb[1], rgb[2]),
        unlit: true,
        cull_mode: Some(cull),
        opaque_render_method: OpaqueRendererMethod::Forward,
        ..default()
    }
}

/// Returns the lit, see-through material of a smoke puff of one grey.
fn smoke((grey, alpha): (f32, f32)) -> StandardMaterial {
    StandardMaterial {
        base_color: Color::srgba(grey, grey, grey * 1.04, alpha),
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 1.0,
        reflectance: 0.0,
        opaque_render_method: OpaqueRendererMethod::Forward,
        ..default()
    }
}

/// Creates the shared flame, ember and smoke assets.
///
/// The outer flame shells show their far side only, so the paler shells inside stay visible without blending.
pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let culls = [Face::Front, Face::Front, Face::Back];
    let shells = [0, 1, 2].map(|i| materials.add(bright(SHELL_COLORS[i], culls[i])));
    commands.insert_resource(FireAssets {
        flame: meshes.add(teardrop()),
        shells,
        puff: meshes.add(Sphere::new(1.0).mesh().uv(6, 4)),
        smoke: SMOKE.map(|shade| materials.add(smoke(shade))),
        ember: materials.add(bright([6.0, 1.8, 0.2], Face::Back)),
    });
}

/// Spawns one switched-off light entity per point as a child of `chunk`, marked [`Embedded`] when the
/// points already are in the frame of the chunk.
pub fn spawn_lights(commands: &mut Commands, chunk: Entity, points: &[LightPoint], rigid: bool) {
    commands.entity(chunk).with_children(|c| {
        for p in points {
            let phase = (p.pos.x * 12.9898 + p.pos.z * 78.233).sin() * 43.0;
            let mut light = c.spawn((
                Glow {
                    kind: p.kind,
                    phase,
                    near: false,
                    level: 0.0,
                },
                PointLight {
                    color: p.kind.tint(),
                    intensity: 0.0,
                    range: power(p.kind).1,
                    radius: LIGHT_RADIUS,
                    shadow_maps_enabled: false,
                    ..default()
                },
                Transform::from_translation(p.pos).with_rotation(p.rot),
                Visibility::Hidden,
            ));
            if rigid {
                light.insert(Embedded);
            }
        }
    });
}

/// Gives every new light its flame shells, which cast no shadow.
pub fn dress(
    mut commands: Commands,
    assets: Res<FireAssets>,
    added: Query<(Entity, &Glow), Added<Glow>>,
) {
    for (entity, glow) in &added {
        commands.entity(entity).with_children(|c| {
            for (index, layer) in shells(glow.kind, glow.phase) {
                c.spawn((
                    layer,
                    Mesh3d(assets.flame.clone()),
                    MeshMaterial3d(assets.shells[index].clone()),
                    pose(&layer, 0.0, Vec3::ZERO),
                    NotShadowCaster,
                ));
            }
        });
    }
}
