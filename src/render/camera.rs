//! The game camera look: render target, graphics options and projection.

use bevy::camera::{Hdr, RenderTarget};
use bevy::core_pipeline::prepass::{DepthPrepass, NormalPrepass};
use bevy::ecs::system::SystemParam;
use bevy::pbr::ScreenSpaceAmbientOcclusion;
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy::render::view::{Msaa, Tonemapping};

use super::GameCamera;
use super::fog::{FogKey, fog_for};
use super::lowres::LowRes;
use super::murk::Murk;
use super::palette::Palette;
use super::pixelate::Pixelate;
use crate::settings::{Graphics, Persistent, Settings};

/// Strength of the bloom glow.
const BLOOM_INTENSITY: f32 = 0.12;

/// Sets the vertical field of view in degrees on a perspective projection.
pub fn set_fov(projection: &mut Projection, fov: u32) {
    if let Projection::Perspective(perspective) = projection {
        perspective.fov = (fov as f32).to_radians();
    }
}

/// Adds or removes the camera effects that the graphics settings ask for.
pub fn configure_camera(commands: &mut Commands, camera: Entity, graphics: &Graphics) {
    let mut entity = commands.entity(camera);
    entity.insert((
        Pixelate::new(graphics.pixel_size, graphics.color_levels),
        Tonemapping::TonyMcMapface,
        Msaa::Off,
        DepthPrepass,
    ));
    match graphics.ambient_occlusion {
        true => entity.insert(ScreenSpaceAmbientOcclusion::default()),
        false => entity.remove::<(ScreenSpaceAmbientOcclusion, NormalPrepass)>(),
    };
    match graphics.bloom {
        true => entity.insert(Bloom {
            intensity: BLOOM_INTENSITY,
            ..Bloom::NATURAL
        }),
        false => entity.remove::<(Bloom, Hdr)>(),
    };
}

/// What a new game camera is configured from.
#[derive(SystemParam)]
struct Look<'w> {
    settings: Res<'w, Persistent<Settings>>,
    palette: Res<'w, Palette>,
    target: Option<Res<'w, LowRes>>,
}

/// Gives the first-person camera its post-process, fog, field of view and effects. Without the
/// low-resolution game view, as in a run without a window, the camera keeps its own target.
fn setup_camera(
    add: On<Add<GameCamera>>,
    mut commands: Commands,
    look: Look,
    mut projections: Query<&mut Projection>,
) {
    let graphics = &look.settings.graphics;
    configure_camera(&mut commands, add.entity, graphics);
    if let Some(target) = look.target {
        commands
            .entity(add.entity)
            .insert(RenderTarget::from(target.0.clone()));
    }
    if let Ok(mut projection) = projections.get_mut(add.entity) {
        set_fov(&mut projection, graphics.fov);
    }
    let key = FogKey::ground(look.palette.horizon, graphics.render_distance);
    commands
        .entity(add.entity)
        .insert((fog_for(key), Murk::default()));
}

/// Registers the camera setup.
pub(super) fn plugin(app: &mut App) {
    app.add_observer(setup_camera);
}
