//! Light entities of structures: point lights spawned per chunk, dressed with layered flames, ray traced on demand.

use bevy::ecs::system::SystemParam;
use bevy::light::{NotShadowCaster, PointLight};
use bevy::material::OpaqueRendererMethod;
use bevy::prelude::*;
use bevy::render::render_resource::Face;
use bevy::solari::prelude::RaytracingMesh3d;

use super::Embedded;
use super::flame::{SHELL_COLORS, emitter, pose, radiance, shells, teardrop};
use super::flicker::power;
use super::glow::{LightKind, LightPoint};
use crate::render::RayTracing;

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
    /// Emissive materials of the ray traced emitters, indexed by `LightKind as usize`.
    pub emitters: [Handle<StandardMaterial>; 6],
    /// Smoke puff and ember mesh.
    pub puff: Handle<Mesh>,
    /// Lit smoke materials from young to old.
    pub smoke: [Handle<StandardMaterial>; 3],
    /// Unlit HDR ember material.
    pub ember: Handle<StandardMaterial>,
}

/// The ray tracing side of the lights: the switch, the emitter mesh while it is on and every light.
#[derive(SystemParam)]
pub struct Tracer<'w, 's> {
    /// Whether ray traced lighting is active.
    ray_tracing: Res<'w, RayTracing>,
    /// Mesh assets, to create the emitter mesh.
    meshes: ResMut<'w, Assets<Mesh>>,
    /// The emitter mesh. It exists only while ray tracing is active, so no acceleration structure is built without it.
    emitter: Local<'s, Option<Handle<Mesh>>>,
    /// Every light and whether it is in the ray traced scene.
    lights: Query<'w, 's, (Entity, &'static Glow, Has<RaytracingMesh3d>)>,
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
    let emitters = LightKind::ALL.map(|kind| {
        materials.add(StandardMaterial {
            base_color: Color::BLACK,
            emissive: radiance(kind),
            ..default()
        })
    });
    commands.insert_resource(FireAssets {
        flame: meshes.add(teardrop()),
        shells,
        emitters,
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

/// Puts the lit lights in the ray traced scene as small emissive meshes, and takes them out when they go dark.
///
/// Solari has no point lights and ignores visibility, so a light exists there only through this mesh.
fn trace(commands: &mut Commands, assets: &FireAssets, tracer: &mut Tracer) {
    let on = tracer.ray_tracing.0;
    if !on {
        *tracer.emitter = None;
    }
    for (entity, glow, traced) in &tracer.lights {
        let wanted = on && glow.level > 0.0;
        if wanted && !traced {
            let mesh = tracer
                .emitter
                .get_or_insert_with(|| tracer.meshes.add(emitter()))
                .clone();
            let material = assets.emitters[glow.kind as usize].clone();
            commands
                .entity(entity)
                .try_insert((RaytracingMesh3d(mesh), MeshMaterial3d(material)));
        } else if traced && !wanted {
            commands
                .entity(entity)
                .try_remove::<(RaytracingMesh3d, MeshMaterial3d<StandardMaterial>)>();
        }
    }
}

/// Gives every new light its flame shells, which cast no shadow, and keeps the ray traced emitters in step.
pub fn dress(
    mut commands: Commands,
    assets: Res<FireAssets>,
    added: Query<(Entity, &Glow), Added<Glow>>,
    mut tracer: Tracer,
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
    trace(&mut commands, &assets, &mut tracer);
}
