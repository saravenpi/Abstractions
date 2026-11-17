//! The game camera look: render target, graphics options, projection and fog, including the underwater fog.

use bevy::camera::{CameraMainTextureUsages, Hdr, RenderTarget};
use bevy::core_pipeline::prepass::{
    DeferredPrepass, DepthPrepass, MotionVectorPrepass, NormalPrepass,
};
use bevy::ecs::system::SystemParam;
use bevy::pbr::{DistanceFog, FogFalloff, ScreenSpaceAmbientOcclusion};
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy::render::render_resource::TextureUsages;
use bevy::render::view::{Msaa, Tonemapping};
use bevy::solari::prelude::SolariLighting;

use super::GameCamera;
use super::clouds::CloudCrossing;
use super::lowres::LowRes;
use super::palette::{Palette, srgb};
use super::pixelate::Pixelate;
use super::ray_tracing::RayTracing;
use super::vantage::Vantage;
use crate::settings::{Graphics, Persistent, Settings};
use crate::world::CHUNK_SIZE;

/// Fog and sky color seen from below the surface, in sRGB.
pub const UNDERWATER: Vec3 = Vec3::new(0.04, 0.27, 0.35);

/// Distance in meters at which the water hides everything.
const UNDERWATER_VISIBILITY: f32 = 25.0;

/// Strength of the bloom glow.
const BLOOM_INTENSITY: f32 = 0.12;

/// Height above sea level in meters, over every summit, from which the fog starts to open up.
const CLIMB_FLOOR: f32 = 300.0;

/// Meters of sight gained per meter climbed above that height, so the planet below shows from high up.
const CLIMB_SIGHT: f32 = 10.0;

/// Least share of air the fog still counts: above the air the fog reaches a thousand times farther.
const THIN_AIR: f32 = 1.0e-3;

/// Distance in meters at which the middle of the cloud layer hides everything.
const WHITEOUT_VISIBILITY: f32 = 70.0;

/// What the fog looks like, so the camera is only written when it changes.
#[derive(Clone, Copy, PartialEq)]
struct FogKey {
    submerged: bool,
    color: Vec3,
    reach: f32,
}

impl FogKey {
    /// Builds the key for a camera in clear air on the ground.
    fn ground(color: Vec3, render_distance: u32) -> Self {
        Self {
            submerged: false,
            color,
            reach: fog_reach(render_distance, 0.0, 1.0),
        }
    }
}

/// Returns the distance in meters the fog lets the camera see: the streamed terrain on foot,
/// farther with `altitude` once above every summit, and without limit as the share of `air` left
/// goes to zero.
pub fn fog_reach(render_distance: u32, altitude: f32, air: f32) -> f32 {
    let ground = render_distance as f32 * CHUNK_SIZE;
    (ground + (altitude - CLIMB_FLOOR).max(0.0) * CLIMB_SIGHT) / air.max(THIN_AIR)
}

/// Builds the camera fog for a key: dense water fog below the surface, or linear fog that fades
/// the terrain out before its reach.
fn fog_for(key: FogKey) -> DistanceFog {
    match key.submerged {
        true => DistanceFog {
            color: srgb(UNDERWATER),
            falloff: FogFalloff::from_visibility(UNDERWATER_VISIBILITY),
            ..default()
        },
        false => DistanceFog {
            color: srgb(key.color),
            falloff: FogFalloff::Linear {
                start: key.reach * 0.35,
                end: key.reach * 0.9,
            },
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
/// Without it every component Solari added goes, so the camera is exactly the normal one.
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
        entity.insert_if_new(SolariLighting {
            restir: true,
            confidence_weight_cap: 16.0,
            ..default()
        });
    } else {
        entity.insert(CameraMainTextureUsages::default());
        entity.remove::<(SolariLighting, DeferredPrepass, MotionVectorPrepass)>();
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
    target: Res<'w, LowRes>,
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
    commands
        .entity(add.entity)
        .insert(RenderTarget::from(look.target.0.clone()));
    if let Ok(mut projection) = projections.get_mut(add.entity) {
        set_fov(&mut projection, graphics.fov);
    }
    let key = FogKey::ground(look.palette.horizon, graphics.render_distance);
    commands.entity(add.entity).insert(fog_for(key));
}

/// Matches the fog to the sky horizon and the height of the camera, to the whiteout inside the
/// cloud layer, or to dense water fog below the surface.
fn update_fog(
    palette: Res<Palette>,
    settings: Res<Persistent<Settings>>,
    place: (Res<Vantage>, Res<CloudCrossing>),
    mut cameras: Query<&mut DistanceFog, With<GameCamera>>,
    mut last: Local<Option<FogKey>>,
) {
    let (vantage, crossing) = place;
    let clear = fog_reach(
        settings.graphics.render_distance,
        vantage.altitude,
        vantage.air,
    );
    let key = FogKey {
        submerged: vantage.altitude < 0.0,
        color: palette.horizon.lerp(palette.cloud, crossing.depth),
        reach: clear * (WHITEOUT_VISIBILITY / clear).powf(crossing.depth),
    };
    for mut fog in &mut cameras {
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
