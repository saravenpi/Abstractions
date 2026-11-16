//! Applies saved settings to the window, audio and renderer whenever they change.

use bevy::audio::{GlobalVolume, Volume};
use bevy::prelude::*;
use bevy::window::{MonitorSelection, PresentMode, PrimaryWindow, WindowMode};

use super::{Persistent, Settings};
use crate::render::{GameCamera, RayTracing, configure_camera, set_fov};

/// Applies vsync and fullscreen to the primary window.
fn apply_window(
    settings: Res<Persistent<Settings>>,
    mut window: Single<&mut Window, With<PrimaryWindow>>,
) {
    window.present_mode = match settings.graphics.vsync {
        true => PresentMode::AutoVsync,
        false => PresentMode::AutoNoVsync,
    };
    window.mode = match settings.graphics.fullscreen {
        true => WindowMode::BorderlessFullscreen(MonitorSelection::Current),
        false => WindowMode::Windowed,
    };
}

/// Applies the master volume.
fn apply_audio(settings: Res<Persistent<Settings>>, mut volume: ResMut<GlobalVolume>) {
    volume.volume = Volume::Linear(settings.audio.master.clamp(0.0, 1.0));
}

/// Applies pixel size, color levels, field of view, ambient occlusion and bloom to the game camera.
///
/// Shadows and fog distance follow the settings through the lighting and fog systems.
fn apply_graphics(
    settings: Res<Persistent<Settings>>,
    ray_tracing: Res<RayTracing>,
    mut commands: Commands,
    mut cameras: Query<(Entity, &mut Projection), With<GameCamera>>,
) {
    for (camera, mut projection) in &mut cameras {
        configure_camera(&mut commands, camera, &settings.graphics, ray_tracing.0);
        set_fov(&mut projection, settings.graphics.fov);
    }
}

/// Registers the apply systems, which also run once on the first frame.
pub(super) fn plugin(app: &mut App) {
    let changed = resource_changed::<Persistent<Settings>>;
    app.add_systems(
        Update,
        (apply_window, apply_audio, apply_graphics).run_if(changed),
    );
}
