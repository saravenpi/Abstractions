//! The game camera look: graphics options, projection and fog, including the underwater fog.

use bevy::camera::{CameraMainTextureUsages, Hdr};
use bevy::core_pipeline::prepass::{DepthPrepass, NormalPrepass};
use bevy::ecs::system::SystemParam;
use bevy::pbr::{DistanceFog, FogFalloff, ScreenSpaceAmbientOcclusion};
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy::render::render_resource::TextureUsages;
use bevy::render::view::{Msaa, Tonemapping};
use bevy::solari::prelude::SolariLighting;

use super::GameCamera;
use super::palette::{Palette, srgb};
use super::pixelate::Pixelate;
use super::ray_tracing::RayTracing;
use crate::settings::{Graphics, Persistent, Settings};
use crate::world::{CHUNK_SIZE, SEA_LEVEL};

/// Fog and sky color seen from below the surface, in sRGB.
pub const UNDERWATER: Vec3 = Vec3::new(0.04, 0.27, 0.35);

/// Distance in meters at which the water hides everything.
const UNDERWATER_VISIBILITY: f32 = 25.0;

/// Strength of the bloom glow.
const BLOOM_INTENSITY: f32 = 0.12;

/// What the fog looks like, so the camera is only written when it changes.
#[derive(Clone, Copy, PartialEq)]
struct FogKey {
    submerged: bool,
    color: Vec3,
    render_distance: u32,
}

/// Returns true when the camera is below the sea surface.
pub fn submerged(camera: &GlobalTransform) -> bool {
    camera.translation().y < SEA_LEVEL
}

/// Returns linear fog that fades terrain out before the last streamed chunk.
fn fog_falloff(render_distance: u32) -> FogFalloff {
    let reach = render_distance as f32 * CHUNK_SIZE;
    FogFalloff::Linear {
        start: reach * 0.35,
        end: reach * 0.9,
    }
}

/// Builds the camera fog for a key.
fn fog_for(key: FogKey) -> DistanceFog {
    match key.submerged {
        true => DistanceFog {
            color: srgb(UNDERWATER),
            falloff: FogFalloff::from_visibility(UNDERWATER_VISIBILITY),
            ..default()
        },
        false => DistanceFog {
            color: srgb(key.color),
            falloff: fog_falloff(key.render_distance),
            ..default()
        },
    }
}

/// Sets the vertical field of view in degrees on a perspective projection.
pub fn set_fov(projection: &mut Projection, fov: u32) {
    if let Projection::Perspective(perspective) = projection {
        perspective.fov = (fov as f32).to_radians();
    }
}

/// Adds or removes the camera effects that the graphics settings ask for.
///
/// With `ray_tracing` the camera gets Solari lighting, which replaces ambient occlusion and needs HDR.
pub fn configure_camera(
    commands: &mut Commands,
    camera: Entity,
    graphics: &Graphics,
    ray_tracing: bool,
) {
    let mut entity = commands.entity(camera);
    entity.insert((
        Pixelate::new(graphics.pixel_size, graphics.color_levels),
        Tonemapping::TonyMcMapface,
        Msaa::Off,
        DepthPrepass,
    ));
    if ray_tracing {
        entity.insert(CameraMainTextureUsages::default().with(TextureUsages::STORAGE_BINDING));
        entity.insert_if_new(SolariLighting::default());
    }
    match graphics.ambient_occlusion && !ray_tracing {
        true => entity.insert(ScreenSpaceAmbientOcclusion::default()),
        false => entity.remove::<(ScreenSpaceAmbientOcclusion, NormalPrepass)>(),
    };
    match (graphics.bloom, ray_tracing) {
        (true, _) => entity.insert(Bloom {
            intensity: BLOOM_INTENSITY,
            ..Bloom::NATURAL
        }),
        (false, true) => entity.remove::<Bloom>(),
        (false, false) => entity.remove::<(Bloom, Hdr)>(),
    };
}

/// What a new game camera is configured from.
#[derive(SystemParam)]
struct Look<'w> {
    settings: Res<'w, Persistent<Settings>>,
    palette: Res<'w, Palette>,
    ray_tracing: Res<'w, RayTracing>,
}

/// Gives the first-person camera its post-process, fog, field of view and effects.
fn setup_camera(
    add: On<Add<GameCamera>>,
    mut commands: Commands,
    look: Look,
    mut projections: Query<&mut Projection>,
) {
    let graphics = &look.settings.graphics;
    configure_camera(&mut commands, add.entity, graphics, look.ray_tracing.0);
    if let Ok(mut projection) = projections.get_mut(add.entity) {
        set_fov(&mut projection, graphics.fov);
    }
    let key = FogKey {
        submerged: false,
        color: look.palette.horizon,
        render_distance: graphics.render_distance,
    };
    commands.entity(add.entity).insert(fog_for(key));
}

/// Matches the fog to the sky horizon, or to dense water fog below the surface.
fn update_fog(
    palette: Res<Palette>,
    settings: Res<Persistent<Settings>>,
    mut cameras: Query<(&GlobalTransform, &mut DistanceFog), With<GameCamera>>,
    mut last: Local<Option<FogKey>>,
) {
    for (transform, mut fog) in &mut cameras {
        let key = FogKey {
            submerged: submerged(transform),
            color: palette.horizon,
            render_distance: settings.graphics.render_distance,
        };
        if *last != Some(key) {
            *last = Some(key);
            *fog = fog_for(key);
        }
    }
}

/// Registers the camera setup and the fog update.
pub(super) fn plugin(app: &mut App) {
    app.add_observer(setup_camera)
        .add_systems(Update, update_fog);
}
