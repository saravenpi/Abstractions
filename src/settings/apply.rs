//! Applies saved settings to the window, audio and renderer whenever they change.

use bevy::audio::{GlobalVolume, Volume};
use bevy::light::DirectionalLight;
use bevy::pbr::DistanceFog;
use bevy::prelude::*;
use bevy::window::{MonitorSelection, PresentMode, PrimaryWindow, WindowMode};

use super::{Persistent, Settings};
use crate::render::{Pixelate, Sun, fog_falloff};

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

/// Applies shadows, pixel size and fog distance to the sun and cameras.
fn apply_graphics(
    settings: Res<Persistent<Settings>>,
    mut suns: Query<&mut DirectionalLight, With<Sun>>,
    mut cameras: Query<(&mut Pixelate, &mut DistanceFog)>,
) {
    for mut sun in &mut suns {
        sun.shadow_maps_enabled = settings.graphics.shadows;
    }
    for (mut pixelate, mut fog) in &mut cameras {
        *pixelate = Pixelate::new(settings.graphics.pixel_size);
        fog.falloff = fog_falloff(settings.graphics.render_distance);
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
