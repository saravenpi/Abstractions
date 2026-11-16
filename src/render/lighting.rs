//! Sun, ambient light, sky color and the camera look: fog, tonemapping and depth prepass.

use bevy::camera::visibility::Visibility;
use bevy::core_pipeline::prepass::DepthPrepass;
use bevy::light::{CascadeShadowConfigBuilder, DirectionalLight, GlobalAmbientLight};
use bevy::pbr::{DistanceFog, FogFalloff};
use bevy::prelude::*;
use bevy::render::view::{Msaa, Tonemapping};

use super::GameCamera;
use super::pixelate::Pixelate;
use crate::screens::Screen;
use crate::settings::{Persistent, Settings};
use crate::world::CHUNK_SIZE;

/// Soft blue used for the sky and the distance fog.
pub const SKY_COLOR: Color = Color::srgb(0.55, 0.74, 0.92);

/// Marker for the world's directional sun light.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Sun;

/// Returns linear fog that fades terrain out before the last streamed chunk.
pub fn fog_falloff(render_distance: u32) -> FogFalloff {
    let reach = render_distance as f32 * CHUNK_SIZE;
    FogFalloff::Linear {
        start: reach * 0.35,
        end: reach * 0.9,
    }
}

/// Spawns the warm sun with cascaded shadows sized for roughly 150 meters.
fn spawn_sun(mut commands: Commands, settings: Res<Persistent<Settings>>) {
    let cascades = CascadeShadowConfigBuilder {
        num_cascades: 3,
        minimum_distance: 0.1,
        first_cascade_far_bound: 20.0,
        maximum_distance: 150.0,
        overlap_proportion: 0.2,
    };
    commands.spawn((
        Sun,
        DirectionalLight {
            color: Color::srgb(1.0, 0.93, 0.78),
            illuminance: 12_000.0,
            shadow_maps_enabled: settings.graphics.shadows,
            ..default()
        },
        cascades.build(),
        Transform::from_xyz(0.0, 0.0, 0.0).looking_to(Vec3::new(-0.45, -0.75, -0.35), Vec3::Y),
        Visibility::default(),
        DespawnOnEnter(Screen::Title),
    ));
}

/// Gives the first-person camera its post-process, fog, tonemapping and prepass.
fn setup_camera(
    add: On<Add<GameCamera>>,
    mut commands: Commands,
    settings: Res<Persistent<Settings>>,
) {
    commands.entity(add.entity).insert((
        Pixelate::new(settings.graphics.pixel_size),
        DistanceFog {
            color: SKY_COLOR,
            falloff: fog_falloff(settings.graphics.render_distance),
            ..default()
        },
        Tonemapping::TonyMcMapface,
        Msaa::Off,
        DepthPrepass,
    ));
}

/// Registers sky, ambient light, the sun and the camera setup.
pub(super) fn plugin(app: &mut App) {
    app.insert_resource(ClearColor(SKY_COLOR))
        .insert_resource(GlobalAmbientLight {
            color: Color::srgb(0.7, 0.8, 1.0),
            brightness: 350.0,
            ..default()
        })
        .add_observer(setup_camera)
        .add_systems(OnEnter(Screen::Loading), spawn_sun);
}
